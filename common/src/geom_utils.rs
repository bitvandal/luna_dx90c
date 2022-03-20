use std::ffi::CStr;
use std::slice::from_raw_parts_mut;
use libc::c_void;
use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D9::*,
};
use d3dx::*;
use crate::*;
use crate::mtrl::Mtrl;

pub fn gen_tri_grid(num_vert_rows: i32, num_vert_cols: i32, dx: f32, dz: f32,
                    center: D3DXVECTOR3, verts: &mut Vec<D3DXVECTOR3>, indices: &mut Vec<u16>) {
    let num_vertices = num_vert_rows * num_vert_cols;
    let num_cell_rows = num_vert_rows - 1;
    let num_cell_cols = num_vert_cols - 1;

    let num_tris = num_cell_rows * num_cell_cols * 2;

    let width: f32 = num_cell_cols as f32 * dx;
    let depth: f32 = num_cell_rows as f32 * dz;

    //===========================================
    // Build vertices.

    // We first build the grid geometry centered about the origin and on
    // the xz-plane, row-by-row and in a top-down fashion.  We then translate
    // the grid vertices so that they are centered about the specified
    // parameter 'center'.

    verts.resize(num_vertices as usize, D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 });

    // Offsets to translate grid from quadrant 4 to center of
    // coordinate system.
    let x_offset: f32 = -width * 0.5;
    let z_offset: f32 =  depth * 0.5;

    let mut k = 0;
    for i in 0..num_vert_rows {
        for j in 0..num_vert_cols {
            // Negate the depth coordinate to put in quadrant four.
            // Then offset to center about coordinate system.
            verts[k].x = j as f32 * dx + x_offset;
            verts[k].z = -i as f32 * dz + z_offset;
            verts[k].y = 0.0;

            // Translate so that the center of the grid is at the
            // specified 'center' parameter.

            unsafe {
                let mut t: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixTranslation(&mut t, center.x, center.y, center.z);
                D3DXVec3TransformCoord(&mut verts[k], &verts[k], &t);
            }

            // Next vertex
            k += 1;
        }
    }

    //===========================================
    // Build indices.

    indices.resize((num_tris * 3) as usize, 0);

    // Generate indices for each quad.
    k = 0;
    for i in 0..num_cell_rows {
        for j in 0..num_cell_cols {
            indices[k]     = (i * num_vert_cols + j) as u16;
            indices[k + 1] = (i * num_vert_cols + j + 1) as u16;
            indices[k + 2] = ((i + 1) * num_vert_cols + j) as u16;

            indices[k + 3] = ((i + 1) * num_vert_cols + j) as u16;
            indices[k + 4] = (i * num_vert_cols + j + 1) as u16;
            indices[k + 5] = ((i + 1) * num_vert_cols + j + 1) as u16;

            // next quad
            k += 6;
        }
    }
}

pub fn gen_tri_grid_32(num_vert_rows: i32, num_vert_cols: i32, dx: f32, dz: f32,
                    center: D3DXVECTOR3, verts: &mut Vec<D3DXVECTOR3>, indices: &mut Vec<u32>) {
    let num_vertices = num_vert_rows * num_vert_cols;
    let num_cell_rows = num_vert_rows - 1;
    let num_cell_cols = num_vert_cols - 1;

    let num_tris = num_cell_rows * num_cell_cols * 2;

    let width: f32 = num_cell_cols as f32 * dx;
    let depth: f32 = num_cell_rows as f32 * dz;

    //===========================================
    // Build vertices.

    // We first build the grid geometry centered about the origin and on
    // the xz-plane, row-by-row and in a top-down fashion.  We then translate
    // the grid vertices so that they are centered about the specified
    // parameter 'center'.

    verts.resize(num_vertices as usize, D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 });

    // Offsets to translate grid from quadrant 4 to center of
    // coordinate system.
    let x_offset: f32 = -width * 0.5;
    let z_offset: f32 =  depth * 0.5;

    let mut k = 0;
    for i in 0..num_vert_rows {
        for j in 0..num_vert_cols {
            // Negate the depth coordinate to put in quadrant four.
            // Then offset to center about coordinate system.
            verts[k].x = j as f32 * dx + x_offset;
            verts[k].z = -i as f32 * dz + z_offset;
            verts[k].y = 0.0;

            // Translate so that the center of the grid is at the
            // specified 'center' parameter.

            unsafe {
                let mut t: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixTranslation(&mut t, center.x, center.y, center.z);
                D3DXVec3TransformCoord(&mut verts[k], &verts[k], &t);
            }

            // Next vertex
            k += 1;
        }
    }

    //===========================================
    // Build indices.

    indices.resize((num_tris * 3) as usize, 0);

    // Generate indices for each quad.
    k = 0;
    for i in 0..num_cell_rows {
        for j in 0..num_cell_cols {
            indices[k]     = (i * num_vert_cols + j) as u32;
            indices[k + 1] = (i * num_vert_cols + j + 1) as u32;
            indices[k + 2] = ((i + 1) * num_vert_cols + j) as u32;

            indices[k + 3] = ((i + 1) * num_vert_cols + j) as u32;
            indices[k + 4] = (i * num_vert_cols + j + 1) as u32;
            indices[k + 5] = ((i + 1) * num_vert_cols + j + 1) as u32;

            // next quad
            k += 6;
        }
    }
}

