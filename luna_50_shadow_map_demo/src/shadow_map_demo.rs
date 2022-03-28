use std::ffi::CStr;
use libc::c_void;
use windows::{Win32::Foundation::*, Win32::Graphics::Direct3D9::*};
use common::mtrl::Mtrl;

use crate::*;
use crate::drawable_tex2d::DrawableTex2D;
use crate::sky::Sky;

pub const BASE_PATH: &str = "luna_50_shadow_map_demo/";

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
pub struct ShadowMapDemo {
    d3d_pp: *const D3DPRESENT_PARAMETERS,
    gfx_stats: Option<GfxStats>,

    sky: Sky,
    scene_mesh: LPD3DXMESH,
    scene_world: D3DXMATRIX,
    scene_mtrls: Vec<Mtrl>,
    scene_textures: Vec<*mut c_void>,

    car_mesh: LPD3DXMESH,
    car_world: D3DXMATRIX,
    car_mtrls: Vec<Mtrl>,
    car_textures: Vec<*mut c_void>,

    white_tex: *mut c_void, // IDirect3DTexture9
    shadow_map: DrawableTex2D,

    light_vp: D3DXMATRIX,

    fx: LPD3DXEFFECT,
    h_build_shadow_map_tech: D3DXHANDLE,
    h_light_wvp: D3DXHANDLE,

    h_tech: D3DXHANDLE,
    h_wvp: D3DXHANDLE,
    h_eye_pos_w: D3DXHANDLE,
    h_world: D3DXHANDLE,
    h_tex: D3DXHANDLE,
    h_shadow_map: D3DXHANDLE,
    h_mtrl: D3DXHANDLE,
    h_light: D3DXHANDLE,

    light: SpotLight,
}

impl ShadowMapDemo {
    pub fn new(d3d_device: IDirect3DDevice9, d3d_object: &IDirect3D9, device_type: D3DDEVTYPE,
               d3d_pp: *const D3DPRESENT_PARAMETERS) -> Option<ShadowMapDemo> {
        unsafe {
            if !ShadowMapDemo::check_device_caps(d3d_object, device_type) {
                display_error_then_quit("checkDeviceCaps() Failed");
            }

            init_all_vertex_declarations(d3d_device.clone());

            let mut gfx_stats = GfxStats::new(d3d_device.clone(), D3DCOLOR_XRGB!(0, 0, 0));

            let sky = Sky::new(BASE_PATH, d3d_device.clone(),
                               "grassenvmap1024.dds", 10000.0);

            let (scene_mesh, scene_mtrls, scene_textures) =
                load_x_file(BASE_PATH, "BasicColumnScene.x", d3d_device.clone());

            let mut scene_world = D3DXMATRIX::default();
            D3DXMatrixIdentity(&mut scene_world);

            let (car_mesh, car_mtrls, car_textures) =
                load_x_file(BASE_PATH, "car.x", d3d_device.clone());

            let mut s = D3DXMATRIX::default();
            let mut t = D3DXMATRIX::default();
            let mut car_world = D3DXMATRIX::default();
            D3DXMatrixScaling(&mut s, 1.5, 1.5, 1.5);
            D3DXMatrixTranslation(&mut t, 6.0, 3.5, -3.0);
            D3DXMatrixMultiply(&mut car_world, &s, &t);

            let mut white_tex = std::mem::zeroed();
            HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
                PSTR(c_resource_path(BASE_PATH, "whitetex.dds").as_str().as_ptr() as _), &mut white_tex));

            // Set some light properties; other properties are set in update function,
            // where they are animated.
            let light = SpotLight {
                pos_w: D3DXVECTOR3::default(),
                dir_w: D3DXVECTOR3::default(),
                ambient: D3DXCOLOR { r: 0.5, g: 0.5, b: 0.5, a: 1.0 },
                diffuse: D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
                spec: D3DXCOLOR { r: 0.5, g: 0.5, b: 0.5, a: 1.0 },
                spot_power: 32.0
            };

            // Create shadow map.
            let vp = D3DVIEWPORT9 { X: 0, Y: 0, Width: 512, Height: 512, MinZ: 0.0, MaxZ: 1.0 };
            let shadow_map = DrawableTex2D::new(512, 512, 1,
                                                D3DFMT_R32F, true,
                                                D3DFMT_D24X8, &vp, false);

