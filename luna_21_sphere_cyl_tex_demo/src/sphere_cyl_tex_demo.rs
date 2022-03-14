// Controls: Use mouse to orbit and zoom; use the 'W' and 'S' keys to
//           alter the height of the camera.

use libc::c_void;
use std::slice::{from_raw_parts_mut};
use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D9::*, Win32::System::SystemServices::*,
};

use crate::*;

// Colors
const WHITE: D3DXCOLOR = D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

// Material
struct Mtrl {
    pub ambient: D3DXCOLOR,
    pub diffuse: D3DXCOLOR,
    pub spec: D3DXCOLOR,
    pub spec_power: f32,
}

#[allow(unused)]
// Cylinder Axis
enum Axis {
    X,
    Y,
    Z
}

// Sample demo
pub struct SphereCylTexDemo {
    d3d_pp: *const D3DPRESENT_PARAMETERS,
    gfx_stats: Option<GfxStats>,

    num_grid_vertices: u32,
    num_grid_triangles: u32,

    cylinder: LPD3DXMESH,
    sphere: LPD3DXMESH,

    vb: IDirect3DVertexBuffer9,
    ib: IDirect3DIndexBuffer9,

    sphere_tex: *mut c_void, //IDirect3DTexture9,
    cyl_tex: *mut c_void,    //IDirect3DTexture9,
    grid_tex: *mut c_void,   //IDirect3DTexture9,

    fx: LPD3DXEFFECT,

    h_tech: D3DXHANDLE,
    h_wvp: D3DXHANDLE,
    h_world_inverse_transpose: D3DXHANDLE,
    h_ambient_light: D3DXHANDLE,
    h_diffuse_light: D3DXHANDLE,
    h_specular_light: D3DXHANDLE,
    h_light_vec_w: D3DXHANDLE,
    h_ambient_mtrl: D3DXHANDLE,
    h_diffuse_mtrl: D3DXHANDLE,
    h_specular_mtrl: D3DXHANDLE,
    h_specular_power: D3DXHANDLE,
    h_eye_pos: D3DXHANDLE,
    h_world: D3DXHANDLE,
    h_tex: D3DXHANDLE,

    ambient_light: D3DXCOLOR,
    diffuse_light: D3DXCOLOR,
    specular_light: D3DXCOLOR,
    light_vec_w: D3DXVECTOR3,

    grid_mtrl: Mtrl,
    cylinder_mtrl: Mtrl,
    sphere_mtrl: Mtrl,

    camera_rotation_y: f32,
    camera_radius: f32,
    camera_height: f32,

    view: D3DXMATRIX,
    proj: D3DXMATRIX,
}

impl SphereCylTexDemo {
    pub fn new(d3d_device: IDirect3DDevice9, d3d_pp: *const D3DPRESENT_PARAMETERS) -> Option<SphereCylTexDemo> {
        if !SphereCylTexDemo::check_device_caps() {
            display_error_then_quit("checkDeviceCaps() Failed");
        }

        let mut gfx_stats = GfxStats::new(d3d_device.clone(), D3DCOLOR_XRGB!(0, 0, 0));

        let ambient_light = WHITE;
        let diffuse_light = WHITE;
        let specular_light = WHITE;
        let light_vec_w = D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 };

        let grid_mtrl = Mtrl {
            ambient: WHITE.mult(0.7),
            diffuse: WHITE,
            spec: WHITE.mult(0.5),
            spec_power: 16.0
        };

        let cylinder_mtrl = Mtrl {
            ambient: WHITE.mult(0.4),
            diffuse: WHITE,
            spec: WHITE.mult(0.8),
            spec_power: 8.0
        };

        let sphere_mtrl = Mtrl {
            ambient: WHITE.mult(0.4),
            diffuse: WHITE,
            spec: WHITE.mult(0.8),
            spec_power: 8.0
        };

