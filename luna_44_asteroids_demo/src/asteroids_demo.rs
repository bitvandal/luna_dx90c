use std::ffi::CStr;
use libc::c_void;
use rand::rngs::ThreadRng;
use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D9::*, Win32::System::SystemServices::*,
};
use common::mtrl::Mtrl;

use crate::*;
use crate::firework_psystem::FireWorkPSystem;

pub const BASE_PATH: &str = "luna_44_asteroids_demo/";

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

// In order to not duplicate firework systems, we create
// a firework "instance" structure, which stores the position of
// the instance in world space and its relative time since being
// created.  In this way, we can draw several fireworks with only
// one actual system by drawing the system in different world space
// positions and at different times.
pub struct FireWorkInstance  {
    time: f32,
    to_world: D3DXMATRIX,
}

// A simple asteroid structure to maintain the rotation, position,
// and velocity of an asteroid.
#[derive(Default)]
pub struct Asteroid {
    axis: D3DXVECTOR3,
    theta: f32,
    pos: D3DXVECTOR3,
    vel: D3DXVECTOR3,
}

const NUM_ASTEROIDS: i32 = 300;

// Sample demo
pub struct AsteroidsDemo {
    d3d_pp: *const D3DPRESENT_PARAMETERS,
    gfx_stats: Option<GfxStats>,

    hwnd: HWND,

    // We only need one firework system, as we just draw the same system
    // several times per frame in different positions and at different
    // relative times to simulate multiple systems.
    fire_work: FireWorkPSystem,

    // A list of firework *instances*
    fire_work_instances: Vec<FireWorkInstance>,

    // A list of asteroids.
    asteroids: Vec<Asteroid>,

    // We only need one actual mesh, as we just draw the same mesh several
    // times per frame in different positions to simulate multiple asteroids.
    asteroid_mesh: LPD3DXMESH,
    asteroid_mtrls: Vec<Mtrl>,
    asteroid_textures: Vec<*mut c_void>,
    asteroid_box: AABB,

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

impl AsteroidsDemo {
    pub fn new(hwnd: HWND, d3d_device: IDirect3DDevice9,
               d3d_pp: *const D3DPRESENT_PARAMETERS, mut rng: ThreadRng) -> Option<AsteroidsDemo> {
        unsafe {
            if !AsteroidsDemo::check_device_caps() {
                display_error_then_quit("checkDeviceCaps() Failed");
            }

            init_all_vertex_declarations(d3d_device.clone());

            let gfx_stats = GfxStats::new(d3d_device.clone(), D3DCOLOR_XRGB!(0, 0, 0));

            // Load the asteroid mesh and compute its bounding box in local space.
            let (asteroid_mesh, asteroid_mtrls, asteroid_textures) =
                load_x_file(BASE_PATH, "asteroid.x", d3d_device.clone());

            let mut v: *mut c_void = std::ptr::null_mut();
            HR!(ID3DXBaseMesh_LockVertexBuffer(asteroid_mesh, 0, &mut v));

            let mut asteroid_box = AABB::default();
            HR!(D3DXComputeBoundingBox(v.cast(), ID3DXBaseMesh_GetNumVertices(asteroid_mesh),
                    ID3DXBaseMesh_GetNumBytesPerVertex(asteroid_mesh),
                    &mut asteroid_box.min_pt, &mut asteroid_box.max_pt));

            HR!(ID3DXBaseMesh_UnlockVertexBuffer(asteroid_mesh));

            // Initialize camera.
            if let Some(camera) = &mut CAMERA {
                camera.set_pos(D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 });
                camera.set_speed(40.0);
            }

            let asteroids: Vec<Asteroid> = AsteroidsDemo::init_asteroids(&mut rng);

            // Initialize the particle system.
            let mut psys_world = D3DXMATRIX::default();
            D3DXMatrixIdentity(&mut psys_world);

            let mut psys_box = AABB::default();
            psys_box.max_pt = D3DXVECTOR3 { x: f32::MAX, y: f32::MAX, z: f32::MAX };
            psys_box.min_pt = D3DXVECTOR3 { x: f32::MIN, y: f32::MIN, z: f32::MIN };

            let mut psys =
                FireWorkPSystem::new(BASE_PATH, "fireworks.fx", "FireWorksTech", "bolt.dds",
                                     &D3DXVECTOR3 { x: 0.0, y: -9.8, z: 0.0 }, &psys_box,
                                     500, -1.0, hwnd, d3d_device.clone(), rng);
            psys.set_world_mtx(&psys_world);

            // Call update once to put all particles in the "alive" list.
            psys.update(0.0);

            // Load the default texture.
            let mut white_tex = std::mem::zeroed();
            HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
                PSTR(c_resource_path(BASE_PATH, "whitetex.dds").as_str().as_ptr() as _), &mut white_tex));

