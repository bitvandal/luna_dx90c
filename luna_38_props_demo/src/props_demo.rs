use std::ffi::CStr;
use std::slice::from_raw_parts_mut;
use libc::c_void;
use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D9::*, Win32::System::SystemServices::*,
};
use common::mtrl::Mtrl;
use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;

use crate::*;
use crate::terrain::Terrain;
use crate::water::Water;

pub const BASE_PATH: &str = "luna_38_props_demo/Art/";

const EPSILON: f32 = 0.001;

// Colors
const WHITE: D3DXCOLOR = D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

// Directional Light
#[repr(C)]
struct DirLight {
    ambient: D3DXCOLOR,
    diffuse: D3DXCOLOR,
    spec: D3DXCOLOR,
    dir_w: D3DXVECTOR3,
}

pub struct Object3D {
    mesh: LPD3DXMESH,
    mtrls: Vec<Mtrl>,
    textures: Vec<*mut c_void>,
    bounding_box: AABB,
}

impl Object3D {
    pub fn new() -> Object3D {
        Object3D {
            mesh: std::ptr::null_mut(),
            mtrls: vec![],
            textures: vec![],
            bounding_box: AABB::default()
        }
    }

    pub fn release_com_objects(&self) {
        ReleaseCOM(self.mesh);
        for tex in &self.textures {
            ReleaseCOM(tex.cast());
        }
    }
}

const NUM_TREES: usize = 200;
const NUM_GRASS_BLOCKS: u32 = 4000;

// Sample demo
pub struct PropsDemo {
    d3d_pp: *const D3DPRESENT_PARAMETERS,
    gfx_stats: Option<GfxStats>,

    terrain: Terrain,
    water: Water,

    time: f32, // Time elapsed from program start.

    castle: Object3D,
    castle_world: D3DXMATRIX,

    trees: [Object3D; 4],
    tree_worlds: [D3DXMATRIX; NUM_TREES],

    grass_mesh: LPD3DXMESH,
    grass_tex: *mut c_void,

    // Grass FX
    grass_fx: LPD3DXEFFECT,
    h_grass_view_proj : D3DXHANDLE,
    h_grass_time : D3DXHANDLE,
    h_grass_eye_pos_w : D3DXHANDLE,

    // General light/texture FX
    fx: LPD3DXEFFECT,
    h_tech: D3DXHANDLE,
    h_wvp: D3DXHANDLE,
    h_world_inv_trans: D3DXHANDLE,
    h_eye_pos_w: D3DXHANDLE,
    h_world: D3DXHANDLE,
    h_tex: D3DXHANDLE,
    h_mtrl: D3DXHANDLE,

    // Camera fixed to ground or can fly?
    free_camera: bool,

    // Default texture if no texture present for subset.
    white_tex: *mut c_void, // IDirect3DTexture9
}

