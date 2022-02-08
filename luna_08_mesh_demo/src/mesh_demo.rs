use libc::c_void;
use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D9::*, Win32::System::SystemServices::*,
};

use crate::*;

// Sample demo
pub struct MeshDemo {
    vb: IDirect3DVertexBuffer9,
    ib: IDirect3DIndexBuffer9,
    camera_rotation_y: f32,
    camera_radius: f32,
    camera_height: f32,
    view: D3DXMATRIX,
    proj: D3DXMATRIX,
    decl: IDirect3DVertexDeclaration9,
    num_grid_vertices: u32,
    num_grid_triangles: u32,
    fx: LPD3DXEFFECT,
    h_tech: D3DXHANDLE,
    h_wvp: D3DXHANDLE,
    cylinder: LPD3DXMESH,
    sphere: LPD3DXMESH,
    gfx_stats: Option<GfxStats>,
    d3d_pp: *const D3DPRESENT_PARAMETERS,
}

impl MeshDemo {
    pub fn new(d3d_device: IDirect3DDevice9, d3d_pp: *const D3DPRESENT_PARAMETERS) -> Option<MeshDemo> {
        if !MeshDemo::check_device_caps() {
            display_error_then_quit("checkDeviceCaps() Failed");
        }

        let mut gfx_stats = GfxStats::new(d3d_device.clone());

        let (vb, ib) = build_geo_buffers(d3d_device.clone());

        let num_grid_vertices = 100 * 100;
        let num_grid_triangles = 99 * 99 * 2;

        let (fx, h_tech, h_wvp) = build_fx(d3d_device.clone());

        let mut cylinder: LPD3DXMESH = std::ptr::null_mut();
        HR!(D3DXCreateCylinder(d3d_device.clone(), 1.0, 1.0, 6.0, 20, 20, &mut cylinder, std::ptr::null_mut()));

        let mut sphere: LPD3DXMESH = std::ptr::null_mut();
        HR!(D3DXCreateSphere(d3d_device.clone(), 1.0, 20, 20, &mut sphere, std::ptr::null_mut()));

        if let Some(gfx_stats) = &mut gfx_stats {
            // If you look at the drawCylinders and drawSpheres functions, you see
            // that we draw 14 cylinders and 14 spheres.
            let num_cyl_verts: u32 = ID3DXBaseMesh_GetNumVertices(cylinder) * 14;
            let num_sphere_verts: u32 = ID3DXBaseMesh_GetNumVertices(sphere) * 14;
            let num_cyl_tris: u32 = ID3DXBaseMesh_GetNumFaces(cylinder) * 14;
            let num_sphere_tris: u32 = ID3DXBaseMesh_GetNumFaces(sphere) * 14;

            gfx_stats.add_vertices(num_grid_vertices);
            gfx_stats.add_vertices(num_cyl_verts);
            gfx_stats.add_vertices(num_sphere_verts);
            gfx_stats.add_triangles(num_grid_triangles);
            gfx_stats.add_triangles(num_cyl_tris);
            gfx_stats.add_triangles(num_sphere_tris);
        }

        let mut mesh_demo = MeshDemo {
            vb,
            ib,
            camera_rotation_y: 1.2 * D3DX_PI,
            camera_radius: 10.0,
            camera_height: 5.0,
            view: unsafe { std::mem::zeroed() },
            proj: unsafe { std::mem::zeroed() },
            decl: init_all_vertex_declarations(d3d_device.clone()).unwrap(),
            num_grid_vertices,
            num_grid_triangles,
            fx,
            h_tech,
            h_wvp,
            cylinder,
            sphere,
            gfx_stats,
            d3d_pp,
        };

        mesh_demo.on_reset_device();

        Some(mesh_demo)
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        ReleaseCOM(self.fx);
        ReleaseCOM(self.cylinder);
        ReleaseCOM(self.sphere);

        destroy_all_vertex_declarations(&self.decl);
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

                // Divide to make mouse less sensitive.
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
                    0xFFFFFFFF,
                    1.0,
                    0));

                HR!(d3d_device.BeginScene());

                // Let Direct3D know the vertex buffer, index buffer and vertex
                // declaration we are using.
                HR!(d3d_device.SetStreamSource(0, &self.vb, 0, std::mem::size_of::<VertexPos>() as u32));
                HR!(d3d_device.SetIndices(&self.ib));
                HR!(d3d_device.SetVertexDeclaration(&self.decl));

                // Setup the rendering FX
                HR!(ID3DXEffect_SetTechnique(self.fx, self.h_tech));

                // Begin passes.
                let mut num_passes: u32 = 0;
                HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));
                for i in 0..num_passes {
                    HR!(ID3DXEffect_BeginPass(self.fx, i));

                    let mut viewproj: D3DXMATRIX = std::mem::zeroed();
                    D3DXMatrixMultiply(&mut viewproj, &self.view, &self.proj);
                    HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &viewproj));
                    HR!(ID3DXEffect_CommitChanges(self.fx));
                    HR!(d3d_device.DrawIndexedPrimitive(D3DPT_TRIANGLELIST, 0, 0, self.num_grid_vertices,
                        0, self.num_grid_triangles));

                    self.draw_cylinders();
                    self.draw_spheres();

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

    fn draw_cylinders(&self) {
        unsafe {
            let mut r: D3DXMATRIX = std::mem::zeroed();
            D3DXMatrixRotationX(&mut r, D3DX_PI * 0.5);

            let mut z: i32 = -30;
            while z <= 30 {
                let mut t: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixTranslation(&mut t, -10.0, 3.0, z as f32);

                let mut res: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixMultiply(&mut res, &r, &t);
                D3DXMatrixMultiply(&mut res, &res, &self.view);
                D3DXMatrixMultiply(&mut res, &res, &self.proj);
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &res));
                HR!(ID3DXEffect_CommitChanges(self.fx));
                HR!(ID3DXBaseMesh_DrawSubset(self.cylinder, 0));

                D3DXMatrixTranslation(&mut t, 10.0, 3.0, z as f32);
                D3DXMatrixMultiply(&mut res, &r, &t);
                D3DXMatrixMultiply(&mut res, &res, &self.view);
                D3DXMatrixMultiply(&mut res, &res, &self.proj);
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &res));
                HR!(ID3DXEffect_CommitChanges(self.fx));
                HR!(ID3DXBaseMesh_DrawSubset(self.cylinder, 0));

                z += 10;
            }
        }
    }

    fn draw_spheres(&self) {
        unsafe {
            let mut t: D3DXMATRIX = std::mem::zeroed();

            let mut z: i32 = -30;
            while z <= 30 {
                D3DXMatrixTranslation(&mut t, -10.0, 7.5, z as f32);
                let mut res: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixMultiply(&mut res, &t, &self.view);
                D3DXMatrixMultiply(&mut res, &res, &self.proj);
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &res));
                HR!(ID3DXEffect_CommitChanges(self.fx));
                HR!(ID3DXBaseMesh_DrawSubset(self.sphere, 0));

                D3DXMatrixTranslation(&mut t, 10.0, 7.5, z as f32);
                let mut res: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixMultiply(&mut res, &t, &self.view);
                D3DXMatrixMultiply(&mut res, &res, &self.proj);
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &res));
                HR!(ID3DXEffect_CommitChanges(self.fx));
                HR!(ID3DXBaseMesh_DrawSubset(self.sphere, 0));

                z += 10;
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
    }

    fn build_proj_mtx(&mut self) {
        let w: f32 = (unsafe { *self.d3d_pp }).BackBufferWidth as f32;
        let h: f32 = (unsafe { *self.d3d_pp }).BackBufferHeight as f32;
        D3DXMatrixPerspectiveFovLH(&mut self.proj, D3DX_PI * 0.25, w/h, 1.0, 5000.0);
    }
}

