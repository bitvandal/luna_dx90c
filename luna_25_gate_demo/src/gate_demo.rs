use std::slice::from_raw_parts_mut;
use libc::c_void;
use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D9::*, Win32::System::SystemServices::*,
};
use common::mtrl::Mtrl;

use crate::*;

// Sample demo
pub struct GateDemo {
    d3d_pp: *const D3DPRESENT_PARAMETERS,
    gfx_stats: Option<GfxStats>,

    num_grid_vertices: u32,
    num_grid_triangles: u32,

    grid_vb: IDirect3DVertexBuffer9,
    grid_ib: IDirect3DIndexBuffer9,
    gate_vb: IDirect3DVertexBuffer9,
    gate_ib: IDirect3DIndexBuffer9,

    ground_tex: *mut c_void,      //IDirect3DTexture9,
    gate_tex: *mut c_void,      //IDirect3DTexture9,

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

    ground_mtrl: Mtrl,
    gate_mtrl: Mtrl,

    light_vec_w: D3DXVECTOR3,
    ambient_light: D3DXCOLOR,
    diffuse_light: D3DXCOLOR,
    specular_light: D3DXCOLOR,

    camera_rotation_y: f32,
    camera_radius: f32,
    camera_height: f32,

    ground_world: D3DXMATRIX,
    gate_world: D3DXMATRIX,
    view: D3DXMATRIX,
    proj: D3DXMATRIX,
}

impl GateDemo {
    pub fn new(d3d_device: IDirect3DDevice9, d3d_pp: *const D3DPRESENT_PARAMETERS) -> Option<GateDemo> {
        if !GateDemo::check_device_caps() {
            display_error_then_quit("checkDeviceCaps() Failed");
        }

        let mut gfx_stats = GfxStats::new(d3d_device.clone(), D3DCOLOR_XRGB!(0, 0, 0));

        let light_vec_w = D3DXVECTOR3 { x: 0.0, y: 0.707, z: -0.707 };
        let diffuse_light = D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        let ambient_light = D3DXCOLOR { r: 0.6, g: 0.6, b: 0.6, a: 1.0 };
        let specular_light = D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

        let ground_mtrl = Mtrl {
            ambient: D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            diffuse: D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            spec: D3DXCOLOR { r: 0.4, g: 0.4, b: 0.4, a: 1.0 },
            spec_power: 8.0
        };

        let gate_mtrl = Mtrl {
            ambient: D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            diffuse: D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            spec: D3DXCOLOR { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
            spec_power: 8.0
        };

        let mut ground_world: D3DXMATRIX = unsafe { std::mem::zeroed() };
        D3DXMatrixIdentity(&mut ground_world);

        let mut gate_world: D3DXMATRIX = unsafe { std::mem::zeroed() };
        D3DXMatrixIdentity(&mut gate_world);

        let mut ground_tex: *mut c_void = std::ptr::null_mut();
        let mut gate_tex: *mut c_void = std::ptr::null_mut();

        HR!(D3DXCreateTextureFromFile(d3d_device.clone(), PSTR(b"luna_25_gate_demo/ground0.dds\0".as_ptr() as _), &mut ground_tex));
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(), PSTR(b"luna_25_gate_demo/gatea.dds\0".as_ptr() as _), &mut gate_tex));

        let (grid_vb, grid_ib) = GateDemo::build_grid_geometry(d3d_device.clone());
        let (gate_vb, gate_ib) = GateDemo::build_gate_geometry(d3d_device.clone());

        let num_grid_vertices = 100 * 100;
        let num_grid_triangles = 99 * 99 * 2;

        if let Some(gfx_stats) = &mut gfx_stats {
            gfx_stats.add_vertices(num_grid_vertices);
            gfx_stats.add_triangles(num_grid_triangles);

            // Add gate quad vertices.
            gfx_stats.add_vertices(4);
            gfx_stats.add_triangles(2);
        }

        let (fx, h_tech, h_wvp, h_world_inverse_transpose, h_light_vec_w,
            h_diffuse_mtrl, h_diffuse_light, h_ambient_mtrl, h_ambient_light,
            h_specular_mtrl, h_specular_light, h_specular_power,
            h_eyepos, h_world, h_tex) =
            GateDemo::build_fx(d3d_device.clone());

