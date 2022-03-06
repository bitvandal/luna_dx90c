use libc::c_void;
use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D9::*, Win32::System::SystemServices::*,
};

use crate::*;

// Sample demo
pub struct TiledGroundDemo {
    d3d_pp: *const D3DPRESENT_PARAMETERS,
    gfx_stats: Option<GfxStats>,

    num_grid_vertices: u32,
    num_grid_triangles: u32,

    grid_vb: IDirect3DVertexBuffer9,
    grid_ib: IDirect3DIndexBuffer9,
    ground_tex: *mut c_void, //IDirect3DTexture9,

    fx: LPD3DXEFFECT,

    h_tech: D3DXHANDLE,
    h_wvp: D3DXHANDLE,
    h_world_inverse_transpose: D3DXHANDLE,
    h_light_vec_w: D3DXHANDLE,
    h_diffuse_mtrl: D3DXHANDLE,
    h_diffuse_light: D3DXHANDLE,
    h_ambient_mtrl: D3DXHANDLE,
    h_ambient_light: D3DXHANDLE,
    h_specular_mtrl: D3DXHANDLE,
    h_specular_light: D3DXHANDLE,
    h_specular_power: D3DXHANDLE,
    h_eyepos: D3DXHANDLE,
    h_world: D3DXHANDLE,
    h_tex: D3DXHANDLE,

    light_vec_w: D3DXVECTOR3,
    ambient_mtrl: D3DXCOLOR,
    ambient_light: D3DXCOLOR,
    diffuse_mtrl: D3DXCOLOR,
    diffuse_light: D3DXCOLOR,
    specular_mtrl: D3DXCOLOR,
    specular_light: D3DXCOLOR,
    specular_power: f32,

    camera_rotation_y: f32,
    camera_radius: f32,
    camera_height: f32,

    world: D3DXMATRIX,
    view: D3DXMATRIX,
    proj: D3DXMATRIX,
}

