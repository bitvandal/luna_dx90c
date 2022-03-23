use std::ffi::{c_void, CStr};
use std::slice::from_raw_parts_mut;
use windows::Win32::Foundation::PSTR;
use windows::Win32::Graphics::Direct3D9::*;
use common::*;
use d3dx::*;
use crate::{Camera, CAMERA};

const EPSILON: f32 = 0.001;

pub struct Water {
    mesh: LPD3DXMESH,
    to_world: D3DXMATRIX,
    fx: LPD3DXEFFECT,

    h_wvp: D3DXHANDLE,
    h_eye_pos_w: D3DXHANDLE,
}

impl Water {
    pub fn new(base_path: &str, m: i32, n: i32, dx: f32, dz: f32,
               to_world: &D3DXMATRIX, d3d_device: &IDirect3DDevice9) -> Water {
        unsafe {
            // let vert_rows = m;
            // let vert_cols = n;

            // let width: f32 = (m - 1) as f32 * dz;
            // let depth: f32 = (n - 1) as f32 * dx;

            let num_tris = (m - 1) * (n - 1) * 2;
            let num_verts = m * n;

            //===============================================================
            // Allocate the mesh.

            let mut elems: [D3DVERTEXELEMENT9; MAX_FVF_DECL_SIZE as usize] = [D3DVERTEXELEMENT9::default(); MAX_FVF_DECL_SIZE as usize];
            let mut num_elements: u32 = 0;

            if let Some(decl) = &VERTEX_POS_DECL {
                HR!(decl.GetDeclaration(elems.as_mut_ptr(), &mut num_elements));
            }

            let mut mesh = std::ptr::null_mut();
            HR!(D3DXCreateMesh(num_tris as u32, num_verts as u32, D3DXMESH_MANAGED,
                elems.as_mut_ptr(), d3d_device.clone(), &mut mesh));

            //===============================================================
            // Write the grid vertices and triangles to the mesh.

            let mut v: *mut c_void = std::ptr::null_mut();
            HR!(ID3DXBaseMesh_LockVertexBuffer(mesh, 0, &mut v));
            let mut v_slice: &mut [VertexPos] = from_raw_parts_mut(v.cast(), num_verts as usize);

            let mut verts: Vec<D3DXVECTOR3> = Vec::new();
            let mut indices: Vec<u32> = Vec::new();
            gen_tri_grid_32(m, n, dx, dz,
                            D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 }, &mut verts, &mut indices);

            let num_vertices: usize = ID3DXBaseMesh_GetNumVertices(mesh) as usize;

            for i in 0..num_vertices {
                v_slice[i].pos = verts[i];
            }

            HR!(ID3DXBaseMesh_UnlockVertexBuffer(mesh));

            //===============================================================
            // Write triangle data so we can compute normals.

            let num_faces: usize = ID3DXBaseMesh_GetNumFaces(mesh) as usize;

            let mut index_buff_ptr: *mut c_void = std::ptr::null_mut();
            HR!(ID3DXBaseMesh_LockIndexBuffer(mesh, 0, &mut index_buff_ptr));
            let indices_slice: &mut[u16] = from_raw_parts_mut(index_buff_ptr.cast(), num_faces * 3);

            let mut att_buffer = std::ptr::null_mut();
            HR!(ID3DXMesh_LockAttributeBuffer(mesh, 0, &mut att_buffer));
            let att_buffer_slice: &mut[u32] = from_raw_parts_mut(att_buffer, num_faces);

            for i in 0..num_faces as usize {
                indices_slice[i * 3 + 0] = indices[i * 3 + 0] as u16;
                indices_slice[i * 3 + 1] = indices[i * 3 + 1] as u16;
                indices_slice[i * 3 + 2] = indices[i * 3 + 2] as u16;

                att_buffer_slice[i] = 0; // Always subset 0
            }

            HR!(ID3DXBaseMesh_UnlockIndexBuffer(mesh));
            HR!(ID3DXMesh_UnlockAttributeBuffer(mesh));

            //===============================================================
            // Optimize for the vertex cache and build attribute table.

            let mut adj: Vec<u32> = Vec::new();
            adj.resize((ID3DXBaseMesh_GetNumFaces(mesh) * 3) as usize, 0) ;
            HR!(ID3DXBaseMesh_GenerateAdjacency(mesh, EPSILON, adj.as_mut_ptr()));
            HR!(ID3DXMesh_OptimizeInPlace(mesh, D3DXMESHOPT_VERTEXCACHE | D3DXMESHOPT_ATTRSORT,
                adj.as_ptr(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut()));

            //===============================================================
            // Build the water effect.

            let (fx,
                h_tech,
                h_wvp,
                h_world,
                h_eye_pos_w)
                = Water::build_fx(base_path, d3d_device.clone());

            // We don't need to set these every frame since they do not change.
            HR!(ID3DXEffect_SetTechnique(fx, h_tech));
            HR!(ID3DXBaseEffect_SetMatrix(fx, h_world, to_world));

            Water {
                mesh,
                to_world: to_world.clone(),
                fx,
                h_wvp,
                h_eye_pos_w,
            }
        }
    }

    pub fn release_com_objects(&self) {
        ReleaseCOM(self.mesh);
        ReleaseCOM(self.fx);
    }

    pub fn get_num_triangles(&self) -> u32 {
        ID3DXBaseMesh_GetNumFaces(self.mesh)
    }

    pub fn get_num_vertices(&self) -> u32 {
        ID3DXBaseMesh_GetNumVertices(self.mesh)
    }

    pub fn on_lost_device(&self) {
        HR!(ID3DXEffect_OnLostDevice(self.fx));
    }

    pub fn on_reset_device(&mut self) {
        HR!(ID3DXEffect_OnResetDevice(self.fx));
    }

    pub fn update(&mut self, _dt: f32) {
    }

    pub fn draw(&mut self) {
        unsafe {
            let camera: &Camera = &CAMERA.expect("Camera has not been created");

            let mut wvp: D3DXMATRIX = std::mem::zeroed();
            D3DXMatrixMultiply(&mut wvp, &self.to_world, camera.get_view_proj());
            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &wvp));

            HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_eye_pos_w,
                    &camera.get_pos() as *const _ as _,
                    std::mem::size_of::<D3DXVECTOR3>() as u32));

            let mut num_passes: u32 = 0;
            HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));
            HR!(ID3DXEffect_BeginPass(self.fx, 0));

            HR!(ID3DXBaseMesh_DrawSubset(self.mesh, 0));

            HR!(ID3DXEffect_EndPass(self.fx));
            HR!(ID3DXEffect_End(self.fx));
        }
    }

    fn build_fx(base_path: &str, d3d_device: IDirect3DDevice9)
        -> (LPD3DXEFFECT, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
            D3DXHANDLE) {
        // Create the generic Light & Tex FX from a .fx file.
        let mut fx: LPD3DXEFFECT = std::ptr::null_mut();
        let mut errors: LPD3DXBUFFER = std::ptr::null_mut();

        HR!(D3DXCreateEffectFromFile(d3d_device,
            PSTR(c_resource_path(base_path, "Water.fx").as_str().as_ptr() as _),
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
        let h_tech = ID3DXBaseEffect_GetTechniqueByName(fx, PSTR(b"WaterTech\0".as_ptr() as _));
        let h_wvp = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWVP\0".as_ptr() as _));
        let h_world = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWorld\0".as_ptr() as _));
        let h_eye_pos_w = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gEyePosW\0".as_ptr() as _));

        (fx, h_tech, h_wvp, h_world, h_eye_pos_w)
    }
}