use std::ffi::CStr;
use std::slice::from_raw_parts_mut;
use libc::c_void;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct3D9::*;
use common::*;
use common::mtrl::Mtrl;
use d3dx::*;

use crate::*;

const EPSILON: f32 = 0.001;

pub struct WaterDMap {
    mesh: LPD3DXMESH,
    fx: LPD3DXEFFECT,

    // The two normal maps to scroll.
    wave_map0: *mut c_void, // IDirect3DTexture9
    wave_map1: *mut c_void, // IDirect3DTexture9

    // The two displacement maps to scroll.
    disp_map0: *mut c_void, // IDirect3DTexture9
    disp_map1: *mut c_void, // IDirect3DTexture9

    // Offset of normal maps for scrolling (vary as a function of time)
    wave_nmap_offset0: D3DXVECTOR2,
    wave_nmap_offset1: D3DXVECTOR2,

    // Offset of displacement maps for scrolling (vary as a function of time)
    wave_dmap_offset0: D3DXVECTOR2,
    wave_dmap_offset1: D3DXVECTOR2,

    init_info: WaterDMapInitInfo,
    #[allow(unused)]
    width: f32,

    #[allow(unused)]
    depth: f32,

    h_wvp: D3DXHANDLE,
    h_eye_pos_w: D3DXHANDLE,
    h_wave_nmap_offset0: D3DXHANDLE,
    h_wave_nmap_offset1: D3DXHANDLE,
    h_wave_dmap_offset0: D3DXHANDLE,
    h_wave_dmap_offset1: D3DXHANDLE,
}

pub struct WaterDMapInitInfo {
    pub dir_light: DirLight,
    pub mtrl: Mtrl,
    pub fx_filename: String,
    pub vert_rows: u32,
    pub vert_cols: u32,
    pub dx: f32,
    pub dz: f32,
    pub wave_map_filename0: String,
    pub wave_map_filename1: String,
    pub dmap_filename0: String,
    pub dmap_filename1: String,
    pub wave_nmap_velocity0: D3DXVECTOR2,
    pub wave_nmap_velocity1: D3DXVECTOR2,
    pub wave_dmap_velocity0: D3DXVECTOR2,
    pub wave_dmap_velocity1: D3DXVECTOR2,
    pub scale_heights: D3DXVECTOR2,
    pub tex_scale: f32,
    pub to_world: D3DXMATRIX,
}

