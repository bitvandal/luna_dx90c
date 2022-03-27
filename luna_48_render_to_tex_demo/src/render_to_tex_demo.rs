use std::ffi::CStr;
use libc::c_void;
use windows::{Win32::Foundation::*, Win32::Graphics::Direct3D9::*};
use windows::core::HSTRING;

use crate::*;
use crate::drawable_tex2d::DrawableTex2D;
use crate::sky::Sky;
use crate::terrain::Terrain;

pub const BASE_PATH: &str = "luna_48_render_to_tex_demo/";

// Sample demo
pub struct RenderToTexDemo {
    d3d_pp: *const D3DPRESENT_PARAMETERS,
    gfx_stats: Option<GfxStats>,

    terrain: Terrain,
    sky: Sky,

    // Two camera's for this demo.
    first_person_camera: Camera,
    birds_eye_camera: Camera,

    // The texture we draw into.
    radar_map: DrawableTex2D,

    radar_vb: Option<IDirect3DVertexBuffer9>,

    // General light/texture FX
    radar_fx: LPD3DXEFFECT,
    h_tex: D3DXHANDLE,
}

impl RenderToTexDemo {
    pub fn new(d3d_device: IDirect3DDevice9, d3d_object: &IDirect3D9, device_type: D3DDEVTYPE,
               d3d_pp: *const D3DPRESENT_PARAMETERS) -> Option<RenderToTexDemo> {
        unsafe {
            let (suitable, auto_gen_mips) = RenderToTexDemo::check_device_caps(d3d_object, device_type);
            if !suitable {
                display_error_then_quit("checkDeviceCaps() Failed");
            }

            let mut first_person_camera = Camera::new();
            let birds_eye_camera = Camera::new();

            init_all_vertex_declarations(d3d_device.clone());

            let mut gfx_stats = GfxStats::new(d3d_device.clone(), D3DCOLOR_XRGB!(0, 0, 0));

            let sky = Sky::new(BASE_PATH, d3d_device.clone(),
                               "grassenvmap1024.dds", 10000.0);

            // Viewport is entire texture.
            let vp = D3DVIEWPORT9 { X: 0, Y: 0, Width: 256, Height: 256, MinZ: 0.0, MaxZ: 1.0 };
            let radar_map = DrawableTex2D::new(256, 256, 0,
                                               D3DFMT_X8R8G8B8, true, D3DFMT_D24X8,
                                               &vp, auto_gen_mips);

            let mut radar_vb: Option<IDirect3DVertexBuffer9> = None;
            HR!(d3d_device.CreateVertexBuffer(6 * std::mem::size_of::<VertexPT>() as u32,
                D3DUSAGE_WRITEONLY as u32, 0, D3DPOOL_MANAGED, &mut radar_vb, std::ptr::null_mut()));

            // Radar quad takes up quadrant IV.  Note that we specify coordinate directly in
            // normalized device coordinates.  I.e., world, view, projection matrices are all
            // identity.
            if let Some(radar_vb) = &mut radar_vb {
                let mut verts_pt: Vec<VertexPT> = Vec::with_capacity(6);
                verts_pt.insert(0, VertexPT { pos: D3DXVECTOR3 { x: 0.0, y:  0.0, z: 0.0 }, tex0: D3DXVECTOR2 { x: 0.0, y: 0.0 } });
                verts_pt.insert(1, VertexPT { pos: D3DXVECTOR3 { x: 1.0, y:  0.0, z: 0.0 }, tex0: D3DXVECTOR2 { x: 1.0, y: 0.0 } });
                verts_pt.insert(2, VertexPT { pos: D3DXVECTOR3 { x: 0.0, y: -1.0, z: 0.0 }, tex0: D3DXVECTOR2 { x: 0.0, y: 1.0 } });
                verts_pt.insert(3, VertexPT { pos: D3DXVECTOR3 { x: 0.0, y: -1.0, z: 0.0 }, tex0: D3DXVECTOR2 { x: 0.0, y: 1.0 } });
                verts_pt.insert(4, VertexPT { pos: D3DXVECTOR3 { x: 1.0, y:  0.0, z: 0.0 }, tex0: D3DXVECTOR2 { x: 1.0, y: 0.0 } });
                verts_pt.insert(5, VertexPT { pos: D3DXVECTOR3 { x: 1.0, y: -1.0, z: 0.0 }, tex0: D3DXVECTOR2 { x: 1.0, y: 1.0 } });

                let mut v: *mut c_void = std::ptr::null_mut();
                HR!(radar_vb.Lock(0, 0, &mut v, 0));
                std::ptr::copy_nonoverlapping(verts_pt.as_ptr(), v as *mut VertexPT, 6);
                HR!(&radar_vb.Unlock());
            }

            let terrain =
                Terrain::new(d3d_device.clone(),
                             513, 513, 4.0, 4.0, "coastMountain513.raw",
                             "grass.dds", "dirt.dds", "rock.dds",
                             "blend_hm17.dds", BASE_PATH, 1.5, 0.0);

            let mut to_sun = D3DXVECTOR3 { x: 1.0, y: 1.0, z: 1.0 };
            D3DXVec3Normalize(&mut to_sun, &to_sun);
            terrain.set_dir_to_sun_w(to_sun);

            // Initialize camera.
            first_person_camera.set_pos(D3DXVECTOR3 { x: 0.0, y: 250.0, z: 0.0 });
            first_person_camera.set_speed(50.0);

            if let Some(gfx_stats) = &mut gfx_stats {
                gfx_stats.add_vertices(terrain.get_num_vertices());
                gfx_stats.add_triangles(terrain.get_num_triangles());
            }

            let (radar_fx,
                h_tech,
                h_tex)
                = RenderToTexDemo::build_fx(d3d_device.clone());

            HR!(ID3DXEffect_SetTechnique(radar_fx, h_tech));

            let mut render_to_tex_demo = RenderToTexDemo {
                d3d_pp,
                gfx_stats,
                terrain,
                sky,
                first_person_camera,
                birds_eye_camera,
                radar_map,
                radar_vb,
                radar_fx,
                h_tex
            };

            render_to_tex_demo.on_reset_device();

            Some(render_to_tex_demo)
        }
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        self.terrain.release_com_objects();
        self.sky.release_com_objects();
        self.radar_map.release_com_objects();

        ReleaseCOM(self.radar_fx);

        destroy_all_vertex_declarations();
    }

