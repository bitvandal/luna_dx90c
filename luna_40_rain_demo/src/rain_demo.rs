use rand::prelude::ThreadRng;
use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D9::*, Win32::System::SystemServices::*,
};

use crate::*;
use crate::rain_psystem::RainPSystem;
use crate::terrain::Terrain;

pub const BASE_PATH: &str = "luna_40_rain_demo/";

// Colors
pub const WHITE: D3DXCOLOR = D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

// Sample demo
pub struct RainDemo {
    d3d_pp: *const D3DPRESENT_PARAMETERS,
    gfx_stats: Option<GfxStats>,

    terrain: Terrain,
    psys: RainPSystem,
}

impl RainDemo {
    pub fn new(hwnd: HWND, d3d_device: IDirect3DDevice9, d3d_pp: *const D3DPRESENT_PARAMETERS, rng: ThreadRng) -> Option<RainDemo> {
        if !RainDemo::check_device_caps() {
            display_error_then_quit("checkDeviceCaps() Failed");
        }

        init_all_vertex_declarations(d3d_device.clone());

        let mut gfx_stats = GfxStats::new(d3d_device.clone(), D3DCOLOR_XRGB!(0, 0, 0));

        // World space units are meters.  So (256*10.0f)x(256*10.0f) is (2.56)^2 square
        // kilometers.
        let terrain =
            Terrain::new(d3d_device.clone(),
                         257,
                         257,
                         2.0,
                         2.0,
                         "heightmap1_257.raw",
                         "mud.dds",
                         "stone.dds",
                         "snow.dds",
                         "blend_hm1.dds",
                         BASE_PATH,
                         0.4,
                         0.0);

        let mut to_sun = D3DXVECTOR3 { x: 1.0, y: 1.0, z: 1.0 };
        D3DXVec3Normalize(&mut to_sun, &to_sun);
        terrain.set_dir_to_sun_w(to_sun);

        // Initialize camera.
        unsafe {
            if let Some(camera) = &mut CAMERA {
                camera.set_pos(D3DXVECTOR3 { x: 55.0, y: 50.0, z: 25.0 });
                camera.set_speed(40.0);
            }
        }

        // Initialize the particle system.
        let mut psys_world = D3DXMATRIX::default();
        D3DXMatrixIdentity(&mut psys_world);

        // Rain always visible, so make infinitely huge bounding box
        // so that it is always seen.
        let mut psys_box = AABB::default();
        psys_box.max_pt = D3DXVECTOR3 { x: f32::MAX, y: f32::MAX, z: f32::MAX };
        psys_box.min_pt = D3DXVECTOR3 { x: f32::MIN, y: f32::MIN, z: f32::MIN };

        // Accelerate due to wind and gravity.
        let mut psys =
            RainPSystem::new(BASE_PATH, "rain.fx", "RainTech", "raindrop.dds",
                             &D3DXVECTOR3 { x: -1.0, y: -9.8, z: 0.0 }, &psys_box,
                             4000, 0.001, hwnd, d3d_device.clone(), rng);
        psys.set_world_mtx(&psys_world);

        if let Some(gfx_stats) = &mut gfx_stats {
            gfx_stats.add_vertices(terrain.get_num_vertices());
            gfx_stats.add_triangles(terrain.get_num_triangles());
        }

        let mut fire_ring_demo = RainDemo {
            d3d_pp,
            gfx_stats,

            terrain,
            psys,
        };

        fire_ring_demo.on_reset_device();

        Some(fire_ring_demo)
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        self.terrain.release_com_objects();
        self.psys.release_com_objects();

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

        self.terrain.on_lost_device();
        self.psys.on_lost_device();
    }

    pub fn on_reset_device(&mut self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.on_reset_device();
        }

        self.terrain.on_reset_device();
        self.psys.on_reset_device();

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

            self.psys.update(dt);
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
                    0xFF666666,
                    1.0,
                    0));

                HR!(d3d_device.BeginScene());

                self.terrain.draw();
                self.psys.draw();

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
}