            // Init a light.
            let mut light_dir_w = D3DXVECTOR3 { x: 0.707, y: 0.0, z: 0.707 };
            D3DXVec3Normalize(&mut light_dir_w, &light_dir_w);

            let light = DirLight {
                ambient: D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
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
                AsteroidsDemo::build_fx(d3d_device.clone());

            HR!(ID3DXBaseEffect_SetValue(fx, h_light, &light as *const _ as _, std::mem::size_of::<DirLight>() as u32));

            let mut asteroids_demo = AsteroidsDemo {
                d3d_pp,
                gfx_stats,

                hwnd,

                fire_work: psys,

                // A list of firework *instances*
                fire_work_instances: Vec::new(),

                // A list of asteroids.
                asteroids,

                // We only need one actual mesh, as we just draw the same mesh several
                // times per frame in different positions to simulate multiple asteroids.
                asteroid_mesh,
                asteroid_mtrls,
                asteroid_textures,
                asteroid_box,

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

            asteroids_demo.on_reset_device();

            Some(asteroids_demo)
        }
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        ReleaseCOM(self.white_tex);
        ReleaseCOM(self.fx);

        self.fire_work.release_com_objects();

        ReleaseCOM(self.asteroid_mesh);
        for tex in &self.asteroid_textures {
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
        self.fire_work.on_lost_device();
    }

    pub fn on_reset_device(&mut self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.on_reset_device();
        }

        HR!(ID3DXEffect_OnResetDevice(self.fx));

        self.fire_work.on_reset_device();

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
            gfx_stats.set_tri_count(ID3DXBaseMesh_GetNumFaces(self.asteroid_mesh) * self.asteroids.len() as u32);
            gfx_stats.set_vertex_count(ID3DXBaseMesh_GetNumVertices(self.asteroid_mesh) * self.asteroids.len() as u32);
        }

