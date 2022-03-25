use std::ffi::CStr;
use std::slice::from_raw_parts_mut;
use libc::c_void;
use rand::rngs::ThreadRng;
use windows::Win32::Foundation::PSTR;
use windows::Win32::Graphics::Direct3D9::*;
use common::*;
use d3dx::*;

use crate::*;

pub struct GunPSystem {
    // In practice, some sort of ID3DXEffect and IDirect3DTexture9 manager should
    // be used so that you do not duplicate effects/textures by having several
    // instances of a particle system.
    fx: LPD3DXEFFECT,
    h_tech: D3DXHANDLE,
    h_wvp: D3DXHANDLE,
    h_eye_pos_l: D3DXHANDLE,
    h_tex: D3DXHANDLE,
    h_time: D3DXHANDLE,
    h_accel: D3DXHANDLE,
    h_viewport_height: D3DXHANDLE,

    tex: *mut c_void, // IDirect3DTexture9
    vb: Option<IDirect3DVertexBuffer9>,
    world: D3DXMATRIX,
    inv_world: D3DXMATRIX,
    time: f32,
    accel: D3DXVECTOR3,
    bounding_box: AABB,
    max_num_particles: u32,
    time_per_particle: f32,

    particles: Vec<Particle>,
    alive_particles: Vec<usize>,
    dead_particles: Vec<usize>,

    rng: ThreadRng,

    hwnd: HWND,
}

