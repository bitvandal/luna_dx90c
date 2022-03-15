// Controls: Use mouse to orbit and zoom; use the 'W' and 'S' keys to
//           alter the height of the camera.
//           Use '1', '2', '3', '4', and '5' keys to select the bone
//           to rotate.  Use the 'A' and 'D' keys to rotate the bone.

use std::slice::from_raw_parts_mut;
use libc::c_void;
use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D9::*, Win32::System::SystemServices::*,
};

use crate::*;

// Material
#[repr(C)]
struct Mtrl {
    pub ambient: D3DXCOLOR,
    pub diffuse: D3DXCOLOR,
    pub spec: D3DXCOLOR,
    pub spec_power: f32,
}

// Directional Light
#[repr(C)]
struct DirLight {
    ambient: D3DXCOLOR,
    diffuse: D3DXCOLOR,
    spec: D3DXCOLOR,
    dir_w: D3DXVECTOR3,
}

// Bounding Volumes
#[repr(C)]
struct AABB  {
    min_pt: D3DXVECTOR3,
    max_pt: D3DXVECTOR3,
}

impl Default for AABB {
    fn default() -> Self {
        AABB {
            min_pt: D3DXVECTOR3 {
                x: f32::MAX,
                y: f32::MAX,
                z: f32::MAX
            },
            max_pt: D3DXVECTOR3 {
                x: f32::MIN,
                y: f32::MIN,
                z: f32::MIN
            }
        }
    }
}

impl AABB {
    #[allow(unused)]
    pub fn center(&self) -> D3DXVECTOR3 {
        D3DXVECTOR3 {
            x: 0.5 * (self.min_pt.x + self.max_pt.x),
            y: 0.5 * (self.min_pt.y + self.max_pt.y),
            z: 0.5 * (self.min_pt.z + self.max_pt.z)
        }
    }
}

// Bone frame
#[derive(Copy, Clone)]
pub struct BoneFrame {
    // Note: The root bone's "parent" frame is the world space.

    pub pos: D3DXVECTOR3,       // Relative to parent frame.
    pub z_angle: f32,           // Relative to parent frame.
    pub to_parent_x_form: D3DXMATRIX,
    pub to_world_x_form: D3DXMATRIX,
}

impl Default for BoneFrame {
    fn default() -> Self {
        BoneFrame {
            pos: Default::default(),
            z_angle: 0.0,
            to_parent_x_form: Default::default(),
            to_world_x_form: Default::default()
        }
    }
}

// Our robot arm has five bones.
const NUM_BONES: usize = 5;

// Sample demo
pub struct RobotArmDemo {
    d3d_pp: *const D3DPRESENT_PARAMETERS,
    gfx_stats: Option<GfxStats>,

    // We only need one bone mesh.  To draw several bones we just draw the
    // same mesh several times, but with a different transformation
    // applied so that it is drawn in a different place.
    bone_mesh: LPD3DXMESH,
    mtrl: Vec<Mtrl>,
    tex: Vec<*mut c_void>,

    bones: [BoneFrame; NUM_BONES],

    // Index into the bone array to the currently selected bone.
    // The user can select a bone and rotate it.
    bone_selected: usize,

    white_tex: *mut c_void,  //IDirect3DTexture9,

    fx: LPD3DXEFFECT,

    h_tech: D3DXHANDLE,
    h_wvp: D3DXHANDLE,
    h_world_inverse_transpose: D3DXHANDLE,
    h_mtrl: D3DXHANDLE,
    h_light: D3DXHANDLE,
    h_eye_pos: D3DXHANDLE,
    h_world: D3DXHANDLE,
    h_tex: D3DXHANDLE,

    light: DirLight,

    camera_rotation_y: f32,
    camera_radius: f32,
    camera_height: f32,

    world: D3DXMATRIX,
    view: D3DXMATRIX,
    proj: D3DXMATRIX,
}