        unsafe {
            if let Some(dinput) = &mut DIRECT_INPUT {
                dinput.poll();
            }

            if let Some(camera) = &mut CAMERA {
                camera.update(dt, None, 0.0);
            }

            // Update the asteroids' orientation and position.
            for a in &mut self.asteroids {
                a.theta += 4.0 * dt;

                let mut res = D3DXVECTOR3::default();
                D3DXVec3Scale(&mut res, &a.vel, dt);
                D3DXVec3Add(&mut a.pos, &a.pos, &res);
            }

            // Update and delete dead firework systems.
            let mut last_index: usize = 0;
            for index in 0..self.fire_work_instances.len() {
                let fire_work = &mut self.fire_work_instances[index];
                fire_work.time += dt;

                // Kill system after 1 seconds.
                if fire_work.time < 1.0 {
                    self.fire_work_instances.swap(last_index, index);
                    last_index += 1;
                }
            }

            self.fire_work_instances.truncate(last_index);
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
                    0xFF333333,
                    1.0,
                    0));

                HR!(d3d_device.BeginScene());

                let camera: &Camera = &CAMERA.expect("Camera has not been created");

                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_eye_pos, &camera.get_pos() as *const _ as _, std::mem::size_of::<D3DXVECTOR3>() as u32));
                HR!(ID3DXEffect_SetTechnique(self.fx, self.h_tech));

                let mut num_passes: u32 = 0;
                HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));
                HR!(ID3DXEffect_BeginPass(self.fx, 0));

                // Did we pick anything?
                let mut origin_w = D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 };
                let mut dir_w = D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 };

                let mut mouse_button_pressed = false;

                if let Some(dinput) = &mut DIRECT_INPUT {
                    if dinput.mouse_button_down(0) {
                        self.get_world_picking_ray(&mut origin_w, &mut dir_w);
                        mouse_button_pressed = true;
                    }
                }

                let mut last_index: usize = 0;
                for index in 0..self.asteroids.len() {
                    let asteroid = &self.asteroids[index];

                    // Build world matrix based on current rotation and position settings.
                    let mut r = D3DXMATRIX::default();
                    let mut t = D3DXMATRIX::default();
                    D3DXMatrixRotationAxis(&mut r, &asteroid.axis, asteroid.theta);
                    D3DXMatrixTranslation(&mut t, asteroid.pos.x, asteroid.pos.y, asteroid.pos.z);

                    let mut to_world = D3DXMATRIX::default();
                    D3DXMatrixMultiply(&mut to_world, &r, &t);

                    // Transform AABB to world space.
                    let mut bounding_box = AABB::default();
                    self.asteroid_box.xform(&to_world, &mut bounding_box);

                    // Did we pick it?
                    if mouse_button_pressed {
                        if D3DXBoxBoundProbe(&bounding_box.min_pt, &bounding_box.max_pt,
                                             &origin_w, &dir_w) != 0 {
                            // Create a firework instance.
                            let inst = FireWorkInstance {
                                time: 0.0,
                                to_world,
                            };

                            self.fire_work_instances.push(inst);
                            continue;
                        } else {
                            // Moves erasable items to the end of the list, so that we can truncate when finish iteration
                            self.asteroids.swap(last_index, index);
                            last_index += 1;
                        }
                    } else {
                        last_index += 1;
                    }

                    // Only draw if AABB is visible.
                    if camera.is_visible(&bounding_box) {
                        let mut wvp: D3DXMATRIX = std::mem::zeroed();
                        D3DXMatrixMultiply(&mut wvp, &to_world, camera.get_view_proj());
                        HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &wvp));

                        let mut world_inverse_transpose: D3DXMATRIX = std::mem::zeroed();
                        D3DXMatrixInverse(&mut world_inverse_transpose, 0.0, &to_world);
                        D3DXMatrixTranspose(&mut world_inverse_transpose, &world_inverse_transpose);
                        HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world_inv_trans, &world_inverse_transpose));
                        HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world, &to_world));

                        for j in 0..self.asteroid_mtrls.len() {
                            HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_mtrl, &self.asteroid_mtrls[j] as *const _ as _, std::mem::size_of::<Mtrl>() as u32));

                            // If there is a texture, then use.
                            if !self.asteroid_textures[j].is_null() {
                                HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.asteroid_textures[j]));
                            } else {
                                // But if not, then set a pure white texture.  When the texture color
                                // is multiplied by the color from lighting, it is like multiplying by
                                // 1 and won't change the color from lighting.

                                HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.white_tex));
                            }

                            HR!(ID3DXEffect_CommitChanges(self.fx));
                            HR!(ID3DXBaseMesh_DrawSubset(self.asteroid_mesh, j as u32));
                        }
                    }
                }

                self.asteroids.truncate(last_index);

                HR!(ID3DXEffect_EndPass(self.fx));
                HR!(ID3DXEffect_End(self.fx));

                // Draw fireworks.
                for inst in &self.fire_work_instances {
                    self.fire_work.set_time(inst.time);
                    self.fire_work.set_world_mtx(&inst.to_world);
                    self.fire_work.draw();
                }

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

    fn init_asteroids(mut rng: &mut ThreadRng) -> Vec<Asteroid> {
        let mut asteroids: Vec<Asteroid> = Vec::new();

        for _i in 0..NUM_ASTEROIDS {
            // Generate a random rotation axis.
            let mut a = Asteroid::default();
            get_random_vec(&mut rng, &mut a.axis);

            // No rotation to start, but we will rotate as
            // a function of time.
            a.theta = 0.0;

            // Random position in world space.
            a.pos.x = get_random_float(&mut rng, -500.0, 500.0);
            a.pos.y = get_random_float(&mut rng, -500.0, 500.0);
            a.pos.z = get_random_float(&mut rng, -500.0, 500.0);

            // Random velocity in world space.
            let speed: f32 = get_random_float(&mut rng, 10.0, 20.0);
            let mut dir = D3DXVECTOR3::default();
            get_random_vec(&mut rng, &mut dir);
            D3DXVec3Scale(&mut a.vel, &dir, speed);

            asteroids.push(a);
        }

        asteroids
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