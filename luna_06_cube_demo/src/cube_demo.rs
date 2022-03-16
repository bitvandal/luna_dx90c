use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D9::*, Win32::System::SystemServices::*,
};

use common::*;
use crate::*;

// Sample demo
pub struct CubeDemo {
    vb: Option<IDirect3DVertexBuffer9>,
    ib: Option<IDirect3DIndexBuffer9>,
    camera_rotation_y: f32,
    camera_radius: f32,
    camera_height: f32,
    view: D3DXMATRIX,
    proj: D3DXMATRIX,
    gfx_stats: Option<GfxStats>,
    d3d_pp: *const D3DPRESENT_PARAMETERS,
}

impl CubeDemo {
    pub fn new(d3d_device: IDirect3DDevice9, d3d_pp: *const D3DPRESENT_PARAMETERS) -> Option<CubeDemo> {
        if !CubeDemo::check_device_caps() {
            display_error_then_quit("checkDeviceCaps() Failed");
        }

        let gfx_stats = GfxStats::new(d3d_device.clone(), D3DCOLOR_XRGB!(0, 0, 0));

        init_all_vertex_declarations(d3d_device.clone());

        let mut cube_demo = CubeDemo {
            vb: build_vertex_buffer(d3d_device.clone()),
            ib: build_index_buffer(d3d_device.clone()),
            camera_rotation_y: 1.2 * D3DX_PI,
            camera_radius: 10.0,
            camera_height: 5.0,
            view: unsafe { std::mem::zeroed() },
            proj: unsafe { std::mem::zeroed() },
            gfx_stats,
            d3d_pp,
        };

        cube_demo.on_reset_device();

        Some(cube_demo)
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        destroy_all_vertex_declarations();
    }

    fn check_device_caps() -> bool {
        // Nothing to check.
        true
    }

    pub fn on_lost_device(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.on_lost_device();
        }
    }

    pub fn on_reset_device(&mut self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.on_reset_device();
        }

        // The aspect ratio depends on the backbuffer dimensions, which can
        // possibly change after a reset.  So rebuild the projection matrix.
        self.build_proj_mtx();
    }

    pub fn update_scene(&mut self, dt: f32) {
        // One cube has 8 vertice and 12 triangles.
        if let Some(gfx_stats) = &mut self.gfx_stats {
            gfx_stats.set_vertex_count(8);
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
                    0xFFFFFFFF,
                    1.0,
                    0));

                HR!(d3d_device.BeginScene());

                // Let Direct3D know the vertex buffer, index buffer and vertex
                // declaration we are using.
                HR!(d3d_device.SetStreamSource(0, &self.vb, 0, std::mem::size_of::<VertexPos>() as u32));
                HR!(d3d_device.SetIndices(&self.ib));
                HR!(d3d_device.SetVertexDeclaration(&VERTEX_POS_DECL));

                // World matrix is identity.
                let mut w: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixIdentity(&mut w);
                HR!(d3d_device.SetTransform(D3DTS_WORLD, &w));
                HR!(d3d_device.SetTransform(D3DTS_VIEW, &self.view));
                HR!(d3d_device.SetTransform(D3DTS_PROJECTION, &self.proj));

                HR!(d3d_device.SetRenderState(D3DRS_FILLMODE, D3DFILL_WIREFRAME.0 as u32));
                HR!(d3d_device.DrawIndexedPrimitive(D3DPT_TRIANGLELIST, 0, 0, 8, 0, 12));

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
    }

    fn build_proj_mtx(&mut self) {
        let w: f32 = (unsafe { *self.d3d_pp }).BackBufferWidth as f32;
        let h: f32 = (unsafe { *self.d3d_pp }).BackBufferHeight as f32;
        D3DXMatrixPerspectiveFovLH(&mut self.proj, D3DX_PI * 0.25, w/h, 1.0, 5000.0);
        unsafe { dbg!(self.proj.Anonymous.m); }
    }
}

fn build_vertex_buffer(d3d_device: IDirect3DDevice9) -> Option<IDirect3DVertexBuffer9> {
    unsafe {
        let mut vb: Option<IDirect3DVertexBuffer9> = None;

        // Obtain a pointer to a new vertex buffer.
        HR!(d3d_device.CreateVertexBuffer(8 * std::mem::size_of::<VertexPos>() as u32,
            D3DUSAGE_WRITEONLY as u32, 0, D3DPOOL_MANAGED, &mut vb, std::ptr::null_mut()));

        let cube_vertices: [VertexPos; 8] = [
            VertexPos { pos: D3DXVECTOR3 { x: -1.0, y: -1.0, z: -1.0 }},
            VertexPos { pos: D3DXVECTOR3 { x: -1.0, y:  1.0, z: -1.0 }},
            VertexPos { pos: D3DXVECTOR3 { x:  1.0, y:  1.0, z: -1.0 }},
            VertexPos { pos: D3DXVECTOR3 { x:  1.0, y: -1.0, z: -1.0 }},
            VertexPos { pos: D3DXVECTOR3 { x: -1.0, y: -1.0, z:  1.0 }},
            VertexPos { pos: D3DXVECTOR3 { x: -1.0, y:  1.0, z:  1.0 }},
            VertexPos { pos: D3DXVECTOR3 { x:  1.0, y:  1.0, z:  1.0 }},
            VertexPos { pos: D3DXVECTOR3 { x:  1.0, y: -1.0, z:  1.0 }}
        ];

        if let Some(vb) = &mut vb {
            // Now lock it to obtain a pointer to its internal data, and write the
            // cube's vertex data.
            let mut v = std::ptr::null_mut();
            HR!(vb.Lock(0, 0, &mut v, 0));
            std::ptr::copy_nonoverlapping(cube_vertices.as_ptr(),
                                          v as *mut VertexPos,
                                          cube_vertices.len());
            HR!(vb.Unlock());
        }

        vb
    }
}

fn build_index_buffer(d3d_device: IDirect3DDevice9) -> Option<IDirect3DIndexBuffer9> {
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
        k[6] = 4; k[7]  = 6; k[8]  = 5;
        k[9] = 4; k[10] = 7; k[11] = 6;

        // Left face.
        k[12] = 4; k[13] = 5; k[14] = 1;
        k[15] = 4; k[16] = 1; k[17] = 0;

        // Right face.
        k[18] = 3; k[19] = 2; k[20] = 6;
        k[21] = 3; k[22] = 6; k[23] = 7;

        // Top face.
        k[24] = 1; k[25] = 5; k[26] = 6;
        k[27] = 1; k[28] = 6; k[29] = 2;

        // Bottom face.
        k[30] = 4; k[31] = 0; k[32] = 3;
        k[33] = 4; k[34] = 3; k[35] = 7;

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

        ib
    }
}