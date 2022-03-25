use std::ffi::CStr;
use libc::c_void;
use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D9::*, Win32::System::SystemServices::*,
};
use common::mtrl::Mtrl;

use crate::*;

pub const BASE_PATH: &str = "luna_43_tri_pick_demo/";

// Directional Light
#[repr(C)]
struct DirLight {
    ambient: D3DXCOLOR,
    diffuse: D3DXCOLOR,
    spec: D3DXCOLOR,
    dir_w: D3DXVECTOR3,
}

// Colors
pub const WHITE: D3DXCOLOR = D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

// Sample demo
pub struct TriPickDemo {
    d3d_pp: *const D3DPRESENT_PARAMETERS,
    gfx_stats: Option<GfxStats>,

    hwnd: HWND,

    // Car mesh
    mesh: LPD3DXMESH,
    mesh_mtrls: Vec<Mtrl>,
    mesh_textures: Vec<*mut c_void>,

    // General light/texture FX
    fx: LPD3DXEFFECT,
    h_tech: D3DXHANDLE,
    h_wvp: D3DXHANDLE,
    h_world_inv_trans: D3DXHANDLE,
    h_eye_pos: D3DXHANDLE,
    h_world: D3DXHANDLE,
    h_tex: D3DXHANDLE,
    h_mtrl: D3DXHANDLE,

    // Default texture if no texture present for subset.
    white_tex: *mut c_void, // IDirect3DTexture9
}

