use std::ffi::CStr;
use std::slice::from_raw_parts_mut;
use libc::c_void;
use windows::Win32::Foundation::{PSTR, RECT};
use windows::Win32::Graphics::Direct3D9::*;
use common::*;
use d3dx::*;
use crate::{CAMERA, Camera};

const EPSILON: f32 = 0.001;

mod sub_grid {
    use common::AABB;
    use d3dx::LPD3DXMESH;

    pub const NUM_ROWS: u32  = 33;
    pub const NUM_COLS: u32  = 33;
    pub const NUM_TRIS: u32  = (NUM_ROWS - 1) * (NUM_COLS - 1) * 2;
    pub const NUM_VERTS: u32 = NUM_ROWS * NUM_COLS;

    #[derive(Clone)]
    pub struct SubGrid {
        pub mesh: LPD3DXMESH,
        pub bounding_box: AABB,
    }
}

pub struct Terrain {
    sub_grids: Vec<sub_grid::SubGrid>,

    width: f32,
    depth: f32,

    dx: f32,
    dz: f32,

    heightmap: Heightmap,

    tex0: *mut c_void,       //IDirect3DTexture9,
    tex1: *mut c_void,       //IDirect3DTexture9,
    tex2: *mut c_void,       //IDirect3DTexture9,
    blend_map: *mut c_void,  //IDirect3DTexture9,

    fx: LPD3DXEFFECT,

    h_tech: D3DXHANDLE,
    h_view_proj: D3DXHANDLE,
    h_dir_to_sun_w: D3DXHANDLE,
}

