use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D9::*, Win32::UI::WindowsAndMessaging::*,
    Win32::System::SystemServices::*,
};

use libc::*;
use crate::*;

// Sample demo
pub struct PageFlipDemo {
    sprite: *mut c_void,
    frames: *mut c_void,
    curr_frame: i32,
    sprite_center: D3DXVECTOR3,
    gfx_stats: Option<GfxStats>,
}

impl PageFlipDemo {
    pub fn new(d3d_device: IDirect3DDevice9) -> Option<PageFlipDemo> {
        if !PageFlipDemo::check_device_caps() {
            display_error_then_quit("checkDeviceCaps() Failed");
        }

        let gfx_stats = GfxStats::new(d3d_device.clone());

        let mut sprite: *mut c_void = std::ptr::null_mut();
        HR!(D3DXCreateSprite(d3d_device.clone(), &mut sprite));

        let mut frames: *mut c_void = std::ptr::null_mut();
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(), PSTR(b"fireatlas.bmp\0".as_ptr() as _), &mut frames));

        let sprite_demo = PageFlipDemo {
            sprite,
            frames,
            curr_frame: 0, 	// The current frame of animation to render.
            sprite_center: D3DXVECTOR3 { x: 32.0, y: 32.0, z: 0.0 },
            gfx_stats,
        };

        sprite_demo.on_reset_device();

        Some(sprite_demo)
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        ReleaseCOM(self.sprite);
        ReleaseCOM(self.frames);
    }

    fn check_device_caps() -> bool {
        // Nothing to check.
        true
    }

    pub fn on_lost_device(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.on_lost_device();
        }

        HR!(ID3DXSprite_OnLostDevice(self.sprite));
    }

    pub fn on_reset_device(&self) {
        // Call the onResetDevice of other objects.
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.on_reset_device();
        }
        HR!(ID3DXSprite_OnResetDevice(self.sprite));

        unsafe {
            if let Some(d3d_device) = &D3D_DEVICE {
                // Sets up the camera 1000 units back looking at the origin.
                let pos = D3DXVECTOR3 { x: 0.0, y: 0.0, z: -100.0 };
                let up = D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 };
                let target = D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 };

                let mut v: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixLookAtLH(&mut v, &pos, &target, &up);
                HR!(d3d_device.SetTransform(D3DTS_VIEW, &v));

                // The following code defines the volume of space the camera sees.
                let mut r = RECT::default();

                if let Some(d3d_app) = &D3D_APP {
                    GetClientRect(d3d_app.main_wnd, &mut r);
                }

                let width: f32 = r.right as f32;
                let height: f32 = r.bottom as f32;

                let mut p: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixPerspectiveFovLH(&mut p, D3DX_PI * 0.25, width / height, 1.0, 5000.0);
                HR!(d3d_device.SetTransform(D3DTS_PROJECTION, &p));

                // This code sets texture filters, which helps to smooth out distortions
                // when you scale a texture.
                HR!(d3d_device.SetSamplerState(0, D3DSAMP_MAGFILTER, D3DTEXF_LINEAR.0 as u32));
                HR!(d3d_device.SetSamplerState(0, D3DSAMP_MINFILTER, D3DTEXF_LINEAR.0 as u32));
                HR!(d3d_device.SetSamplerState(0, D3DSAMP_MIPFILTER, D3DTEXF_LINEAR.0 as u32));

                // This line of code disables Direct3D lighting.
                HR!(d3d_device.SetRenderState(D3DRS_LIGHTING, 0));

                // The following code is used to setup alpha blending.
                HR!(d3d_device.SetTextureStageState(0, D3DTSS_ALPHAARG1, D3DTA_TEXTURE));
                HR!(d3d_device.SetTextureStageState(0, D3DTSS_ALPHAOP, D3DTOP_SELECTARG1.0 as u32));
                HR!(d3d_device.SetRenderState(D3DRS_SRCBLEND, D3DBLEND_SRCALPHA.0));
                HR!(d3d_device.SetRenderState(D3DRS_DESTBLEND, D3DBLEND_INVSRCALPHA.0));

                // Indicates that we are using 2D texture coordinates.
                HR!(d3d_device.SetTextureStageState(0, D3DTSS_TEXTURETRANSFORMFLAGS, D3DTTFF_COUNT2.0 as u32));
            }
        }
    }

    pub fn update_scene(&mut self, dt: f32) {
        // Just Drawing 1 sprite at a time.
        if let Some(gfx_stats) = &mut self.gfx_stats {
            gfx_stats.set_tri_count(2);
            gfx_stats.set_vertex_count(4);
            gfx_stats.update(dt);
        }

        // Get snapshot of input devices
        unsafe {
            if let Some(dinput) = &mut DIRECT_INPUT {
                dinput.poll();
            }
        }

        unsafe {
            // Keep track of how much time has accumulated.
            static mut TIME_ACCUM: f32 = 0.0;
            TIME_ACCUM += dt;

            // Play animation at 30 frames per second.
            if TIME_ACCUM >= 1.0 / 30.0 {
                // After 1/30 seconds has passed, move on to the next frame.
                self.curr_frame += 1;
                TIME_ACCUM = 0.0;

                // This animation has have 30 frames indexed from 0, 1, ..., 29,
                // so start back at the beginning if we go over.
                if self.curr_frame > 29 {
                    self.curr_frame = 0;
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
                    0xFF000000,
                    1.0,
                    0));

                HR!(d3d_device.BeginScene());

                HR!(ID3DXSprite_Begin(self.sprite, D3DXSPRITE_OBJECTSPACE | D3DXSPRITE_DONOTMODIFY_RENDERSTATE));

                // Compute rectangle on texture atlas of the current frame
                // we want to use.
                let i: i32 = self.curr_frame / 6; // Row
                let j: i32 = self.curr_frame % 6; // Column
                let r: RECT = RECT { left: j * 64, top: i * 64, right: (j + 1) * 64, bottom: (i + 1) * 64 };

                // Turn on alpha blending.
                HR!(d3d_device.SetRenderState(D3DRS_ALPHABLENDENABLE, 1));

                // Don't move explosion--set identity matrix.
                let mut m: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixIdentity(&mut m);
                HR!(ID3DXSprite_SetTransform(self.sprite, &m));
                HR!(ID3DXSprite_Draw(self.sprite, self.frames, &r, &self.sprite_center, std::ptr::null(), D3DCOLOR_XRGB!(255, 255, 255)));
                HR!(ID3DXSprite_End(self.sprite));

                // Turn off alpha blending.
                HR!(d3d_device.SetRenderState(D3DRS_ALPHABLENDENABLE, 0));

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
}