    fn check_device_caps(d3d_object: &IDirect3D9, device_type: D3DDEVTYPE) -> (bool, bool) {
        unsafe {
            let mut auto_gen_mips = true;

            if let Some(d3d_device) = &D3D_DEVICE {
                let mut caps: D3DCAPS9 = std::mem::zeroed();
                HR!(d3d_device.GetDeviceCaps(&mut caps));

                // Check for vertex shader version 2.0 support.
                if caps.VertexShaderVersion < D3DVS_VERSION!(2, 0) {
                    return (false, auto_gen_mips);
                }

                // Check for pixel shader version 2.0 support.
                if caps.PixelShaderVersion < D3DPS_VERSION!(2, 0) {
                    return (false, auto_gen_mips);
                }

                // Check render target support.  The adapter format can be either the display mode format
                // for windowed mode, or D3DFMT_X8R8G8B8 for fullscreen mode, so we need to test against
                // both.  We use D3DFMT_X8R8G8B8 as the render texture format and D3DFMT_D24X8 as the
                // render texture depth format.

                let mut mode = D3DDISPLAYMODE::default();
                HR!(d3d_object.GetAdapterDisplayMode(D3DADAPTER_DEFAULT, &mut mode));

                // Windowed.
                if FAILED!(d3d_object.CheckDeviceFormat(D3DADAPTER_DEFAULT, device_type, mode.Format,
                    D3DUSAGE_RENDERTARGET as u32, D3DRTYPE_TEXTURE, D3DFMT_X8R8G8B8)) {
                    return (false, auto_gen_mips);
                }

                if FAILED!(d3d_object.CheckDepthStencilMatch(D3DADAPTER_DEFAULT, device_type, mode.Format,
                    D3DFMT_X8R8G8B8, D3DFMT_D24X8)) {
                    return (false, auto_gen_mips);
                }

                // Fullscreen.
                if FAILED!(d3d_object.CheckDeviceFormat(D3DADAPTER_DEFAULT, device_type, D3DFMT_X8R8G8B8,
                    D3DUSAGE_RENDERTARGET as u32, D3DRTYPE_TEXTURE, D3DFMT_X8R8G8B8)) {
                    return (false, auto_gen_mips);
                }

                if FAILED!(d3d_object.CheckDepthStencilMatch(D3DADAPTER_DEFAULT, device_type, D3DFMT_X8R8G8B8,
                    D3DFMT_X8R8G8B8, D3DFMT_D24X8)) {
                    return (false, auto_gen_mips);
                }

                if caps.Caps2 & D3DCAPS2_CANAUTOGENMIPMAP as u32 != 0 {
                    let mut hr: windows::core::Result<()>;

                    // Windowed.
                    hr = d3d_object.CheckDeviceFormat(D3DADAPTER_DEFAULT,
                                                      D3DDEVTYPE_HAL,
                                                      mode.Format,
                                                      D3DUSAGE_AUTOGENMIPMAP as u32,
                                                      D3DRTYPE_TEXTURE,
                                                      D3DFMT_X8R8G8B8);

                    // Note: this likely don't work as the code is not an error, but not sure how to
                    //       do this, but cannot test it anyway.
                    if hr == windows::core::Result::Err(windows::core::Error::new(D3DOK_NOAUTOGEN, HSTRING::from("D3DOK_NOAUTOGEN"))) {
                        auto_gen_mips = false;
                    }

                    // Fullscreen.
                    hr = d3d_object.CheckDeviceFormat(D3DADAPTER_DEFAULT,
                                                      D3DDEVTYPE_HAL,
                                                      D3DFMT_X8R8G8B8,
                                                      D3DUSAGE_AUTOGENMIPMAP as u32,
                                                      D3DRTYPE_TEXTURE,
                                                      D3DFMT_X8R8G8B8);

                    // Note: this likely don't work, but not sure how to do this, and cannot test it anyway.
                    if hr == windows::core::Result::Err(windows::core::Error::new(D3DOK_NOAUTOGEN, HSTRING::from("D3DOK_NOAUTOGEN"))) {
                        auto_gen_mips = false;
                    }
                }
            }

            (true, auto_gen_mips)
        }
    }