            // Initialize camera.
            if let Some(camera) = &mut CAMERA {
                camera.set_pos(D3DXVECTOR3 { x: 0.0, y: 20.0, z: -100.0 });
                camera.set_speed(50.0);
            }

            if let Some(gfx_stats) = &mut gfx_stats {
                gfx_stats.add_vertices(ID3DXBaseMesh_GetNumVertices(scene_mesh));
                gfx_stats.add_triangles(ID3DXBaseMesh_GetNumFaces(scene_mesh));

                gfx_stats.add_vertices(ID3DXBaseMesh_GetNumVertices(car_mesh));
                gfx_stats.add_triangles(ID3DXBaseMesh_GetNumFaces(car_mesh));

                gfx_stats.add_vertices(sky.get_num_vertices());
                gfx_stats.add_triangles(sky.get_num_triangles());
            }

            let (fx,
                h_tech,
                h_build_shadow_map_tech,
                h_light_wvp,
                h_wvp,
                h_world,
                h_mtrl,
                h_light,
                h_eye_pos_w,
                h_tex,
                h_shadow_map)
                = ShadowMapDemo::build_fx(d3d_device.clone());

            let mut shadow_map_demo = ShadowMapDemo {
                d3d_pp,
                gfx_stats,

                sky,
                scene_mesh,
                scene_world,
                scene_mtrls,
                scene_textures,

                car_mesh,
                car_world,
                car_mtrls,
                car_textures,

                white_tex,
                shadow_map,

                light_vp: Default::default(),

                fx,
                h_build_shadow_map_tech,
                h_light_wvp,
                h_tech,
                h_wvp,
                h_eye_pos_w,
                h_world,
                h_tex,
                h_shadow_map,
                h_mtrl,
                h_light,

                light
            };

            shadow_map_demo.on_reset_device();

