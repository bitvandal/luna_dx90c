use libc::c_void;
use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D9::*, Win32::System::SystemServices::*,
};

use crate::*;

// Sample demo
pub struct DiffuseCubeDemo {
    d3d_pp: *const D3DPRESENT_PARAMETERS,
    gfx_stats: Option<GfxStats>,

    vb: IDirect3DVertexBuffer9,
    ib: IDirect3DIndexBuffer9,
    fx: LPD3DXEFFECT,

    h_tech: D3DXHANDLE,
    h_wvp: D3DXHANDLE,
    h_world_inverse_transpose: D3DXHANDLE,
    h_light_vec_w: D3DXHANDLE,
    h_diffuse_mtrl: D3DXHANDLE,
    h_diffuse_light: D3DXHANDLE,

    light_vec_w: D3DXVECTOR3,
    diffuse_mtrl: D3DXCOLOR,
    diffuse_light: D3DXCOLOR,

    camera_rotation_y: f32,
    camera_radius: f32,
    camera_height: f32,

    world: D3DXMATRIX,
    view: D3DXMATRIX,
    proj: D3DXMATRIX,
}

impl DiffuseCubeDemo {
    pub fn new(d3d_device: IDirect3DDevice9, d3d_pp: *const D3DPRESENT_PARAMETERS) -> Option<DiffuseCubeDemo> {
        if !DiffuseCubeDemo::check_device_caps() {
            display_error_then_quit("checkDeviceCaps() Failed");
        }

        let gfx_stats = GfxStats::new(d3d_device.clone());

        let vb = DiffuseCubeDemo::build_vertex_buffer(d3d_device.clone());
        let ib = DiffuseCubeDemo::build_index_buffer(d3d_device.clone());

        let (fx,
            h_tech,
            h_wvp,
            h_world_inverse_transpose,
            h_light_vec_w,
            h_diffuse_mtrl,
            h_diffuse_light) =
            DiffuseCubeDemo::build_fx(d3d_device.clone());

        let mut light_vec_w = D3DXVECTOR3 { x: -0.5, y: 0.75, z: -2.0 };
        D3DXVec3Normalize(&mut light_vec_w, &light_vec_w);

        let diffuse_mtrl = D3DXCOLOR { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
        let diffuse_light = D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

        let mut world: D3DXMATRIX = unsafe { std::mem::zeroed() };
        D3DXMatrixIdentity(&mut world);

        init_all_vertex_declarations(d3d_device.clone());

        let mut diffuse_cube_demo = DiffuseCubeDemo {
            d3d_pp,
            gfx_stats,

            vb,
            ib,
            fx,

            h_tech,
            h_wvp,
            h_world_inverse_transpose,
            h_light_vec_w,
            h_diffuse_mtrl,
            h_diffuse_light,

            light_vec_w,
            diffuse_mtrl,
            diffuse_light,

            camera_radius: 10.0,
            camera_rotation_y: 1.2 * D3DX_PI,
            camera_height: 5.0,

            world,
            view: unsafe { std::mem::zeroed() },
            proj: unsafe { std::mem::zeroed() },
        };

        diffuse_cube_demo.on_reset_device();

        Some(diffuse_cube_demo)
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        ReleaseCOM(self.fx);

        destroy_all_vertex_declarations();
    }

    fn check_device_caps() -> bool {
        unsafe {
            if let Some(d3d_device) = &D3D_DEVICE {
                let mut caps: D3DCAPS9 = std::mem::zeroed();
                HR!(d3d_device.GetDeviceCaps(&mut caps));

                // Check for vertex shader version 2.0 support.
                if caps.VertexShaderVersion < D3DVS_VERSION!(2, 0) {
                    return false
                }

                // Check for pixel shader version 2.0 support.
                if caps.PixelShaderVersion < D3DPS_VERSION!(2, 0) {
                    return false
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
            gfx_stats.set_vertex_count(24);
            gfx_stats.set_tri_count(12);
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
                if self.camera_radius < 5.0 {
                    self.camera_radius = 5.0;
                }
            }
        }

        // The camera position/orientation relative to world space can
        // change every frame based on input, so we need to rebuild the
        // view matrix every frame with the latest changes.
        self.build_view_mtx();
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

                // Let Direct3D know the vertex buffer, index buffer and vertex
                // declaration we are using.
                HR!(d3d_device.SetStreamSource(0, &self.vb, 0, std::mem::size_of::<VertexPN>() as u32));
                HR!(d3d_device.SetIndices(&self.ib));
                HR!(d3d_device.SetVertexDeclaration(&VERTEX_PN_DECL));

                // Setup the rendering FX
                HR!(ID3DXEffect_SetTechnique(self.fx, self.h_tech));

                let mut r: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixMultiply(&mut r, &self.world, &self.view);
                D3DXMatrixMultiply(&mut r, &r, &self.proj);
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &r));

                let mut world_inverse_transpose: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixInverse(&mut world_inverse_transpose, 0.0, &self.world);
                D3DXMatrixTranspose(&mut world_inverse_transpose, &world_inverse_transpose);
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world_inverse_transpose, &world_inverse_transpose));

                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_light_vec_w, &self.light_vec_w as *const _ as _, std::mem::size_of::<D3DXVECTOR3>() as u32));
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_diffuse_mtrl, &self.diffuse_mtrl as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_diffuse_light, &self.diffuse_light as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));

                // Begin passes.
                let mut num_passes: u32 = 0;
                HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));
                for i in 0..num_passes {
                    HR!(ID3DXEffect_BeginPass(self.fx, i));
                    HR!(d3d_device.DrawIndexedPrimitive(D3DPT_TRIANGLELIST, 0, 0, 8, 0, 12));
                    HR!(ID3DXEffect_EndPass(self.fx));
                }
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

    fn build_vertex_buffer(d3d_device: IDirect3DDevice9) -> IDirect3DVertexBuffer9 {
        unsafe {
            let mut vb: Option<IDirect3DVertexBuffer9> = None;

            // Obtain a pointer to a new vertex buffer.
            HR!(d3d_device.CreateVertexBuffer((24 * std::mem::size_of::<VertexPN>()) as u32,
                D3DUSAGE_WRITEONLY as u32, 0, D3DPOOL_MANAGED, &mut vb, std::ptr::null_mut()));

            let cube_vertices: [VertexPN; 24] = [
                // fill in the front face vertex data
                VertexPN { pos: D3DXVECTOR3 { x: -1.0, y: -1.0, z: -1.0 }, normal: D3DXVECTOR3 { x:  0.0, y:  0.0, z: -1.0 }},
                VertexPN { pos: D3DXVECTOR3 { x: -1.0, y:  1.0, z: -1.0 }, normal: D3DXVECTOR3 { x:  0.0, y:  0.0, z: -1.0 }},
                VertexPN { pos: D3DXVECTOR3 { x:  1.0, y:  1.0, z: -1.0 }, normal: D3DXVECTOR3 { x:  0.0, y:  0.0, z: -1.0 }},
                VertexPN { pos: D3DXVECTOR3 { x:  1.0, y: -1.0, z: -1.0 }, normal: D3DXVECTOR3 { x:  0.0, y:  0.0, z: -1.0 }},

                // fill in the back face vertex data
                VertexPN { pos: D3DXVECTOR3 { x: -1.0, y: -1.0, z:  1.0 }, normal: D3DXVECTOR3 { x:  0.0, y:  0.0, z:  1.0 }},
                VertexPN { pos: D3DXVECTOR3 { x:  1.0, y: -1.0, z:  1.0 }, normal: D3DXVECTOR3 { x:  0.0, y:  0.0, z:  1.0 }},
                VertexPN { pos: D3DXVECTOR3 { x:  1.0, y:  1.0, z:  1.0 }, normal: D3DXVECTOR3 { x:  0.0, y:  0.0, z:  1.0 }},
                VertexPN { pos: D3DXVECTOR3 { x: -1.0, y:  1.0, z:  1.0 }, normal: D3DXVECTOR3 { x:  0.0, y:  0.0, z:  1.0 }},

                // fill in the top face vertex data
                VertexPN { pos: D3DXVECTOR3 { x: -1.0, y:  1.0, z: -1.0 }, normal: D3DXVECTOR3 { x:  0.0, y:  1.0, z:  0.0 }},
                VertexPN { pos: D3DXVECTOR3 { x: -1.0, y:  1.0, z:  1.0 }, normal: D3DXVECTOR3 { x:  0.0, y:  1.0, z:  0.0 }},
                VertexPN { pos: D3DXVECTOR3 { x:  1.0, y:  1.0, z:  1.0 }, normal: D3DXVECTOR3 { x:  0.0, y:  1.0, z:  0.0 }},
                VertexPN { pos: D3DXVECTOR3 { x:  1.0, y:  1.0, z: -1.0 }, normal: D3DXVECTOR3 { x:  0.0, y:  1.0, z:  0.0 }},

                // fill in the bottom face vertex data
                VertexPN { pos: D3DXVECTOR3 { x: -1.0, y: -1.0, z: -1.0 }, normal: D3DXVECTOR3 { x:  0.0, y: -1.0, z:  0.0 }},
                VertexPN { pos: D3DXVECTOR3 { x:  1.0, y: -1.0, z: -1.0 }, normal: D3DXVECTOR3 { x:  0.0, y: -1.0, z:  0.0 }},
                VertexPN { pos: D3DXVECTOR3 { x:  1.0, y: -1.0, z:  1.0 }, normal: D3DXVECTOR3 { x:  0.0, y: -1.0, z:  0.0 }},
                VertexPN { pos: D3DXVECTOR3 { x: -1.0, y: -1.0, z:  1.0 }, normal: D3DXVECTOR3 { x:  0.0, y: -1.0, z:  0.0 }},

                // fill in the left face vertex data
                VertexPN { pos: D3DXVECTOR3 { x: -1.0, y: -1.0, z:  1.0 }, normal: D3DXVECTOR3 { x: -1.0, y:  0.0, z:  0.0 }},
                VertexPN { pos: D3DXVECTOR3 { x: -1.0, y:  1.0, z:  1.0 }, normal: D3DXVECTOR3 { x: -1.0, y:  0.0, z:  0.0 }},
                VertexPN { pos: D3DXVECTOR3 { x: -1.0, y:  1.0, z: -1.0 }, normal: D3DXVECTOR3 { x: -1.0, y:  0.0, z:  0.0 }},
                VertexPN { pos: D3DXVECTOR3 { x: -1.0, y: -1.0, z: -1.0 }, normal: D3DXVECTOR3 { x: -1.0, y:  0.0, z:  0.0 }},

                // fill in the right face vertex data
                VertexPN { pos: D3DXVECTOR3 { x:  1.0, y: -1.0, z: -1.0 }, normal: D3DXVECTOR3 { x:  1.0, y:  0.0, z:  0.0 }},
                VertexPN { pos: D3DXVECTOR3 { x:  1.0, y:  1.0, z: -1.0 }, normal: D3DXVECTOR3 { x:  1.0, y:  0.0, z:  0.0 }},
                VertexPN { pos: D3DXVECTOR3 { x:  1.0, y:  1.0, z:  1.0 }, normal: D3DXVECTOR3 { x:  1.0, y:  0.0, z:  0.0 }},
                VertexPN { pos: D3DXVECTOR3 { x:  1.0, y: -1.0, z:  1.0 }, normal: D3DXVECTOR3 { x:  1.0, y:  0.0, z:  0.0 }},
            ];

            // Now lock it to obtain a pointer to its internal data, and write the
            // cube's vertex data.  Note also that in this demo we don't use an index
            // buffer.
            if let Some(vb) = &mut vb {
                let mut v = std::ptr::null_mut();
                HR!(vb.Lock(0, 0, &mut v, 0));

                std::ptr::copy_nonoverlapping(cube_vertices.as_ptr(),
                                              v as *mut VertexPN,
                                              cube_vertices.len());
                HR!(vb.Unlock());
            }

            vb.unwrap()
        }
    }

    fn build_index_buffer(d3d_device: IDirect3DDevice9) -> IDirect3DIndexBuffer9 {
        unsafe {
            let mut ib: Option<IDirect3DIndexBuffer9> = None;

            // Obtain a pointer to a new index buffer.
            HR!(d3d_device.CreateIndexBuffer(36 * std::mem::size_of::<u16>() as u32,
            D3DUSAGE_WRITEONLY as u32, D3DFMT_INDEX16, D3DPOOL_MANAGED, &mut ib, std::ptr::null_mut()));

            let mut k: [u16; 36] = [0; 36];

            // Front face.
            k[0] = 0; k[1] = 1; k[2] = 2;
            k[3] = 0; k[4] = 2; k[5] = 3;

            // Back face.
            k[6] = 4; k[7]  = 5; k[8]  = 6;
            k[9] = 4; k[10] = 6; k[11] = 7;

            // Top face.
            k[12] = 8; k[13] = 9; k[14] = 10;
            k[15] = 8; k[16] = 10; k[17] = 11;

            // Bottom face.
            k[18] = 12; k[19] = 13; k[20] = 14;
            k[21] = 12; k[22] = 14; k[23] = 15;

            // Left face.
            k[24] = 16; k[25] = 17; k[26] = 18;
            k[27] = 16; k[28] = 18; k[29] = 19;

            // Right face.
            k[30] = 20; k[31] = 21; k[32] = 22;
            k[33] = 20; k[34] = 22; k[35] = 23;

            if let Some(ib) = &mut ib {
                // Now lock it to obtain a pointer to its internal data, and write the
                // cube's index data.
                let mut i = std::ptr::null_mut();
                HR!(ib.Lock(0, 0, &mut i, 0));
                std::ptr::copy_nonoverlapping(k.as_ptr(),
                                              i as *mut u16,
                                              k.len());
                HR!(ib.Unlock());
            }

            ib.unwrap()
        }
    }

    fn build_view_mtx(&mut self) {
        let x: f32 = self.camera_radius * self.camera_rotation_y.cos();
        let z: f32 = self.camera_radius * self.camera_rotation_y.sin();
        let pos = D3DXVECTOR3 { x, y: self.camera_height, z };
        let target = D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 };
        let up = D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 };
        D3DXMatrixLookAtLH(&mut self.view, &pos, &target, &up);
    }

    fn build_proj_mtx(&mut self) {
        let w: f32 = (unsafe { *self.d3d_pp }).BackBufferWidth as f32;
        let h: f32 = (unsafe { *self.d3d_pp }).BackBufferHeight as f32;
        D3DXMatrixPerspectiveFovLH(&mut self.proj, D3DX_PI * 0.25, w / h, 1.0, 5000.0);
    }

    fn build_fx(d3d_device: IDirect3DDevice9) -> (LPD3DXEFFECT, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE) {
        // Create the FX from a .fx file.
        let mut fx: LPD3DXEFFECT = std::ptr::null_mut();
        let mut errors: LPD3DXBUFFER = std::ptr::null_mut();

        HR!(D3DXCreateEffectFromFile(d3d_device, PSTR(b"luna_13_diffuse_cube_demo/diffuse.fx\0".as_ptr() as _),
        std::ptr::null(), std::ptr::null(), D3DXSHADER_DEBUG,
        std::ptr::null(), &mut fx, &mut errors));

        unsafe {
            if !errors.is_null() {
                let errors_ptr: *mut c_void = ID3DXBuffer_GetBufferPointer(errors);

                let c_str: &CStr = CStr::from_ptr(errors_ptr.cast());
                let str_slice: &str = c_str.to_str().unwrap_or("<unkonwn error>");
                message_box(str_slice);
                // the original sample code will also crash at this point
            }
        }

        // Obtain handles.
        let h_tech = ID3DXBaseEffect_GetTechniqueByName(fx, PSTR(b"DiffuseTech\0".as_ptr() as _));
        let h_wvp = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWVP\0".as_ptr() as _));
        let h_world_inverse_transpose = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWorldInverseTranspose\0".as_ptr() as _));
        let h_light_vec_w = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gLightVecW\0".as_ptr() as _));
        let h_diffuse_mtrl = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gDiffuseMtrl\0".as_ptr() as _));
        let h_diffuse_light = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gDiffuseLight\0".as_ptr() as _));

        (fx, h_tech, h_wvp, h_world_inverse_transpose, h_light_vec_w, h_diffuse_mtrl, h_diffuse_light)
    }
}