impl PropsDemo {
    pub fn new(d3d_device: IDirect3DDevice9, d3d_pp: *const D3DPRESENT_PARAMETERS) -> Option<PropsDemo> {
        if !PropsDemo::check_device_caps() {
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
                         "castlehm257.raw",
                         "grass.dds",
                         "dirt.dds",
                         "rock.dds",
                         "blend_castle.dds",
                         BASE_PATH,
                         0.5,
                         0.0);

        let mut to_sun = D3DXVECTOR3 { x: -1.0, y: 3.0, z: 1.0 };
        D3DXVec3Normalize(&mut to_sun, &to_sun);
        terrain.set_dir_to_sun_w(to_sun);

        // Setup water.
        let mut water_world: D3DXMATRIX = D3DXMATRIX::default();
        D3DXMatrixTranslation(&mut water_world, 8.0, 35.0, -80.0);
        let water = Water::new(BASE_PATH, 33, 33, 20.0, 20.0,
                               &water_world, &d3d_device);

        // Initialize camera.
        unsafe {
            if let Some(camera) = &mut CAMERA {
                camera.set_pos(D3DXVECTOR3 { x: 8.0, y: 35.0, z: -100.0 });
                camera.set_speed(20.0);
            }
        }

        let (castle, castle_world) = PropsDemo::build_castle(d3d_device.clone());
        let (trees, tree_worlds) = PropsDemo::build_trees(&terrain, d3d_device.clone());
        let grass_mesh = PropsDemo::build_grass(&terrain, d3d_device.clone());

        let mut grass_tex = unsafe { std::mem::zeroed() };
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
            PSTR(c_resource_path(BASE_PATH, "grassfin0.dds").as_str().as_ptr() as _), &mut grass_tex));

        let mut white_tex = unsafe { std::mem::zeroed() };
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
            PSTR(c_resource_path(BASE_PATH, "whitetex.dds").as_str().as_ptr() as _), &mut white_tex));

        let (grass_fx,
            h_grass_tech,
            h_grass_view_proj,
            h_grass_tex,
            h_grass_time,
            h_grass_eye_pos_w,
            h_grass_dir_to_sun_w,
            fx,
            h_tech,
            h_wvp,
            h_world_inv_trans,
            h_mtrl,
            h_light,
            h_eye_pos_w,
            h_world,
            h_tex) =
            PropsDemo::build_fx(d3d_device.clone());

        // The Sun.
        let mut light_dir_w = D3DXVECTOR3::default();
        D3DXVec3Scale(&mut light_dir_w, &to_sun, -1.0);
        D3DXVec3Normalize(&mut light_dir_w, &light_dir_w);

        let light = DirLight {
            ambient: D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            diffuse: D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            spec: D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            dir_w: light_dir_w,
        };

        HR!(ID3DXBaseEffect_SetValue(fx, h_light, &light as *const _ as _, std::mem::size_of::<DirLight>() as u32));
        HR!(ID3DXBaseEffect_SetValue(grass_fx, h_grass_dir_to_sun_w, &to_sun as *const _ as _, std::mem::size_of::<D3DXVECTOR3>() as u32));

        HR!(ID3DXEffect_SetTechnique(grass_fx, h_grass_tech));
        HR!(ID3DXBaseEffect_SetTexture(grass_fx, h_grass_tex, grass_tex));

        if let Some(gfx_stats) = &mut gfx_stats {
            gfx_stats.add_vertices(terrain.get_num_vertices());
            gfx_stats.add_triangles(terrain.get_num_triangles());
            gfx_stats.add_vertices(water.get_num_vertices());
            gfx_stats.add_triangles(water.get_num_triangles());
            gfx_stats.add_vertices(ID3DXBaseMesh_GetNumVertices(castle.mesh));
            gfx_stats.add_triangles(ID3DXBaseMesh_GetNumFaces(castle.mesh));

            for i in 0..4 {
                gfx_stats.add_vertices(ID3DXBaseMesh_GetNumVertices(trees[i].mesh) * NUM_TREES as u32 / 4);
                gfx_stats.add_triangles(ID3DXBaseMesh_GetNumFaces(trees[i].mesh) * NUM_TREES as u32 / 4);
            }

            gfx_stats.add_vertices(ID3DXBaseMesh_GetNumVertices(grass_mesh));
            gfx_stats.add_triangles(ID3DXBaseMesh_GetNumFaces(grass_mesh));
        }

        let mut props_demo = PropsDemo {
            d3d_pp,
            gfx_stats,

            terrain,
            water,

            time: 0.0,

            castle,
            castle_world,
            trees,
            tree_worlds,

            grass_mesh,
            grass_tex,

            grass_fx,
            h_grass_view_proj,
            h_grass_time,
            h_grass_eye_pos_w,

            fx,
            h_tech,
            h_wvp,
            h_world_inv_trans,
            h_eye_pos_w,
            h_world,
            h_tex,
            h_mtrl,

            free_camera: false,

            white_tex,
        };

        props_demo.on_reset_device();

        Some(props_demo)
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        self.terrain.release_com_objects();
        self.water.release_com_objects();

        ReleaseCOM(self.white_tex);
        ReleaseCOM(self.fx);
        ReleaseCOM(self.grass_mesh);
        ReleaseCOM(self.grass_tex);
        ReleaseCOM(self.grass_fx);

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
        self.water.on_lost_device();
        HR!(ID3DXEffect_OnLostDevice(self.fx));
        HR!(ID3DXEffect_OnLostDevice(self.grass_fx));
    }

    pub fn on_reset_device(&mut self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.on_reset_device();
        }

        self.terrain.on_reset_device();
        self.water.on_reset_device();
        HR!(ID3DXEffect_OnResetDevice(self.fx));
        HR!(ID3DXEffect_OnResetDevice(self.grass_fx));

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
        if let Some(gfx_stats) = &mut self.gfx_stats {
            gfx_stats.update(dt);
        }

        self.time += dt;

        // Get snapshot of input devices.
        unsafe {
            if let Some(dinput) = &mut DIRECT_INPUT {
                dinput.poll();

                // Fix camera to ground or free flying camera?
                if dinput.key_down(DIK_N as usize) {
                    self.free_camera = false;
                }

                if dinput.key_down(DIK_M as usize) {
                    self.free_camera = true;
                }

                if let Some(camera) = &mut CAMERA {
                    if self.free_camera {
                        camera.update(dt, None, 0.0);
                    } else {
                        camera.update(dt, Some(&self.terrain), 2.5);
                    }
                }
            }

            self.water.update(dt);
        }
    }

    pub fn draw_scene(&mut self) {
        unsafe {
            let camera: &Camera = &CAMERA.expect("Camera has not been created");

            if let Some(d3d_device) = &D3D_DEVICE {
                // Clear the backbuffer and depth buffer.
                HR!(d3d_device.Clear(
                    0,
                    std::ptr::null(),
                    (D3DCLEAR_TARGET | D3DCLEAR_ZBUFFER) as u32,
                    0xFF888888,
                    1.0,
                    0));

                HR!(d3d_device.BeginScene());

                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_eye_pos_w,
                    &camera.get_pos() as *const _ as _,
                    std::mem::size_of::<D3DXVECTOR3>() as u32));

                HR!(ID3DXEffect_SetTechnique(self.fx, self.h_tech));

                let mut num_passes: u32 = 0;
                HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));
                HR!(ID3DXEffect_BeginPass(self.fx, 0));

                self.draw_object(&self.castle, &self.castle_world);

                // Use alpha test to block non leaf pixels from being rendered in the
                // trees (i.e., use alpha mask).
                HR!(d3d_device.SetRenderState(D3DRS_ALPHATESTENABLE, 1));
                HR!(d3d_device.SetRenderState(D3DRS_ALPHAFUNC, D3DCMP_GREATEREQUAL.0 as u32));
                HR!(d3d_device.SetRenderState(D3DRS_ALPHAREF, 200));

                // Draw the trees: NUM_TREES/4 of each of the four types.
                for i in 0..NUM_TREES {
                    if i < NUM_TREES / 4 {
                        self.draw_object(&self.trees[0], &self.tree_worlds[i]);
                    } else if i < 2 * NUM_TREES / 4 {
                        self.draw_object(&self.trees[1], &self.tree_worlds[i]);
                    }else if i < 3 * NUM_TREES / 4 {
                        self.draw_object(&self.trees[2], &self.tree_worlds[i]);
                    } else {
                        self.draw_object(&self.trees[3], &self.tree_worlds[i]);
                    }
                }

                HR!(d3d_device.SetRenderState(D3DRS_ALPHATESTENABLE, 0));

                HR!(ID3DXEffect_EndPass(self.fx));
                HR!(ID3DXEffect_End(self.fx));

                HR!(ID3DXBaseEffect_SetValue(self.grass_fx, self.h_grass_eye_pos_w,
                        &camera.get_pos() as *const _ as _,
                        std::mem::size_of::<D3DXVECTOR3>() as u32));
                HR!(ID3DXBaseEffect_SetMatrix(self.grass_fx, self.h_grass_view_proj, camera.get_view_proj()));
                HR!(ID3DXBaseEffect_SetFloat(self.grass_fx, self.h_grass_time, self.time));

                HR!(ID3DXEffect_Begin(self.grass_fx, &mut num_passes, 0));
                HR!(ID3DXEffect_BeginPass(self.grass_fx, 0));

                // Draw to depth buffer only.
                HR!(ID3DXBaseMesh_DrawSubset(self.grass_mesh, 0));

                HR!(ID3DXEffect_EndPass(self.grass_fx));
                HR!(ID3DXEffect_End(self.grass_fx));

                self.terrain.draw();

                self.water.draw(); // draw alpha blended objects last.

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

    fn build_fx(d3d_device: IDirect3DDevice9) -> (LPD3DXEFFECT, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
                                                  D3DXHANDLE, D3DXHANDLE, D3DXHANDLE, LPD3DXEFFECT,
                                                  D3DXHANDLE, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
                                                  D3DXHANDLE, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE) {
        // Create the generic Light & Tex FX from a .fx file.
        let mut fx: LPD3DXEFFECT = std::ptr::null_mut();
        let mut errors: LPD3DXBUFFER = std::ptr::null_mut();

        HR!(D3DXCreateEffectFromFile(d3d_device.clone(),
            PSTR(c_resource_path(BASE_PATH, "DirLightTex.fx").as_str().as_ptr() as _),
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
        let h_tech = ID3DXBaseEffect_GetTechniqueByName(fx, PSTR(b"DirLightTexTech\0".as_ptr() as _));
        let h_wvp = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWVP\0".as_ptr() as _));
        let h_world_inv_trans = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWorldInvTrans\0".as_ptr() as _));
        let h_mtrl = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gMtrl\0".as_ptr() as _));
        let h_light = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gLight\0".as_ptr() as _));
        let h_eye_pos_w = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gEyePosW\0".as_ptr() as _));
        let h_world = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWorld\0".as_ptr() as _));
        let h_tex = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gTex\0".as_ptr() as _));


        // Create the grass FX from a .fx file.
        let mut grass_fx: LPD3DXEFFECT = std::ptr::null_mut();

        HR!(D3DXCreateEffectFromFile(d3d_device.clone(),
            PSTR(c_resource_path(BASE_PATH, "grass.fx").as_str().as_ptr() as _),
            std::ptr::null(), std::ptr::null(), 0,
            std::ptr::null(), &mut grass_fx, &mut errors));


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
        let h_grass_tech = ID3DXBaseEffect_GetTechniqueByName(grass_fx, PSTR(b"GrassTech\0".as_ptr() as _));
        let h_grass_view_proj = ID3DXBaseEffect_GetParameterByName(grass_fx, std::ptr::null(), PSTR(b"gViewProj\0".as_ptr() as _));
        let h_grass_tex = ID3DXBaseEffect_GetParameterByName(grass_fx, std::ptr::null(), PSTR(b"gTex\0".as_ptr() as _));
        let h_grass_time = ID3DXBaseEffect_GetParameterByName(grass_fx, std::ptr::null(), PSTR(b"gTime\0".as_ptr() as _));
        let h_grass_eye_pos_w = ID3DXBaseEffect_GetParameterByName(grass_fx, std::ptr::null(), PSTR(b"gEyePosW\0".as_ptr() as _));
        let h_grass_dir_to_sun_w = ID3DXBaseEffect_GetParameterByName(grass_fx, std::ptr::null(), PSTR(b"gDirToSunW\0".as_ptr() as _));

        (grass_fx, h_grass_tech, h_grass_view_proj, h_grass_tex, h_grass_time, h_grass_eye_pos_w, h_grass_dir_to_sun_w,
         fx, h_tech, h_wvp, h_world_inv_trans, h_mtrl, h_light, h_eye_pos_w, h_world, h_tex)
    }

    pub fn draw_object(&self, obj: &Object3D, to_world: &D3DXMATRIX) {
        unsafe {
            let camera: &Camera = &CAMERA.expect("Camera has not been created");

            // Transform AABB into the world space.
            let mut bounding_box: AABB = std::mem::zeroed();
            obj.bounding_box.xform(to_world, &mut bounding_box);

            // Only draw if AABB is visible.
            if camera.is_visible(&bounding_box) {
                let mut wvp: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixMultiply(&mut wvp, to_world, camera.get_view_proj());
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &wvp));

                let mut world_inverse_transpose: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixInverse(&mut world_inverse_transpose, 0.0, to_world);
                D3DXMatrixTranspose(&mut world_inverse_transpose, &world_inverse_transpose);
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world_inv_trans, &world_inverse_transpose));
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world, to_world));

                for j in 0..obj.mtrls.len() {
                    HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_mtrl,
                        &obj.mtrls[j] as *const _ as _,
                        std::mem::size_of::<Mtrl>() as u32));

                    // If there is a texture, then use.
                    if !obj.textures[j].is_null() {
                        HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, obj.textures[j]));
                    } else {
                        // But if not, then set a pure white texture.  When the texture color
                        // is multiplied by the color from lighting, it is like multiplying by
                        // 1 and won't change the color from lighting.
                        HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.white_tex));
                    }

                    HR!(ID3DXEffect_CommitChanges(self.fx));
                    HR!(ID3DXBaseMesh_DrawSubset(obj.mesh, j as u32));
                }
            }
        }
    }

    fn build_castle(d3d_device: IDirect3DDevice9) -> (Object3D, D3DXMATRIX) {
        // Load the castle mesh.
        let mut castle = Object3D::new();
        let (mesh, mtrls, textures) =
            load_x_file(BASE_PATH, "castle.x", d3d_device.clone());
        castle.mesh = mesh;
        castle.mtrls = mtrls;
        castle.textures = textures;

        // Compute castle AABB.
        let mut v: *mut c_void = std::ptr::null_mut();
        HR!(ID3DXBaseMesh_LockVertexBuffer(castle.mesh, 0, &mut v));

        let num_vertices = ID3DXBaseMesh_GetNumVertices(castle.mesh);
        let num_bytes_per_vertex = ID3DXBaseMesh_GetNumBytesPerVertex(castle.mesh);

        HR!(D3DXComputeBoundingBox(v.cast(), num_vertices, num_bytes_per_vertex,
            &mut castle.bounding_box.min_pt, &mut castle.bounding_box.max_pt));

        HR!(ID3DXBaseMesh_UnlockVertexBuffer(castle.mesh));

        // Manually set castle materials.
        for i in 0..castle.mtrls.len() {
            castle.mtrls[i].ambient = WHITE.mult(0.5);
            castle.mtrls[i].diffuse = WHITE;
            castle.mtrls[i].spec = WHITE.mult(0.8);
            castle.mtrls[i].spec_power = 28.0;
        }

        // Build castle's world matrix.
        let mut t: D3DXMATRIX = unsafe { std::mem::zeroed() };
        let mut ry: D3DXMATRIX = unsafe { std::mem::zeroed() };

        D3DXMatrixRotationY(&mut ry, D3DX_PI);
        D3DXMatrixTranslation(&mut t, 8.0, 35.0, -80.0);

        let mut castle_world: D3DXMATRIX = unsafe { std::mem::zeroed() };
        D3DXMatrixMultiply(&mut castle_world, &ry, &t);

        (castle, castle_world)
    }

    fn build_trees(terrain: &Terrain, d3d_device: IDirect3DDevice9) -> ([Object3D; 4], [D3DXMATRIX; NUM_TREES]) {
        // Load 4 unique meshes.  To draw more than 4 trees, we just draw these
        // 4 trees repeatedly, with different world matrices applied.
        let mut trees: [Object3D; 4] = unsafe { std::mem::zeroed() };
        let mut tree_worlds = [D3DXMATRIX::default(); NUM_TREES];

        let (mesh, mtrls, textures) =
            load_x_file(BASE_PATH, "tree0.x", d3d_device.clone());
        trees[0].mesh = mesh;
        trees[0].mtrls = mtrls;
        trees[0].textures = textures;

        let (mesh, mtrls, textures) =
            load_x_file(BASE_PATH, "tree1.x", d3d_device.clone());
        trees[1].mesh = mesh;
        trees[1].mtrls = mtrls;
        trees[1].textures = textures;

        let (mesh, mtrls, textures) =
            load_x_file(BASE_PATH, "tree2.x", d3d_device.clone());
        trees[2].mesh = mesh;
        trees[2].mtrls = mtrls;
        trees[2].textures = textures;

        let (mesh, mtrls, textures) =
            load_x_file(BASE_PATH, "tree3.x", d3d_device.clone());
        trees[3].mesh = mesh;
        trees[3].mtrls = mtrls;
        trees[3].textures = textures;

        // Build tree bounding boxes.
        for i in 0..4 {
            let mut v: *mut c_void = std::ptr::null_mut();
            HR!(ID3DXBaseMesh_LockVertexBuffer(trees[i].mesh, 0, &mut v));

            HR!(D3DXComputeBoundingBox(v.cast(),
                ID3DXBaseMesh_GetNumVertices(trees[i].mesh),
                ID3DXBaseMesh_GetNumBytesPerVertex(trees[i].mesh),
                &mut trees[i].bounding_box.min_pt, &mut trees[i].bounding_box.max_pt));

            HR!(ID3DXBaseMesh_UnlockVertexBuffer(trees[i].mesh));
        }

        // Build world matrices for NUM_TREES trees.  To do this, we generate a
        // random position on the terrain surface for each tree.  In reality,
        // this is not the best way to do it, as we'd like to have more control and
        // manually place trees in the scene by an artist.  Nevertheless, this is
        // an easy way to get trees in the scene of our demo.  To prevent trees
        // from being placed on mountain peaks, or in the water, we can specify to
        // only generate trees in an allowed height range.  By inspecting the heightmap
        // used in this demo, castlehm257.raw, the range [35, 50] seems to be a good
        // one to generate trees in.  Note that this method does not prevent trees from
        // interpenetrating with one another and it does not prevent the trees from
        // interpenetrating with the castle.

        // Scale down a bit do we ignore the borders of the terrain as candidates.
        let w: i32 = (terrain.get_width() * 0.8) as i32;
        let d: i32 = (terrain.get_depth() * 0.8) as i32;

        let mut s = D3DXMATRIX::default();
        let mut t = D3DXMATRIX::default();

        let mut rng = thread_rng();

        for i in 0..NUM_TREES {
            loop {
                let x: f32 = (rng.gen_range(0..32767) % w) as f32 - (w as f32 * 0.5);
                let z: f32 = (rng.gen_range(0..32767) % d) as f32 - (d as f32 * 0.5);

                // Subtract off height to embed trunk in ground.
                let y: f32 = terrain.get_height(x, z) - 0.5;

                // Trees modeled to a different scale then ours, so scale them down to make sense.
                // Also randomize the height a bit.
                let rand_int = rng.gen_range(0..32767);
                let tree_scale = get_random_float(rand_int, 0.15, 0.25);

                // Build tree's world matrix.
                D3DXMatrixTranslation(&mut t, x, y, z);
                D3DXMatrixScaling(&mut s, tree_scale, tree_scale, tree_scale);
                D3DXMatrixMultiply(&mut tree_worlds[i], &s, &t);

                // Only generate trees in this height range.  If the height
                // is outside this range, generate a new random position and
                // try again.
                if !(y < 35.0 || y > 50.0) { // We are trying again if out of valid height range
                    break;
                }
            }
        }

        (trees, tree_worlds)
    }

    fn build_grass(terrain: &Terrain, d3d_device: IDirect3DDevice9) -> LPD3DXMESH {
        unsafe {
            let mut elems: [D3DVERTEXELEMENT9; MAX_FVF_DECL_SIZE as usize] = [D3DVERTEXELEMENT9::default(); MAX_FVF_DECL_SIZE as usize];
            let mut num_elems = 0;

            if let Some(decl) = &VERTEX_GRASS {
                HR!(decl.GetDeclaration(elems.as_mut_ptr(), &mut num_elems));
            }

            let mut grass_mesh = std::ptr::null_mut();
            HR!(D3DXCreateMesh(NUM_GRASS_BLOCKS * 2, NUM_GRASS_BLOCKS * 4, D3DXMESH_MANAGED,
                elems.as_mut_ptr(), d3d_device.clone(), &mut grass_mesh));

            let mut v: *mut c_void = std::ptr::null_mut();
            HR!(ID3DXBaseMesh_LockVertexBuffer(grass_mesh, 0, &mut v));

            let mut k: *mut c_void = std::ptr::null_mut();
            HR!(ID3DXBaseMesh_LockIndexBuffer(grass_mesh, 0, &mut k));

            let mut index_offset = 0;

            // Scale down the region in which we generate grass.
            let w: i32 = (terrain.get_width() * 0.15) as i32;
            let d: i32 = (terrain.get_depth() * 0.15) as i32;

            let mut rng = thread_rng();

            let mut v_offset = 0;
            let mut k_offset = 0;

            // Randomly generate a grass block (three intersecting quads) around the
            // terrain in the height range [35, 50] (similar to the trees).
            for _i in 0..NUM_GRASS_BLOCKS {
                let mut x: f32;
                let mut z: f32;
                let mut y: f32;

                loop {
                    //============================================
                    // Construct vertices.

                    // Generate random position in region.  Note that we also shift
                    // this region to place it in the world.
                    x = ((rng.gen_range(0..32767) % w) as f32 - (w as f32 * 0.5)) - 30.0;
                    z = ((rng.gen_range(0..32767) % d) as f32 - (d as f32 * 0.5)) - 20.0;
                    y = terrain.get_height(x, z);

                    // Only generate grass blocks in this height range.  If the height
                    // is outside this range, generate a new random position and
                    // try again.
                    if !(y < 35.0 || y > 50.0) { // We are trying again if out of valid height range
                        break;
                    }
                }

                let rand_int1 = rng.gen_range(0..32767);
                let sx: f32 = get_random_float(rand_int1, 0.75, 1.25);

                let rand_int2 = rng.gen_range(0..32767);
                let sy: f32 = get_random_float(rand_int2, 0.75, 1.25);

                let rand_int3 = rng.gen_range(0..32767);
                let sz: f32 = get_random_float(rand_int3, 0.75, 1.25);

                let pos = D3DXVECTOR3 { x, y, z };
                let scale = D3DXVECTOR3 { x: sx, y: sy, z: sz };

                PropsDemo::build_grass_fin(&mut rng, v.offset(v_offset),
                                           k.offset(k_offset), &mut index_offset,
                                           pos, scale);

                v_offset += 4 * std::mem::size_of::<VertexGrass>() as isize;
                k_offset += 6 * std::mem::size_of::<u16>() as isize;
            }

            HR!(ID3DXBaseMesh_UnlockVertexBuffer(grass_mesh));
            HR!(ID3DXBaseMesh_UnlockIndexBuffer(grass_mesh));

            let num_faces = ID3DXBaseMesh_GetNumFaces(grass_mesh);

            // Fill in the attribute buffer (everything in subset 0)
            let mut attribute_buffer_ptr = std::ptr::null_mut();
            HR!(ID3DXMesh_LockAttributeBuffer(grass_mesh, 0, &mut attribute_buffer_ptr));
            let att_buffer_slice: &mut[u32] = from_raw_parts_mut(attribute_buffer_ptr, num_faces as usize);

            for i in 0..num_faces as usize {
                att_buffer_slice[i] = 0;
            }

            HR!(ID3DXMesh_UnlockAttributeBuffer(grass_mesh));

            let mut adj: Vec<u32> = Vec::new();
            adj.resize(num_faces as usize * 3, 0);
            HR!(ID3DXBaseMesh_GenerateAdjacency(grass_mesh, EPSILON, adj.as_mut_ptr()));
            HR!(ID3DXMesh_OptimizeInPlace(grass_mesh, D3DXMESHOPT_ATTRSORT | D3DXMESHOPT_VERTEXCACHE,
                adj.as_ptr(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut()));

            grass_mesh
        }
    }

    fn build_grass_fin(rng: &mut ThreadRng, v: *mut c_void /*VertexGrass*/, k: *mut c_void /*u16*/,
                       index_offset: &mut i32, world_pos: D3DXVECTOR3,
                       scale: D3DXVECTOR3) {
        unsafe {
            let mut world_pos_copy = world_pos.clone();

            // Only top vertices have non-zero amplitudes:
            // The bottom vertices are fixed to the ground.

            let rand_int = rng.gen_range(0..32767);
            let amp = get_random_float(rand_int, 0.5, 1.0);

            let mut v_slice: &mut [VertexGrass] = from_raw_parts_mut(v.cast(), 4);
            v_slice[0] = VertexGrass {
                pos: D3DXVECTOR3 { x: -1.0, y: -0.5, z: 0.0 },
                quad_pos: D3DXVECTOR3::default(),
                tex0: D3DXVECTOR2 { x: 0.0, y: 1.0 },
                amplitude: 0.0,
                color_offset: 0
            };

            v_slice[1] = VertexGrass {
                pos: D3DXVECTOR3 { x: -1.0, y: 0.5, z: 0.0 },
                quad_pos: D3DXVECTOR3::default(),
                tex0: D3DXVECTOR2 { x: 0.0, y: 0.0 },
                amplitude: amp,
                color_offset: 0
            };

            v_slice[2] = VertexGrass {
                pos: D3DXVECTOR3 { x: 1.0, y: 0.5, z: 0.0 },
                quad_pos: D3DXVECTOR3::default(),
                tex0: D3DXVECTOR2 { x: 1.0, y: 0.0 },
                amplitude: amp,
                color_offset: 0
            };

            v_slice[3] = VertexGrass {
                pos: D3DXVECTOR3 { x: 1.0, y: -0.5, z: 0.0 },
                quad_pos: D3DXVECTOR3::default(),
                tex0: D3DXVECTOR2 { x: 1.0, y: 1.0 },
                amplitude: 0.0,
                color_offset: 0
            };

            // Set indices of fin.
            let k_slice: &mut[u16] = from_raw_parts_mut(k.cast(), 6);
            k_slice[0] = (0 + *index_offset) as u16;
            k_slice[1] = (1 + *index_offset) as u16;
            k_slice[2] = (2 + *index_offset) as u16;
            k_slice[3] = (0 + *index_offset) as u16;
            k_slice[4] = (2 + *index_offset) as u16;
            k_slice[5] = (3 + *index_offset) as u16;

            // Offset the indices by four to have the indices index into
            // the next four elements of the vertex buffer for the next fin.
            *index_offset += 4;

            // Scale the fins and randomize green color intensity.
            for i in 0..4 {
                v_slice[i].pos.x *= scale.x;
                v_slice[i].pos.y *= scale.y;
                v_slice[i].pos.z *= scale.z;

                // Generate random offset color (mostly green).
                v_slice[i].color_offset = D3DCOLOR_RGBA!(
                    (get_random_float(rng.gen_range(0..32767), 0.0, 0.1) * 255.0) as u32,
                    (get_random_float(rng.gen_range(0..32767), 0.0, 0.2) * 255.0) as u32,
                    (get_random_float(rng.gen_range(0..32767), 0.0, 0.1) * 255.0) as u32,
                    0);
            }

            // Add offset so that the bottom of fin touches the ground
            // when placed on terrain.  Otherwise, the fin's center point
            // will touch the ground and only half of the fin will show.
            let height_over2: f32 = (v_slice[1].pos.y - v_slice[0].pos.y) / 2.0;
            world_pos_copy.y += height_over2;

            // Set world center position for the quad.
            v_slice[0].quad_pos = world_pos_copy.clone();
            v_slice[1].quad_pos = world_pos_copy.clone();
            v_slice[2].quad_pos = world_pos_copy.clone();
            v_slice[3].quad_pos = world_pos_copy.clone();
        }
    }
}