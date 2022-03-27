use std::ffi::CStr;
use libc::c_void;
use windows::Win32::Foundation::PSTR;
use windows::Win32::Graphics::Direct3D9::*;
use common::*;
use d3dx::*;
use crate::*;

pub struct Sky {
    sphere: LPD3DXMESH,
    radius: f32,
    env_map: *mut c_void, // IDirect3DCubeTexture9,
    fx: LPD3DXEFFECT,
    h_wvp: D3DXHANDLE,
}

impl Sky {
    pub fn new(base_path: &str, d3d_device: IDirect3DDevice9, env_map_file_name: &str, sky_radius: f32) -> Sky {
        let mut sphere: LPD3DXMESH = std::ptr::null_mut();
        HR!(D3DXCreateSphere(d3d_device.clone(), sky_radius, 30, 30, &mut sphere, std::ptr::null_mut()));

        let mut env_map: *mut c_void = std::ptr::null_mut();
        HR!(D3DXCreateCubeTextureFromFile(d3d_device.clone(),
            PSTR(c_resource_path(base_path, env_map_file_name).as_str().as_ptr() as _),
            &mut env_map));

        let mut fx: LPD3DXEFFECT = std::ptr::null_mut();
        let mut errors: LPD3DXBUFFER = std::ptr::null_mut();

        HR!(D3DXCreateEffectFromFile(d3d_device,
            PSTR(c_resource_path(base_path, "sky.fx").as_str().as_ptr() as _),
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

        let h_tech = ID3DXBaseEffect_GetTechniqueByName(fx, PSTR(b"SkyTech\0".as_ptr() as _));
        let h_wvp = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWVP\0".as_ptr() as _));
        let h_env_map = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gEnvMap\0".as_ptr() as _));

        // Set effect parameters that do not vary.
        HR!(ID3DXEffect_SetTechnique(fx, h_tech));
        HR!(ID3DXBaseEffect_SetTexture(fx, h_env_map, env_map));

        Sky {
            sphere,
            radius: sky_radius,
            env_map,
            fx,
            h_wvp,
        }
    }

    pub fn release_com_objects(&self) {
        ReleaseCOM(self.sphere);
        ReleaseCOM(self.env_map);
        ReleaseCOM(self.fx);
    }

    pub fn get_num_triangles(&self) -> u32 {
        ID3DXBaseMesh_GetNumFaces(self.sphere)
    }

    pub fn get_num_vertices(&self) -> u32 {
        ID3DXBaseMesh_GetNumVertices(self.sphere)
    }

    pub fn get_env_map(&self) -> *mut c_void {
        self.env_map.clone()
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn on_lost_device(&self) {
        HR!(ID3DXEffect_OnLostDevice(self.fx));
    }

    pub fn on_reset_device(&self) {
        HR!(ID3DXEffect_OnResetDevice(self.fx));
    }

    pub fn draw(&self) {
        unsafe {
            let camera: &Camera = &CAMERA.expect("Camera has not been created");

            // Sky always centered about camera's position.
            let p = camera.get_pos();

            let mut w = D3DXMATRIX::default();
            D3DXMatrixTranslation(&mut w, p.x, p.y, p.z);
            D3DXMatrixMultiply(&mut w, &w, camera.get_view_proj());
            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &w));

            let mut num_passes: u32 = 0;
            HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));
            HR!(ID3DXEffect_BeginPass(self.fx, 0));
            HR!(ID3DXBaseMesh_DrawSubset(self.sphere, 0));
            HR!(ID3DXEffect_EndPass(self.fx));
            HR!(ID3DXEffect_End(self.fx));
        }
    }
}