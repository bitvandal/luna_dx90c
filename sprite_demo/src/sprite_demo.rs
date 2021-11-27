use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D9::*, Win32::UI::WindowsAndMessaging::*,
    Win32::System::SystemServices::*, Win32::Devices::HumanInterfaceDevice::*
};

use libc::*;
use crate::*;

// Sample demo
pub struct SpriteDemo {
    bullet_speed: f32,
    max_ship_speed: f32,
    ship_accel: f32,
    ship_drag: f32,
    sprite: *mut c_void,
    bkgd_tex: *mut c_void,
    bkgd_center: D3DXVECTOR3,
    ship_tex: *mut c_void,
    ship_center: D3DXVECTOR3,
    ship_pos: D3DXVECTOR3,
    ship_speed: f32,
    ship_rotation: f32,
    bullet_tex: *mut c_void,
    bullet_center: D3DXVECTOR3,
    gfx_stats: Option<GfxStats>,
    bullet_list: Vec<BulletInfo>
}

pub struct SpriteDemoOptions {
    pub bullet_speed: f32,
    pub max_ship_speed: f32,
    pub ship_accel: f32,
    pub ship_drag: f32,
}

struct BulletInfo {
    pos: D3DXVECTOR3,
    rotation: f32,
    life: f32,
}