            Some(shadow_map_demo)
        }
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        self.sky.release_com_objects();

        ReleaseCOM(self.fx);
        ReleaseCOM(self.white_tex);
        self.shadow_map.release_com_objects();

        ReleaseCOM(self.scene_mesh);
        for tex in &self.scene_textures {
            ReleaseCOM(tex.cast());
        }

        ReleaseCOM(self.car_mesh);
        for tex in &self.car_textures {
            ReleaseCOM(tex.cast());
        }

        destroy_all_vertex_declarations();
    }

    fn check_device_caps(d3d_object: &IDirect3D9, device_type: D3DDEVTYPE) -> bool {
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

            // Check render target support.  The adapter format can be either the display mode format
            // for windowed mode, or D3DFMT_X8R8G8B8 for fullscreen mode, so we need to test against
            // both.  We use D3DFMT_X8R8G8B8 as the render texture format and D3DFMT_D24X8 as the
            // render texture depth format.

            let mut mode = D3DDISPLAYMODE::default();
            HR!(d3d_object.GetAdapterDisplayMode(D3DADAPTER_DEFAULT, &mut mode));

            // Windowed.
            if FAILED!(d3d_object.CheckDeviceFormat(D3DADAPTER_DEFAULT, device_type, mode.Format,
                    D3DUSAGE_RENDERTARGET as u32, D3DRTYPE_TEXTURE, D3DFMT_R32F)) {
                return false
            }

            if FAILED!(d3d_object.CheckDepthStencilMatch(D3DADAPTER_DEFAULT, device_type, mode.Format,
                    D3DFMT_R32F, D3DFMT_D24X8)) {
                return false
            }

            // Fullscreen.
            if FAILED!(d3d_object.CheckDeviceFormat(D3DADAPTER_DEFAULT, device_type, D3DFMT_X8R8G8B8,
                    D3DUSAGE_RENDERTARGET as u32, D3DRTYPE_TEXTURE, D3DFMT_R32F)) {
                return false
            }

            if FAILED!(d3d_object.CheckDepthStencilMatch(D3DADAPTER_DEFAULT, device_type, D3DFMT_X8R8G8B8,
                    D3DFMT_R32F, D3DFMT_D24X8)) {
                return false
            }

            true
        }
    }

    pub fn on_lost_device(&mut self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.on_lost_device();
        }

        self.sky.on_lost_device();
        self.shadow_map.on_lost_device();
        HR!(ID3DXEffect_OnLostDevice(self.fx));
    }

    pub fn on_reset_device(&mut self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.on_reset_device();
        }

        self.sky.on_reset_device();
        self.shadow_map.on_reset_device();
        HR!(ID3DXEffect_OnResetDevice(self.fx));

        // The aspect ratio depends on the backbuffer dimensions, which can
        // possibly change after a reset.  So rebuild the projection matrix.
        unsafe {
            let w: f32 = (*self.d3d_pp).BackBufferWidth as f32;
            let h: f32 = (*self.d3d_pp).BackBufferHeight as f32;

            if let Some(camera) = &mut CAMERA {
                camera.set_lens(D3DX_PI * 0.25, w / h, 1.0, 2000.0);
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

            if let Some(camera) = &mut CAMERA {
                camera.update(dt, None, 0.0);
            }

            // Animate spot light by rotating it on y-axis with respect to time.
            let mut light_view = D3DXMATRIX::default();
            let mut light_pos_w = D3DXVECTOR3 { x: 125.0, y: 50.0, z: 0.0 };
            let light_target_w = D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 };
            let light_up_w = D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 };

            static mut T: f32 = 0.0;
            T += dt;

            if T >= 2.0 * D3DX_PI {
                T = 0.0;
            }

            let mut ry = D3DXMATRIX::default();
            D3DXMatrixRotationY(&mut ry, T);
            D3DXVec3TransformCoord(&mut light_pos_w, &light_pos_w, &ry);

            D3DXMatrixLookAtLH(&mut light_view, &light_pos_w, &light_target_w, &light_up_w);

            let mut light_lens = D3DXMATRIX::default();
            let light_fov: f32 = D3DX_PI * 0.25;
            D3DXMatrixPerspectiveFovLH(&mut light_lens, light_fov, 1.0, 1.0, 200.0);

            D3DXMatrixMultiply(&mut self.light_vp, &light_view, &light_lens);

            // Setup a spotlight corresponding to the projector.
            let mut light_dir_w = D3DXVECTOR3::default();
            D3DXVec3Subtract(&mut light_dir_w, &light_target_w, &light_pos_w);
            D3DXVec3Normalize(&mut light_dir_w, &light_dir_w);
            self.light.pos_w = light_pos_w;
            self.light.dir_w = light_dir_w;
        }
    }

    pub fn draw_scene(&mut self) {
        unsafe {
            if let Some(d3d_device) = &D3D_DEVICE {
                let camera: &Camera = &CAMERA.expect("Camera has not been created");

                self.draw_shadow_map(d3d_device.clone());

                HR!(d3d_device.BeginScene());

                self.sky.draw();

                HR!(ID3DXEffect_SetTechnique(self.fx, self.h_tech));

                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_light, &self.light as *const _ as _,
                    std::mem::size_of::<SpotLight>() as u32));
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_eye_pos_w,
                    &camera.get_pos() as *const _ as _, std::mem::size_of::<D3DXVECTOR3>() as u32));
                HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_shadow_map, self.shadow_map.d3d_tex()));

                let mut num_passes: u32 = 0;
                HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));
                HR!(ID3DXEffect_BeginPass(self.fx, 0));

                // Draw Scene mesh.
                let mut light_wvp: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixMultiply(&mut light_wvp, &self.scene_world, &self.light_vp);
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_light_wvp, &light_wvp));

                let mut wvp: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixMultiply(&mut wvp, &self.scene_world, camera.get_view_proj());
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &wvp));
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world, &self.scene_world));

                for j in 0..self.scene_mtrls.len() {
                    HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_mtrl,
                        &self.scene_mtrls[j] as *const _ as _, std::mem::size_of::<Mtrl>() as u32));

                    if !self.scene_textures[j].is_null() {
                        HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.scene_textures[j]));
                    } else {
                        HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.white_tex));
                    }

                    HR!(ID3DXEffect_CommitChanges(self.fx));
                    HR!(ID3DXBaseMesh_DrawSubset(self.scene_mesh, j as u32));
                }

                // Draw car mesh.
                light_wvp = std::mem::zeroed();
                D3DXMatrixMultiply(&mut light_wvp, &self.car_world, &self.light_vp);
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_light_wvp, &light_wvp));

                wvp = std::mem::zeroed();
                D3DXMatrixMultiply(&mut wvp, &self.car_world, camera.get_view_proj());
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &wvp));
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world, &self.car_world));

                for j in 0..self.car_mtrls.len() {
                    HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_mtrl,
                        &self.car_mtrls[j] as *const _ as _, std::mem::size_of::<Mtrl>() as u32));

                    if !self.car_textures[j].is_null() {
                        HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.car_textures[j]));
                    } else {
                        HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.white_tex));
                    }

                    HR!(ID3DXEffect_CommitChanges(self.fx));
                    HR!(ID3DXBaseMesh_DrawSubset(self.car_mesh, j as u32));
                }

                HR!(ID3DXEffect_EndPass(self.fx));
                HR!(ID3DXEffect_End(self.fx));

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

    fn draw_shadow_map(&self, d3d_device: IDirect3DDevice9) {
        unsafe {
            self.shadow_map.begin_scene();

            HR!(d3d_device.Clear(
                        0,
                        std::ptr::null(),
                        (D3DCLEAR_TARGET | D3DCLEAR_ZBUFFER) as u32,
                        0x00000000,
                        1.0,
                        0));

            HR!(ID3DXEffect_SetTechnique(self.fx, self.h_build_shadow_map_tech));

            let mut num_passes: u32 = 0;
            HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));
            HR!(ID3DXEffect_BeginPass(self.fx, 0));

            // Draw scene mesh.
            let mut light_wvp: D3DXMATRIX = std::mem::zeroed();
            D3DXMatrixMultiply(&mut light_wvp, &self.scene_world, &self.light_vp);
            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_light_wvp, &light_wvp));
            HR!(ID3DXEffect_CommitChanges(self.fx));

            for j in 0..self.scene_mtrls.len() {
                HR!(ID3DXBaseMesh_DrawSubset(self.scene_mesh, j as u32));
            }

            // Draw car mesh.
            light_wvp = std::mem::zeroed();
            D3DXMatrixMultiply(&mut light_wvp, &self.car_world, &self.light_vp);
            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_light_wvp, &light_wvp));
            HR!(ID3DXEffect_CommitChanges(self.fx));

            for j in 0..self.car_mtrls.len() {
                HR!(ID3DXBaseMesh_DrawSubset(self.car_mesh, j as u32));
            }

            HR!(ID3DXEffect_EndPass(self.fx));
            HR!(ID3DXEffect_End(self.fx));

            self.shadow_map.end_scene();
        }
    }

    fn build_fx(d3d_device: IDirect3DDevice9) -> (LPD3DXEFFECT, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
                                                  D3DXHANDLE, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
                                                  D3DXHANDLE, D3DXHANDLE, D3DXHANDLE) {
        // Create the FX from a .fx file.
        let mut fx: LPD3DXEFFECT = std::ptr::null_mut();
        let mut errors: LPD3DXBUFFER = std::ptr::null_mut();

        HR!(D3DXCreateEffectFromFile(d3d_device,
            PSTR(c_resource_path(BASE_PATH, "LightShadow.fx").as_str().as_ptr() as _),
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
        let h_tech = ID3DXBaseEffect_GetTechniqueByName(fx, PSTR(b"LightShadowTech\0".as_ptr() as _));
        let h_build_shadow_map_tech = ID3DXBaseEffect_GetTechniqueByName(fx, PSTR(b"BuildShadowMapTech\0".as_ptr() as _));
        let h_light_wvp = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gLightWVP\0".as_ptr() as _));
        let h_wvp = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWVP\0".as_ptr() as _));
        let h_world = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWorld\0".as_ptr() as _));
        let h_mtrl = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gMtrl\0".as_ptr() as _));
        let h_light = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gLight\0".as_ptr() as _));
        let h_eye_pos_w = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gEyePosW\0".as_ptr() as _));
        let h_tex = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gTex\0".as_ptr() as _));
        let h_shadow_map = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gShadowMap\0".as_ptr() as _));

        (fx, h_tech, h_build_shadow_map_tech, h_light_wvp, h_wvp, h_world, h_mtrl, h_light, h_eye_pos_w, h_tex, h_shadow_map)
    }
}