pub fn gen_spherical_tex_coords(d3d_device: IDirect3DDevice9, sphere: &mut LPD3DXMESH) {
    // D3DXCreate* functions generate vertices with position
    // and normal data.  But for texturing, we also need
    // tex-coords.  So clone the mesh to change the vertex
    // format to a format with tex-coords.
    let mut elements: [D3DVERTEXELEMENT9; 64] = [D3DVERTEXELEMENT9::default(); 64];
    let mut num_elements: u32 = 0;
    unsafe {
        if let Some(decl) = &VERTEX_PNT_DECL {
            HR!(decl.GetDeclaration(elements.as_mut_ptr(), &mut num_elements));
        }

        let mut temp: LPD3DXMESH = std::ptr::null_mut();
        HR!(ID3DXBaseMesh_CloneMesh(*sphere, D3DXMESH_SYSTEMMEM, elements.as_mut_ptr(),
                d3d_device.clone(), &mut temp));

        ReleaseCOM(*sphere);

        // Now generate texture coordinates for each vertex.
        let mut verts: *mut c_void = std::ptr::null_mut();
        HR!(ID3DXBaseMesh_LockVertexBuffer(temp, 0, &mut verts));

        let num_vertices: usize = ID3DXBaseMesh_GetNumVertices(temp) as usize;
        let vertices: &mut [VertexPNT] = from_raw_parts_mut(verts as *mut VertexPNT, num_vertices);

        for i in 0..num_vertices {
            // Convert to spherical coordinates.
            let p: D3DXVECTOR3 = vertices[i].pos;

            let theta: f32 = p.z.atan2(p.x);
            let phi: f32 = (p.y / (p.x * p.x + p.y * p.y + p.z * p.z).sqrt()).acos();

            // Phi and theta give the texture coordinates, but are not in
            // the range [0, 1], so scale them into that range.

            let u: f32 = theta / (2.0 * D3DX_PI);
            let v: f32 = phi / D3DX_PI;

            // Save texture coordinates.
            vertices[i].tex0.x = u;
            vertices[i].tex0.y = v;
        }

        HR!(ID3DXBaseMesh_UnlockVertexBuffer(temp));

        // Clone back to a hardware mesh.
        HR!(ID3DXBaseMesh_CloneMesh(temp, D3DXMESH_MANAGED | D3DXMESH_WRITEONLY, elements.as_mut_ptr(),
                d3d_device.clone(), sphere));

        ReleaseCOM(temp);
    }
}

// Cylinder Axis
pub enum Axis {
    X,
    Y,
    Z
}

