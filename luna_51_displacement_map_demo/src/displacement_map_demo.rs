use windows::{Win32::Foundation::*, Win32::Graphics::Direct3D9::*};
use common::mtrl::Mtrl;

use crate::*;
use crate::sky::Sky;

pub const BASE_PATH: &str = "luna_51_displacement_map_demo/";

// Colors
const WHITE: D3DXCOLOR = D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

// Spot Light
#[repr(C)]
#[derive(Clone)]
pub struct SpotLight {
    ambient: D3DXCOLOR,
    diffuse: D3DXCOLOR,
    spec: D3DXCOLOR,
    pos_w: D3DXVECTOR3,
    dir_w: D3DXVECTOR3,
    spot_power: f32,
}

// Sample demo
pub struct DisplacementMapDemo {
    d3d_pp: *const D3DPRESENT_PARAMETERS,
    gfx_stats: Option<GfxStats>,
    sky: Sky,
    water: WaterDMap,
}

impl DisplacementMapDemo {
    pub fn new(d3d_device: IDirect3DDevice9, d3d_pp: *const D3DPRESENT_PARAMETERS) -> Option<DisplacementMapDemo> {
        unsafe {
            if !DisplacementMapDemo::check_device_caps() {
                display_error_then_quit("checkDeviceCaps() Failed");
            }

            init_all_vertex_declarations(d3d_device.clone());

            let mut light_dir_w = D3DXVECTOR3 { x: 0.0, y: -1.0, z: -3.0 };
            D3DXVec3Normalize(&mut light_dir_w, &light_dir_w);

            let light = DirLight {
                dir_w: light_dir_w,
                ambient: D3DXCOLOR { r: 0.3, g: 0.3, b: 0.3, a: 1.0 },
                diffuse: D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
                spec: D3DXCOLOR { r: 0.7, g: 0.7, b: 0.7, a: 1.0 },
            };

            let mut gfx_stats = GfxStats::new(d3d_device.clone(), D3DCOLOR_XRGB!(0, 0, 0));

            let sky = Sky::new(BASE_PATH, d3d_device.clone(),
                               "grassenvmap1024.dds", 10000.0);

            let mut water_world = D3DXMATRIX::default();
            D3DXMatrixIdentity(&mut water_world);

            let water_mtrl = Mtrl {
                ambient: D3DXCOLOR { r: 0.4, g: 0.4, b: 0.7, a: 1.0 },
                diffuse: D3DXCOLOR { r: 0.4, g: 0.4, b: 0.7, a: 1.0 },
                spec: WHITE.mult(0.8),
                spec_power: 128.0
            };

            let water_init_info = WaterDMapInitInfo {
                dir_light: light,
                mtrl: water_mtrl,
                fx_filename: "waterdmap.fx".to_string(),
                vert_rows: 128,
                vert_cols: 128,
                dx: 0.25,
                dz: 0.25,
                wave_map_filename0: "wave0.dds".to_string(),
                wave_map_filename1: "wave1.dds".to_string(),
                dmap_filename0: "waterdmap0.dds".to_string(),
                dmap_filename1: "waterdmap1.dds".to_string(),
                wave_nmap_velocity0: D3DXVECTOR2 { x: 0.05, y: 0.07 },
                wave_nmap_velocity1: D3DXVECTOR2 { x: -0.01, y: 0.13 },
                wave_dmap_velocity0: D3DXVECTOR2 { x: 0.012, y: 0.015 },
                wave_dmap_velocity1: D3DXVECTOR2 { x: 0.014, y: 0.05 },
                scale_heights: D3DXVECTOR2 { x: 0.7, y: 1.1 },
                tex_scale: 8.0,
                to_world: water_world
            };

            let water = WaterDMap::new(BASE_PATH, water_init_info, d3d_device.clone());

            // Initialize camera.
            if let Some(camera) = &mut CAMERA {
                camera.set_pos(D3DXVECTOR3 { x: 0.0, y: 1.0, z: -15.0 });
                camera.set_speed(5.0);
            }

            if let Some(gfx_stats) = &mut gfx_stats {
                gfx_stats.add_vertices(water.get_num_vertices());
                gfx_stats.add_triangles(water.get_num_triangles());

                gfx_stats.add_vertices(sky.get_num_vertices());
                gfx_stats.add_triangles(sky.get_num_triangles());
            }

            let mut displacement_map_demo = DisplacementMapDemo {
                d3d_pp,
                gfx_stats,
                sky,
                water,
            };

            displacement_map_demo.on_reset_device();

            Some(displacement_map_demo)
        }
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        self.sky.release_com_objects();
        self.water.release_com_objects();

        destroy_all_vertex_declarations();
    }

    fn check_device_caps() -> bool {
        unsafe {
            if let Some(d3d_device) = &D3D_DEVICE {
                let mut caps: D3DCAPS9 = std::mem::zeroed();
                HR!(d3d_device.GetDeviceCaps(&mut caps));

                // Check for vertex shader version 3.0 support.
                if caps.VertexShaderVersion < D3DVS_VERSION!(3, 0) {
                    return false;
                }

                // Check for pixel shader version 3.0 support.
                if caps.PixelShaderVersion < D3DPS_VERSION!(3, 0) {
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

        self.sky.on_lost_device();
        self.water.on_lost_device();
    }

    pub fn on_reset_device(&mut self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.on_reset_device();
        }

        self.sky.on_reset_device();
        self.water.on_reset_device();

        // The aspect ratio depends on the backbuffer dimensions, which can
        // possibly change after a reset.  So rebuild the projection matrix.
        unsafe {
            let w: f32 = (*self.d3d_pp).BackBufferWidth as f32;
            let h: f32 = (*self.d3d_pp).BackBufferHeight as f32;

            if let Some(camera) = &mut CAMERA {
                camera.set_lens(D3DX_PI * 0.25, w / h, 1.0, 1000.0);
            }
        }
    }

    pub fn update_scene(&mut self, dt: f32) {
        unsafe {
            if let Some(gfx_stats) = &mut self.gfx_stats {
                gfx_stats.update(dt);
            }

            if let Some(dinput) = &mut DIRECT_INPUT {
                dinput.poll();
            }

            // Prevent camera from getting too close to water


            if let Some(camera) = &mut CAMERA {
                let mut camera_pos = camera.get_pos();
                if camera_pos.y < 2.0 {
                    camera_pos.y = 2.0;
                    camera.set_pos(camera_pos);
                }
                camera.update(dt, None, 0.0);
            }

            self.water.update(dt);
        }
    }

    pub fn draw_scene(&mut self) {
        unsafe {
            if let Some(d3d_device) = &D3D_DEVICE {
                HR!(d3d_device.BeginScene());

                self.sky.draw();
                self.water.draw();

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