use std::ffi::CStr;
use libc::c_void;
use windows::{Win32::Foundation::*, Win32::Graphics::Direct3D9::*};
use common::mtrl::Mtrl;

use crate::*;
use crate::sky::Sky;

pub const BASE_PATH: &str = "luna_46_bricks_demo/";

// Directional Light
#[repr(C)]
struct DirLight {
    ambient: D3DXCOLOR,
    diffuse: D3DXCOLOR,
    spec: D3DXCOLOR,
    dir_w: D3DXVECTOR3,
}

// Sample demo
pub struct NormalMapDemo {
    d3d_pp: *const D3DPRESENT_PARAMETERS,
    gfx_stats: Option<GfxStats>,

    sky: Sky,

    scene_mesh: LPD3DXMESH,
    scene_world: D3DXMATRIX,
    scene_mtrls: Vec<Mtrl>,
    scene_textures: Vec<*mut c_void>,

    // Hack for this particular scene--usually you'd want to come up
    // with a more general method of loading normal maps such that
    // the ith normal map corresponds with the ith mesh subset.
    // For example, you might call each color texture name_color and
    // its corresponding normal map name_nmap.  Then when you load the
    // name_color texture you also load the corresponding normal map.
    // If a texture doesn't have a normal map, you could use a default one
    // like we use the default white texture.
    scene_normal_maps: [*mut c_void; 2], // [IDirect3DTexture9]

    white_tex: *mut c_void, // IDirect3DTexture9

    // General light/texture FX
    fx: LPD3DXEFFECT,
    h_wvp: D3DXHANDLE,
    h_eye_pos_w: D3DXHANDLE,
    h_tex: D3DXHANDLE,
    h_mtrl: D3DXHANDLE,
    h_light: D3DXHANDLE,
    h_normal_map: D3DXHANDLE,

    light: DirLight,
}