pub fn gen_cyl_tex_coords(d3d_device: IDirect3DDevice9, cylinder: &mut LPD3DXMESH, axis: Axis) {
    // D3DXCreate* functions generate vertices with position
    // and normal data.  But for texturing, we also need
    // tex-coords.  So clone the mesh to change the vertex
    // format to a format with tex-coords.
    let mut elements: [D3DVERTEXELEMENT9; 64] = [D3DVERTEXELEMENT9::default(); 64];
    let mut num_elements: u32 = 0;
    unsafe {
        if let Some(decl) = &VERTEX_PNT_DECL {
            HR!(decl.GetDeclaration(elements.as_mut_ptr(), &mut num_elements));
        }

        let mut temp: LPD3DXMESH = std::ptr::null_mut();
        HR!(ID3DXBaseMesh_CloneMesh(*cylinder, D3DXMESH_SYSTEMMEM, elements.as_mut_ptr(),
                d3d_device.clone(), &mut temp));

        ReleaseCOM(*cylinder);

        // Now generate texture coordinates for each vertex.
        let mut verts: *mut c_void = std::ptr::null_mut();
        HR!(ID3DXBaseMesh_LockVertexBuffer(temp, 0, &mut verts));

        // We need to get the height of the cylinder we are projecting the
        // vertices onto.  That height depends on which axis the client has
        // specified that the cylinder lies on.  The height is determined by
        // finding the height of the bounding cylinder on the specified axis.

        let mut max_point = D3DXVECTOR3 { x: -f32::MAX, y: -f32::MAX, z: -f32::MAX };
        let mut min_point = D3DXVECTOR3 { x: f32::MAX, y: f32::MAX, z: f32::MAX };

        let num_vertices: usize = ID3DXBaseMesh_GetNumVertices(temp) as usize;
        let vertices: &mut [VertexPNT] = from_raw_parts_mut(verts as *mut VertexPNT, num_vertices);

        for i in 0..num_vertices {
            D3DXVec3Maximize(&mut max_point, &max_point, &vertices[i].pos);
            D3DXVec3Minimize(&mut min_point, &min_point, &vertices[i].pos);
        }

        let a: f32;
        let b: f32;
        let h: f32;

        match axis {
            Axis::X => {
                a = min_point.x;
                b = max_point.x;
                h = b - a;
            },
            Axis::Y => {
                a = min_point.y;
                b = max_point.y;
                h = b - a;
            },
            Axis::Z => {
                a = min_point.z;
                b = max_point.z;
                h = b - a;
            }
        }

        // Iterate over each vertex and compute its texture coordinate.

        for i in 0..num_vertices {
            // Get the coordinates along the axes orthogonal to the
            // axis the cylinder is aligned with.
            let x: f32;
            let y: f32;
            let z: f32;

            match axis {
                Axis::X => {
                    x = vertices[i].pos.y;
                    z = vertices[i].pos.z;
                    y = vertices[i].pos.x;
                },
                Axis::Y => {
                    x = vertices[i].pos.x;
                    z = vertices[i].pos.z;
                    y = vertices[i].pos.y;
                },
                Axis::Z => {
                    x = vertices[i].pos.x;
                    z = vertices[i].pos.y;
                    y = vertices[i].pos.z;
                }
            }

            // Convert to cylindrical coordinates.

            let theta = z.atan2(x);
            let y2 = y - b; // Transform [a, b] --> [-h, 0]

            // Transform theta from [0, 2*pi] to [0, 1] range and
            // transform y2 from [-h, 0] to [0, 1].

            let u: f32 = theta / (2.0 * D3DX_PI);
            let v = y2 / -h;

            // Save texture coordinates.

            vertices[i].tex0.x = u;
            vertices[i].tex0.y = v;
        }

        HR!(ID3DXBaseMesh_UnlockVertexBuffer(temp));

        // Clone back to a hardware mesh.
        HR!(ID3DXBaseMesh_CloneMesh(temp, D3DXMESH_MANAGED | D3DXMESH_WRITEONLY, elements.as_mut_ptr(),
                d3d_device.clone(), cylinder));

        ReleaseCOM(temp);
    }
}