impl WaterDMap {
    pub fn new(base_path: &str, init_info: WaterDMapInitInfo, d3d_device: IDirect3DDevice9) -> WaterDMap {
        unsafe {
            let width: f32 = (init_info.vert_cols - 1) as f32 * init_info.dx;
            let depth: f32 = (init_info.vert_rows - 1) as f32  * init_info.dz;

            let wave_nmap_offset0 = D3DXVECTOR2 { x: 0.0, y: 0.0 };
            let wave_nmap_offset1 = D3DXVECTOR2 { x: 0.0, y: 0.0 };

            let wave_dmap_offset0 = D3DXVECTOR2 { x: 0.0, y: 0.0 };
            let wave_dmap_offset1 = D3DXVECTOR2 { x: 0.0, y: 0.0 };

            let num_tris: u32 = (init_info.vert_rows - 1) * (init_info.vert_cols - 1) * 2;
            let num_verts: u32 = init_info.vert_rows * init_info.vert_cols;

            //===============================================================
            // Allocate the mesh.

            // Get the vertex declaration for the NMapVertex.
            let mut elems: [D3DVERTEXELEMENT9; MAX_FVF_DECL_SIZE as usize] = [D3DVERTEXELEMENT9::default(); MAX_FVF_DECL_SIZE as usize];
            let mut num_elems: u32 = 0;
            if let Some(decl) = &WATER_DMAP_VERTEX_DECL {
                HR!(decl.GetDeclaration(elems.as_mut_ptr(), &mut num_elems));
            }

            let mut mesh = std::ptr::null_mut();
            HR!(D3DXCreateMesh(num_tris, num_verts, D3DXMESH_MANAGED,
                elems.as_mut_ptr(), d3d_device.clone(), &mut mesh));

            //===============================================================
            // Write the grid vertices and triangles to the mesh.

            let mut v: *mut c_void = std::ptr::null_mut();
            HR!(ID3DXBaseMesh_LockVertexBuffer(mesh, 0, &mut v));
            let mut v_slice: &mut [WaterDMapVertex] = from_raw_parts_mut(v.cast(), num_verts as usize);

            let mut verts: Vec<D3DXVECTOR3> = Vec::new();
            let mut indices: Vec<u32> = Vec::new();
            gen_tri_grid_32(init_info.vert_rows as i32, init_info.vert_cols as i32,
                            init_info.dx, init_info.dz, D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 },
                            &mut verts, &mut indices);

            for i in 0..init_info.vert_rows {
                for j in 0..init_info.vert_cols {
                    let index: usize = (i * init_info.vert_cols + j) as usize;
                    v_slice[index].pos  = verts[index];
                    v_slice[index].scaled_tex_c = D3DXVECTOR2 {
                        x: j as f32 / init_info.vert_cols as f32 * init_info.tex_scale,
                        y: i as f32 / init_info.vert_rows as f32 * init_info.tex_scale
                    };
                    v_slice[index].normalized_tex_c = D3DXVECTOR2 {
                        x: j as f32 / init_info.vert_cols as f32,
                        y: i as f32 / init_info.vert_rows as f32
                    };
                }
            }

            HR!(ID3DXBaseMesh_UnlockVertexBuffer(mesh));

            //===============================================================
            // Write triangle data so we can compute normals.

            let mut index_buff_ptr: *mut c_void = std::ptr::null_mut();
            HR!(ID3DXBaseMesh_LockIndexBuffer(mesh, 0, &mut index_buff_ptr));
            let indices_slice: &mut[u16] = from_raw_parts_mut(index_buff_ptr.cast(), ID3DXBaseMesh_GetNumFaces(mesh) as usize * 3);

            let mut att_buffer = std::ptr::null_mut();
            HR!(ID3DXMesh_LockAttributeBuffer(mesh, 0, &mut att_buffer));
            let att_buffer_slice: &mut[u32] = from_raw_parts_mut(att_buffer, ID3DXBaseMesh_GetNumFaces(mesh) as usize);

            for i in 0..ID3DXBaseMesh_GetNumFaces(mesh) as usize {
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
            adj.resize(ID3DXBaseMesh_GetNumFaces(mesh) as usize * 3, 0);
            HR!(ID3DXBaseMesh_GenerateAdjacency(mesh, EPSILON, adj.as_mut_ptr()));
            HR!(ID3DXMesh_OptimizeInPlace(mesh, D3DXMESHOPT_VERTEXCACHE | D3DXMESHOPT_ATTRSORT,
                adj.as_ptr(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut()));

            //===============================================================
            // Create textures/effect.

            let m = init_info.vert_rows;
            let n = init_info.vert_cols;

            let mut wave_map0 = std::mem::zeroed();
            let mut wave_map1 = std::mem::zeroed();

            HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
                PSTR(c_resource_path(base_path, init_info.wave_map_filename0.as_str()).as_str().as_ptr() as _), &mut wave_map0));
            HR!(D3DXCreateTextureFromFile(d3d_device.clone(),
                PSTR(c_resource_path(base_path, init_info.wave_map_filename1.as_str()).as_str().as_ptr() as _), &mut wave_map1));

            let mut disp_map0 = std::mem::zeroed();
            let mut disp_map1 = std::mem::zeroed();

            HR!(D3DXCreateTextureFromFileEx(d3d_device.clone(),
                PSTR(c_resource_path(base_path, init_info.dmap_filename0.as_str()).as_str().as_ptr() as _),
                m, n, 1, 0, D3DFMT_R32F, D3DPOOL_MANAGED, D3DX_DEFAULT, D3DX_DEFAULT, 0,
                std::ptr::null_mut(), std::ptr::null_mut(), &mut disp_map0));
            HR!(D3DXCreateTextureFromFileEx(d3d_device.clone(),
                PSTR(c_resource_path(base_path, init_info.dmap_filename1.as_str()).as_str().as_ptr() as _),
                m, n, 1, 0, D3DFMT_R32F, D3DPOOL_MANAGED, D3DX_DEFAULT, D3DX_DEFAULT, 0,
                std::ptr::null_mut(), std::ptr::null_mut(), &mut disp_map1));

            let (fx,
                h_tech,
                h_wvp,
                h_world,
                h_world_inv,
                h_light,
                h_mtrl,
                h_eye_pos_w,
                h_wave_map0,
                h_wave_map1,
                h_wave_nmap_offset0,
                h_wave_nmap_offset1,
                h_wave_dmap_offset0,
                h_wave_dmap_offset1,
                h_wave_disp_map0,
                h_wave_disp_map1,
                h_scale_heights,
                h_grid_step_size_l)
                = WaterDMap::build_fx(base_path, init_info.fx_filename.as_str(), d3d_device.clone());

            // We don't need to set these every frame since they do not change.
            HR!(ID3DXBaseEffect_SetMatrix(fx, h_world, &init_info.to_world));

            let mut world_inv: D3DXMATRIX = std::mem::zeroed();
            D3DXMatrixInverse(&mut world_inv, 0.0, &init_info.to_world);
            HR!(ID3DXBaseEffect_SetMatrix(fx, h_world_inv, &world_inv));

            HR!(ID3DXEffect_SetTechnique(fx, h_tech));

            HR!(ID3DXBaseEffect_SetTexture(fx, h_wave_map0, wave_map0));
            HR!(ID3DXBaseEffect_SetTexture(fx, h_wave_map1, wave_map1));
            HR!(ID3DXBaseEffect_SetTexture(fx, h_wave_disp_map0, disp_map0));
            HR!(ID3DXBaseEffect_SetTexture(fx, h_wave_disp_map1, disp_map1));

            HR!(ID3DXBaseEffect_SetValue(fx, h_light, &init_info.dir_light as *const _ as _, std::mem::size_of::<DirLight>() as u32));
            HR!(ID3DXBaseEffect_SetValue(fx, h_mtrl, &init_info.mtrl as *const _ as _, std::mem::size_of::<Mtrl>() as u32));

            HR!(ID3DXBaseEffect_SetValue(fx, h_scale_heights, &init_info.scale_heights as *const _ as _, std::mem::size_of::<D3DXVECTOR2>() as u32));

            let step_sizes = D3DXVECTOR2 { x: init_info.dx, y: init_info.dz };
            HR!(ID3DXBaseEffect_SetValue(fx, h_grid_step_size_l, &step_sizes as *const _ as _, std::mem::size_of::<D3DXVECTOR2>() as u32));

            WaterDMap {
                mesh,
                fx,
                wave_map0,
                wave_map1,
                disp_map0,
                disp_map1,
                wave_nmap_offset0,
                wave_nmap_offset1,
                wave_dmap_offset0,
                wave_dmap_offset1,
                init_info,
                width,
                depth,
                h_wvp,
                h_eye_pos_w,
                h_wave_nmap_offset0,
                h_wave_nmap_offset1,
                h_wave_dmap_offset0,
                h_wave_dmap_offset1,
            }
        }
    }

    pub fn release_com_objects(&self) {
        ReleaseCOM(self.mesh);
        ReleaseCOM(self.fx);
        ReleaseCOM(self.wave_map0);
        ReleaseCOM(self.wave_map1);
        ReleaseCOM(self.disp_map0);
        ReleaseCOM(self.disp_map1);
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

    pub fn on_reset_device(&self) {
        HR!(ID3DXEffect_OnResetDevice(self.fx));
    }

    pub fn update(&mut self, dt:f32) {
        // Update texture coordinate offsets.  These offsets are added to the
        // texture coordinates in the vertex shader to animate them.
        self.wave_nmap_offset0.x += self.init_info.wave_nmap_velocity0.x as f32 * dt;
        self.wave_nmap_offset1.y += self.init_info.wave_nmap_velocity1.y as f32 * dt;

        self.wave_dmap_offset0.x += self.init_info.wave_dmap_velocity0.x as f32 * dt;
        self.wave_dmap_offset1.y += self.init_info.wave_dmap_velocity1.y as f32 * dt;

        // Textures repeat every 1.0 unit, so reset back down to zero
        // so the coordinates do not grow too large.
        if self.wave_nmap_offset0.x >= 1.0 || self.wave_nmap_offset0.x <= -1.0 {
            self.wave_nmap_offset0.x = 0.0;
        }

        if self.wave_nmap_offset1.x >= 1.0 || self.wave_nmap_offset1.x <= -1.0 {
            self.wave_nmap_offset1.x = 0.0;
        }

        if self.wave_nmap_offset0.y >= 1.0 || self.wave_nmap_offset0.y <= -1.0 {
            self.wave_nmap_offset0.y = 0.0;
        }

        if self.wave_nmap_offset1.y >= 1.0 || self.wave_nmap_offset1.y <= -1.0 {
            self.wave_nmap_offset1.y = 0.0;
        }

        if self.wave_dmap_offset0.x >= 1.0 || self.wave_dmap_offset0.x <= -1.0 {
            self.wave_dmap_offset0.x = 0.0;
        }

        if self.wave_dmap_offset1.x >= 1.0 || self.wave_dmap_offset1.x <= -1.0 {
            self.wave_dmap_offset1.x = 0.0;
        }

        if self.wave_dmap_offset0.y >= 1.0 || self.wave_dmap_offset0.y <= -1.0 {
            self.wave_dmap_offset0.y = 0.0;
        }

        if self.wave_dmap_offset1.y >= 1.0 || self.wave_dmap_offset1.y <= -1.0 {
            self.wave_dmap_offset1.y = 0.0;
        }
    }

    pub fn draw(&self) {
        unsafe {
            let camera: &Camera = &CAMERA.expect("Camera has not been created");

            let mut wvp: D3DXMATRIX = std::mem::zeroed();
            D3DXMatrixMultiply(&mut wvp, &self.init_info.to_world, camera.get_view_proj());
            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &wvp));

            HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_eye_pos_w, &camera.get_pos() as *const _ as _, std::mem::size_of::<D3DXVECTOR3>() as u32));
            HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_wave_nmap_offset0, &self.wave_nmap_offset0 as *const _ as _, std::mem::size_of::<D3DXVECTOR2>() as u32));
            HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_wave_nmap_offset1, &self.wave_nmap_offset1 as *const _ as _, std::mem::size_of::<D3DXVECTOR2>() as u32));
            HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_wave_dmap_offset0, &self.wave_dmap_offset0 as *const _ as _, std::mem::size_of::<D3DXVECTOR2>() as u32));
            HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_wave_dmap_offset1, &self.wave_dmap_offset1 as *const _ as _, std::mem::size_of::<D3DXVECTOR2>() as u32));

            let mut num_passes: u32 = 0;
            HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));
            HR!(ID3DXEffect_BeginPass(self.fx, 0));

            HR!(ID3DXBaseMesh_DrawSubset(self.mesh, 0));

            HR!(ID3DXEffect_EndPass(self.fx));
            HR!(ID3DXEffect_End(self.fx));
        }
    }

    fn build_fx(base_path: &str, fx_filename: &str, d3d_device: IDirect3DDevice9)
        -> (LPD3DXEFFECT, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
            D3DXHANDLE, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
            D3DXHANDLE, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
            D3DXHANDLE, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
            D3DXHANDLE, D3DXHANDLE) {
        // Create the FX from a .fx file.
        let mut fx: LPD3DXEFFECT = std::ptr::null_mut();
        let mut errors: LPD3DXBUFFER = std::ptr::null_mut();

        HR!(D3DXCreateEffectFromFile(d3d_device,
            PSTR(c_resource_path(base_path, fx_filename).as_str().as_ptr() as _),
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
        let h_world = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWorld\0".as_ptr() as _));
        let h_world_inv = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWorldInv\0".as_ptr() as _));
        let h_wvp = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWVP\0".as_ptr() as _));
        let h_eye_pos_w = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gEyePosW\0".as_ptr() as _));
        let h_light = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gLight\0".as_ptr() as _));
        let h_mtrl = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gMtrl\0".as_ptr() as _));
        let h_wave_map0 = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWaveMap0\0".as_ptr() as _));
        let h_wave_map1 = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWaveMap1\0".as_ptr() as _));
        let h_wave_nmap_offset0 = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWaveNMapOffset0\0".as_ptr() as _));
        let h_wave_nmap_offset1 = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWaveNMapOffset1\0".as_ptr() as _));
        let h_wave_dmap_offset0 = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWaveDMapOffset0\0".as_ptr() as _));
        let h_wave_dmap_offset1 = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWaveDMapOffset1\0".as_ptr() as _));
        let h_wave_disp_map0 = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWaveDispMap0\0".as_ptr() as _));
        let h_wave_disp_map1 = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWaveDispMap1\0".as_ptr() as _));
        let h_scale_heights = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gScaleHeights\0".as_ptr() as _));
        let h_grid_step_size_l = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gGridStepSizeL\0".as_ptr() as _));

        (fx,
         h_tech,
         h_wvp,
         h_world,
         h_world_inv,
         h_light,
         h_mtrl,
         h_eye_pos_w,
         h_wave_map0,
         h_wave_map1,
         h_wave_nmap_offset0,
         h_wave_nmap_offset1,
         h_wave_dmap_offset0,
         h_wave_dmap_offset1,
         h_wave_disp_map0,
         h_wave_disp_map1,
         h_scale_heights,
         h_grid_step_size_l)
    }
}