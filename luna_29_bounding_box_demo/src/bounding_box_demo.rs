use common::mtrl::Mtrl;
use libc::c_void;
use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D9::*, Win32::System::SystemServices::*,
};

use crate::*;

const BASE_PATH: &str = "luna_29_bounding_box_demo/";

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
struct AABB {
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
    pub fn center(&self) -> D3DXVECTOR3 {
        D3DXVECTOR3 {
            x: 0.5 * (self.min_pt.x + self.max_pt.x),
            y: 0.5 * (self.min_pt.y + self.max_pt.y),
            z: 0.5 * (self.min_pt.z + self.max_pt.z)
        }
    }
}

// Sample demo
pub struct BoundingBoxDemo {
    d3d_pp: *const D3DPRESENT_PARAMETERS,
    gfx_stats: Option<GfxStats>,

    mesh: LPD3DXMESH,
    mtrl: Vec<Mtrl>,
    tex: Vec<*mut c_void>,

    white_tex: *mut c_void,  //IDirect3DTexture9,

    box_mesh: LPD3DXMESH,
    box_mtrl: Mtrl,
    _bounding_box: AABB,
    bounding_box_offset: D3DXMATRIX,

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

impl BoundingBoxDemo {
    pub fn new(d3d_device: IDirect3DDevice9, d3d_pp: *const D3DPRESENT_PARAMETERS) -> Option<BoundingBoxDemo> {
        if !BoundingBoxDemo::check_device_caps() {
            display_error_then_quit("checkDeviceCaps() Failed");
        }

        init_all_vertex_declarations(d3d_device.clone());

        let mut gfx_stats = GfxStats::new(d3d_device.clone(), D3DCOLOR_XRGB!(0, 0, 0));

        let mut light_dir_w = D3DXVECTOR3 { x: 1.0, y: -1.0, z: -2.0 };
        D3DXVec3Normalize(&mut light_dir_w, &light_dir_w);

        let light = DirLight {
            ambient: D3DXCOLOR { r: 0.5, g: 0.5, b: 0.5, a: 1.0 },
            diffuse: D3DXCOLOR { r: 0.8, g: 0.8, b: 0.8, a: 1.0 },
            spec: D3DXCOLOR { r: 0.8, g: 0.8, b: 0.8, a: 1.0 },
            dir_w: light_dir_w,
        };

        let (mesh, mtrl, tex) =
            load_x_file(BASE_PATH, "bigship1.x", d3d_device.clone());
            // load_x_file(BASE_PATH, "car.x", d3d_device.clone());
            // load_x_file(BASE_PATH, "skullocc.x", d3d_device.clone());
            // load_x_file(BASE_PATH, "tiger.x", d3d_device.clone());

        let mut world = unsafe { std::mem::zeroed() };
        D3DXMatrixIdentity(&mut world);

        // Compute the bounding box.
        let mut v: *mut c_void = std::ptr::null_mut();
        HR!(ID3DXBaseMesh_LockVertexBuffer(mesh, 0, &mut v));

        let num_vertices: usize = ID3DXBaseMesh_GetNumVertices(mesh) as usize;
        let mut bounding_box: AABB = Default::default();

        HR!(D3DXComputeBoundingBox(v.cast(), num_vertices as u32, std::mem::size_of::<VertexPNT>() as u32,
            &mut bounding_box.min_pt, &mut bounding_box.max_pt));

        HR!(ID3DXBaseMesh_UnlockVertexBuffer(mesh));

        let width: f32  = bounding_box.max_pt.x - bounding_box.min_pt.x;
        let height: f32 = bounding_box.max_pt.y - bounding_box.min_pt.y;
        let depth: f32  = bounding_box.max_pt.z - bounding_box.min_pt.z;

        // Build a box mesh so that we can render the bounding box visually.
        let mut box_mesh: LPD3DXMESH = std::ptr::null_mut();
        HR!(D3DXCreateBox(d3d_device.clone(), width, height, depth, &mut box_mesh, std::ptr::null_mut()));

        // It is possible that the mesh was not centered about the origin
        // when it was modeled.  But the bounding box mesh is built around the
        // origin.  So offset the bounding box (mesh) center so that it
        // matches the true mathematical bounding box center.

        let center = bounding_box.center();
        let mut bounding_box_offset: D3DXMATRIX = unsafe { std::mem::zeroed() };

        D3DXMatrixTranslation(&mut bounding_box_offset, center.x, center.y, center.z);

        // Define the box material--make semi-transparent.
        let box_mtrl = Mtrl {
            ambient: D3DXCOLOR { r: 0.0, g: 0.0, b: 1.0, a: 1.0 },
            diffuse: D3DXCOLOR { r: 0.0, g: 0.0, b: 1.0, a: 0.5 },
            spec:    D3DXCOLOR { r: 0.5, g: 0.5, b: 0.5, a: 1.0 },
            spec_power: 8.0
        };

        // Create the white dummy texture.
        let mut white_tex = unsafe { std::mem::zeroed() };
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
            PSTR(c_resource_path(BASE_PATH, "whitetex.dds").as_str().as_ptr() as _), &mut white_tex));