impl Terrain {
    pub fn new(d3d_device: IDirect3DDevice9, vert_rows: u32, vert_cols: u32,
               dx: f32, dz: f32,
               heightmap_file: &str, tex0_file: &str, tex1_file: &str,
               tex2_file: &str, blend_map_file: &str, base_path: &str,
               height_scale: f32, y_offset: f32) -> Terrain {

        let width: f32 = (vert_cols as f32 - 1.0) * dx;
        let depth: f32 = (vert_rows as f32 - 1.0) * dz;

        let mut heightmap = Heightmap::new();
        heightmap.load_raw(vert_rows as i32, vert_cols as i32,
                           c_resource_path(base_path, heightmap_file).as_str(),
                           height_scale, y_offset);

        let mut tex0 = unsafe { std::mem::zeroed() };
        let mut tex1 = unsafe { std::mem::zeroed() };
        let mut tex2 = unsafe { std::mem::zeroed() };
        let mut blend_map = unsafe { std::mem::zeroed() };

        HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
            PSTR(c_resource_path(base_path, tex0_file).as_str().as_ptr() as _), &mut tex0));
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
            PSTR(c_resource_path(base_path, tex1_file).as_str().as_ptr() as _), &mut tex1));
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
            PSTR(c_resource_path(base_path, tex2_file).as_str().as_ptr() as _), &mut tex2));
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
            PSTR(c_resource_path(base_path, blend_map_file).as_str().as_ptr() as _), &mut blend_map));

        let mut sub_grids: Vec<sub_grid::SubGrid> = Vec::new();
        Terrain::build_geometry(d3d_device.clone(), &heightmap, vert_rows, vert_cols,
                                width, depth, dx, dz, &mut sub_grids);

        let (fx,
            h_tech,
            h_view_proj,
            h_dir_to_sun_w,
            h_tex0,
            h_tex1,
            h_tex2,
            h_blend_map) =
            Terrain::build_effect(d3d_device.clone(), base_path);

        HR!(ID3DXBaseEffect_SetTexture(fx, h_tex0, tex0));
        HR!(ID3DXBaseEffect_SetTexture(fx, h_tex1, tex1));
        HR!(ID3DXBaseEffect_SetTexture(fx, h_tex2, tex2));
        HR!(ID3DXBaseEffect_SetTexture(fx, h_blend_map, blend_map));

        Terrain {
            sub_grids,

            heightmap,

            width,
            depth,

            dx,
            dz,

            tex0,
            tex1,
            tex2,
            blend_map,

            fx,

            h_tech,
            h_view_proj,
            h_dir_to_sun_w,
        }
    }

    pub fn release_com_objects(&self) {
        ReleaseCOM(self.fx);

        for sub_grid in &self.sub_grids {
            ReleaseCOM(sub_grid.mesh);
        }

        ReleaseCOM(self.tex0.cast());
        ReleaseCOM(self.tex1.cast());
        ReleaseCOM(self.tex2.cast());
        ReleaseCOM(self.blend_map.cast());
    }

    pub fn on_lost_device(&self) {
        HR!(ID3DXEffect_OnLostDevice(self.fx));
    }

    pub fn on_reset_device(&self) {
        HR!(ID3DXEffect_OnResetDevice(self.fx));
    }

    pub fn set_dir_to_sun_w(&self, d: D3DXVECTOR3) {
        HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_dir_to_sun_w, &d as *const _ as _,
            std::mem::size_of::<D3DXVECTOR3>() as u32));
    }

    pub fn get_num_vertices(&self) -> u32 {
        if self.sub_grids.is_empty() {
            0
        } else {
            let num_vertices: usize = ID3DXBaseMesh_GetNumVertices(self.sub_grids[0].mesh) as usize;
            (self.sub_grids.len() * num_vertices) as u32
        }
    }

    pub fn get_num_triangles(&self) -> u32 {
        if self.sub_grids.is_empty() {
            0
        } else {
            let num_faces: usize = ID3DXBaseMesh_GetNumFaces(self.sub_grids[0].mesh) as usize;
            (self.sub_grids.len() * num_faces) as u32
        }
    }

    pub fn get_height(&self, x: f32, z: f32) -> f32 {
        // Transform from terrain local space to "cell" space.
        let c: f32 = (x + 0.5 * self.width) /  self.dx;
        let d: f32 = (z - 0.5 * self.depth) / -self.dz;

        // Get the row and column we are in.
        let row = d.floor() as usize;
        let col = c.floor() as usize;

        // Grab the heights of the cell we are in.
        // A*--*B
        //  | /|
        //  |/ |
        // C*--*D
        let cell_a = self.heightmap.at(row, col);
        let cell_b = self.heightmap.at(row, col + 1);
        let cell_c = self.heightmap.at(row + 1, col);
        let cell_d = self.heightmap.at(row + 1, col + 1);

        // Where we are relative to the cell.
        let s = c - col as f32;
        let t = d - row as f32;

        // If upper triangle ABC.
        if t < 1.0 - s {
            let uy = cell_b - cell_a;
            let vy = cell_c - cell_a;
            cell_a + s * uy + t * vy
        } else { // lower triangle DCB.
            let uy = cell_c - cell_d;
            let vy = cell_b - cell_d;
            cell_d + (1.0 - s) * uy + (1.0 - t) * vy
        }
    }

    pub fn draw(&self) {
        unsafe {
            let camera: &Camera = &CAMERA.expect("Camera has not been created");

            // Frustum cull sub-grids.
            let mut visible_sub_grids: Vec<sub_grid::SubGrid> = Vec::new();
            for sub_grid in &self.sub_grids {
                if camera.is_visible(&sub_grid.bounding_box) {
                    visible_sub_grids.push(sub_grid.clone())
                }
            }

            // Sort front-to-back from camera.
            visible_sub_grids.sort_by(|a, b| {
                // Sort by distance from nearest to farthest from the camera.  In this
                // way, we draw objects in front to back order to reduce overdraw
                // (i.e., depth test will prevent them from being processed further.
                let mut d1: D3DXVECTOR3 = std::mem::zeroed();
                D3DXVec3Subtract(&mut d1, &a.bounding_box.center(), &camera.get_pos());

                let mut d2: D3DXVECTOR3 = std::mem::zeroed();
                D3DXVec3Subtract(&mut d2, &b.bounding_box.center(), &camera.get_pos());

                let sqlen_a = D3DXVec3LengthSq(&d1);
                let sqlen_b = D3DXVec3LengthSq(&d2);
                sqlen_a.partial_cmp(&sqlen_b).unwrap()
            });

            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_view_proj, camera.get_view_proj()));

            HR!(ID3DXEffect_SetTechnique(self.fx, self.h_tech));

            let mut num_passes: u32 = 0;
            HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));

            HR!(ID3DXEffect_BeginPass(self.fx, 0));

            // dbg!(visible_sub_grids.len());

            for sub_grid in visible_sub_grids {
                HR!(ID3DXBaseMesh_DrawSubset(sub_grid.mesh, 0));
            }

            HR!(ID3DXEffect_EndPass(self.fx));
            HR!(ID3DXEffect_End(self.fx));
        }
    }

    fn build_effect(d3d_device: IDirect3DDevice9, base_path: &str)
        -> (LPD3DXEFFECT, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
            D3DXHANDLE, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE) {
        // Create the FX from a .fx file.
        let mut fx: LPD3DXEFFECT = std::ptr::null_mut();
        let mut errors: LPD3DXBUFFER = std::ptr::null_mut();

        HR!(D3DXCreateEffectFromFile(d3d_device,
            PSTR(c_resource_path(base_path, "Terrain.fx").as_str().as_ptr() as _),
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

    fn build_geometry(d3d_device: IDirect3DDevice9, heightmap: &Heightmap,
                      vert_rows: u32, vert_cols: u32, width: f32, depth: f32,
                      dx: f32, dz: f32,
                      sub_grids: &mut Vec<sub_grid::SubGrid>) {
        unsafe {
            //===============================================================
            // Create one large mesh for the grid in system memory.

            let num_tris = (vert_rows - 1) * (vert_cols - 1) * 2;
            let num_verts = vert_rows * vert_cols;

            let mut elems: [D3DVERTEXELEMENT9; MAX_FVF_DECL_SIZE as usize] = [D3DVERTEXELEMENT9::default(); MAX_FVF_DECL_SIZE as usize];
            let mut num_elems = 0;

            if let Some(decl) = &VERTEX_PNT_DECL {
                HR!(decl.GetDeclaration(elems.as_mut_ptr(), &mut num_elems));
            }

            // Use Scratch pool since we are using this mesh purely for some CPU work,
            // which will be used to create the sub-grids that the graphics card
            // will actually draw.
            let mut mesh = std::ptr::null_mut();
            HR!(D3DXCreateMesh(num_tris as u32, num_verts as u32, D3DPOOL_SCRATCH.0 | D3DXMESH_32BIT,
                elems.as_mut_ptr(), d3d_device.clone(), &mut mesh));

            //===============================================================
            // Write the grid vertices and triangles to the mesh.

            let mut v: *mut c_void = std::ptr::null_mut();
            HR!(ID3DXBaseMesh_LockVertexBuffer(mesh, 0, &mut v));
            let mut v_slice: &mut [VertexPNT] = from_raw_parts_mut(v.cast(), num_verts as usize);

            let mut verts: Vec<D3DXVECTOR3> = Vec::new();
            let mut indices: Vec<u32> = Vec::new();
            gen_tri_grid_32(vert_rows as i32, vert_cols as i32, dx, dz,
                            D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 }, &mut verts, &mut indices);

            let w = width;
            let d = depth;

            let num_vertices: usize = ID3DXBaseMesh_GetNumVertices(mesh) as usize;

            for i in 0..num_vertices as u32 {
                // We store the grid vertices in a linear array, but we can
                // convert the linear array index to an (r, c) matrix index.
                let r = (i / vert_cols) as usize;
                let c = (i % vert_cols) as usize;

                let index = i as usize;
                v_slice[index].pos = verts[index];
                v_slice[index].pos.y = heightmap.at(r, c);

                v_slice[index].tex0.x = (v_slice[index].pos.x + (0.5 * w)) / w;
                v_slice[index].tex0.y = (v_slice[index].pos.z + (0.5 * d)) / -d;
            }

            // Write triangle data so we can compute normals.

            let num_faces: usize = ID3DXBaseMesh_GetNumFaces(mesh) as usize;

            let mut index_buff_ptr: *mut c_void = std::ptr::null_mut();
            HR!(ID3DXBaseMesh_LockIndexBuffer(mesh, 0, &mut index_buff_ptr));
            let indices_slice: &mut[u32] = from_raw_parts_mut(index_buff_ptr.cast(), num_faces as usize * 3);

            for i in 0..num_faces {
                indices_slice[i * 3 + 0] = indices[i * 3 + 0];
                indices_slice[i * 3 + 1] = indices[i * 3 + 1];
                indices_slice[i * 3 + 2] = indices[i * 3 + 2];
            }

            HR!(ID3DXBaseMesh_UnlockIndexBuffer(mesh));

            // Compute Vertex Normals.
            HR!(D3DXComputeNormals(mesh, std::ptr::null()));

            //===============================================================
            // Now break the grid up into subgrid meshes.

            // Find out the number of subgrids we'll have.  For example, if
            // m = 513, n = 257, SUBGRID_VERT_ROWS = SUBGRID_VERT_COLS = 33,
            // then subGridRows = 512/32 = 16 and sibGridCols = 256/32 = 8.
            let sub_grid_rows = (vert_rows - 1) / (sub_grid::NUM_ROWS - 1);
            let sub_grid_cols = (vert_cols - 1) / (sub_grid::NUM_COLS - 1);

            for r in 0..sub_grid_rows {
                for c in 0..sub_grid_cols {
                    // Rectangle that indicates (via matrix indices ij) the
                    // portion of global grid vertices to use for this subgrid.
                    let r: RECT = RECT {
                        left:         (c * (sub_grid::NUM_COLS - 1)) as i32,
                        top:          (r * (sub_grid::NUM_ROWS - 1)) as i32,
                        right:  ((c + 1) * (sub_grid::NUM_COLS - 1)) as i32,
                        bottom: ((r + 1) * (sub_grid::NUM_ROWS - 1)) as i32
                    };

                    Terrain::build_sub_grid_mesh(d3d_device.clone(), r, vert_cols, dx, dz,
                                                 v_slice, sub_grids);
                }
            }

            HR!(ID3DXBaseMesh_UnlockVertexBuffer(mesh));

            ReleaseCOM(mesh);
        }
    }

    fn build_sub_grid_mesh(d3d_device: IDirect3DDevice9, r: RECT, vert_cols: u32, dx: f32, dz: f32,
                           grid_verts: &mut [VertexPNT], sub_grids: &mut Vec<sub_grid::SubGrid>) {
        unsafe {
            //===============================================================
            // Get indices for subgrid (we don't use the verts here--the verts
            // are given by the parameter gridVerts).

            let mut temp_verts: Vec<D3DXVECTOR3> = Vec::new();
            let mut temp_indices: Vec<u32> = Vec::new();

            gen_tri_grid_32(sub_grid::NUM_ROWS as i32, sub_grid::NUM_COLS as i32,
                            dx, dz, D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 },
                            &mut temp_verts, &mut temp_indices);

            let mut sub_mesh = std::mem::zeroed();
            let mut elems: [D3DVERTEXELEMENT9; MAX_FVF_DECL_SIZE as usize] = [D3DVERTEXELEMENT9::default(); MAX_FVF_DECL_SIZE as usize];
            let mut num_elems = 0;

            if let Some(decl) = &VERTEX_PNT_DECL {
                HR!(decl.GetDeclaration(elems.as_mut_ptr(), &mut num_elems));
            }

            HR!(D3DXCreateMesh(sub_grid::NUM_TRIS, sub_grid::NUM_VERTS, D3DXMESH_MANAGED,
                    elems.as_mut_ptr(), d3d_device.clone(), &mut sub_mesh));

            //===============================================================
            // Build Vertex Buffer.  Copy rectangle of vertices from the
            // grid into the subgrid structure.

            let mut v: *mut c_void = std::ptr::null_mut();
            HR!(ID3DXBaseMesh_LockVertexBuffer(sub_mesh, 0, &mut v));
            let v_slice: &mut [VertexPNT] = from_raw_parts_mut(v.cast(), sub_grid::NUM_VERTS as usize);

            let mut k: usize = 0;
            for i in r.top..=r.bottom {
                for j in r.left..=r.right {
                    let index = (i * vert_cols as i32 + j) as usize;
                    v_slice[k] = grid_verts[index].clone();
                    k += 1;
                }
            }

            //===============================================================
            // Compute the bounding box before unlocking the vertex buffer.

            let mut bnd_box: AABB = AABB::default();

            let sub_mesh_num_vertices = ID3DXBaseMesh_GetNumVertices(sub_mesh) as usize;

            HR!(D3DXComputeBoundingBox(v.cast(), sub_mesh_num_vertices as u32,
                std::mem::size_of::<VertexPNT>() as u32,
                &mut bnd_box.min_pt, &mut bnd_box.max_pt));

            HR!(ID3DXBaseMesh_UnlockVertexBuffer(sub_mesh));

            //===============================================================
            // Build Index and Attribute Buffer.

            let mut indices: *mut c_void = std::ptr::null_mut();
            HR!(ID3DXBaseMesh_LockIndexBuffer(sub_mesh, 0, &mut indices));
            let indices_slice: &mut[u16] = from_raw_parts_mut(indices.cast(), sub_grid::NUM_TRIS as usize * 3);

            let mut att_buffer = std::ptr::null_mut();
            HR!(ID3DXMesh_LockAttributeBuffer(sub_mesh, 0, &mut att_buffer));
            let att_buffer_slice: &mut[u32] = from_raw_parts_mut(att_buffer, sub_grid::NUM_TRIS as usize);

            for i in 0..sub_grid::NUM_TRIS as usize {
                indices_slice[i * 3 + 0] = temp_indices[i * 3 + 0] as u16;
                indices_slice[i * 3 + 1] = temp_indices[i * 3 + 1] as u16;
                indices_slice[i * 3 + 2] = temp_indices[i * 3 + 2] as u16;

                att_buffer_slice[i] = 0; // Always subset 0
            }

            HR!(ID3DXBaseMesh_UnlockIndexBuffer(sub_mesh));
            HR!(ID3DXMesh_UnlockAttributeBuffer(sub_mesh));

            //===============================================================
            // Optimize for the vertex cache and build attribute table.

            let sub_mesh_num_faces = ID3DXBaseMesh_GetNumFaces(sub_mesh) * 3;

            let mut adj: Vec<u32> = Vec::new();
            adj.resize(sub_mesh_num_faces as usize * 3, 0);
            HR!(ID3DXBaseMesh_GenerateAdjacency(sub_mesh, EPSILON, adj.as_mut_ptr()));
            HR!(ID3DXMesh_OptimizeInPlace(sub_mesh, D3DXMESHOPT_VERTEXCACHE | D3DXMESHOPT_ATTRSORT,
                adj.as_ptr(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut()));

            //===============================================================
            // Save the mesh and bounding box.

            let g = sub_grid::SubGrid {
                mesh: sub_mesh,
                bounding_box: bnd_box.clone(),
            };

            sub_grids.push(g);
        }
    }
}