        let mut cylinder: LPD3DXMESH = std::ptr::null_mut();
        HR!(D3DXCreateCylinder(d3d_device.clone(), 1.0, 1.0, 6.0, 20, 20, &mut cylinder, std::ptr::null_mut()));

        let mut sphere: LPD3DXMESH = std::ptr::null_mut();
        HR!(D3DXCreateSphere(d3d_device.clone(), 1.0, 20, 20, &mut sphere, std::ptr::null_mut()));

        init_all_vertex_declarations(d3d_device.clone());

        SphereCylTexDemo::gen_spherical_tex_coords(d3d_device.clone(), &mut sphere);

        // D3DXCreateCylinder doc says: The created cylinder is centered at the origin, and its axis is aligned with the z-axis.
        SphereCylTexDemo::gen_cyl_tex_coords(d3d_device.clone(), &mut cylinder, Axis::Z);

        let mut sphere_tex: *mut c_void = std::ptr::null_mut();
        let mut cyl_tex: *mut c_void = std::ptr::null_mut();
        let mut grid_tex: *mut c_void = std::ptr::null_mut();

        HR!(D3DXCreateTextureFromFile(d3d_device.clone(), PSTR(b"luna_21_sphere_cyl_tex_demo/marble.bmp\0".as_ptr() as _), &mut sphere_tex));
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(), PSTR(b"luna_21_sphere_cyl_tex_demo/stone2.dds\0".as_ptr() as _), &mut cyl_tex));
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(), PSTR(b"luna_21_sphere_cyl_tex_demo/ground0.dds\0".as_ptr() as _), &mut grid_tex));

        let (vb, ib) = SphereCylTexDemo::build_geo_buffers(d3d_device.clone());

        let (fx, h_tech, h_wvp, h_world_inverse_transpose,
            h_eye_pos, h_world,
            h_ambient_light, h_diffuse_light, h_specular_light,
            h_light_vec_w,
            h_ambient_mtrl, h_diffuse_mtrl, h_specular_mtrl,
            h_specular_power, h_tex) =
            SphereCylTexDemo::build_fx(d3d_device.clone());

        // Save vertex count and triangle count for DrawIndexedPrimitive arguments.
        let num_grid_vertices = 100 * 100;
        let num_grid_triangles = 99 * 99 * 2;

        // If you look at the drawCylinders and drawSpheres functions, you see
        // that we draw 14 cylinders and 14 spheres.
        let num_cyl_verts = ID3DXBaseMesh_GetNumVertices(cylinder) * 14;
        let num_sphere_verts = ID3DXBaseMesh_GetNumVertices(sphere) * 14;
        let num_cyl_tris = ID3DXBaseMesh_GetNumFaces(cylinder) * 14;
        let num_sphere_tris = ID3DXBaseMesh_GetNumFaces(sphere) * 14;

        if let Some(gfx_stats) = &mut gfx_stats {
            gfx_stats.add_vertices(num_grid_vertices);
            gfx_stats.add_vertices(num_cyl_verts);
            gfx_stats.add_vertices(num_sphere_verts);
            gfx_stats.add_triangles(num_grid_triangles);
            gfx_stats.add_triangles(num_cyl_tris);
            gfx_stats.add_triangles(num_sphere_tris);
        }

        let mut sphere_cyl_tex_demo = SphereCylTexDemo {
            d3d_pp,
            gfx_stats,

            num_grid_vertices,
            num_grid_triangles,

            cylinder,
            sphere,
            vb,
            ib,

            sphere_tex,
            cyl_tex,
            grid_tex,

            fx,

            h_tech,
            h_wvp,
            h_world_inverse_transpose,
            h_ambient_light,
            h_diffuse_light,
            h_specular_light,
            h_light_vec_w,

            h_ambient_mtrl,
            h_diffuse_mtrl,
            h_specular_mtrl,
            h_specular_power,
            h_eye_pos,
            h_world,
            h_tex,

            ambient_light,
            diffuse_light,
            specular_light,
            light_vec_w,

            grid_mtrl,
            cylinder_mtrl,
            sphere_mtrl,

            camera_radius: 50.0,
            camera_rotation_y: 1.2 * D3DX_PI,
            camera_height: 20.0,

            view: unsafe { std::mem::zeroed() },
            proj: unsafe { std::mem::zeroed() },
        };

        sphere_cyl_tex_demo.on_reset_device();

        Some(sphere_cyl_tex_demo)
    }

    pub fn release_com_objects(&self) {
        if let Some(gfx_stats) = &self.gfx_stats {
            gfx_stats.release_com_objects();
        }

        ReleaseCOM(self.fx);

        destroy_all_vertex_declarations();
    }

    fn check_device_caps() -> bool {
        unsafe {
            if let Some(d3d_device) = &D3D_DEVICE {
                let mut caps: D3DCAPS9 = std::mem::zeroed();
                HR!(d3d_device.GetDeviceCaps(&mut caps));

                // Check for vertex shader version 2.0 support.
                if caps.VertexShaderVersion < D3DVS_VERSION!(2, 0) {
                    return false
                }

                // Check for pixel shader version 2.0 support.
                if caps.PixelShaderVersion < D3DPS_VERSION!(2, 0) {
                    return false
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
            }
        }

        // The camera position/orientation relative to world space can
        // change every frame based on input, so we need to rebuild the
        // view matrix every frame with the latest changes.
        self.build_view_mtx();
    }

    pub fn draw_scene(&mut self) {
        unsafe {
            if let Some(d3d_device) = &D3D_DEVICE {
                // Clear the backbuffer and depth buffer.
                HR!(d3d_device.Clear(
                    0,
                    std::ptr::null(),
                    (D3DCLEAR_TARGET | D3DCLEAR_ZBUFFER) as u32,
                    0xFFFFFFFF,
                    1.0,
                    0));

                HR!(d3d_device.BeginScene());

                // Setup the rendering FX
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_ambient_light, &self.ambient_light as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_diffuse_light, &self.diffuse_light as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_specular_light, &self.specular_light as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_light_vec_w, &self.light_vec_w as *const _ as _, std::mem::size_of::<D3DXVECTOR3>() as u32));

                HR!(ID3DXEffect_SetTechnique(self.fx, self.h_tech));

                // Begin passes.
                let mut num_passes: u32 = 0;
                HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));
                for i in 0..num_passes {
                    HR!(ID3DXEffect_BeginPass(self.fx, i));

                    self.draw_grid();
                    self.draw_cylinders(d3d_device);
                    self.draw_spheres(d3d_device);

                    HR!(ID3DXEffect_EndPass(self.fx));
                }
                HR!(ID3DXEffect_End(self.fx));

                if let Some(gfx_stats) = &self.gfx_stats {
                    gfx_stats.display();
                }

                HR!(d3d_device.EndScene());

                HR!(d3d_device.Present(
                    std::ptr::null(),
                    std::ptr::null(),
                    HWND(0),
                    std::ptr::null()));
            }
        }
    }

    fn build_geo_buffers(d3d_device: IDirect3DDevice9) -> (IDirect3DVertexBuffer9, IDirect3DIndexBuffer9){
        unsafe {
            let mut verts: Vec<D3DXVECTOR3> = Vec::new();
            let mut indices: Vec<u16> = Vec::new();

            gen_tri_grid(100, 100, 1.0, 1.0,
                         D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 }, &mut verts, &mut indices);

            let mut vb: Option<IDirect3DVertexBuffer9> = None;

            // Obtain a pointer to a new vertex buffer.
            HR!(d3d_device.CreateVertexBuffer((verts.len() * std::mem::size_of::<VertexPNT>()) as u32,
            D3DUSAGE_WRITEONLY as u32, 0, D3DPOOL_MANAGED, &mut vb, std::ptr::null_mut()));

            if let Some(vb) = &mut vb {
                // Now lock it to obtain a pointer to its internal data, and write the
                // grid's vertex data.
                let mut v = std::ptr::null_mut();
                HR!(vb.Lock(0, 0, &mut v, 0));

                let tex_scale: f32 = 0.2;

                let num_grid_vertices = 100 * 100;
                let mut verts_pnt: Vec<VertexPNT> = Vec::with_capacity(num_grid_vertices);
                for i in 0..100 {
                    for j in 0..100 {
                        let index = i * 100 + j;
                        verts_pnt.insert(index, VertexPNT {
                            pos: verts[index],
                            normal: D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 },
                            tex0: D3DXVECTOR2 { x: j as f32 * tex_scale, y: i as f32 * tex_scale }
                        });
                    }
                }

                std::ptr::copy_nonoverlapping(verts_pnt.as_ptr(),
                                              v as *mut VertexPNT,
                                              num_grid_vertices);

                HR!(vb.Unlock());
            }

            // Obtain a pointer to a new index buffer.
            let mut ib: Option<IDirect3DIndexBuffer9> = None;

            // Obtain a pointer to a new index buffer.
            HR!(d3d_device.CreateIndexBuffer((indices.len() * std::mem::size_of::<u16>()) as u32,
            D3DUSAGE_WRITEONLY as u32, D3DFMT_INDEX16, D3DPOOL_MANAGED, &mut ib, std::ptr::null_mut()));

            if let Some(ib) = &mut ib {
                // Now lock it to obtain a pointer to its internal data, and write the
                // grid's index data.
                let mut i = std::ptr::null_mut();
                HR!(ib.Lock(0, 0, &mut i, 0));
                std::ptr::copy_nonoverlapping(indices.as_ptr(),
                                              i as *mut u16,
                                              indices.len());
                HR!(ib.Unlock());
            }

            (vb.unwrap(), ib.unwrap())
        }
    }

    fn build_fx(d3d_device: IDirect3DDevice9) -> (LPD3DXEFFECT, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
                                                  D3DXHANDLE, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
                                                  D3DXHANDLE, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
                                                  D3DXHANDLE, D3DXHANDLE, D3DXHANDLE) {
        // Create the FX from a .fx file.
        let mut fx: LPD3DXEFFECT = std::ptr::null_mut();
        let mut errors: LPD3DXBUFFER = std::ptr::null_mut();

        HR!(D3DXCreateEffectFromFile(d3d_device, PSTR(b"luna_21_sphere_cyl_tex_demo/dirLightTex.fx\0".as_ptr() as _),
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
        let h_world_inverse_transpose = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWorldInvTrans\0".as_ptr() as _));
        let h_eye_pos = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gEyePosW\0".as_ptr() as _));
        let h_world = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWorld\0".as_ptr() as _));
        let h_ambient_light = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gAmbientLight\0".as_ptr() as _));
        let h_diffuse_light = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gDiffuseLight\0".as_ptr() as _));
        let h_specular_light = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gSpecularLight\0".as_ptr() as _));
        let h_light_vec_w = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gLightVecW\0".as_ptr() as _));
        let h_ambient_mtrl = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gAmbientMtrl\0".as_ptr() as _));
        let h_diffuse_mtrl = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gDiffuseMtrl\0".as_ptr() as _));
        let h_specular_mtrl = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gSpecularMtrl\0".as_ptr() as _));
        let h_specular_power = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gSpecularPower\0".as_ptr() as _));
        let h_tex = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gTex\0".as_ptr() as _));

        (fx, h_tech, h_wvp, h_world_inverse_transpose,
         h_eye_pos, h_world,
         h_ambient_light, h_diffuse_light, h_specular_light,
         h_light_vec_w,
         h_ambient_mtrl, h_diffuse_mtrl, h_specular_mtrl,
         h_specular_power, h_tex)
    }

    fn build_view_mtx(&mut self) {
        let x: f32 = self.camera_radius * self.camera_rotation_y.cos();
        let z: f32 = self.camera_radius * self.camera_rotation_y.sin();
        let pos = D3DXVECTOR3 { x, y: self.camera_height, z };
        let target = D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 };
        let up = D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 };
        D3DXMatrixLookAtLH(&mut self.view, &pos, &target, &up);

        HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_eye_pos, &pos as *const _ as _, std::mem::size_of::<D3DXVECTOR3>() as u32));
    }

    fn build_proj_mtx(&mut self) {
        let w: f32 = (unsafe { *self.d3d_pp }).BackBufferWidth as f32;
        let h: f32 = (unsafe { *self.d3d_pp }).BackBufferHeight as f32;
        D3DXMatrixPerspectiveFovLH(&mut self.proj, D3DX_PI * 0.25, w / h, 1.0, 5000.0);
    }

    fn draw_grid(&self) {
        unsafe {
            if let Some(d3d_device) = &D3D_DEVICE {
                HR!(d3d_device.SetStreamSource(0, &self.vb, 0, std::mem::size_of::<VertexPNT>() as u32));
                HR!(d3d_device.SetIndices(&self.ib));
                HR!(d3d_device.SetVertexDeclaration(&VERTEX_PNT_DECL));

                let mut w: D3DXMATRIX = std::mem::zeroed();
                let mut wit: D3DXMATRIX = std::mem::zeroed();
                let mut res: D3DXMATRIX = std::mem::zeroed();

                D3DXMatrixIdentity(&mut w);
                D3DXMatrixInverse(&mut wit, 0.0, &w);
                D3DXMatrixTranspose(&mut wit, &wit);

                D3DXMatrixMultiply(&mut res, &w, &self.view);
                D3DXMatrixMultiply(&mut res, &res, &self.proj);

                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world, &w));
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &res));
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world_inverse_transpose, &wit));

                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_ambient_mtrl, &self.grid_mtrl.ambient as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_diffuse_mtrl, &self.grid_mtrl.diffuse as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_specular_mtrl, &self.grid_mtrl.spec as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
                HR!(ID3DXBaseEffect_SetFloat(self.fx, self.h_specular_power, self.grid_mtrl.spec_power));

                HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.grid_tex));

                HR!(ID3DXEffect_CommitChanges(self.fx));

                HR!(d3d_device.DrawIndexedPrimitive(D3DPT_TRIANGLELIST, 0, 0, self.num_grid_vertices,
                            0, self.num_grid_triangles));
            }
        }
    }

    fn draw_cylinders(&self, d3d_device: &IDirect3DDevice9) {
        unsafe {
            HR!(d3d_device.SetRenderState(D3DRS_WRAP0, D3DWRAP_U as u32));

            let mut r: D3DXMATRIX = std::mem::zeroed();
            D3DXMatrixRotationX(&mut r, -D3DX_PI * 0.5);

            HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_ambient_mtrl, &self.cylinder_mtrl.ambient as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
            HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_diffuse_mtrl, &self.cylinder_mtrl.diffuse as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
            HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_specular_mtrl, &self.cylinder_mtrl.spec as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
            HR!(ID3DXBaseEffect_SetFloat(self.fx, self.h_specular_power, self.cylinder_mtrl.spec_power));

            HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.cyl_tex));

            let mut z: i32 = -30;
            while z <= 30 {
                let mut t: D3DXMATRIX = std::mem::zeroed();
                let mut w: D3DXMATRIX = std::mem::zeroed();
                let mut wit: D3DXMATRIX = std::mem::zeroed();
                let mut res: D3DXMATRIX = std::mem::zeroed();

                D3DXMatrixTranslation(&mut t, -10.0, 3.0, z as f32);

                D3DXMatrixMultiply(&mut w, &r, &t);

                D3DXMatrixInverse(&mut wit, 0.0, &w);
                D3DXMatrixTranspose(&mut wit, &wit);

                D3DXMatrixMultiply(&mut res, &w, &self.view);
                D3DXMatrixMultiply(&mut res, &res, &self.proj);

                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &res));
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world, &w));
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world_inverse_transpose, &wit));

                HR!(ID3DXEffect_CommitChanges(self.fx));
                HR!(ID3DXBaseMesh_DrawSubset(self.cylinder, 0));

                D3DXMatrixTranslation(&mut t, 10.0, 3.0, z as f32);

                D3DXMatrixMultiply(&mut w, &r, &t);

                D3DXMatrixInverse(&mut wit, 0.0, &w);
                D3DXMatrixTranspose(&mut wit, &wit);

                D3DXMatrixMultiply(&mut res, &w, &self.view);
                D3DXMatrixMultiply(&mut res, &res, &self.proj);

                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &res));
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world, &w));
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world_inverse_transpose, &wit));

                HR!(ID3DXEffect_CommitChanges(self.fx));
                HR!(ID3DXBaseMesh_DrawSubset(self.cylinder, 0));

                z += 10;
            }

            // Disable.
            HR!(d3d_device.SetRenderState(D3DRS_WRAP0, 0));
        }
    }

    fn draw_spheres(&self, d3d_device: &IDirect3DDevice9) {
        unsafe {
            HR!(d3d_device.SetRenderState(D3DRS_WRAP0, D3DWRAP_U as u32));

            let mut w: D3DXMATRIX = std::mem::zeroed();
            let mut wit: D3DXMATRIX = std::mem::zeroed();
            let mut res: D3DXMATRIX = std::mem::zeroed();

            HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_ambient_mtrl, &self.sphere_mtrl.ambient as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
            HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_diffuse_mtrl, &self.sphere_mtrl.diffuse as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
            HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_specular_mtrl, &self.sphere_mtrl.spec as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
            HR!(ID3DXBaseEffect_SetFloat(self.fx, self.h_specular_power, self.sphere_mtrl.spec_power));

            HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.sphere_tex));

            let mut z: i32 = -30;
            while z <= 30 {
                D3DXMatrixTranslation(&mut w, -10.0, 7.5, z as f32);

                D3DXMatrixInverse(&mut wit, 0.0, &w);
                D3DXMatrixTranspose(&mut wit, &wit);

                D3DXMatrixMultiply(&mut res, &w, &self.view);
                D3DXMatrixMultiply(&mut res, &res, &self.proj);

                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &res));
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world, &w));
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world_inverse_transpose, &wit));

                HR!(ID3DXEffect_CommitChanges(self.fx));
                HR!(ID3DXBaseMesh_DrawSubset(self.sphere, 0));

                D3DXMatrixTranslation(&mut w, 10.0, 7.5, z as f32);

                D3DXMatrixInverse(&mut wit, 0.0, &w);
                D3DXMatrixTranspose(&mut wit, &wit);

                D3DXMatrixMultiply(&mut res, &w, &self.view);
                D3DXMatrixMultiply(&mut res, &res, &self.proj);

                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &res));
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world, &w));
                HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world_inverse_transpose, &wit));

                HR!(ID3DXEffect_CommitChanges(self.fx));
                HR!(ID3DXBaseMesh_DrawSubset(self.sphere, 0));

                z += 10;
            }


            // Disable.
            HR!(d3d_device.SetRenderState(D3DRS_WRAP0, 0));
        }
    }

    fn gen_spherical_tex_coords(d3d_device: IDirect3DDevice9, sphere: &mut LPD3DXMESH) {
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

    fn gen_cyl_tex_coords(d3d_device: IDirect3DDevice9, cylinder: &mut LPD3DXMESH, axis: Axis) {
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
}