pub fn load_x_file(base_path: &str, filename: &str, d3d_device: IDirect3DDevice9) -> (LPD3DXMESH, Vec<Mtrl>, Vec<*mut c_void>) {
    unsafe {
        let mut mesh_out: LPD3DXMESH = std::ptr::null_mut();
        let mut mtrls = Vec::new();
        let mut texs = Vec::new();

        // Step 1: Load the .x file from file into a system memory mesh.

        let mut mesh_sys: LPD3DXMESH = std::ptr::null_mut();
        let mut adj_buffer: LPD3DXBUFFER = std::ptr::null_mut();
        let mut mtrl_buffer: LPD3DXBUFFER = std::ptr::null_mut();
        let mut num_mtrls: u32 = 0;

        HR!(D3DXLoadMeshFromX(PSTR(c_resource_path(base_path, filename).as_str().as_ptr() as _),
                D3DXMESH_SYSTEMMEM, d3d_device.clone(), &mut adj_buffer, &mut mtrl_buffer, std::ptr::null_mut(),
                &mut num_mtrls, &mut mesh_sys));

        // Step 2: Find out if the mesh already has normal info?

        let mut elems: [D3DVERTEXELEMENT9; 64] = [D3DVERTEXELEMENT9::default(); 64];
        HR!(ID3DXBaseMesh_GetDeclaration(mesh_sys, elems.as_mut_ptr()));

        let mut has_normals = false;

        for i in 0..=64 {
            // Did we reach D3DDECL_END() {0xFF,0,D3DDECLTYPE_UNUSED, 0,0,0}?
            if elems[i].Stream == 0xff {
                break;
            }

            if elems[i].Type == D3DDECLTYPE_FLOAT3.0 as u8 &&
                elems[i].Usage == D3DDECLUSAGE_NORMAL.0 as u8 &&
                elems[i].UsageIndex == 0 {
                has_normals = true;
                break;
            }
        }

        // Step 3: Change vertex format to VertexPNT.

        let mut elements: [D3DVERTEXELEMENT9; 64] = [D3DVERTEXELEMENT9::default(); 64];
        let mut num_elements: u32 = 0;

        if let Some(decl) = &VERTEX_PNT_DECL {
            HR!(decl.GetDeclaration(elements.as_mut_ptr(), &mut num_elements));

            let mut temp: LPD3DXMESH = std::ptr::null_mut();
            HR!(ID3DXBaseMesh_CloneMesh(*&mesh_sys, D3DXMESH_SYSTEMMEM, elements.as_ptr(),
                        d3d_device.clone(), &mut temp));

            ReleaseCOM(*&mesh_sys);

            mesh_sys = temp;
        }

        // Step 4: If the mesh did not have normals, generate them.

        if has_normals == false {
            HR!(D3DXComputeNormals(mesh_sys, std::ptr::null()));
        }

        // Step 5: Optimize the mesh.

        let buf_pointer: *mut u32 = ID3DXBuffer_GetBufferPointer(adj_buffer).cast();

        HR!(ID3DXMesh_Optimize(*&mesh_sys,
                D3DXMESH_MANAGED | D3DXMESHOPT_COMPACT | D3DXMESHOPT_ATTRSORT | D3DXMESHOPT_VERTEXCACHE,
                buf_pointer,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut mesh_out));

        ReleaseCOM(mesh_sys);   // Done w/ system mesh.
        ReleaseCOM(adj_buffer); // Done with buffer.

        // Step 6: Extract the materials and load the textures.

        if mtrl_buffer != std::ptr::null_mut() && num_mtrls != 0 {
            let d3dxmtrls_ptr: *mut D3DXMATERIAL = ID3DXBuffer_GetBufferPointer(mtrl_buffer).cast();

            let d3dxmtrls: &mut [D3DXMATERIAL] = from_raw_parts_mut(d3dxmtrls_ptr, num_mtrls as usize);

            for i in 0..num_mtrls {
                // Save the ith material.  Note that the MatD3D property does not have an ambient
                // value set when its loaded, so just set it to the diffuse value.

                let index = i as usize;
                let m: Mtrl = Mtrl {
                    ambient: d3dxmtrls[index].MatD3D.Diffuse.into(),
                    diffuse: d3dxmtrls[index].MatD3D.Diffuse.into(),
                    spec: d3dxmtrls[index].MatD3D.Specular.into(),
                    spec_power: d3dxmtrls[index].MatD3D.Power,
                };
                mtrls.push(m);

                // Check if the ith material has an associative texture
                if !d3dxmtrls[index].pTextureFilename.is_null() {
                    // Yes, load the texture for the ith subset
                    let mut tex: *mut c_void = std::ptr::null_mut();

                    let c_str: &CStr = CStr::from_ptr(d3dxmtrls[index].pTextureFilename.0.cast());
                    let str_slice: &str = c_str.to_str().unwrap_or("<unknown error>");
                    let mut tex_fn = c_resource_path(base_path, str_slice);

                    HR!(D3DXCreateTextureFromFile(d3d_device.clone(), PSTR(tex_fn.as_mut_ptr()), &mut tex));

                    texs.push(tex);
                } else {
                    // No texture for the ith subset
                    texs.push(std::ptr::null_mut());
                }
            }
        }

        ReleaseCOM(mtrl_buffer); // done w/ buffer

        (mesh_out, mtrls, texs)
    }
}