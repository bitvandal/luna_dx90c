use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D9::*, Win32::System::SystemServices::*,
};

use crate::*;
use crate::terrain::Terrain;

pub const BASE_PATH: &str = "luna_34_terrain_demo/";

// Sample demo
pub struct TerrainDemo {
    d3d_pp: *const D3DPRESENT_PARAMETERS,
    gfx_stats: Option<GfxStats>,

    terrain: Terrain,

    camera_rotation_y: f32,
    camera_radius: f32,
    camera_height: f32,

    view: D3DXMATRIX,
    proj: D3DXMATRIX,
}

impl TerrainDemo {
    pub fn new(d3d_device: IDirect3DDevice9, d3d_pp: *const D3DPRESENT_PARAMETERS) -> Option<TerrainDemo> {
        if !TerrainDemo::check_device_caps() {
            display_error_then_quit("checkDeviceCaps() Failed");
        }

        init_all_vertex_declarations(d3d_device.clone());

        let mut gfx_stats = GfxStats::new(d3d_device.clone(), D3DCOLOR_XRGB!(0, 0, 0));

        let terrain =
            Terrain::new(d3d_device.clone(),
                         257,
                         257,
                         0.5,
                         0.5,
                         "heightmap17_257.raw",
                         "grass.dds",
                         "dirt.dds",
                         "stone.dds",
                         "blend_hm17.dds",
                         BASE_PATH,
                         0.2,
                         0.0);

        let d = D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 };
        terrain.set_dir_to_sun_w(d);

        let mut world = unsafe { std::mem::zeroed() };
        D3DXMatrixIdentity(&mut world);

        if let Some(gfx_stats) = &mut gfx_stats {
            gfx_stats.add_vertices(terrain.get_num_vertices());
            gfx_stats.add_triangles(terrain.get_num_triangles());
        }

        let mut terrain_demo = TerrainDemo {
            d3d_pp,
            gfx_stats,

            terrain,

            camera_radius: 75.0,
            camera_rotation_y: 1.3 * D3DX_PI,
            camera_height: 35.0,

            view: unsafe { std::mem::zeroed() },
            proj: unsafe { std::mem::zeroed() },
        };

        terrain_demo.on_reset_device();

        Some(terrain_demo)
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        self.terrain.release_com_objects();

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

        self.terrain.on_lost_device();
    }

    pub fn on_reset_device(&mut self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.on_reset_device();
        }

        self.terrain.on_reset_device();

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

                self.terrain.draw(&self.view, &self.proj);

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
}