    pub fn on_lost_device(&mut self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.on_lost_device();
        }

        self.terrain.on_lost_device();
        self.sky.on_lost_device();
        self.radar_map.on_lost_device();
        HR!(ID3DXEffect_OnLostDevice(self.radar_fx));
    }

    pub fn on_reset_device(&mut self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.on_reset_device();
        }

        self.terrain.on_reset_device();
        self.radar_map.on_reset_device();
        self.sky.on_reset_device();
        HR!(ID3DXEffect_OnResetDevice(self.radar_fx));

        // The aspect ratio depends on the backbuffer dimensions, which can
        // possibly change after a reset.  So rebuild the projection matrix.
        unsafe {
            let w: f32 = (*self.d3d_pp).BackBufferWidth as f32;
            let h: f32 = (*self.d3d_pp).BackBufferHeight as f32;

            self.first_person_camera.set_lens(D3DX_PI * 0.25, w / h, 1.0, 2000.0);
            self.birds_eye_camera.set_lens(D3DX_PI * 0.25, w / h, 1.0, 2000.0);
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

            if let Some(camera) = &mut CAMERA {
                // Hack to avoid reference management
                CAMERA = Some(self.first_person_camera.clone());

                camera.update(dt, Some(&self.terrain), 5.5);

                // Hack to avoid reference management
                self.first_person_camera = CAMERA.unwrap().clone();
            }
        }
    }

    pub fn draw_scene(&mut self) {
        unsafe {
            if let Some(d3d_device) = &D3D_DEVICE {
                CAMERA = Some(self.first_person_camera.clone());

                // Draw into radar map.
                let first_person_camera = CAMERA.unwrap().clone();
                let pos = D3DXVECTOR3 {
                    x: first_person_camera.get_pos().x,
                    y: first_person_camera.get_pos().y + 1000.0,
                    z: first_person_camera.get_pos().z
                };

                let up = D3DXVECTOR3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0
                };

                self.birds_eye_camera.look_at(&pos, &first_person_camera.get_pos(), &up);

                self.radar_map.begin_scene();

                HR!(d3d_device.Clear(
                    0,
                    std::ptr::null(),
                    (D3DCLEAR_TARGET | D3DCLEAR_ZBUFFER) as u32,
                    0xFF000000,
                    1.0,
                    0));

                CAMERA = Some(self.birds_eye_camera.clone());

                self.terrain.draw();

                self.radar_map.end_scene();

                CAMERA = Some(self.first_person_camera.clone());

                HR!(d3d_device.BeginScene());
                self.sky.draw();
                self.terrain.draw();

                HR!(d3d_device.SetStreamSource(0, &self.radar_vb, 0, std::mem::size_of::<VertexPT>() as u32));
                HR!(d3d_device.SetVertexDeclaration(&VERTEX_PT_DECL));
                HR!(ID3DXBaseEffect_SetTexture(self.radar_fx, self.h_tex, self.radar_map.d3d_tex()));

                let mut num_passes: u32 = 0;
                HR!(ID3DXEffect_Begin(self.radar_fx, &mut num_passes, 0));
                HR!(ID3DXEffect_BeginPass(self.radar_fx, 0));
                HR!(d3d_device.DrawPrimitive(D3DPT_TRIANGLELIST, 0, 2));
                HR!(ID3DXEffect_EndPass(self.radar_fx));
                HR!(ID3DXEffect_End(self.radar_fx));

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

    fn build_fx(d3d_device: IDirect3DDevice9) -> (LPD3DXEFFECT, D3DXHANDLE, D3DXHANDLE) {
        // Create the FX from a .fx file.
        let mut fx: LPD3DXEFFECT = std::ptr::null_mut();
        let mut errors: LPD3DXBUFFER = std::ptr::null_mut();

        HR!(D3DXCreateEffectFromFile(d3d_device,
            PSTR(c_resource_path(BASE_PATH, "Radar.fx").as_str().as_ptr() as _),
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
        let h_tech = ID3DXBaseEffect_GetTechniqueByName(fx, PSTR(b"RadarTech\0".as_ptr() as _));
        let h_tex = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gTex\0".as_ptr() as _));

        (fx, h_tech, h_tex)
    }
}