impl SpriteDemo {
    pub fn new(d3d_device: IDirect3DDevice9, options: SpriteDemoOptions) -> Option<SpriteDemo> {
        if !SpriteDemo::check_device_caps() {
            display_error_then_quit("checkDeviceCaps() Failed");
        }

        let gfx_stats = GfxStats::new(d3d_device.clone());

        let mut sprite: *mut c_void = std::ptr::null_mut();
        HR!(D3DXCreateSprite(d3d_device.clone(), &mut sprite));

        let mut bkgd_tex: *mut c_void = std::ptr::null_mut();
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(), PSTR(b"bkgd1.bmp\0".as_ptr() as _), &mut bkgd_tex));

        let mut ship_tex: *mut c_void = std::ptr::null_mut();
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(), PSTR(b"alienship.bmp\0".as_ptr() as _), &mut ship_tex));

        let mut bullet_tex: *mut c_void = std::ptr::null_mut();
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(), PSTR(b"bullet.bmp\0".as_ptr() as _), &mut bullet_tex));

        let sprite_demo = SpriteDemo {
            bullet_speed: options.bullet_speed,
            max_ship_speed: options.max_ship_speed,
            ship_accel: options.ship_accel,
            ship_drag: options.ship_drag,
            sprite,
            bkgd_tex,
            bkgd_center: D3DXVECTOR3 { x: 256.0, y: 256.0, z: 0.0 },
            ship_tex,
            ship_center: D3DXVECTOR3 { x: 64.0, y: 64.0, z: 0.0 },
            ship_pos: D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 },
            ship_speed: 0.0,
            ship_rotation: 0.0,
            bullet_tex,
            bullet_center: D3DXVECTOR3 { x: 32.0, y: 32.0, z: 0.0 },
            gfx_stats,
            bullet_list: Vec::new(),
        };

        sprite_demo.on_reset_device();

        Some(sprite_demo)
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        ReleaseCOM(self.sprite);
        ReleaseCOM(self.bkgd_tex);
        ReleaseCOM(self.ship_tex);
        ReleaseCOM(self.bullet_tex);
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
                let pos = D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1000.0 };
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

                // The following code specifies an alpha test and reference value.
                HR!(d3d_device.SetRenderState(D3DRS_ALPHAREF, 10));
                HR!(d3d_device.SetRenderState(D3DRS_ALPHAFUNC, D3DCMP_GREATER.0 as u32));

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
        // Two triangles for each sprite--two for background,
        // two for ship, and two for each bullet.  Similarly,
        // 4 vertices for each sprite.
        if let Some(gfx_stats) = &mut self.gfx_stats {
            gfx_stats.set_tri_count(4 + self.bullet_list.len() as u32 * 2);
            gfx_stats.set_vertex_count(8 + self.bullet_list.len() as u32 * 4);
            gfx_stats.update(dt);
        }

        // Get snapshot of input devices
        unsafe {
            if let Some(dinput) = &mut DIRECT_INPUT {
                dinput.poll();
            }
        }

        // Update game objects.
        self.update_ship(dt);
        self.update_bullets(dt);
    }

    fn update_ship(&mut self, dt: f32) {
        // Check input.
        unsafe {
            if let Some(dinput) = &DIRECT_INPUT {
                if dinput.key_down(DIK_A as usize) {
                    self.ship_rotation += 4.0 * dt;
                }

                if dinput.key_down(DIK_D as usize) {
                    self.ship_rotation -= 4.0 * dt;
                }

                if dinput.key_down(DIK_W as usize) {
                    self.ship_speed += self.ship_accel * dt;
                }

                if dinput.key_down(DIK_S as usize) {
                    self.ship_speed -= self.ship_accel * dt;
                }
            }

            // Clamp top speed.
            if self.ship_speed > self.max_ship_speed { self.ship_speed = self.max_ship_speed }
            if self.ship_speed < -self.max_ship_speed { self.ship_speed = -self.max_ship_speed }

            // Rotate counterclockwise when looking down -z axis (i.e., rotate
            // clockwise when looking down the +z axis.
            let ship_dir: D3DXVECTOR3 = D3DXVECTOR3 {
                x: -self.ship_rotation.sin(),
                y: self.ship_rotation.cos(),
                z: 0.0
            };

            // Update position and speed based on time.
            self.ship_pos = *D3DXVec3Add(&mut std::mem::zeroed(),
                                         &self.ship_pos,
                                         D3DXVec3Scale(&mut std::mem::zeroed(), &ship_dir, self.ship_speed * dt));
            self.ship_speed -= self.ship_drag * self.ship_speed * dt;
        }
    }

    fn update_bullets(&mut self, dt: f32) {
        unsafe {
            // Make static so that its value persists across function calls.
            static mut FIRE_DELAY: f32 = 0.0;

            // Accumulate time.
            FIRE_DELAY += dt;

            if let Some(dinput) = &DIRECT_INPUT {
                // Did the user press the spacebar key and has 0.1 seconds passed?
                // We can only fire one bullet every 0.1 seconds.  If we do not
                // put this delay in, the ship will fire bullets way too fast.
                if dinput.key_down(DIK_SPACE as usize) && FIRE_DELAY > 0.1 {
                    let mut bullet: BulletInfo = std::mem::zeroed();

                    // Remember the ship is always drawn at the center of the window--
                    // the origin.  Therefore, bullets originate from the origin.
                    bullet.pos = D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 };

                    // The bullets rotation should match the ship's rotating at the
                    // instant it is fired.
                    bullet.rotation = self.ship_rotation;

                    // Bullet just born.
                    bullet.life = 0.0;

                    // Add the bullet to the list.
                    self.bullet_list.push(bullet);

                    // A bullet was just fired, so reset the fire delay.
                    FIRE_DELAY = 0.0;
                }

                // Now loop through each bullet, and update its position.
                for element in &mut self.bullet_list.iter_mut() {
                    // Accumulate the time the bullet has lived.
                    element.life += dt;

                    if element.life < 2.0 {
                        // Otherwise, update its position by moving along its directional
                        // path.  Code similar to how we move the ship--but no drag.
                        let dir = D3DXVECTOR3 { x: -element.rotation.sin(), y: element.rotation.cos(), z: 0.0 };
                        element.pos = *D3DXVec3Add(&mut std::mem::zeroed(),
                                                   &element.pos,
                                                   D3DXVec3Scale(&mut std::mem::zeroed(), &dir, self.bullet_speed * dt));
                    }
                }

                // If the bullet has lived for two seconds, kill it.  By now the
                // bullet should have flown off the screen and cannot be seen.
                self.bullet_list.retain(|element| element.life < 2.0);
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

                HR!(ID3DXSprite_Begin(self.sprite, D3DXSPRITE_OBJECTSPACE | D3DXSPRITE_DONOTMODIFY_RENDERSTATE));

                self.draw_bkgd();
                self.draw_ship();
                self.draw_bullets();

                if let Some(gfx_stats) = &self.gfx_stats {
                    gfx_stats.display();
                }

                HR!(ID3DXSprite_End(self.sprite));

                HR!(d3d_device.EndScene());

                HR!(d3d_device.Present(
                    std::ptr::null(),
                    std::ptr::null(),
                    HWND(0),
                    std::ptr::null()));
            }
        }
    }

    fn draw_bkgd(&self) {
        unsafe {
            if let Some(d3d_device) = &D3D_DEVICE {
                // Set a texture coordinate scaling transform.  Here we scale the texture
                // coordinates by 10 in each dimension.  This tiles the texture
                // ten times over the sprite surface.
                let mut tex_scaling: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixScaling(&mut tex_scaling, 10.0, 10.0, 0.0);
                HR!(d3d_device.SetTransform(D3DTS_TEXTURE0, &tex_scaling));

                // Position and size the background sprite--remember that
                // we always draw the ship in the center of the client area
                // rectangle. To give the illusion that the ship is moving,
                // we translate the background in the opposite direction.
                let mut t: D3DXMATRIX = std::mem::zeroed();
                let mut s: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixTranslation(&mut t, -self.ship_pos.x, -self.ship_pos.y, -self.ship_pos.z);
                D3DXMatrixScaling(&mut s, 20.0, 20.0, 0.0);
                let mut r: D3DXMATRIX = std::mem::zeroed();
                HR!(ID3DXSprite_SetTransform(self.sprite, D3DXMatrixMultiply(&mut r, &s, &t)));

                // Draw the background sprite.
                HR!(ID3DXSprite_Draw(self.sprite, self.bkgd_tex, std::ptr::null(), &self.bkgd_center,
                    std::ptr::null(), D3DCOLOR_XRGB!(255, 255, 255)));
                HR!(ID3DXSprite_Flush(self.sprite));

                // Restore defaults texture coordinate scaling transform.
                D3DXMatrixScaling(&mut tex_scaling, 1.0, -1.0, 0.0);
                HR!(d3d_device.SetTransform(D3DTS_TEXTURE0, &tex_scaling));
            }
        }
    }

    fn draw_ship(&self) {
        unsafe {
            if let Some(d3d_device) = &D3D_DEVICE {
                // Turn on the alpha test.
                HR!(d3d_device.SetRenderState(D3DRS_ALPHATESTENABLE, 1));

                // Set ships orientation.
                let mut r: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixRotationZ(&mut r, self.ship_rotation);
                HR!(ID3DXSprite_SetTransform(self.sprite, &r));

                // Draw the ship.
                HR!(ID3DXSprite_Draw(self.sprite, self.ship_tex, std::ptr::null(), &self.ship_center,
                        std::ptr::null(), D3DCOLOR_XRGB!(255, 255, 255)));
                HR!(ID3DXSprite_Flush(self.sprite));

                // Turn off the alpha test.
                HR!(d3d_device.SetRenderState(D3DRS_ALPHATESTENABLE, 0));
            }
        }
    }

    fn draw_bullets(&mut self) {
        unsafe {
            if let Some(d3d_device) = &D3D_DEVICE {
                // Turn on alpha blending.
                HR!(d3d_device.SetRenderState(D3DRS_ALPHABLENDENABLE, 1));

                // For each bullet...
                for element in &mut self.bullet_list.iter_mut() {
                    // Set its position and orientation.
                    let mut r: D3DXMATRIX = std::mem::zeroed();
                    let mut t: D3DXMATRIX = std::mem::zeroed();
                    D3DXMatrixRotationZ(&mut r, element.rotation);
                    D3DXMatrixTranslation(&mut t, element.pos.x, element.pos.y, element.pos.z);
                    HR!(ID3DXSprite_SetTransform(self.sprite, D3DXMatrixMultiply(&mut r, &r, &t)));

                    // Add it to the batch.
                    HR!(ID3DXSprite_Draw(self.sprite, self.bullet_tex, std::ptr::null(), &self.bullet_center,
                        std::ptr::null(), D3DCOLOR_XRGB!(255, 255, 255)));
                }

                // Draw all the bullets at once.
                HR!(ID3DXSprite_Flush(self.sprite));

                // Turn off alpha blending.
                HR!(d3d_device.SetRenderState(D3DRS_ALPHABLENDENABLE, 0));
            }
        }
    }
}
