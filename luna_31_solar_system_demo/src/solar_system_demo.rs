use common::mtrl::Mtrl;
use libc::c_void;
use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D9::*, Win32::System::SystemServices::*,
};

use crate::*;

const BASE_PATH: &str = "luna_31_solar_system_demo/";

// Colors
const WHITE: D3DXCOLOR = D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

// Directional Light
#[repr(C)]
struct DirLight {
    ambient: D3DXCOLOR,
    diffuse: D3DXCOLOR,
    spec: D3DXCOLOR,
    dir_w: D3DXVECTOR3,
}

// We classify the objects in our scene as one of three types.
enum SolarType {
    SUN,
    PLANET,
    MOON
}

// Solar Object
struct SolarObject {
    // Note: The root's "parent" frame is the world space.
    type_id: SolarType,
    pos: D3DXVECTOR3,       // Relative to parent frame.
    y_angle: f32,           // Relative to parent frame.
    parent: i32,            // Index to parent frame (-1 if root, i.e., no parent)
    size: f32,              // Relative to world frame.
    tex: *mut c_void,       // IDirect3DTexture9
    to_parent_x_form: D3DXMATRIX,
    to_world_x_form: D3DXMATRIX,
}

const NUM_OBJECTS: usize = 10;

// Sample demo
pub struct SolarSystemDemo {
    d3d_pp: *const D3DPRESENT_PARAMETERS,
    gfx_stats: Option<GfxStats>,

    // We only need one sphere mesh.  To draw several solar objects we just
    // draw the same mesh several times, but with a different transformation
    // applied so that it is drawn in a different place.
    sphere: LPD3DXMESH,

    object: [SolarObject; NUM_OBJECTS],

    sun_tex: *mut c_void,       //IDirect3DTexture9,
    planet1_tex: *mut c_void,   //IDirect3DTexture9,
    planet2_tex: *mut c_void,   //IDirect3DTexture9,
    planet3_tex: *mut c_void,   //IDirect3DTexture9,
    moon_tex: *mut c_void,      //IDirect3DTexture9,

    // Use a white material--the color will come from the texture.
    white_mtrl: Mtrl,

    light: DirLight,

    fx: LPD3DXEFFECT,

    h_tech: D3DXHANDLE,
    h_wvp: D3DXHANDLE,
    h_world_inverse_transpose: D3DXHANDLE,
    h_mtrl: D3DXHANDLE,
    h_light: D3DXHANDLE,
    h_eye_pos: D3DXHANDLE,
    h_world: D3DXHANDLE,
    h_tex: D3DXHANDLE,

    camera_rotation_y: f32,
    camera_radius: f32,
    camera_height: f32,

    world: D3DXMATRIX,
    view: D3DXMATRIX,
    proj: D3DXMATRIX,
}