impl TiledGroundDemo {
    pub fn new(d3d_device: IDirect3DDevice9, d3d_pp: *const D3DPRESENT_PARAMETERS) -> Option<TiledGroundDemo> {
        if !TiledGroundDemo::check_device_caps() {
            display_error_then_quit("checkDeviceCaps() Failed");
        }

        let mut gfx_stats = GfxStats::new(d3d_device.clone(), D3DCOLOR_XRGB!(0, 0, 0));

        let (box_vb, box_ib) = TiledGroundDemo::build_grid_geometry(d3d_device.clone());

        let (fx, h_tech, h_wvp, h_world_inverse_transpose, h_light_vec_w,
            h_diffuse_mtrl, h_diffuse_light, h_ambient_mtrl, h_ambient_light,
            h_specular_mtrl, h_specular_light, h_specular_power,
            h_eyepos, h_world, h_tex) =
            TiledGroundDemo::build_fx(d3d_device.clone());

        // Save vertex count and triangle count for DrawIndexedPrimitive arguments.
        let num_grid_vertices = 100 * 100;
        let num_grid_triangles = 99 * 99 * 2;

        if let Some(gfx_stats) = &mut gfx_stats {
            gfx_stats.add_vertices(num_grid_vertices);
            gfx_stats.add_triangles(num_grid_triangles);
        }

        let light_vec_w = D3DXVECTOR3 { x: 0.0, y: 0.707, z: -0.707 };
        let diffuse_mtrl = D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        let diffuse_light = D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        let ambient_mtrl = D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        let ambient_light = D3DXCOLOR { r: 0.6, g: 0.6, b: 0.6, a: 1.0 };
        let specular_mtrl = D3DXCOLOR { r: 0.4, g: 0.4, b: 0.4, a: 1.0 };
        let specular_light = D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        let specular_power = 8.0;

        let mut world: D3DXMATRIX = unsafe { std::mem::zeroed() };
        D3DXMatrixIdentity(&mut world);

        init_all_vertex_declarations(d3d_device.clone());

        let mut ground_tex: *mut c_void = std::ptr::null_mut();
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(), PSTR(b"luna_19_tiled_ground_demo/ground0.dds\0".as_ptr() as _), &mut ground_tex));

        let mut tiled_ground_demo = TiledGroundDemo {
            d3d_pp,
            gfx_stats,

            num_grid_vertices,
            num_grid_triangles,

            grid_vb: box_vb,
            grid_ib: box_ib,
            ground_tex,

            fx,

            h_tech,
            h_wvp,
            h_world_inverse_transpose,
            h_light_vec_w,
            h_diffuse_mtrl,
            h_diffuse_light,
            h_ambient_mtrl,
            h_ambient_light,
            h_specular_mtrl,
            h_specular_light,
            h_specular_power,
            h_eyepos,
            h_world,
            h_tex,

            light_vec_w,
            ambient_mtrl,
            ambient_light,
            diffuse_mtrl,
            diffuse_light,
            specular_mtrl,
            specular_light,
            specular_power,

            camera_radius: 6.0,
            camera_rotation_y: 1.2 * D3DX_PI,
            camera_height: 3.0,

            world,
            view: unsafe { std::mem::zeroed() },
            proj: unsafe { std::mem::zeroed() },
        };

        tiled_ground_demo.on_reset_device();

        Some(tiled_ground_demo)
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        ReleaseCOM(self.fx);
        ReleaseCOM(self.ground_tex);

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
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_ambient_mtrl, &self.ambient_mtrl as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_ambient_light, &self.ambient_light as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_specular_mtrl, &self.specular_mtrl as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_specular_light, &self.specular_light as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
                HR!(ID3DXBaseEffect_SetFloat(self.fx, self.h_specular_power, self.specular_power));
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world, &self.world));
                HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.ground_tex));

                // Let Direct3D know the vertex buffer, index buffer and vertex
                // declaration we are using.
                HR!(d3d_device.SetVertexDeclaration(&VERTEX_PNT_DECL));
                HR!(d3d_device.SetStreamSource(0, &self.grid_vb, 0, std::mem::size_of::<VertexPNT>() as u32));
                HR!(d3d_device.SetIndices(&self.grid_ib));

                // Begin passes.
                let mut num_passes: u32 = 0;
                HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));
                for i in 0..num_passes {
                    HR!(ID3DXEffect_BeginPass(self.fx, i));
                    HR!(d3d_device.DrawIndexedPrimitive(D3DPT_TRIANGLELIST, 0, 0, self.num_grid_vertices, 0, self.num_grid_triangles));
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

    fn build_grid_geometry(d3d_device: IDirect3DDevice9) -> (IDirect3DVertexBuffer9, IDirect3DIndexBuffer9) {
        unsafe {
            let mut verts: Vec<D3DXVECTOR3> = Vec::new();
            let mut indices: Vec<u16> = Vec::new();

            gen_tri_grid(100, 100, 1.0, 1.0,
                         D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 }, &mut verts, &mut indices);

            let mut vb: Option<IDirect3DVertexBuffer9> = None;

            // Obtain a pointer to a new vertex buffer.
            HR!(d3d_device.CreateVertexBuffer((verts.len() * std::mem::size_of::<VertexPNT>()) as u32,
            D3DUSAGE_WRITEONLY as u32, 0, D3DPOOL_MANAGED, &mut vb, std::ptr::null_mut()));

            if let Some(vb) = &mut vb {
                // Now lock it to obtain a pointer to its internal data, and write the
                // grid's vertex data.
                let mut v = std::ptr::null_mut();
                HR!(vb.Lock(0, 0, &mut v, 0));

                let tex_scale: f32 = 0.2;

                let num_grid_vertices = 100 * 100;
                let mut verts_pnt: Vec<VertexPNT> = Vec::with_capacity(num_grid_vertices);
                for i in 0..100 {
                    for j in 0..100 {
                        let index = i * 100 + j;
                        verts_pnt.insert(index, VertexPNT {
                            pos: verts[index],
                            normal: D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 },
                            tex0: D3DXVECTOR2 { x: j as f32 * tex_scale, y: i as f32 * tex_scale }
                        });
                    }
                }

                std::ptr::copy_nonoverlapping(verts_pnt.as_ptr(),
                                              v as *mut VertexPNT,
                                              num_grid_vertices);

                HR!(vb.Unlock());
            }

            // Obtain a pointer to a new index buffer.
            let mut ib: Option<IDirect3DIndexBuffer9> = None;

            // Obtain a pointer to a new index buffer.
            HR!(d3d_device.CreateIndexBuffer((indices.len() * std::mem::size_of::<u16>()) as u32,
            D3DUSAGE_WRITEONLY as u32, D3DFMT_INDEX16, D3DPOOL_MANAGED, &mut ib, std::ptr::null_mut()));

            if let Some(ib) = &mut ib {
                // Now lock it to obtain a pointer to its internal data, and write the
                // grid's index data.
                let mut i = std::ptr::null_mut();
                HR!(ib.Lock(0, 0, &mut i, 0));
                std::ptr::copy_nonoverlapping(indices.as_ptr(),
                                              i as *mut u16,
                                              indices.len());
                HR!(ib.Unlock());
            }

            (vb.unwrap(), ib.unwrap())
        }
    }

    fn build_view_mtx(&mut self) {
        let x: f32 = self.camera_radius * self.camera_rotation_y.cos();
        let z: f32 = self.camera_radius * self.camera_rotation_y.sin();
        let pos = D3DXVECTOR3 { x, y: self.camera_height, z };
        let target = D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 };
        let up = D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 };
        D3DXMatrixLookAtLH(&mut self.view, &pos, &target, &up);

        HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_eyepos, &pos as *const _ as _, std::mem::size_of::<D3DXVECTOR3>() as u32));
    }

    fn build_proj_mtx(&mut self) {
        let w: f32 = (unsafe { *self.d3d_pp }).BackBufferWidth as f32;
        let h: f32 = (unsafe { *self.d3d_pp }).BackBufferHeight as f32;
        D3DXMatrixPerspectiveFovLH(&mut self.proj, D3DX_PI * 0.25, w / h, 1.0, 5000.0);
    }

    fn build_fx(d3d_device: IDirect3DDevice9) -> (LPD3DXEFFECT, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
                                                  D3DXHANDLE, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
                                                  D3DXHANDLE, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
                                                  D3DXHANDLE, D3DXHANDLE, D3DXHANDLE) {
        // Create the FX from a .fx file.
        let mut fx: LPD3DXEFFECT = std::ptr::null_mut();
        let mut errors: LPD3DXBUFFER = std::ptr::null_mut();

        HR!(D3DXCreateEffectFromFile(d3d_device, PSTR(b"luna_19_tiled_ground_demo/dirLightTex.fx\0".as_ptr() as _),
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
        let h_tech = ID3DXBaseEffect_GetTechniqueByName(fx, PSTR(b"DirLightTexTech\0".as_ptr() as _));
        let h_wvp = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWVP\0".as_ptr() as _));
        let h_world_inverse_transpose = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWorldInvTrans\0".as_ptr() as _));
        let h_light_vec_w = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gLightVecW\0".as_ptr() as _));
        let h_diffuse_mtrl = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gDiffuseMtrl\0".as_ptr() as _));
        let h_diffuse_light = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gDiffuseLight\0".as_ptr() as _));
        let h_ambient_mtrl = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gAmbientMtrl\0".as_ptr() as _));
        let h_ambient_light = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gAmbientLight\0".as_ptr() as _));
        let h_specular_mtrl = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gSpecularMtrl\0".as_ptr() as _));
        let h_specular_light = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gSpecularLight\0".as_ptr() as _));
        let h_specular_power = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gSpecularPower\0".as_ptr() as _));
        let h_eyepos = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gEyePosW\0".as_ptr() as _));
        let h_world = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWorld\0".as_ptr() as _));
        let h_tex = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gTex\0".as_ptr() as _));

        (fx, h_tech, h_wvp, h_world_inverse_transpose, h_light_vec_w,
         h_diffuse_mtrl, h_diffuse_light, h_ambient_mtrl, h_ambient_light,
         h_specular_mtrl, h_specular_light, h_specular_power,
         h_eyepos, h_world, h_tex)
    }
}