        if let Some(gfx_stats) = &mut gfx_stats {
            // Add main mesh geometry count.
            gfx_stats.add_vertices(ID3DXBaseMesh_GetNumVertices(mesh));
            gfx_stats.add_triangles(ID3DXBaseMesh_GetNumFaces(mesh));

            // Add bounding box geometry count.
            gfx_stats.add_vertices(ID3DXBaseMesh_GetNumVertices(box_mesh));
            gfx_stats.add_triangles(ID3DXBaseMesh_GetNumFaces(box_mesh));
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
            BoundingBoxDemo::build_fx(d3d_device.clone());

        let mut bounding_box_demo = BoundingBoxDemo {
            d3d_pp,
            gfx_stats,

            mesh,

            mtrl,
            tex,

            white_tex,

            box_mesh,
            box_mtrl,
            _bounding_box: bounding_box,
            bounding_box_offset,

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

            camera_radius: 30.0,
            camera_rotation_y: 1.2 * D3DX_PI,
            camera_height: 10.0,

            world,

            view: unsafe { std::mem::zeroed() },
            proj: unsafe { std::mem::zeroed() },
        };

        bounding_box_demo.on_reset_device();

        Some(bounding_box_demo)
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        ReleaseCOM(self.fx);
        ReleaseCOM(self.mesh);
        ReleaseCOM(self.box_mesh);

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
                if self.camera_radius < 3.0 {
                    self.camera_radius = 3.0;
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
                    0xFFEEEEEE,
                    1.0,
                    0));

                HR!(d3d_device.BeginScene());

                // Setup the rendering FX
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_light, &self.light as *const _ as _, std::mem::size_of::<DirLight>() as u32));

                let mut res: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixMultiply(&mut res, &self.world, &self.view);
                D3DXMatrixMultiply(&mut res, &res, &self.proj);
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &res));

                let mut world_inverse_transpose: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixInverse(&mut world_inverse_transpose, 0.0, &self.world);
                D3DXMatrixTranspose(&mut world_inverse_transpose, &world_inverse_transpose);
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world_inverse_transpose, &world_inverse_transpose));

                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world, &self.world));

                HR!(ID3DXEffect_SetTechnique(self.fx, self.h_tech));

                let mut num_passes: u32 = 0;
                HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));

                HR!(ID3DXEffect_BeginPass(self.fx, 0));

                // Draw the mesh.
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
                    HR!(ID3DXBaseMesh_DrawSubset(self.mesh, j as u32));
                }

                // Draw the bounding box with alpha blending.
                HR!(d3d_device.SetRenderState(D3DRS_ALPHABLENDENABLE, 1));
                HR!(d3d_device.SetRenderState(D3DRS_SRCBLEND, D3DBLEND_SRCALPHA.0));
                HR!(d3d_device.SetRenderState(D3DRS_DESTBLEND, D3DBLEND_INVSRCALPHA.0));

                res = std::mem::zeroed();
                D3DXMatrixMultiply(&mut res, &self.bounding_box_offset, &self.view);
                D3DXMatrixMultiply(&mut res, &res, &self.proj);
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &res));

                world_inverse_transpose = std::mem::zeroed();
                D3DXMatrixInverse(&mut world_inverse_transpose, 0.0, &self.bounding_box_offset);
                D3DXMatrixTranspose(&mut world_inverse_transpose, &world_inverse_transpose);
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world_inverse_transpose, &world_inverse_transpose));
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world, &self.bounding_box_offset));

                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_mtrl, &self.box_mtrl as *const _ as _, std::mem::size_of::<Mtrl>() as u32));
                HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.white_tex));

                HR!(ID3DXEffect_CommitChanges(self.fx));
                HR!(ID3DXBaseMesh_DrawSubset(self.box_mesh, 0));

                HR!(d3d_device.SetRenderState(D3DRS_ALPHABLENDENABLE, 0));

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
}