impl SolarSystemDemo {
    pub fn new(d3d_device: IDirect3DDevice9, d3d_pp: *const D3DPRESENT_PARAMETERS) -> Option<SolarSystemDemo> {
        if !SolarSystemDemo::check_device_caps() {
            display_error_then_quit("checkDeviceCaps() Failed");
        }

        init_all_vertex_declarations(d3d_device.clone());

        let mut gfx_stats = GfxStats::new(d3d_device.clone(), D3DCOLOR_XRGB!(255, 255, 255));

        // Setup a directional light.
        let mut light_dir_w = D3DXVECTOR3 { x: 0.0, y: 1.0, z: 2.0 };
        D3DXVec3Normalize(&mut light_dir_w, &light_dir_w);

        let light = DirLight {
            ambient: D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0},
            diffuse: D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0},
            spec: D3DXCOLOR { r: 0.6, g: 0.6, b: 0.6, a: 1.0 },
            dir_w: light_dir_w
        };

        // Create a sphere to represent a solar object.
        let mut sphere: LPD3DXMESH = std::ptr::null_mut();
        HR!(D3DXCreateSphere(d3d_device.clone(), 1.0, 30, 30, &mut sphere, std::ptr::null_mut()));
        gen_spherical_tex_coords(d3d_device.clone(), &mut sphere);

        let mut world = unsafe { std::mem::zeroed() };
        D3DXMatrixIdentity(&mut world);

        // Create the textures.
        let mut sun_tex = unsafe { std::mem::zeroed() };
        let mut planet1_tex = unsafe { std::mem::zeroed() };
        let mut planet2_tex = unsafe { std::mem::zeroed() };
        let mut planet3_tex = unsafe { std::mem::zeroed() };
        let mut moon_tex = unsafe { std::mem::zeroed() };

        HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
            PSTR(c_resource_path(BASE_PATH, "sun.dds").as_str().as_ptr() as _), &mut sun_tex));
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
            PSTR(c_resource_path(BASE_PATH, "planet1.dds").as_str().as_ptr() as _), &mut planet1_tex));
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
            PSTR(c_resource_path(BASE_PATH, "planet2.dds").as_str().as_ptr() as _), &mut planet2_tex));
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
            PSTR(c_resource_path(BASE_PATH, "planet3.dds").as_str().as_ptr() as _), &mut planet3_tex));
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
            PSTR(c_resource_path(BASE_PATH, "moon.dds").as_str().as_ptr() as _), &mut moon_tex));

        // Initialize default white material.
        let white_mtrl = Mtrl {
            ambient: WHITE,
            diffuse: WHITE,
            spec: WHITE.mult(0.5),
            spec_power: 48.0
        };

        //==================================================
        // Specify how the solar object frames are related.

        let pos: [D3DXVECTOR3; NUM_OBJECTS] = [
            D3DXVECTOR3 { x:  0.0, y: 0.0, z:  0.0 },
            D3DXVECTOR3 { x:  7.0, y: 0.0, z:  7.0 },
            D3DXVECTOR3 { x: -9.0, y: 0.0, z:  0.0 },
            D3DXVECTOR3 { x:  7.0, y: 0.0, z: -6.0 },
            D3DXVECTOR3 { x:  5.0, y: 0.0, z:  0.0 },
            D3DXVECTOR3 { x: -5.0, y: 0.0, z:  0.0 },
            D3DXVECTOR3 { x:  3.0, y: 0.0, z:  0.0 },
            D3DXVECTOR3 { x:  2.0, y: 0.0, z: -2.0 },
            D3DXVECTOR3 { x: -2.0, y: 0.0, z:  0.0 },
            D3DXVECTOR3 { x:  0.0, y: 0.0, z:  2.0 },
        ];

        let object: [SolarObject; NUM_OBJECTS] = [
            SolarObject { // SUN
                type_id: SolarType::SUN,
                pos: pos[0],
                y_angle: 0.0,
                parent: -1,
                size: 2.5,
                tex: sun_tex,
                to_parent_x_form: Default::default(),
                to_world_x_form: Default::default()
            },
            SolarObject { // P1
                type_id: SolarType::PLANET,
                pos: pos[1],
                y_angle: 0.0,
                parent: 0,
                size: 1.5,
                tex: planet1_tex,
                to_parent_x_form: Default::default(),
                to_world_x_form: Default::default()
            },
            SolarObject { // P2
                type_id: SolarType::PLANET,
                pos: pos[2],
                y_angle: 0.0,
                parent: 0,
                size: 1.2,
                tex: planet2_tex,
                to_parent_x_form: Default::default(),
                to_world_x_form: Default::default()
            },
            SolarObject { // P3
                type_id: SolarType::PLANET,
                pos: pos[3],
                y_angle: 0.0,
                parent: 0,
                size: 0.8,
                tex: planet3_tex,
                to_parent_x_form: Default::default(),
                to_world_x_form: Default::default()
            },
            SolarObject { // M1P1
                type_id: SolarType::MOON,
                pos: pos[4],
                y_angle: 0.0,
                parent: 1,
                size: 0.5,
                tex: moon_tex,
                to_parent_x_form: Default::default(),
                to_world_x_form: Default::default()
            },
            SolarObject { // M2P1
                type_id: SolarType::MOON,
                pos: pos[5],
                y_angle: 0.0,
                parent: 1,
                size: 0.5,
                tex: moon_tex,
                to_parent_x_form: Default::default(),
                to_world_x_form: Default::default()
            },
            SolarObject { // M1P2
                type_id: SolarType::MOON,
                pos: pos[6],
                y_angle: 0.0,
                parent: 2,
                size: 0.4,
                tex: moon_tex,
                to_parent_x_form: Default::default(),
                to_world_x_form: Default::default()
            },
            SolarObject { // M1P3
                type_id: SolarType::MOON,
                pos: pos[7],
                y_angle: 0.0,
                parent: 3,
                size: 0.3,
                tex: moon_tex,
                to_parent_x_form: Default::default(),
                to_world_x_form: Default::default()
            },
            SolarObject { // M2P3
                type_id: SolarType::MOON,
                pos: pos[8],
                y_angle: 0.0,
                parent: 3,
                size: 0.3,
                tex: moon_tex,
                to_parent_x_form: Default::default(),
                to_world_x_form: Default::default()
            },
            SolarObject { // M3P3
                type_id: SolarType::MOON,
                pos: pos[9],
                y_angle: 0.0,
                parent: 3,
                size: 0.3,
                tex: moon_tex,
                to_parent_x_form: Default::default(),
                to_world_x_form: Default::default()
            },
        ];

        //==================================================

        if let Some(gfx_stats) = &mut gfx_stats {
            gfx_stats.add_vertices(ID3DXBaseMesh_GetNumVertices(sphere));
            gfx_stats.add_triangles(ID3DXBaseMesh_GetNumFaces(sphere));
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
            SolarSystemDemo::build_fx(d3d_device.clone());

        let mut solar_system_demo = SolarSystemDemo {
            d3d_pp,
            gfx_stats,

            sphere,

            object,
            sun_tex,
            planet1_tex,
            planet2_tex,
            planet3_tex,
            moon_tex,

            white_mtrl,

            light,

            fx,

            h_tech,
            h_wvp,
            h_world_inverse_transpose,
            h_mtrl,
            h_light,
            h_eye_pos,
            h_world,
            h_tex,

            camera_radius: 25.0,
            camera_rotation_y: 1.2 * D3DX_PI,
            camera_height: 10.0,

            world,
            view: unsafe { std::mem::zeroed() },
            proj: unsafe { std::mem::zeroed() },
        };

        solar_system_demo.on_reset_device();

        Some(solar_system_demo)
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        ReleaseCOM(self.fx);
        ReleaseCOM(self.sphere);

        ReleaseCOM(self.sun_tex.cast());
        ReleaseCOM(self.planet1_tex.cast());
        ReleaseCOM(self.planet2_tex.cast());
        ReleaseCOM(self.planet3_tex.cast());
        ReleaseCOM(self.moon_tex.cast());

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

                //================================================
                // Animate the solar objects with respect to time.

                for i in 0..NUM_OBJECTS {
                    match &self.object[i].type_id {
                        SolarType::SUN => {
                            self.object[i].y_angle += 1.5 * dt;
                        },
                        SolarType::PLANET => {
                            self.object[i].y_angle += 2.0 * dt;
                        },
                        SolarType::MOON => {
                            self.object[i].y_angle += 2.5 * dt;
                        }
                    }

                    // If we rotate over 360 degrees, just roll back to 0.
                    if self.object[i].y_angle >= 2.0 * D3DX_PI {
                        self.object[i].y_angle = 0.0;
                    }
                }
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
                    0xFF000000,
                    1.0,
                    0));

                HR!(d3d_device.BeginScene());

                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_light, &self.light as *const _ as _, std::mem::size_of::<DirLight>() as u32));

                HR!(ID3DXEffect_SetTechnique(self.fx, self.h_tech));

                let mut num_passes: u32 = 0;
                HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));

                HR!(ID3DXEffect_BeginPass(self.fx, 0));

                // Wrap the texture coordinates that get assigned to TEXCOORD2 in the pixel shader.
                HR!(d3d_device.SetRenderState(D3DRS_WRAP2, D3DWRAP_U as u32));

                // Build the world transforms for each frame, then render them.
                self.build_object_world_transforms();

                let mut s: D3DXMATRIX = std::mem::zeroed();

                for i in 0..NUM_OBJECTS {
                    let scale = self.object[i].size;
                    D3DXMatrixScaling(&mut s, scale, scale, scale);

                    // Prefix the frame matrix with a scaling transformation to
                    // size it relative to the world.
                    self.world = std::mem::zeroed();
                    D3DXMatrixMultiply(&mut self.world, &s, &self.object[i].to_world_x_form);

                    let mut res: D3DXMATRIX = std::mem::zeroed();
                    D3DXMatrixMultiply(&mut res, &self.world, &self.view);
                    D3DXMatrixMultiply(&mut res, &res, &self.proj);
                    HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &res));

                    let mut world_inverse_transpose: D3DXMATRIX = std::mem::zeroed();
                    D3DXMatrixInverse(&mut world_inverse_transpose, 0.0, &self.world);
                    D3DXMatrixTranspose(&mut world_inverse_transpose, &world_inverse_transpose);
                    HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world_inverse_transpose, &world_inverse_transpose));

                    HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world, &self.world));

                    HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_mtrl, &self.white_mtrl as *const _ as _, std::mem::size_of::<Mtrl>() as u32));
                    HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.object[i].tex));

                    HR!(ID3DXEffect_CommitChanges(self.fx));
                    HR!(ID3DXBaseMesh_DrawSubset(self.sphere, 0));
                }

                HR!(d3d_device.SetRenderState(D3DRS_WRAP2, 0));

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

        HR!(D3DXCreateEffectFromFile(d3d_device,
            PSTR(c_resource_path(BASE_PATH, "PhongDirLtTex.fx").as_str().as_ptr() as _),
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

    fn build_object_world_transforms(&mut self) {
        unsafe {
            // First, construct the transformation matrix that transforms
            // the ith bone into the coordinate system of its parent.
            let mut r: D3DXMATRIX = std::mem::zeroed();
            let mut t: D3DXMATRIX = std::mem::zeroed();
            let mut p: D3DXVECTOR3;

            for i in 0..NUM_OBJECTS {
                p = self.object[i].pos;
                D3DXMatrixRotationY(&mut r, self.object[i].y_angle);
                D3DXMatrixTranslation(&mut t, p.x, p.y, p.z);

                D3DXMatrixMultiply(&mut self.object[i].to_parent_x_form, &r, &t);
            }

            // For each object...
            for i in 0..NUM_OBJECTS {
                // Initialize to identity matrix.
                D3DXMatrixIdentity(&mut self.object[i].to_world_x_form);

                // The ith object's world transform is given by its
                // to-parent transform, followed by its parent's to-parent transform,
                // followed by its grandparent's to-parent transform, and
                // so on, up to the root's to-parent transform.
                let mut k: i32 = i as i32;
                while k != -1 {
                    D3DXMatrixMultiply(&mut self.object[i].to_world_x_form,
                                       &self.object[i].to_world_x_form,
                                       &self.object[k as usize].to_parent_x_form);

                    k = self.object[k as usize].parent;
                }
            }
        }
    }
}