fn build_geo_buffers(d3d_device: IDirect3DDevice9) -> (IDirect3DVertexBuffer9, IDirect3DIndexBuffer9){
    unsafe {
        let mut verts: Vec<D3DXVECTOR3> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();

        gen_tri_grid(100, 100, 1.0, 1.0,
                     D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 }, &mut verts, &mut indices);

        let mut vb: Option<IDirect3DVertexBuffer9> = None;

        // Obtain a pointer to a new vertex buffer.
        HR!(d3d_device.CreateVertexBuffer((verts.len() * std::mem::size_of::<VertexPos>()) as u32,
            D3DUSAGE_WRITEONLY as u32, 0, D3DPOOL_MANAGED, &mut vb, std::ptr::null_mut()));

        if let Some(vb) = &mut vb {
            // Now lock it to obtain a pointer to its internal data, and write the
            // grid's vertex data.
            let mut v = std::ptr::null_mut();
            HR!(vb.Lock(0, 0, &mut v, 0));

            std::ptr::copy_nonoverlapping(verts.as_ptr(),
                                          v as *mut D3DXVECTOR3,
                                          verts.len());
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

fn build_fx(d3d_device: IDirect3DDevice9) -> (LPD3DXEFFECT, D3DXHANDLE, D3DXHANDLE) {
    // Create the FX from a .fx file.
    let mut fx: LPD3DXEFFECT = std::ptr::null_mut();
    let mut errors: LPD3DXBUFFER = std::ptr::null_mut();

    HR!(D3DXCreateEffectFromFile(d3d_device, PSTR(b"luna_08_mesh_demo/transform.fx\0".as_ptr() as _),
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
    let h_tech = ID3DXBaseEffect_GetTechniqueByName(fx, PSTR(b"TransformTech\0".as_ptr() as _));
    let h_wvp = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWVP\0".as_ptr() as _));

    (fx, h_tech, h_wvp)
}