impl TriPickDemo {
    pub fn new(hwnd: HWND, d3d_device: IDirect3DDevice9,
               d3d_pp: *const D3DPRESENT_PARAMETERS) -> Option<TriPickDemo> {
        if !TriPickDemo::check_device_caps() {
            display_error_then_quit("checkDeviceCaps() Failed");
        }

        init_all_vertex_declarations(d3d_device.clone());

        let mut gfx_stats = GfxStats::new(d3d_device.clone(), D3DCOLOR_XRGB!(0, 0, 0));

        // Load the mesh data.
        let (mesh, mesh_mtrls, mesh_textures) =
            load_x_file(BASE_PATH, "car.x", d3d_device.clone());

        // Initialize camera.
        unsafe {
            if let Some(camera) = &mut CAMERA {
                camera.set_pos(D3DXVECTOR3 { x: -0.0, y: 2.0, z: -15.0 });
                camera.set_speed(40.0);
            }
        }

        // Load the default texture.
        let mut white_tex = unsafe { std::mem::zeroed() };
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
            PSTR(c_resource_path(BASE_PATH, "whitetex.dds").as_str().as_ptr() as _), &mut white_tex));

        // Init a light.
        let mut light_dir_w = D3DXVECTOR3 { x: 0.707, y: 0.0, z: 0.707 };
        D3DXVec3Normalize(&mut light_dir_w, &light_dir_w);

        let light = DirLight {
            ambient: D3DXCOLOR { r: 0.5, g: 0.5, b: 0.5, a: 1.0 },
            diffuse: D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            spec: D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            dir_w: light_dir_w,
        };

        let (fx,
            h_tech,
            h_wvp,
            h_world_inv_trans,
            h_eye_pos,
            h_world,
            h_tex,
            h_mtrl,
            h_light) =
            TriPickDemo::build_fx(d3d_device.clone());

        HR!(ID3DXBaseEffect_SetValue(fx, h_light, &light as *const _ as _, std::mem::size_of::<DirLight>() as u32));

        if let Some(gfx_stats) = &mut gfx_stats {
            gfx_stats.add_vertices(ID3DXBaseMesh_GetNumVertices(mesh));
            gfx_stats.add_triangles(ID3DXBaseMesh_GetNumFaces(mesh));
        }

        let mut tri_pick_demo = TriPickDemo {
            d3d_pp,
            gfx_stats,

            hwnd,

            mesh,
            mesh_mtrls,
            mesh_textures,

            fx,
            h_tech,
            h_wvp,
            h_world_inv_trans,
            h_eye_pos,
            h_world,
            h_tex,
            h_mtrl,

            white_tex
        };

        tri_pick_demo.on_reset_device();

        Some(tri_pick_demo)
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        ReleaseCOM(self.white_tex);
        ReleaseCOM(self.fx);

        ReleaseCOM(self.mesh);
        for tex in &self.mesh_textures {
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

    pub fn on_lost_device(&mut self) {
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
        unsafe {
            let w: f32 = (*self.d3d_pp).BackBufferWidth as f32;
            let h: f32 = (*self.d3d_pp).BackBufferHeight as f32;

            if let Some(camera) = &mut CAMERA {
                camera.set_lens(D3DX_PI * 0.25, w / h, 0.01, 5000.0);
            }
        }
    }

    pub fn update_scene(&mut self, dt: f32) {
        if let Some(gfx_stats) = &mut self.gfx_stats {
            gfx_stats.update(dt);
        }

        unsafe {
            if let Some(dinput) = &mut DIRECT_INPUT {
                dinput.poll();
            }

            if let Some(camera) = &mut CAMERA {
                camera.update(dt, None, 0.0);
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

                let camera: &Camera = &CAMERA.expect("Camera has not been created");

                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_eye_pos, &camera.get_pos() as *const _ as _, std::mem::size_of::<D3DXVECTOR3>() as u32));
                HR!(ID3DXEffect_SetTechnique(self.fx, self.h_tech));

                let mut num_passes: u32 = 0;
                HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));
                HR!(ID3DXEffect_BeginPass(self.fx, 0));

                // Specify mesh directly in world space.
                let mut to_world = D3DXMATRIX::default();
                D3DXMatrixIdentity(&mut to_world);

                // Set FX parameters.
                let mut viewproj: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixMultiply(&mut viewproj, &to_world, camera.get_view_proj());
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &viewproj));

                let mut world_inverse_transpose: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixInverse(&mut world_inverse_transpose, 0.0, &to_world);
                D3DXMatrixTranspose(&mut world_inverse_transpose, &world_inverse_transpose);
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world_inv_trans, &world_inverse_transpose));
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world, &to_world));

                // Draw the car in wireframe mode.
                HR!(d3d_device.SetRenderState(D3DRS_FILLMODE, D3DFILL_WIREFRAME.0 as u32));

                for j in 0..self.mesh_mtrls.len() {
                    HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_mtrl, &self.mesh_mtrls[j] as *const _ as _, std::mem::size_of::<Mtrl>() as u32));

                    // If there is a texture, then use.
                    if !self.mesh_textures[j].is_null() {
                        HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.mesh_textures[j]));
                    } else {
                        // But if not, then set a pure white texture.  When the texture color
                        // is multiplied by the color from lighting, it is like multiplying by
                        // 1 and won't change the color from lighting.

                        HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.white_tex));
                    }

                    HR!(ID3DXEffect_CommitChanges(self.fx));
                    HR!(ID3DXBaseMesh_DrawSubset(self.mesh, j as u32));
                }

                // Switch back to solid mode.
                HR!(d3d_device.SetRenderState(D3DRS_FILLMODE, D3DFILL_SOLID.0 as u32));

                // Did we pick anything?
                let mut origin_w = D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 };
                let mut dir_w = D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 };

                if let Some(dinput) = &mut DIRECT_INPUT {
                    if dinput.mouse_button_down(0) {

                        self.get_world_picking_ray(&mut origin_w, &mut dir_w);

                        let mut hit: i32 = 0;
                        let mut face_index: u32 = u32::MAX;
                        let mut u: f32 = 0.0;
                        let mut v: f32 = 0.0;
                        let mut dist: f32 = 0.0;

                        let mut all_hits: LPD3DXBUFFER = std::ptr::null_mut();
                        let mut num_hits: u32 = 0;
                        HR!(D3DXIntersect(self.mesh, &origin_w, &dir_w, &mut hit,
                            &mut face_index, &mut u, &mut v, &mut dist, &mut all_hits, &mut num_hits));
                        ReleaseCOM(all_hits);

                        // We hit anything?
                        if hit != 0 {
                            // Yes, draw the picked triangle in solid mode.
                            let mut vb: Option<IDirect3DVertexBuffer9> = None;
                            let mut ib: Option<IDirect3DIndexBuffer9> = None;
                            HR!(ID3DXBaseMesh_GetVertexBuffer(self.mesh, &mut vb as *mut _ as _));
                            HR!(ID3DXBaseMesh_GetIndexBuffer(self.mesh, &mut ib as *mut _ as _));
                            HR!(d3d_device.SetIndices(ib));
                            HR!(d3d_device.SetVertexDeclaration(&VERTEX_PNT_DECL));
                            HR!(d3d_device.SetStreamSource(0, vb, 0, std::mem::size_of::<VertexPNT>() as u32));

                            // faceIndex identifies the picked triangle to draw.
                            HR!(d3d_device.DrawIndexedPrimitive(D3DPT_TRIANGLELIST, 0, 0,
                                ID3DXBaseMesh_GetNumVertices(self.mesh), face_index * 3, 1));
                        }
                    }
                }

                HR!(ID3DXEffect_EndPass(self.fx));
                HR!(ID3DXEffect_End(self.fx));

                if let Some(gfx_stats) = &self.gfx_stats {
                    gfx_stats.display();
                }

                HR!(d3d_device.EndScene());

                // Present the backbuffer.
                HR!(d3d_device.Present(
                    std::ptr::null(),
                    std::ptr::null(),
                    HWND(0),
                    std::ptr::null()));
            }
        }
    }

    pub fn get_world_picking_ray(&self, origin_w: &mut D3DXVECTOR3, dir_w: &mut D3DXVECTOR3) {
        unsafe {
            let camera: &Camera = &CAMERA.expect("Camera has not been created");

            // Get the screen point clicked.
            let mut s: POINT = POINT::default();
            GetCursorPos(&mut s);

            // Make it relative to the client area window.
            ScreenToClient(self.hwnd, &mut s);

            // By the way we've been constructing things, the entire
            // backbuffer is the viewport.
            let w: f32 = (*self.d3d_pp).BackBufferWidth as f32;
            let h: f32 = (*self.d3d_pp).BackBufferHeight as f32;

            let proj: &D3DXMATRIX = camera.get_proj();
            let x: f32 = ( 2.0 * (s.x as f32) / w - 1.0) / (*proj).Anonymous.m[0]; //proj(0, 0)
            let y: f32 = (-2.0 * (s.y as f32) / h + 1.0) / (*proj).Anonymous.m[5]; //proj(1, 1)

            // Build picking ray in view space.
            let origin = D3DXVECTOR3 { x:0.0, y: 0.0, z: 0.0 };
            let dir = D3DXVECTOR3 { x, y, z: 1.0 };

            // So if the view matrix transforms coordinates from
            // world space to view space, then the inverse of the
            // view matrix transforms coordinates from view space
            // to world space.
            let mut inv_view = D3DXMATRIX::default();
            D3DXMatrixInverse(&mut inv_view, 0.0, camera.get_view());

            // Transform picking ray to world space.
            D3DXVec3TransformCoord(origin_w, &origin, &inv_view);
            D3DXVec3TransformNormal(dir_w, &dir, &inv_view);
            D3DXVec3Normalize(dir_w, dir_w);
        }
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
        let h_world_inv_trans = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWorldInvTrans\0".as_ptr() as _));
        let h_eye_pos = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gEyePosW\0".as_ptr() as _));
        let h_world = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWorld\0".as_ptr() as _));
        let h_tex = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gTex\0".as_ptr() as _));
        let h_mtrl = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gMtrl\0".as_ptr() as _));
        let h_light = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gLight\0".as_ptr() as _));

        (fx, h_tech, h_wvp, h_world_inv_trans, h_eye_pos, h_world, h_tex, h_mtrl, h_light)
    }
}