impl RobotArmDemo {
    pub fn new(d3d_device: IDirect3DDevice9, d3d_pp: *const D3DPRESENT_PARAMETERS) -> Option<RobotArmDemo> {
        dbg!("QUE PASA!");

        if !RobotArmDemo::check_device_caps() {
            display_error_then_quit("checkDeviceCaps() Failed");
        }

        init_all_vertex_declarations(d3d_device.clone());

        let mut gfx_stats = GfxStats::new(d3d_device.clone(), D3DCOLOR_XRGB!(0, 0, 0));

        let mut light_dir_w = D3DXVECTOR3 { x: 0.0, y: 1.0, z: 2.0 };
        D3DXVec3Normalize(&mut light_dir_w, &light_dir_w);

        let light = DirLight {
            ambient: D3DXCOLOR { r: 0.8, g: 0.8, b: 0.8, a: 1.0 },
            diffuse: D3DXCOLOR { r: 0.8, g: 0.8, b: 0.8, a: 1.0 },
            spec: D3DXCOLOR { r: 0.8, g: 0.8, b: 0.8, a: 1.0 },
            dir_w: light_dir_w,
        };

        let (bone_mesh, mtrl, tex) =
            RobotArmDemo::load_x_file("bone.x", d3d_device.clone());

        let mut world = unsafe { std::mem::zeroed() };
        D3DXMatrixIdentity(&mut world);

        // Create the white dummy texture.
        let mut white_tex = unsafe { std::mem::zeroed() };
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(), PSTR(b"luna_30_robot_arm_demo/whitetex.dds\0".as_ptr() as _), &mut white_tex));

        // Initialize the bones relative to their parent frame.
        // The root is special--its parent frame is the world space.
        // As such, its position and angle are ignored--its
        // toWorldXForm should be set explicitly (that is, the world
        // transform of the entire skeleton).
        //
        // *------*------*------*------
        //    0      1      2      3

        let mut bones: [BoneFrame; NUM_BONES] = [BoneFrame::default(); NUM_BONES];

        for i in 1..NUM_BONES { // Ignore root.
            // Describe each bone frame relative to its parent frame.
            bones[i].pos = D3DXVECTOR3 { x: 2.0, y: 0.0, z: 0.0 };
            bones[i].z_angle = 0.0;
        }

        // Root frame at center of world.
        bones[0].pos = D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 };
        bones[0].z_angle = 0.0;

        // Start off with the last (leaf) bone:
        let bone_selected = NUM_BONES - 1;

        if let Some(gfx_stats) = &mut gfx_stats {
            gfx_stats.add_vertices(ID3DXBaseMesh_GetNumVertices(bone_mesh));
            gfx_stats.add_triangles(ID3DXBaseMesh_GetNumFaces(bone_mesh));
        }

        let (fx,
            h_tech,
            h_wvp,
            h_world_inverse_transpose,
            h_mtrl,
            h_light,
            h_eye_pos,
            h_world,
            h_tex) =
            RobotArmDemo::build_fx(d3d_device.clone());

        let mut robot_arm_demo = RobotArmDemo {
            d3d_pp,
            gfx_stats,

            bone_mesh,

            mtrl,
            tex,

            bones,
            bone_selected,

            white_tex,

            fx,

            h_tech,
            h_wvp,
            h_world_inverse_transpose,
            h_mtrl,
            h_light,
            h_eye_pos,
            h_world,
            h_tex,

            light,

            camera_radius: 9.0,
            camera_rotation_y: 1.5 * D3DX_PI,
            camera_height: 0.0,

            world,

            view: unsafe { std::mem::zeroed() },
            proj: unsafe { std::mem::zeroed() },
        };

        robot_arm_demo.on_reset_device();

        Some(robot_arm_demo)
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        ReleaseCOM(self.fx);
        ReleaseCOM(self.bone_mesh);

        for tex in &self.tex {
            ReleaseCOM(tex.cast());
        }

        destroy_all_vertex_declarations();
    }

    fn check_device_caps() -> bool {
        unsafe {
            if let Some(d3d_device) = &D3D_DEVICE {
                let mut caps: D3DCAPS9 = std::mem::zeroed();
                HR!(d3d_device.GetDeviceCaps(&mut caps));

                // Check for vertex shader version 2.0 support.
                if caps.VertexShaderVersion < D3DVS_VERSION!(2, 0) {
                    return false;
                }

                // Check for pixel shader version 2.0 support.
                if caps.PixelShaderVersion < D3DPS_VERSION!(2, 0) {
                    return false;
                }
            }

            true
        }
    }

    pub fn on_lost_device(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.on_lost_device();
        }

        HR!(ID3DXEffect_OnLostDevice(self.fx));
    }

    pub fn on_reset_device(&mut self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.on_reset_device();
        }

        HR!(ID3DXEffect_OnResetDevice(self.fx));

        // The aspect ratio depends on the backbuffer dimensions, which can
        // possibly change after a reset.  So rebuild the projection matrix.
        self.build_proj_mtx();
    }

    pub fn update_scene(&mut self, dt: f32) {
        if let Some(gfx_stats) = &mut self.gfx_stats {
            gfx_stats.update(dt);
        }

        // Get snapshot of input devices.
        unsafe {
            if let Some(dinput) = &mut DIRECT_INPUT {
                dinput.poll();

                // Check input.
                if dinput.key_down(DIK_W as usize) {
                    self.camera_height += 25.0 * dt;
                }

                if dinput.key_down(DIK_S as usize) {
                    self.camera_height -= 25.0 * dt;
                }

                // Allow the user to select a bone (zero based index)
                if dinput.key_down(DIK_1 as usize) {
                    self.bone_selected = 0;
                }

                if dinput.key_down(DIK_2 as usize) {
                    self.bone_selected = 1;
                }

                if dinput.key_down(DIK_3 as usize) {
                    self.bone_selected = 2;
                }

                if dinput.key_down(DIK_4 as usize) {
                    self.bone_selected = 3;
                }

                if dinput.key_down(DIK_5 as usize) {
                    self.bone_selected = 4;
                }

                // Allow the user to rotate a bone.
                if dinput.key_down(DIK_A as usize) {
                    self.bones[self.bone_selected].z_angle += 1.0 * dt;
                }

                if dinput.key_down(DIK_D as usize) {
                    self.bones[self.bone_selected].z_angle -= 1.0 * dt;
                }

                // If we rotate over 360 degrees, just roll back to 0
                if self.bones[self.bone_selected].z_angle.abs() >= 2.0 * D3DX_PI {
                    self.bones[self.bone_selected].z_angle = 0.0;
                }

                // Divide by 50 to make mouse less sensitive.
                self.camera_rotation_y += dinput.mouse_dx() / 100.0;
                self.camera_radius += dinput.mouse_dy() / 25.0;

                // If we rotate over 360 degrees, just roll back to 0
                if self.camera_rotation_y.abs() >= 2.0 * D3DX_PI {
                    self.camera_rotation_y = 0.0;
                }

                // Don't let radius get too small.
                if self.camera_radius < 2.0 {
                    self.camera_radius = 2.0;
                }

                // The camera position/orientation relative to world space can
                // change every frame based on input, so we need to rebuild the
                // view matrix every frame with the latest changes.
                self.build_view_mtx();
            }
        }
    }

    pub fn draw_scene(&mut self) {
        unsafe {
            if let Some(d3d_device) = &D3D_DEVICE {
                // Clear the backbuffer and depth buffer.
                HR!(d3d_device.Clear(
                    0,
                    std::ptr::null(),
                    (D3DCLEAR_TARGET | D3DCLEAR_ZBUFFER) as u32,
                    0xFFFFFFFF,
                    1.0,
                    0));

                HR!(d3d_device.BeginScene());

                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_light, &self.light as *const _ as _, std::mem::size_of::<DirLight>() as u32));

                HR!(ID3DXEffect_SetTechnique(self.fx, self.h_tech));

                let mut num_passes: u32 = 0;
                HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));

                HR!(ID3DXEffect_BeginPass(self.fx, 0));

                // Build the world transforms for each bone, then render them.
                self.build_bone_world_transforms();

                let mut t: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixTranslation(&mut t, -(NUM_BONES as f32), 0.0, 0.0);

                for i in 0..NUM_BONES {
                    // Append the transformation with a slight translation to better
                    // center the skeleton at the center of the scene.
                    self.world = std::mem::zeroed();
                    D3DXMatrixMultiply(&mut self.world, &self.bones[i].to_world_x_form, &t);

                    let mut res: D3DXMATRIX = std::mem::zeroed();
                    D3DXMatrixMultiply(&mut res, &self.world, &self.view);
                    D3DXMatrixMultiply(&mut res, &res, &self.proj);
                    HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &res));

                    let mut world_inverse_transpose: D3DXMATRIX = std::mem::zeroed();
                    D3DXMatrixInverse(&mut world_inverse_transpose, 0.0, &self.world);
                    D3DXMatrixTranspose(&mut world_inverse_transpose, &world_inverse_transpose);
                    HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world_inverse_transpose, &world_inverse_transpose));

                    HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world, &self.world));

                    for j in 0..self.mtrl.len() {
                        HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_mtrl, &self.mtrl[j] as *const _ as _, std::mem::size_of::<Mtrl>() as u32));

                        // If there is a texture, then use.
                        if !self.tex[j].is_null() {
                            HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.tex[j]));
                        } else {
                            // But if not, then set a pure white texture.  When the texture color
                            // is multiplied by the color from lighting, it is like multiplying by
                            // 1 and won't change the color from lighting.

                            HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.white_tex));
                        }

                        HR!(ID3DXEffect_CommitChanges(self.fx));
                        HR!(ID3DXBaseMesh_DrawSubset(self.bone_mesh, j as u32));
                    }
                }

                HR!(ID3DXEffect_EndPass(self.fx));
                HR!(ID3DXEffect_End(self.fx));

                if let Some(gfx_stats) = &self.gfx_stats {
                    gfx_stats.display();
                }

                HR!(d3d_device.EndScene());

                HR!(d3d_device.Present(
                    std::ptr::null(),
                    std::ptr::null(),
                    HWND(0),
                    std::ptr::null()));
            }
        }
    }

    fn build_view_mtx(&mut self) {
        let x: f32 = self.camera_radius * self.camera_rotation_y.cos();
        let z: f32 = self.camera_radius * self.camera_rotation_y.sin();
        let pos = D3DXVECTOR3 { x, y: self.camera_height, z };
        let target = D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 };
        let up = D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 };
        D3DXMatrixLookAtLH(&mut self.view, &pos, &target, &up);

        HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_eye_pos, &pos as *const _ as _, std::mem::size_of::<D3DXVECTOR3>() as u32));
    }

    fn build_proj_mtx(&mut self) {
        let w: f32 = (unsafe { *self.d3d_pp }).BackBufferWidth as f32;
        let h: f32 = (unsafe { *self.d3d_pp }).BackBufferHeight as f32;
        D3DXMatrixPerspectiveFovLH(&mut self.proj, D3DX_PI * 0.25, w / h, 1.0, 5000.0);
    }

    fn build_fx(d3d_device: IDirect3DDevice9) -> (LPD3DXEFFECT, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
                                                  D3DXHANDLE, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
                                                  D3DXHANDLE) {
        // Create the FX from a .fx file.
        let mut fx: LPD3DXEFFECT = std::ptr::null_mut();
        let mut errors: LPD3DXBUFFER = std::ptr::null_mut();

        HR!(D3DXCreateEffectFromFile(d3d_device, PSTR(b"luna_30_robot_arm_demo/PhongDirLtTex.fx\0".as_ptr() as _),
            std::ptr::null(), std::ptr::null(), D3DXSHADER_DEBUG,
            std::ptr::null(), &mut fx, &mut errors));

        unsafe {
            if !errors.is_null() {
                let errors_ptr: *mut c_void = ID3DXBuffer_GetBufferPointer(errors);

                let c_str: &CStr = CStr::from_ptr(errors_ptr.cast());
                let str_slice: &str = c_str.to_str().unwrap_or("<unknown error>");
                message_box(str_slice);
                // the original sample code will also crash at this point
            }
        }

        // Obtain handles.
        let h_tech = ID3DXBaseEffect_GetTechniqueByName(fx, PSTR(b"PhongDirLtTexTech\0".as_ptr() as _));
        let h_wvp = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWVP\0".as_ptr() as _));
        let h_world_inverse_transpose = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWorldInvTrans\0".as_ptr() as _));
        let h_mtrl = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gMtrl\0".as_ptr() as _));
        let h_light = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gLight\0".as_ptr() as _));
        let h_eye_pos = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gEyePosW\0".as_ptr() as _));
        let h_world = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWorld\0".as_ptr() as _));
        let h_tex = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gTex\0".as_ptr() as _));

        (fx, h_tech, h_wvp, h_world_inverse_transpose, h_mtrl, h_light, h_eye_pos, h_world, h_tex)
    }

    fn load_x_file(filename: &str, d3d_device: IDirect3DDevice9) -> (LPD3DXMESH, Vec<Mtrl>, Vec<*mut c_void>) {
        unsafe {
            let mut mesh_out: LPD3DXMESH = std::ptr::null_mut();
            let mut mtrls = Vec::new();
            let mut texs = Vec::new();

            // Step 1: Load the .x file from file into a system memory mesh.

            let mut mesh_sys: LPD3DXMESH = std::ptr::null_mut();
            let mut adj_buffer: LPD3DXBUFFER = std::ptr::null_mut();
            let mut mtrl_buffer: LPD3DXBUFFER = std::ptr::null_mut();
            let mut num_mtrls: u32 = 0;

            HR!(D3DXLoadMeshFromX(PSTR(build_file_path(filename).as_str().as_ptr() as _),
                D3DXMESH_SYSTEMMEM, d3d_device.clone(), &mut adj_buffer, &mut mtrl_buffer, std::ptr::null_mut(),
                &mut num_mtrls, &mut mesh_sys));

            // Step 2: Find out if the mesh already has normal info?

            let mut elems: [D3DVERTEXELEMENT9; 64] = [D3DVERTEXELEMENT9::default(); 64];
            HR!(ID3DXBaseMesh_GetDeclaration(mesh_sys, elems.as_mut_ptr()));

            let mut has_normals = false;

            for i in 0..=64 {
                // Did we reach D3DDECL_END() {0xFF,0,D3DDECLTYPE_UNUSED, 0,0,0}?
                if elems[i].Stream == 0xff {
                    break;
                }

                if elems[i].Type == D3DDECLTYPE_FLOAT3.0 as u8 &&
                    elems[i].Usage == D3DDECLUSAGE_NORMAL.0 as u8 &&
                    elems[i].UsageIndex == 0 {
                    has_normals = true;
                    break;
                }
            }

            // Step 3: Change vertex format to VertexPNT.

            let mut elements: [D3DVERTEXELEMENT9; 64] = [D3DVERTEXELEMENT9::default(); 64];
            let mut num_elements: u32 = 0;

            if let Some(decl) = &VERTEX_PNT_DECL {
                HR!(decl.GetDeclaration(elements.as_mut_ptr(), &mut num_elements));

                let mut temp: LPD3DXMESH = std::ptr::null_mut();
                HR!(ID3DXBaseMesh_CloneMesh(*&mesh_sys, D3DXMESH_SYSTEMMEM, elements.as_ptr(),
                        d3d_device.clone(), &mut temp));

                ReleaseCOM(*&mesh_sys);

                mesh_sys = temp;
            }

            // Step 4: If the mesh did not have normals, generate them.

            if has_normals == false {
                HR!(D3DXComputeNormals(mesh_sys, std::ptr::null()));
            }

            // Step 5: Optimize the mesh.

            let buf_pointer: *mut u32 = ID3DXBuffer_GetBufferPointer(adj_buffer).cast();

            HR!(ID3DXMesh_Optimize(*&mesh_sys,
                D3DXMESH_MANAGED | D3DXMESHOPT_COMPACT | D3DXMESHOPT_ATTRSORT | D3DXMESHOPT_VERTEXCACHE,
                buf_pointer,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut mesh_out));

            ReleaseCOM(mesh_sys);   // Done w/ system mesh.
            ReleaseCOM(adj_buffer); // Done with buffer.

            // Step 6: Extract the materials and load the textures.

            if mtrl_buffer != std::ptr::null_mut() && num_mtrls != 0 {
                let d3dxmtrls_ptr: *mut D3DXMATERIAL = ID3DXBuffer_GetBufferPointer(mtrl_buffer).cast();

                let d3dxmtrls: &mut [D3DXMATERIAL] = from_raw_parts_mut(d3dxmtrls_ptr, num_mtrls as usize);

                for i in 0..num_mtrls as usize {
                    // Save the ith material.  Note that the MatD3D property does not have an ambient
                    // value set when its loaded, so just set it to the diffuse value.

                    let m: Mtrl = Mtrl {
                        ambient: d3dxmtrls[i].MatD3D.Diffuse.into(),
                        diffuse: d3dxmtrls[i].MatD3D.Diffuse.into(),
                        spec: d3dxmtrls[i].MatD3D.Specular.into(),
                        spec_power: d3dxmtrls[i].MatD3D.Power,
                    };
                    mtrls.push(m);

                    // Check if the ith material has an associative texture
                    if !d3dxmtrls[i].pTextureFilename.is_null() {
                        // Yes, load the texture for the ith subset
                        let mut tex: *mut c_void = std::ptr::null_mut();

                        let c_str: &CStr = CStr::from_ptr(d3dxmtrls[i].pTextureFilename.0.cast());
                        let str_slice: &str = c_str.to_str().unwrap_or("<unknown error>");
                        let mut tex_fn = build_file_path(str_slice);

                        HR!(D3DXCreateTextureFromFile(d3d_device.clone(), PSTR(tex_fn.as_mut_ptr()), &mut tex));

                        texs.push(tex);
                    } else {
                        // No texture for the ith subset
                        texs.push(std::ptr::null_mut());
                    }
                }
            }

            ReleaseCOM(mtrl_buffer); // done w/ buffer

            (mesh_out, mtrls, texs)
        }
    }

    fn build_bone_world_transforms(&mut self) {
        unsafe {
            // First, construct the transformation matrix that transforms
            // the ith bone into the coordinate system of its parent.
            let mut r: D3DXMATRIX = std::mem::zeroed();
            let mut t: D3DXMATRIX = std::mem::zeroed();
            let mut p: D3DXVECTOR3;

            for i in 0..NUM_BONES {
                p = self.bones[i].pos;
                D3DXMatrixRotationZ(&mut r, self.bones[i].z_angle);
                D3DXMatrixTranslation(&mut t, p.x, p.y, p.z);

                D3DXMatrixMultiply(&mut self.bones[i].to_parent_x_form, &r, &t);
            }

            // Now, the ith object's world transform is given by its
            // to-parent transform, followed by its parent's to-parent transform,
            // followed by its grandparent's to-parent transform, and
            // so on, up to the root's to-parent transform.

            // For each bone...
            for i in 0..NUM_BONES {
                // Initialize to identity matrix.
                D3DXMatrixIdentity(&mut self.bones[i].to_world_x_form);

                // Combine  W[i] = W[i]*W[i-1]*...*W[0].
                let mut j = i as i32;
                while j >= 0 {
                    D3DXMatrixMultiply(&mut self.bones[i].to_world_x_form,
                                       &self.bones[i].to_world_x_form,
                                       &self.bones[j as usize].to_parent_x_form);

                    j = j - 1;
                }
            }
        }
    }
}

fn build_file_path(filename: &str) -> String {
    let mut filepath = String::from("luna_30_robot_arm_demo/");
    filepath.push_str(filename);
    filepath.push_str("\0");
    filepath
}