impl GunPSystem {
    pub fn new(base_path: &str, fx_name: &str, tech_name: &str, tex_name: &str,
               accel: &D3DXVECTOR3, bounding_box: &AABB, max_num_particles: u32,
               time_per_particle: f32, hwnd: HWND, d3d_device: IDirect3DDevice9,
               rng: ThreadRng) -> GunPSystem {

        // Allocate memory for maximum number of particles.
        let mut particles: Vec<Particle> = Vec::new();
        particles.resize(max_num_particles as usize, Particle::default());

        let mut alive_particles: Vec<usize> = Vec::new();
        alive_particles.reserve(max_num_particles as usize);

        let mut dead_particles: Vec<usize> = Vec::new();
        dead_particles.reserve(max_num_particles as usize);

        // They start off all dead.
        for i in 0..max_num_particles as usize {
            particles[i].life_time = -1.0;
            particles[i].initial_time = 0.0;
        }

        let mut world = D3DXMATRIX::default();
        let mut inv_world = D3DXMATRIX::default();
        D3DXMatrixIdentity(&mut world);
        D3DXMatrixIdentity(&mut inv_world);

        // Create the texture.
        let mut tex = unsafe { std::mem::zeroed() };
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
            PSTR(c_resource_path(base_path, tex_name).as_str().as_ptr() as _), &mut tex));

        // Create the FX.
        // Create the generic Light & Tex FX from a .fx file.
        let mut fx: LPD3DXEFFECT = std::ptr::null_mut();
        let mut errors: LPD3DXBUFFER = std::ptr::null_mut();
        HR!(D3DXCreateEffectFromFile(d3d_device.clone(),
            PSTR(c_resource_path(base_path, fx_name).as_str().as_ptr() as _),
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

        let mut c_tech_name = String::from(tech_name);
        c_tech_name.push(0 as char);

        let h_tech = ID3DXBaseEffect_GetTechniqueByName(fx, PSTR(c_tech_name.as_ptr() as _));
        let h_wvp = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWVP\0".as_ptr() as _));
        let h_eye_pos_l = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gEyePosL\0".as_ptr() as _));
        let h_tex = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gTex\0".as_ptr() as _));
        let h_time = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gTime\0".as_ptr() as _));
        let h_accel = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gAccel\0".as_ptr() as _));
        let h_viewport_height = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gViewportHeight\0".as_ptr() as _));

        let mut vb: Option<IDirect3DVertexBuffer9> = None;

        unsafe {
            HR!(d3d_device.CreateVertexBuffer(max_num_particles * std::mem::size_of::<Particle>() as u32,
                (D3DUSAGE_DYNAMIC | D3DUSAGE_WRITEONLY | D3DUSAGE_POINTS) as u32, 0, D3DPOOL_DEFAULT,
                &mut vb, std::ptr::null_mut()));
        }

        GunPSystem {
            fx,
            h_tech,
            h_wvp,
            h_eye_pos_l,
            h_tex,
            h_time,
            h_accel,
            h_viewport_height,
            tex,
            vb,
            world,
            inv_world,
            time: 0.0,
            accel: accel.clone(),
            bounding_box: bounding_box.clone(),
            max_num_particles,
            time_per_particle,
            particles,
            alive_particles,
            dead_particles,
            rng,
            hwnd,
        }
    }

    pub fn release_com_objects(&self) {
        ReleaseCOM(self.fx);
        ReleaseCOM(self.tex);
    }

    // pub fn get_time(&self) -> f32 {
    //     self.time
    // }

    // pub fn set_time(&mut self, t: f32) {
    //     self.time = t;
    // }

    // pub fn get_aabb(&self) -> AABB {
    //     self.bounding_box.clone()
    // }

    pub fn set_world_mtx(&mut self, world: &D3DXMATRIX) {
        self.world = world.clone();

        // Compute the change of coordinates matrix that changes coordinates
        // relative to world space so that they are relative to the particle
        // system's local space.
        D3DXMatrixInverse(&mut self.inv_world, 0.0, world);
    }

    // FIXME not as fast as in C++ version that mutates the particle in place, also alive
    //       and dead lists contains indices to the particle instead of particle references
    //       for simplicity
    pub fn add_particle(&mut self) {
        if self.dead_particles.len() > 0 {
            // Reinitialize a particle.
            let particle_index = self.dead_particles.pop();

            if let Some(particle_index) = particle_index {
                let mut p = Particle::default();
                self.init_particle(&mut p);

                self.particles[particle_index].initial_pos = p.initial_pos;
                self.particles[particle_index].initial_velocity = p.initial_velocity;
                self.particles[particle_index].initial_size = p.initial_size;
                self.particles[particle_index].initial_time = p.initial_time;
                self.particles[particle_index].life_time = p.life_time;
                self.particles[particle_index].mass = p.mass;
                self.particles[particle_index].initial_color = p.initial_color;

                self.alive_particles.push(particle_index);
            }
        }
    }

    pub fn init_particle(&mut self, out: &mut Particle) {
        unsafe {
            let camera: &Camera = &CAMERA.expect("Camera has not been created");

            // Generate at camera.
            out.initial_pos = camera.get_pos();

            // Set down a bit so it looks like player is carrying the gun.
            out.initial_pos.y -= 3.0;

            // Fire in camera's look direction.
            let speed: f32 = 500.0;
            D3DXVec3Scale(&mut out.initial_velocity, &camera.get_look(), speed);
        }

        out.initial_time = self.time;
        out.life_time = 4.0;
        out.initial_color = D3DCOLOR_XRGB!(255, 255, 255);
        out.initial_size = get_random_float(&mut self.rng, 80.0, 90.0);
        out.mass = 1.0;
    }

    pub fn on_lost_device(&mut self) {
        HR!(ID3DXEffect_OnLostDevice(self.fx));

        // Default pool resources need to be freed before reset.
        self.vb = None;
    }

    pub fn on_reset_device(&mut self) {
        HR!(ID3DXEffect_OnResetDevice(self.fx));

        // Default pool resources need to be recreated after reset.
        if self.vb.is_none() {
            unsafe {
                if let Some(d3d_device) = &D3D_DEVICE {
                    HR!(d3d_device.CreateVertexBuffer(self.max_num_particles * std::mem::size_of::<Particle>() as u32,
                        (D3DUSAGE_DYNAMIC | D3DUSAGE_WRITEONLY | D3DUSAGE_POINTS) as u32, 0, D3DPOOL_DEFAULT,
                        &mut self.vb, std::ptr::null_mut()));
                }
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;

        // Rebuild the dead and alive list.  Note that resize(0) does
        // not deallocate memory (i.e., the capacity of the vector does
        // not change).
        self.dead_particles.resize(0, usize::MAX);
        self.alive_particles.resize(0, usize::MAX);

        // For each particle.
        for i in 0..self.max_num_particles as usize {
            // Is the particle dead?
            if (self.time - self.particles[i].initial_time) > self.particles[i].life_time {
                self.dead_particles.push(i);
            } else {
                self.alive_particles.push(i);
            }
        }

        // A negative or zero mTimePerParticle value denotes
        // not to emit any particles.
        if self.time_per_particle > 0.0 {
            // Emit particles.
            static mut TIME_ACCUM: f32 = 0.0;

            unsafe {
                TIME_ACCUM += dt;

                while TIME_ACCUM >= self.time_per_particle {
                    self.add_particle();
                    TIME_ACCUM -= self.time_per_particle;
                }
            }
        }
    }

    pub fn draw(&mut self) {
        unsafe {
            let camera: &Camera = &CAMERA.expect("Camera has not been created");

            if let Some(d3d_device) = &D3D_DEVICE {
                // Get camera position relative to world space system and make it
                // relative to the particle system's local system.
                let eye_pos_w = camera.get_pos();
                let mut eye_pos_l = D3DXVECTOR3::default();
                D3DXVec3TransformCoord(&mut eye_pos_l, &eye_pos_w, &self.inv_world);

                HR!(ID3DXEffect_SetTechnique(self.fx, self.h_tech));
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_accel, &self.accel as *const _ as _, std::mem::size_of::<D3DXVECTOR3>() as u32));
                HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.tex));

                // Set FX parameters.
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_eye_pos_l,
                    &eye_pos_l as *const _ as _,
                    std::mem::size_of::<D3DXVECTOR3>() as u32));

                HR!(ID3DXBaseEffect_SetFloat(self.fx, self.h_time, self.time));

                let mut wvp: D3DXMATRIX = D3DXMATRIX::default();
                D3DXMatrixMultiply(&mut wvp, &self.world, camera.get_view_proj());
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &wvp));

                // Point sprite sizes are given in pixels.  So if the viewport size
                // is changed, then more or less pixels become available, which alters
                // the perceived size of the particles.  For example, if the viewport
                // is 32x32, then a 32x32 sprite covers the entire viewport!  But if
                // the viewport is 1024x1024, then a 32x32 sprite only covers a small
                // portion of the viewport.  Thus, we scale the particle's
                // size by the viewport height to keep them in proportion to the
                // viewport dimensions.

                let mut client_rect = RECT::default();
                GetClientRect(self.hwnd, &mut client_rect);
                HR!(ID3DXBaseEffect_SetInt(self.fx, self.h_viewport_height, client_rect.bottom));

                let mut num_passes: u32 = 0;
                HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));
                HR!(ID3DXEffect_BeginPass(self.fx, 0));

                HR!(d3d_device.SetStreamSource(0, &self.vb, 0, std::mem::size_of::<Particle>() as u32));
                HR!(d3d_device.SetVertexDeclaration(&PARTICLE_DECL));

                let mut box_world: AABB = AABB::default();
                self.bounding_box.xform(&self.world, &mut box_world);

                if camera.is_visible(&box_world) {
                    if let Some(vb) = &mut self.vb {
                        // Initial lock of VB for writing.
                        let mut p = std::ptr::null_mut();
                        HR!(vb.Lock(0, 0, &mut p, D3DLOCK_DISCARD as u32));
                        let p_slice: &mut [Particle] =
                            from_raw_parts_mut(p as *mut Particle,
                                               self.max_num_particles as usize);

                        let mut vb_index: usize = 0;

                        // For each living particle.
                        for i in 0..self.alive_particles.len() {
                            // Copy particle to VB
                            p_slice[vb_index] = self.particles[self.alive_particles[i]].clone();
                            vb_index += 1;
                        }

                        HR!(vb.Unlock());

                        // Render however many particles we copied over.
                        if vb_index > 0 {
                            HR!(d3d_device.DrawPrimitive(D3DPT_POINTLIST, 0, vb_index as u32));
                        }
                    }
                }

                HR!(ID3DXEffect_EndPass(self.fx));
                HR!(ID3DXEffect_End(self.fx));
            }
        }
    }
}