use std::slice::from_raw_parts_mut;
use libc::c_void;
use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D9::*, Win32::System::SystemServices::*,
};

use crate::*;
use crate::heightmap::Heightmap;

const BASE_PATH: &str = "luna_33_basic_terrain_demo/";

const EPSILON: f32 = 0.001;

// Sample demo
pub struct BasicTerrainDemo {
    d3d_pp: *const D3DPRESENT_PARAMETERS,
    gfx_stats: Option<GfxStats>,

    terrain_mesh: LPD3DXMESH,

    tex0: *mut c_void,      //IDirect3DTexture9,
    tex1: *mut c_void,      //IDirect3DTexture9,
    tex2: *mut c_void,      //IDirect3DTexture9,
    blend_map: *mut c_void, //IDirect3DTexture9,

    fx: LPD3DXEFFECT,

    h_tech: D3DXHANDLE,
    h_view_proj: D3DXHANDLE,

    camera_rotation_y: f32,
    camera_radius: f32,
    camera_height: f32,

    view: D3DXMATRIX,
    proj: D3DXMATRIX,
}

impl BasicTerrainDemo {
    pub fn new(d3d_device: IDirect3DDevice9, d3d_pp: *const D3DPRESENT_PARAMETERS) -> Option<BasicTerrainDemo> {
        if !BasicTerrainDemo::check_device_caps() {
            display_error_then_quit("checkDeviceCaps() Failed");
        }

        init_all_vertex_declarations(d3d_device.clone());

        let mut gfx_stats = GfxStats::new(d3d_device.clone(), D3DCOLOR_XRGB!(0, 0, 0));

        let mut world = unsafe { std::mem::zeroed() };
        D3DXMatrixIdentity(&mut world);

        let mut heightmap = Heightmap::new();
        heightmap.load_raw(129, 129, c_resource_path(BASE_PATH, "heightmap17_129.raw").as_str(), 0.25, 0.0);

        // Load textures from file.
        let mut tex0 = unsafe { std::mem::zeroed() };
        let mut tex1 = unsafe { std::mem::zeroed() };
        let mut tex2 = unsafe { std::mem::zeroed() };
        let mut blend_map = unsafe { std::mem::zeroed() };

        HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
            PSTR(c_resource_path(BASE_PATH, "grass.dds").as_str().as_ptr() as _), &mut tex0));
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
            PSTR(c_resource_path(BASE_PATH, "dirt.dds").as_str().as_ptr() as _), &mut tex1));
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
            PSTR(c_resource_path(BASE_PATH, "stone.dds").as_str().as_ptr() as _), &mut tex2));
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
            PSTR(c_resource_path(BASE_PATH, "blend_hm17.dds").as_str().as_ptr() as _), &mut blend_map));

        let terrain_mesh = BasicTerrainDemo::build_grid_geometry(d3d_device.clone(), &heightmap);

        if let Some(gfx_stats) = &mut gfx_stats {
            gfx_stats.add_vertices(ID3DXBaseMesh_GetNumVertices(terrain_mesh));
            gfx_stats.add_triangles(ID3DXBaseMesh_GetNumFaces(terrain_mesh));
        }

        let (fx,
            h_tech,
            h_view_proj,
            h_dir_to_sun_w,
            h_tex0,
            h_tex1,
            h_tex2,
            h_blend_map) =
            BasicTerrainDemo::build_fx(d3d_device.clone());

        HR!(ID3DXBaseEffect_SetTexture(fx, h_tex0, tex0));
        HR!(ID3DXBaseEffect_SetTexture(fx, h_tex1, tex1));
        HR!(ID3DXBaseEffect_SetTexture(fx, h_tex2, tex2));
        HR!(ID3DXBaseEffect_SetTexture(fx, h_blend_map, blend_map));

        let d = D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 };
        HR!(ID3DXBaseEffect_SetValue(fx, h_dir_to_sun_w, &d as *const _ as _, std::mem::size_of::<D3DXVECTOR3>() as u32));

        let mut basic_terrain_demo = BasicTerrainDemo {
            d3d_pp,
            gfx_stats,

            terrain_mesh,

            tex0,
            tex1,
            tex2,
            blend_map,

            fx,

            h_tech,
            h_view_proj,

            camera_radius: 80.0,
            camera_rotation_y: 1.2 * D3DX_PI,
            camera_height: 40.0,

            view: unsafe { std::mem::zeroed() },
            proj: unsafe { std::mem::zeroed() },
        };

        basic_terrain_demo.on_reset_device();

        Some(basic_terrain_demo)
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        ReleaseCOM(self.fx);
        ReleaseCOM(self.terrain_mesh);

        ReleaseCOM(self.tex0.cast());
        ReleaseCOM(self.tex1.cast());
        ReleaseCOM(self.tex2.cast());
        ReleaseCOM(self.blend_map.cast());

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

        HR!(ID3DXEffect_OnLostDevice(self.fx));
    }

    pub fn on_reset_device(&mut self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.on_reset_device();
        }

        HR!(ID3DXEffect_OnResetDevice(self.fx));

        // The aspect ratio depends on the backbuffer dimensions, which can
        // possibly change after a reset.  So rebuild the projection matrix.
        self.build_proj_mtx();
    }

    pub fn update_scene(&mut self, dt: f32) {
        if let Some(gfx_stats) = &mut self.gfx_stats {
            gfx_stats.update(dt);
        }

        // Get snapshot of input devices.
        unsafe {
            if let Some(dinput) = &mut DIRECT_INPUT {
                dinput.poll();

                // Check input.
                if dinput.key_down(DIK_W as usize) {
                    self.camera_height += 25.0 * dt;
                }

                if dinput.key_down(DIK_S as usize) {
                    self.camera_height -= 25.0 * dt;
                }

                // Divide by 50 to make mouse less sensitive.
                self.camera_rotation_y += dinput.mouse_dx() / 100.0;
                self.camera_radius += dinput.mouse_dy() / 25.0;

                // If we rotate over 360 degrees, just roll back to 0
                if self.camera_rotation_y.abs() >= 2.0 * D3DX_PI {
                    self.camera_rotation_y = 0.0;
                }

                // Don't let radius get too small.
                if self.camera_radius < 5.0 {
                    self.camera_radius = 5.0;
                }

                // The camera position/orientation relative to world space can
                // change every frame based on input, so we need to rebuild the
                // view matrix every frame with the latest changes.
                self.build_view_mtx();
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

                let mut view_proj: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixMultiply(&mut view_proj, &self.view, &self.proj);
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_view_proj, &view_proj));

                HR!(ID3DXEffect_SetTechnique(self.fx, self.h_tech));

                let mut num_passes: u32 = 0;
                HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));

                HR!(ID3DXEffect_BeginPass(self.fx, 0));

                HR!(ID3DXBaseMesh_DrawSubset(self.terrain_mesh, 0));

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

    fn build_grid_geometry(d3d_device: IDirect3DDevice9, heightmap: &Heightmap) -> LPD3DXMESH {
        unsafe {
            let mut verts: Vec<D3DXVECTOR3> = Vec::new();
            let mut indices: Vec<u16> = Vec::new();

            let vert_rows = 129;
            let vert_cols = 129;
            let dx = 1.0;
            let dz = 1.0;
            gen_tri_grid(vert_rows, vert_cols, dx, dz,
                         D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 }, &mut verts, &mut indices);

            let num_verts = vert_rows * vert_cols;
            let num_tris = (vert_rows - 1) * (vert_cols - 1) * 2;

            // Create the mesh.
            let mut elems: [D3DVERTEXELEMENT9; MAX_FVF_DECL_SIZE as usize] = [D3DVERTEXELEMENT9::default(); MAX_FVF_DECL_SIZE as usize];
            let mut num_elems = 0;

            if let Some(decl) = &VERTEX_PNT_DECL {
                HR!(decl.GetDeclaration(elems.as_mut_ptr(), &mut num_elems));
            }

            let mut terrain_mesh = std::mem::zeroed();
            HR!(D3DXCreateMesh(num_tris as u32, num_verts as u32, D3DXMESH_MANAGED, elems.as_mut_ptr(),
                d3d_device.clone(), &mut terrain_mesh));

            // Write the vertices.
            let mut v: *mut c_void = std::ptr::null_mut();
            HR!(ID3DXBaseMesh_LockVertexBuffer(terrain_mesh, 0, &mut v));
            let mut v_slice: &mut [VertexPNT] = from_raw_parts_mut(v.cast(), num_verts as usize);

            // width/depth
            let w = (vert_cols as f32 - 1.0) * dx;
            let d = (vert_rows as f32 - 1.0) * dz;

            for i in 0..vert_rows as usize {
                for j in 0..vert_cols as usize {
                    let index: usize = i * vert_cols as usize + j;
                    v_slice[index].pos = verts[index].clone();
                    v_slice[index].pos.y = heightmap.at(i, j);
                    v_slice[index].normal = D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 };
                    v_slice[index].tex0.x = (v_slice[index].pos.x + (0.5 * w)) / w;
                    v_slice[index].tex0.y = (v_slice[index].pos.z - (0.5 * d)) / -d;
                }
            }

            HR!(ID3DXBaseMesh_UnlockVertexBuffer(terrain_mesh));

            // Write the indices and attribute buffer.
            let mut k: *mut c_void = std::ptr::null_mut();
            HR!(ID3DXBaseMesh_LockIndexBuffer(terrain_mesh, 0, &mut k));
            let k_slice: &mut[u16] = from_raw_parts_mut(k.cast(), num_tris as usize * 3);

            let mut att_buffer = std::ptr::null_mut();
            HR!(ID3DXMesh_LockAttributeBuffer(terrain_mesh, 0, &mut att_buffer));
            let att_buffer_slice: &mut[u32] = from_raw_parts_mut(att_buffer, num_tris as usize);

            // Compute the indices for each triangle.
            for i in 0..num_tris as usize {
                k_slice[i * 3 + 0] = indices[i * 3 + 0];
                k_slice[i * 3 + 1] = indices[i * 3 + 1];
                k_slice[i * 3 + 2] = indices[i * 3 + 2];

                att_buffer_slice[i] = 0; // Always subset 0
            }

            HR!(ID3DXBaseMesh_UnlockIndexBuffer(terrain_mesh));
            HR!(ID3DXMesh_UnlockAttributeBuffer(terrain_mesh));

            // Generate normals and then optimize the mesh.

            HR!(D3DXComputeNormals(terrain_mesh, std::ptr::null()));

            let terrain_num_faces = ID3DXBaseMesh_GetNumFaces(terrain_mesh) * 3;

            let mut adj: Vec<u32> = Vec::new();
            adj.resize(terrain_num_faces as usize, 0);
            HR!(ID3DXBaseMesh_GenerateAdjacency(terrain_mesh, EPSILON, adj.as_mut_ptr()));
            HR!(ID3DXMesh_OptimizeInPlace(terrain_mesh, D3DXMESHOPT_VERTEXCACHE | D3DXMESHOPT_ATTRSORT,
                adj.as_ptr(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut()));

            terrain_mesh
        }
    }

    fn build_fx(d3d_device: IDirect3DDevice9) -> (LPD3DXEFFECT, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
                                                  D3DXHANDLE, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE) {
        // Create the FX from a .fx file.
        let mut fx: LPD3DXEFFECT = std::ptr::null_mut();
        let mut errors: LPD3DXBUFFER = std::ptr::null_mut();

        HR!(D3DXCreateEffectFromFile(d3d_device,
            PSTR(c_resource_path(BASE_PATH, "Terrain.fx").as_str().as_ptr() as _),
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
        let h_tech = ID3DXBaseEffect_GetTechniqueByName(fx, PSTR(b"TerrainTech\0".as_ptr() as _));
        let h_view_proj = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gViewProj\0".as_ptr() as _));
        let h_dir_to_sun_w = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gDirToSunW\0".as_ptr() as _));
        let h_tex0 = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gTex0\0".as_ptr() as _));
        let h_tex1 = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gTex1\0".as_ptr() as _));
        let h_tex2 = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gTex2\0".as_ptr() as _));
        let h_blend_map = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gBlendMap\0".as_ptr() as _));

        (fx, h_tech, h_view_proj, h_dir_to_sun_w, h_tex0, h_tex1, h_tex2, h_blend_map)
    }

    fn build_view_mtx(&mut self) {
        let x: f32 = self.camera_radius * self.camera_rotation_y.cos();
        let z: f32 = self.camera_radius * self.camera_rotation_y.sin();
        let pos = D3DXVECTOR3 { x, y: self.camera_height, z };
        let target = D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 };
        let up = D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 };
        D3DXMatrixLookAtLH(&mut self.view, &pos, &target, &up);
    }

    fn build_proj_mtx(&mut self) {
        let w: f32 = (unsafe { *self.d3d_pp }).BackBufferWidth as f32;
        let h: f32 = (unsafe { *self.d3d_pp }).BackBufferHeight as f32;
        D3DXMatrixPerspectiveFovLH(&mut self.proj, D3DX_PI * 0.25, w / h, 1.0, 5000.0);
    }
}