        let mut gate_demo = GateDemo {
            d3d_pp,
            gfx_stats,

            num_grid_vertices,
            num_grid_triangles,

            grid_vb,
            grid_ib,
            gate_vb,
            gate_ib,

            ground_tex,
            gate_tex,

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

            ground_mtrl,
            gate_mtrl,

            light_vec_w,
            ambient_light,
            diffuse_light,
            specular_light,

            camera_radius: 6.0,
            camera_rotation_y: 1.2 * D3DX_PI,
            camera_height: 3.0,

            ground_world,
            gate_world,
            view: unsafe { std::mem::zeroed() },
            proj: unsafe { std::mem::zeroed() },
        };

        gate_demo.on_reset_device();

        init_all_vertex_declarations(d3d_device.clone());

        Some(gate_demo)
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        ReleaseCOM(self.ground_tex);
        ReleaseCOM(self.gate_tex);
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

                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_light_vec_w, &self.light_vec_w as *const _ as _, std::mem::size_of::<D3DXVECTOR3>() as u32));
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_diffuse_light, &self.diffuse_light as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_ambient_light, &self.ambient_light as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_specular_light, &self.specular_light as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));

                self.draw_ground(d3d_device.clone());
                self.draw_gate(d3d_device.clone());

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

                let w: f32 = 99.0;
                let d: f32 = 99.0;

                let num_grid_vertices = 100 * 100;
                let mut verts_pnt: Vec<VertexPNT> = Vec::with_capacity(num_grid_vertices);

                for i in 0..100 {
                    for j in 0..100 {
                        let index = i * 100 + j;
                        verts_pnt.insert(index, VertexPNT {
                            pos: verts[index],
                            normal: D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 },
                            tex0: D3DXVECTOR2 {
                                x: (verts[index].x + (0.5 * w)) / w,
                                y: (verts[index].z - (0.5 * d)) / -d
                            }
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

    fn build_gate_geometry(d3d_device: IDirect3DDevice9) -> (IDirect3DVertexBuffer9, IDirect3DIndexBuffer9) {
        unsafe {
            // Gate is just a rectangle aligned with the xy-plane.

            let mut vb: Option<IDirect3DVertexBuffer9> = None;

            // Obtain a pointer to a new vertex buffer.
            HR!(d3d_device.CreateVertexBuffer((4 * std::mem::size_of::<VertexPNT>()) as u32,
                    D3DUSAGE_WRITEONLY as u32, 0, D3DPOOL_MANAGED, &mut vb, std::ptr::null_mut()));

            if let Some(vb) = &mut vb {
                // Now lock it to obtain a pointer to its internal data, and write the
                // grid's vertex data.
                let mut vertices_ptr = std::ptr::null_mut();
                HR!(vb.Lock(0, 0, &mut vertices_ptr, 0));

                // Scale texture coordinates by 4 units in the v-direction for tiling.
                let vertices: &mut [VertexPNT] = from_raw_parts_mut(vertices_ptr as *mut VertexPNT, 4);

                vertices[0] = VertexPNT { pos: D3DXVECTOR3 { x: -20.0, y: 0.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 0.0, y: 1.0 } };
                vertices[1] = VertexPNT { pos: D3DXVECTOR3 { x: -20.0, y: 5.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 0.0, y: 0.0 } };
                vertices[2] = VertexPNT { pos: D3DXVECTOR3 { x:  20.0, y: 5.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 4.0, y: 0.0 } };
                vertices[3] = VertexPNT { pos: D3DXVECTOR3 { x:  20.0, y: 0.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 4.0, y: 1.0 } };

                HR!(vb.Unlock());
            }

            // Obtain a pointer to a new index buffer.

            let mut ib: Option<IDirect3DIndexBuffer9> = None;

            HR!(d3d_device.CreateIndexBuffer((6 * std::mem::size_of::<u16>()) as u32,
                D3DUSAGE_WRITEONLY as u32, D3DFMT_INDEX16, D3DPOOL_MANAGED, &mut ib, std::ptr::null_mut()));

            if let Some(ib) = &mut ib {
                // Now lock it to obtain a pointer to its internal data, and write the
                // grid's index data.
                let mut indices_ptr = std::ptr::null_mut();
                HR!(ib.Lock(0, 0, &mut indices_ptr, 0));

                let k: &mut [u16] = from_raw_parts_mut(indices_ptr as *mut u16, 6);

                k[0] = 0;  k[1] = 1;  k[2] = 2; // Triangle 0
                k[3] = 0;  k[4] = 2;  k[5] = 3; // Triangle 1

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

        HR!(D3DXCreateEffectFromFile(d3d_device, PSTR(b"luna_25_gate_demo/dirLightTex.fx\0".as_ptr() as _),
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

    fn draw_ground(&self, d3d_device: IDirect3DDevice9) {
        unsafe {
            HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_ambient_mtrl, &self.ground_mtrl.ambient as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
            HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_diffuse_mtrl, &self.ground_mtrl.diffuse as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
            HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_specular_mtrl, &self.ground_mtrl.spec as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
            HR!(ID3DXBaseEffect_SetFloat(self.fx, self.h_specular_power, self.ground_mtrl.spec_power));

            let mut res: D3DXMATRIX = std::mem::zeroed();
            D3DXMatrixMultiply(&mut res, &self.ground_world, &self.view);
            D3DXMatrixMultiply(&mut res, &res, &self.proj);
            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &res));

            let mut world_inverse_transpose: D3DXMATRIX = std::mem::zeroed();
            D3DXMatrixInverse(&mut world_inverse_transpose, 0.0, &self.ground_world);
            D3DXMatrixTranspose(&mut world_inverse_transpose, &world_inverse_transpose);
            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world_inverse_transpose, &world_inverse_transpose));

            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world, &self.ground_world));
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
        }
    }

    fn draw_gate(&self, d3d_device: IDirect3DDevice9) {
        unsafe {
            // Enable alpha test.
            HR!(d3d_device.SetRenderState(D3DRS_ALPHATESTENABLE, 1));
            HR!(d3d_device.SetRenderState(D3DRS_ALPHAFUNC, D3DCMP_GREATEREQUAL.0 as u32));
            HR!(d3d_device.SetRenderState(D3DRS_ALPHAREF, 100));

            // Turn off backface culling so you can see both sides of the gate.
            HR!(d3d_device.SetRenderState(D3DRS_CULLMODE, D3DCULL_NONE.0));

            HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_ambient_mtrl, &self.gate_mtrl.ambient as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
            HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_diffuse_mtrl, &self.gate_mtrl.diffuse as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
            HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_specular_mtrl, &self.gate_mtrl.spec as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
            HR!(ID3DXBaseEffect_SetFloat(self.fx, self.h_specular_power, self.gate_mtrl.spec_power));

            let mut res: D3DXMATRIX = std::mem::zeroed();
            D3DXMatrixMultiply(&mut res, &self.gate_world, &self.view);
            D3DXMatrixMultiply(&mut res, &res, &self.proj);
            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &res));

            let mut world_inverse_transpose: D3DXMATRIX = std::mem::zeroed();
            D3DXMatrixInverse(&mut world_inverse_transpose, 0.0, &self.gate_world);
            D3DXMatrixTranspose(&mut world_inverse_transpose, &world_inverse_transpose);
            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world_inverse_transpose, &world_inverse_transpose));

            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world, &self.gate_world));
            HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.gate_tex));

            // Let Direct3D know the vertex buffer, index buffer and vertex
            // declaration we are using.
            HR!(d3d_device.SetVertexDeclaration(&VERTEX_PNT_DECL));
            HR!(d3d_device.SetStreamSource(0, &self.gate_vb, 0, std::mem::size_of::<VertexPNT>() as u32));
            HR!(d3d_device.SetIndices(&self.gate_ib));

            // Begin passes.
            let mut num_passes: u32 = 0;
            HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));
            for i in 0..num_passes {
                HR!(ID3DXEffect_BeginPass(self.fx, i));
                HR!(d3d_device.DrawIndexedPrimitive(D3DPT_TRIANGLELIST, 0, 0, 4, 0, 2));
                HR!(ID3DXEffect_EndPass(self.fx));
            }
            HR!(ID3DXEffect_End(self.fx));

            // Disable alpha test.
            HR!(d3d_device.SetRenderState(D3DRS_ALPHATESTENABLE, 0));

            // Turn culling back on.
            HR!(d3d_device.SetRenderState(D3DRS_CULLMODE, D3DCULL_CCW.0));
        }
    }
}