impl NormalMapDemo {
    pub fn new(d3d_device: IDirect3DDevice9, d3d_pp: *const D3DPRESENT_PARAMETERS) -> Option<NormalMapDemo> {
        unsafe {
            if !NormalMapDemo::check_device_caps() {
                display_error_then_quit("checkDeviceCaps() Failed");
            }

            init_all_vertex_declarations(d3d_device.clone());

            let mut gfx_stats = GfxStats::new(d3d_device.clone(), D3DCOLOR_XRGB!(0, 0, 0));

            let sky = Sky::new(BASE_PATH, d3d_device.clone(),
                               "grassenvmap1024.dds", 10000.0);

            let light = DirLight {
                ambient: D3DXCOLOR { r: 0.4, g: 0.4, b: 0.4, a: 1.0 },
                diffuse: D3DXCOLOR { r: 0.8, g: 0.8, b: 0.8, a: 1.0 },
                spec: D3DXCOLOR { r: 0.6, g: 0.6, b: 0.6, a: 1.0 },
                dir_w: D3DXVECTOR3::default(),
            };

            let (temp_mesh, scene_mtrls, scene_textures) =
                load_x_file(BASE_PATH, "BasicColumnScene.x", d3d_device.clone());

            // Get the vertex declaration for the NMapVertex.
            let mut elems: [D3DVERTEXELEMENT9; MAX_FVF_DECL_SIZE as usize] = [D3DVERTEXELEMENT9::default(); MAX_FVF_DECL_SIZE as usize];
            let mut num_elems: u32 = 0;
            if let Some(decl) = &NMAP_VERTEX_DECL {
                HR!(decl.GetDeclaration(elems.as_mut_ptr(), &mut num_elems));
            }

            // Clone the mesh to the NMapVertex format.
            let mut cloned_temp_mesh: LPD3DXMESH = std::ptr::null_mut();
            HR!(ID3DXBaseMesh_CloneMesh(temp_mesh, D3DXMESH_MANAGED, elems.as_mut_ptr(),
                d3d_device.clone(), &mut cloned_temp_mesh));

            // Now use D3DXComputeTangentFrameEx to build the TNB-basis for each vertex
            // in the mesh.
            let mut scene_mesh: LPD3DXMESH = std::ptr::null_mut();
            HR!(D3DXComputeTangentFrameEx(
                cloned_temp_mesh,                   // Input mesh
                D3DDECLUSAGE_TEXCOORD.0 as u32, 0,  // Vertex element of input tex-coords.
                D3DDECLUSAGE_BINORMAL.0 as u32, 0,  // Vertex element to output binormal.
                D3DDECLUSAGE_TANGENT.0 as u32, 0,   // Vertex element to output tangent.
                D3DDECLUSAGE_NORMAL.0 as u32, 0,    // Vertex element to output normal.
                0,                                  // Options
                std::ptr::null(),                   // Adjacency
                0.01, 0.25, 0.01,                   // Thresholds for handling errors
                &mut scene_mesh,                    // Output mesh
                std::ptr::null_mut()));             // Vertex Remapping

            // Done with temps.
            ReleaseCOM(temp_mesh);
            ReleaseCOM(cloned_temp_mesh);

            let mut scene_world = D3DXMATRIX::default();
            D3DXMatrixIdentity(&mut scene_world);

            let mut scene_world_inv = D3DXMATRIX::default();
            D3DXMatrixIdentity(&mut scene_world_inv);

            let mut scene_normal_maps = [std::ptr::null_mut(); 2];
            HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
                PSTR(c_resource_path(BASE_PATH, "floor_nmap.bmp").as_str().as_ptr() as _), &mut scene_normal_maps[0]));
            HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
                PSTR(c_resource_path(BASE_PATH, "bricks_nmap.bmp").as_str().as_ptr() as _), &mut scene_normal_maps[1]));

            let mut white_tex = std::mem::zeroed();
            HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
                PSTR(c_resource_path(BASE_PATH, "whitetex.dds").as_str().as_ptr() as _), &mut white_tex));

            // Initialize camera.
            if let Some(camera) = &mut CAMERA {
                camera.set_pos(D3DXVECTOR3 { x: 0.0, y: 3.0, z: -10.0 });
                camera.set_speed(10.0);
            }

            if let Some(gfx_stats) = &mut gfx_stats {
                gfx_stats.add_vertices(ID3DXBaseMesh_GetNumVertices(scene_mesh));
                gfx_stats.add_triangles(ID3DXBaseMesh_GetNumFaces(scene_mesh));

                gfx_stats.add_vertices(sky.get_num_vertices());
                gfx_stats.add_triangles(sky.get_num_triangles());
            }

            let (fx,
                h_tech,
                h_wvp,
                h_world_inv,
                h_eye_pos_w,
                h_tex,
                h_mtrl,
                h_light,
                h_normal_map)
                = NormalMapDemo::build_fx(d3d_device.clone());

            // Set parameters that do not vary:

            // World is the identity, so inverse-transpose also identity.
            HR!(ID3DXBaseEffect_SetMatrix(fx, h_world_inv, &scene_world_inv));
            HR!(ID3DXEffect_SetTechnique(fx, h_tech));

            let mut normal_map_demo = NormalMapDemo {
                d3d_pp,
                gfx_stats,

                sky,

                scene_mesh,
                scene_world,
                scene_mtrls,
                scene_textures,

                scene_normal_maps,

                white_tex,

                fx,
                h_wvp,
                h_eye_pos_w,
                h_tex,
                h_mtrl,
                h_light,
                h_normal_map,

                light,
            };

            normal_map_demo.on_reset_device();

            Some(normal_map_demo)
        }
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        self.sky.release_com_objects();

        ReleaseCOM(self.fx);

        ReleaseCOM(self.scene_mesh);
        for tex in &self.scene_textures {
            ReleaseCOM(tex.cast());
        }

        ReleaseCOM(self.white_tex);
        ReleaseCOM(self.scene_normal_maps[0]);
        ReleaseCOM(self.scene_normal_maps[1]);

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

        self.sky.on_lost_device();
        HR!(ID3DXEffect_OnLostDevice(self.fx));
    }

    pub fn on_reset_device(&mut self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.on_reset_device();
        }

        self.sky.on_reset_device();
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


            // Animate light by spinning it around.

            static mut TIME: f32 = 0.0;
            TIME += dt;

            self.light.dir_w = D3DXVECTOR3 { x: 5.0 * TIME.cos(), y: -1.0, z: 5.0 * TIME.sin() };
            D3DXVec3Normalize(&mut self.light.dir_w, &self.light.dir_w);
        }
    }

    pub fn draw_scene(&mut self) {
        unsafe {
            if let Some(d3d_device) = &D3D_DEVICE {
                let camera: &Camera = &CAMERA.expect("Camera has not been created");

                HR!(d3d_device.BeginScene());

                // Draw sky first--this also replaces our gd3dDevice->Clear call.
                self.sky.draw();

                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_light, &self.light as *const _ as _,
                    std::mem::size_of::<DirLight>() as u32));

                let mut wvp: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixMultiply(&mut wvp, &self.scene_world, camera.get_view_proj());
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &wvp));

                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_eye_pos_w,
                    &camera.get_pos() as *const _ as _, std::mem::size_of::<D3DXVECTOR3>() as u32));

                let mut num_passes: u32 = 0;
                HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));
                HR!(ID3DXEffect_BeginPass(self.fx, 0));

                for j in 0..self.scene_mtrls.len() {
                    HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_mtrl,
                        &self.scene_mtrls[j] as *const _ as _, std::mem::size_of::<Mtrl>() as u32));

                    // If there is a texture, then use.
                    if !self.scene_textures[j].is_null() {
                        HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.scene_textures[j]));
                    } else {
                        // But if not, then set a pure white texture.  When the texture color
                        // is multiplied by the color from lighting, it is like multiplying by
                        // 1 and won't change the color from lighting.

                        HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.white_tex));
                    }

                    HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_normal_map, self.scene_normal_maps[j]));

                    HR!(ID3DXEffect_CommitChanges(self.fx));
                    HR!(ID3DXBaseMesh_DrawSubset(self.scene_mesh, j as u32));
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

    fn build_fx(d3d_device: IDirect3DDevice9) -> (LPD3DXEFFECT, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
                                                  D3DXHANDLE, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
                                                  D3DXHANDLE) {
        // Create the FX from a .fx file.
        let mut fx: LPD3DXEFFECT = std::ptr::null_mut();
        let mut errors: LPD3DXBUFFER = std::ptr::null_mut();

        HR!(D3DXCreateEffectFromFile(d3d_device,
            PSTR(c_resource_path(BASE_PATH, "NormalMap.fx").as_str().as_ptr() as _),
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
        let h_tech = ID3DXBaseEffect_GetTechniqueByName(fx, PSTR(b"NormalMapTech\0".as_ptr() as _));
        let h_wvp = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWVP\0".as_ptr() as _));
        let h_world_inv = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWorldInv\0".as_ptr() as _));
        let h_mtrl = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gMtrl\0".as_ptr() as _));
        let h_light = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gLight\0".as_ptr() as _));
        let h_eye_pos_w = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gEyePosW\0".as_ptr() as _));
        let h_tex = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gTex\0".as_ptr() as _));
        let h_normal_map = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gNormalMap\0".as_ptr() as _));

        (fx, h_tech, h_wvp, h_world_inv, h_eye_pos_w, h_tex, h_mtrl, h_light, h_normal_map)
    }
}