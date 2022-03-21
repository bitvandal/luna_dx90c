use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D9::*, Win32::System::SystemServices::*,
};

use crate::*;
use crate::terrain::Terrain;

pub const BASE_PATH: &str = "luna_35_camera_demo/";

// Sample demo
pub struct CameraDemo {
    d3d_pp: *const D3DPRESENT_PARAMETERS,
    gfx_stats: Option<GfxStats>,

    terrain: Terrain,
}

impl CameraDemo {
    pub fn new(d3d_device: IDirect3DDevice9, d3d_pp: *const D3DPRESENT_PARAMETERS) -> Option<CameraDemo> {
        if !CameraDemo::check_device_caps() {
            display_error_then_quit("checkDeviceCaps() Failed");
        }

        init_all_vertex_declarations(d3d_device.clone());

        let mut gfx_stats = GfxStats::new(d3d_device.clone(), D3DCOLOR_XRGB!(0, 0, 0));

        let terrain =
            Terrain::new(d3d_device.clone(),
                         257,
                         257,
                         10.0,
                         10.0,
                         "heightmap17_257.raw",
                         "grass.dds",
                         "dirt.dds",
                         "stone.dds",
                         "blend_hm17.dds",
                         BASE_PATH,
                         3.0,
                         0.0);

        let d = D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 };
        terrain.set_dir_to_sun_w(d);

        // Initialize camera.
        unsafe {
            if let Some(camera) = &mut CAMERA {
                camera.set_pos(D3DXVECTOR3 { x: -200.0, y: 300.0, z: 0.0 });
            }
        }

        if let Some(gfx_stats) = &mut gfx_stats {
            gfx_stats.add_vertices(terrain.get_num_vertices());
            gfx_stats.add_triangles(terrain.get_num_triangles());
        }

        let mut camera_demo = CameraDemo {
            d3d_pp,
            gfx_stats,

            terrain,
        };

        camera_demo.on_reset_device();

        Some(camera_demo)
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

        unsafe {
            let w: f32 = (*self.d3d_pp).BackBufferWidth as f32;
            let h: f32 = (*self.d3d_pp).BackBufferHeight as f32;

            if let Some(camera) = &mut CAMERA {
                camera.set_lens(D3DX_PI * 0.25, w / h, 1.0, 5000.0);
            }
        }
    }

    pub fn update_scene(&mut self, dt: f32) {
        if let Some(gfx_stats) = &mut self.gfx_stats {
            gfx_stats.update(dt);
        }

        // Get snapshot of input devices.
        unsafe {
            if let Some(dinput) = &mut DIRECT_INPUT {
                dinput.poll();

                if let Some(camera) = &mut CAMERA {
                    camera.update(dt);
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
                    0xFFEEEEEE,
                    1.0,
                    0));

                HR!(d3d_device.BeginScene());

                self.terrain.draw();

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