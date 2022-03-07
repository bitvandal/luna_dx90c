use std::slice::from_raw_parts_mut;
use libc::c_void;
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

// Sample demo
pub struct StencilMirrorDemo {
    d3d_pp: *const D3DPRESENT_PARAMETERS,
    gfx_stats: Option<GfxStats>,

    room_vb: IDirect3DVertexBuffer9,
    teapot: LPD3DXMESH,

    floor_tex: *mut c_void,  //IDirect3DTexture9,
    wall_tex: *mut c_void,   //IDirect3DTexture9,
    mirror_tex: *mut c_void, //IDirect3DTexture9,
    teapot_tex: *mut c_void, //IDirect3DTexture9,

    fx: LPD3DXEFFECT,

    h_tech: D3DXHANDLE,
    h_wvp: D3DXHANDLE,
    h_world_inverse_transpose: D3DXHANDLE,
    h_light_vec_w: D3DXHANDLE,
    h_diffuse_mtrl: D3DXHANDLE,
    h_diffuse_light: D3DXHANDLE,
    h_ambient_mtrl: D3DXHANDLE,
    h_ambient_light: D3DXHANDLE,
    h_specular_mtrl: D3DXHANDLE,
    h_specular_light: D3DXHANDLE,
    h_specular_power: D3DXHANDLE,
    h_eye_pos: D3DXHANDLE,
    h_world: D3DXHANDLE,
    h_tex: D3DXHANDLE,

    white_mtrl: Mtrl,

    light_vec_w: D3DXVECTOR3,
    ambient_light: D3DXCOLOR,
    diffuse_light: D3DXCOLOR,
    specular_light: D3DXCOLOR,

    camera_rotation_y: f32,
    camera_radius: f32,
    camera_height: f32,

    room_world: D3DXMATRIX,
    teapot_world: D3DXMATRIX,

    view: D3DXMATRIX,
    proj: D3DXMATRIX,
}

impl StencilMirrorDemo {
    pub fn new(d3d_device: IDirect3DDevice9, d3d_pp: *const D3DPRESENT_PARAMETERS) -> Option<StencilMirrorDemo> {
        if !StencilMirrorDemo::check_device_caps() {
            display_error_then_quit("checkDeviceCaps() Failed");
        }

        init_all_vertex_declarations(d3d_device.clone());

        let mut gfx_stats = GfxStats::new(d3d_device.clone(), D3DCOLOR_XRGB!(0, 0, 0));

        let light_vec_w = D3DXVECTOR3 { x: 0.0, y: 0.707, z: -0.707 };
        let diffuse_light = D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        let ambient_light = D3DXCOLOR { r: 0.6, g: 0.6, b: 0.6, a: 1.0 };
        let specular_light = D3DXCOLOR { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

        let white_mtrl = Mtrl {
            ambient: WHITE,
            diffuse: WHITE,
            spec: WHITE.mult(0.8),
            spec_power: 16.0
        };

        let mut room_world: D3DXMATRIX = unsafe { std::mem::zeroed() };
        D3DXMatrixIdentity(&mut room_world);

        let mut teapot_world: D3DXMATRIX = unsafe { std::mem::zeroed() };
        D3DXMatrixTranslation(&mut teapot_world, 0.0, 3.0, -6.0);

        let mut floor_tex: *mut c_void = std::ptr::null_mut();
        let mut wall_tex: *mut c_void = std::ptr::null_mut();
        let mut mirror_tex: *mut c_void = std::ptr::null_mut();
        let mut teapot_tex: *mut c_void = std::ptr::null_mut();

        HR!(D3DXCreateTextureFromFile(d3d_device.clone(), PSTR(b"luna_26_stencil_mirror_demo/checkboard.dds\0".as_ptr() as _), &mut floor_tex));
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(), PSTR(b"luna_26_stencil_mirror_demo/brick2.dds\0".as_ptr() as _), &mut wall_tex));
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(), PSTR(b"luna_26_stencil_mirror_demo/ice.dds\0".as_ptr() as _), &mut mirror_tex));
        HR!(D3DXCreateTextureFromFile(d3d_device.clone(), PSTR(b"luna_26_stencil_mirror_demo/brick1.dds\0".as_ptr() as _), &mut teapot_tex));

        let mut teapot: LPD3DXMESH = std::ptr::null_mut();
        HR!(D3DXCreateTeapot(d3d_device.clone(), &mut teapot, std::ptr::null_mut()));

        // Generate texture coordinates for the teapot.
        StencilMirrorDemo::gen_spherical_tex_coords(d3d_device.clone(), &mut teapot);

        // Room geometry count.
        if let Some(gfx_stats) = &mut gfx_stats {
            gfx_stats.add_vertices(24);
            gfx_stats.add_triangles(8);

            // We draw the teapot twice--once normal and once reflected.
            gfx_stats.add_vertices(ID3DXBaseMesh_GetNumVertices(teapot) * 2);
            gfx_stats.add_triangles(ID3DXBaseMesh_GetNumFaces(teapot) * 2);
        }

        let room_vb = StencilMirrorDemo::build_room_geometry(d3d_device.clone());

        let (fx,
            h_tech,
            h_wvp,
            h_world_inverse_transpose,
            h_light_vec_w,
            h_ambient_mtrl,
            h_ambient_light,
            h_diffuse_mtrl,
            h_diffuse_light,
            h_specular_mtrl,
            h_specular_light,
            h_specular_power,
            h_eye_pos,
            h_world,
            h_tex) =
            StencilMirrorDemo::build_fx(d3d_device.clone());

        let mut stencil_mirror_demo = StencilMirrorDemo {
            d3d_pp,
            gfx_stats,

            room_vb,
            teapot,

            floor_tex,
            wall_tex,
            mirror_tex,
            teapot_tex,

            fx,

            h_tech,
            h_wvp,
            h_world_inverse_transpose,
            h_light_vec_w,
            h_diffuse_mtrl,
            h_diffuse_light,
            h_ambient_mtrl,
            h_ambient_light,
            h_specular_mtrl,
            h_specular_light,
            h_specular_power,
            h_eye_pos,
            h_world,
            h_tex,

            white_mtrl,

            light_vec_w,
            ambient_light,
            diffuse_light,
            specular_light,

            camera_radius: 15.0,
            camera_rotation_y: 1.4 * D3DX_PI,
            camera_height: 5.0,

            room_world,
            teapot_world,

            view: unsafe { std::mem::zeroed() },
            proj: unsafe { std::mem::zeroed() },
        };

        stencil_mirror_demo.on_reset_device();

        Some(stencil_mirror_demo)
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
                    (D3DCLEAR_TARGET | D3DCLEAR_ZBUFFER | D3DCLEAR_STENCIL) as u32,
                    0xFFEEEEEE,
                    1.0,
                    0));

                HR!(d3d_device.BeginScene());

                // Setup the rendering FX
                HR!(ID3DXEffect_SetTechnique(self.fx, self.h_tech));

                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_light_vec_w, &self.light_vec_w as *const _ as _, std::mem::size_of::<D3DXVECTOR3>() as u32));
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_diffuse_light, &self.diffuse_light as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_ambient_light, &self.ambient_light as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_specular_light, &self.specular_light as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));

                // All objects use the same material.
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_ambient_mtrl, &self.white_mtrl.ambient as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_diffuse_mtrl, &self.white_mtrl.diffuse as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
                HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_specular_mtrl, &self.white_mtrl.spec as *const _ as _, std::mem::size_of::<D3DXCOLOR>() as u32));
                HR!(ID3DXBaseEffect_SetFloat(self.fx, self.h_specular_power, self.white_mtrl.spec_power));

                self.draw_room(d3d_device.clone());
                self.draw_mirror(d3d_device.clone());
                self.draw_teapot(d3d_device.clone());

                self.draw_reflected_teapot(d3d_device.clone());

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

    fn build_fx(d3d_device: IDirect3DDevice9) -> (LPD3DXEFFECT, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
                                                  D3DXHANDLE, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
                                                  D3DXHANDLE, D3DXHANDLE, D3DXHANDLE, D3DXHANDLE,
                                                  D3DXHANDLE, D3DXHANDLE, D3DXHANDLE) {
        // Create the FX from a .fx file.
        let mut fx: LPD3DXEFFECT = std::ptr::null_mut();
        let mut errors: LPD3DXBUFFER = std::ptr::null_mut();

        HR!(D3DXCreateEffectFromFile(d3d_device, PSTR(b"luna_26_stencil_mirror_demo/dirLightTex.fx\0".as_ptr() as _),
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
        let h_light_vec_w = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gLightVecW\0".as_ptr() as _));
        let h_diffuse_mtrl = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gDiffuseMtrl\0".as_ptr() as _));
        let h_diffuse_light = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gDiffuseLight\0".as_ptr() as _));
        let h_ambient_mtrl = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gAmbientMtrl\0".as_ptr() as _));
        let h_ambient_light = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gAmbientLight\0".as_ptr() as _));
        let h_specular_mtrl = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gSpecularMtrl\0".as_ptr() as _));
        let h_specular_light = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gSpecularLight\0".as_ptr() as _));
        let h_specular_power = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gSpecularPower\0".as_ptr() as _));
        let h_eye_pos = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gEyePosW\0".as_ptr() as _));
        let h_world = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gWorld\0".as_ptr() as _));
        let h_tex = ID3DXBaseEffect_GetParameterByName(fx, std::ptr::null(), PSTR(b"gTex\0".as_ptr() as _));

        (fx, h_tech, h_wvp, h_world_inverse_transpose, h_light_vec_w, h_ambient_mtrl,
         h_ambient_light, h_diffuse_mtrl, h_diffuse_light,
         h_specular_mtrl, h_specular_light, h_specular_power, h_eye_pos, h_world, h_tex)
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

    fn build_room_geometry(d3d_device: IDirect3DDevice9) -> IDirect3DVertexBuffer9 {
        unsafe {
            // Create and specify geometry.  For this sample we draw a floor
            // and a wall with a mirror on it.  We put the floor, wall, and
            // mirror geometry in one vertex buffer.
            //
            //   |----|----|----|
            //   |Wall|Mirr|Wall|
            //   |    | or |    |
            //   /--------------/
            //  /   Floor      /
            // /--------------/

            let mut vb: Option<IDirect3DVertexBuffer9> = None;

            // Create the vertex buffer.
            HR!(d3d_device.CreateVertexBuffer((24 * std::mem::size_of::<VertexPNT>()) as u32,
                D3DUSAGE_WRITEONLY as u32, 0, D3DPOOL_MANAGED, &mut vb, std::ptr::null_mut()));

            let vertices: [VertexPNT; 24] = [
                // Floor: Observe we tile texture coordinates.
                VertexPNT { pos: D3DXVECTOR3 { x: -7.5, y: 0.0, z: -10.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 }, tex0: D3DXVECTOR2 { x: 0.0, y: 4.0 }},
                VertexPNT { pos: D3DXVECTOR3 { x: -7.5, y: 0.0, z:   0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 }, tex0: D3DXVECTOR2 { x: 0.0, y: 0.0 }},
                VertexPNT { pos: D3DXVECTOR3 { x:  7.5, y: 0.0, z:   0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 }, tex0: D3DXVECTOR2 { x: 4.0, y: 0.0 }},

                VertexPNT { pos: D3DXVECTOR3 { x: -7.5, y: 0.0, z: -10.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 }, tex0: D3DXVECTOR2 { x: 0.0, y: 4.0 }},
                VertexPNT { pos: D3DXVECTOR3 { x:  7.5, y: 0.0, z:   0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 }, tex0: D3DXVECTOR2 { x: 4.0, y: 0.0 }},
                VertexPNT { pos: D3DXVECTOR3 { x:  7.5, y: 0.0, z: -10.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 }, tex0: D3DXVECTOR2 { x: 4.0, y: 4.0 }},

                // Wall: Observe we tile texture coordinates, and that we
                // leave a gap in the middle for the mirror.
                VertexPNT { pos: D3DXVECTOR3 { x: -7.5, y: 0.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 0.0, y: 2.0 }},
                VertexPNT { pos: D3DXVECTOR3 { x: -7.5, y: 5.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 0.0, y: 0.0 }},
                VertexPNT { pos: D3DXVECTOR3 { x: -2.5, y: 5.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 2.0, y: 0.0 }},

                VertexPNT { pos: D3DXVECTOR3 { x: -7.5, y: 0.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 0.0, y: 2.0 }},
                VertexPNT { pos: D3DXVECTOR3 { x: -2.5, y: 5.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 2.0, y: 0.0 }},
                VertexPNT { pos: D3DXVECTOR3 { x: -2.5, y: 0.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 2.0, y: 2.0 }},

                VertexPNT { pos: D3DXVECTOR3 { x: 2.5, y: 0.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 0.0, y: 2.0 }},
                VertexPNT { pos: D3DXVECTOR3 { x: 2.5, y: 5.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 0.0, y: 0.0 }},
                VertexPNT { pos: D3DXVECTOR3 { x: 7.5, y: 5.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 2.0, y: 0.0 }},

                VertexPNT { pos: D3DXVECTOR3 { x: 2.5, y: 0.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 0.0, y: 2.0 }},
                VertexPNT { pos: D3DXVECTOR3 { x: 7.5, y: 5.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 2.0, y: 0.0 }},
                VertexPNT { pos: D3DXVECTOR3 { x: 7.5, y: 0.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 2.0, y: 2.0 }},

                // Mirror
                VertexPNT { pos: D3DXVECTOR3 { x: -2.5, y: 0.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 0.0, y: 1.0 }},
                VertexPNT { pos: D3DXVECTOR3 { x: -2.5, y: 5.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 0.0, y: 0.0 }},
                VertexPNT { pos: D3DXVECTOR3 { x:  2.5, y: 5.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 1.0, y: 0.0 }},

                VertexPNT { pos: D3DXVECTOR3 { x: -2.5, y: 0.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 0.0, y: 1.0 }},
                VertexPNT { pos: D3DXVECTOR3 { x:  2.5, y: 5.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 1.0, y: 0.0 }},
                VertexPNT { pos: D3DXVECTOR3 { x:  2.5, y: 0.0, z: 0.0 }, normal: D3DXVECTOR3 { x: 0.0, y: 0.0, z: -1.0 }, tex0: D3DXVECTOR2 { x: 1.0, y: 1.0 }},
            ];

            if let Some(vb) = &mut vb {
                let mut v = std::ptr::null_mut();
                HR!(vb.Lock(0, 0, &mut v, 0));

                // Write box vertices to the vertex buffer.
                std::ptr::copy_nonoverlapping(vertices.as_ptr(),
                                              v as *mut VertexPNT,
                                              vertices.len());
                HR!(vb.Unlock());
            }

            vb.unwrap()
        }
    }

    fn draw_room(&self, d3d_device: IDirect3DDevice9) {
        unsafe {
            let mut res: D3DXMATRIX = std::mem::zeroed();
            D3DXMatrixMultiply(&mut res, &self.room_world, &self.view);
            D3DXMatrixMultiply(&mut res, &res, &self.proj);
            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &res));

            let mut world_inverse_transpose: D3DXMATRIX = std::mem::zeroed();
            D3DXMatrixInverse(&mut world_inverse_transpose, 0.0, &self.room_world);
            D3DXMatrixTranspose(&mut world_inverse_transpose, &world_inverse_transpose);
            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world_inverse_transpose, &world_inverse_transpose));

            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world, &self.room_world));

            HR!(d3d_device.SetVertexDeclaration(&VERTEX_PNT_DECL));
            HR!(d3d_device.SetStreamSource(0, &self.room_vb, 0, std::mem::size_of::<VertexPNT>() as u32));

            // Begin passes.
            let mut num_passes: u32 = 0;
            HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));
            for i in 0..num_passes {
                HR!(ID3DXEffect_BeginPass(self.fx, i));

                // draw the floor
                HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.floor_tex));
                HR!(ID3DXEffect_CommitChanges(self.fx));
                HR!(d3d_device.DrawPrimitive(D3DPT_TRIANGLELIST, 0, 2));

                // draw the walls
                HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.wall_tex));
                HR!(ID3DXEffect_CommitChanges(self.fx));
                HR!(d3d_device.DrawPrimitive(D3DPT_TRIANGLELIST, 6, 4));

                HR!(ID3DXEffect_EndPass(self.fx));
            }
            HR!(ID3DXEffect_End(self.fx));
        }
    }

    fn draw_mirror(&self, d3d_device: IDirect3DDevice9) {
        unsafe {
            let mut res: D3DXMATRIX = std::mem::zeroed();
            D3DXMatrixMultiply(&mut res, &self.room_world, &self.view);
            D3DXMatrixMultiply(&mut res, &res, &self.proj);
            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &res));

            let mut world_inverse_transpose: D3DXMATRIX = std::mem::zeroed();
            D3DXMatrixInverse(&mut world_inverse_transpose, 0.0, &self.room_world);
            D3DXMatrixTranspose(&mut world_inverse_transpose, &world_inverse_transpose);
            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world_inverse_transpose, &world_inverse_transpose));

            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world, &self.room_world));
            HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.mirror_tex));

            HR!(d3d_device.SetVertexDeclaration(&VERTEX_PNT_DECL));
            HR!(d3d_device.SetStreamSource(0, &self.room_vb, 0, std::mem::size_of::<VertexPNT>() as u32));

            // Begin passes.
            let mut num_passes: u32 = 0;
            HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));
            for i in 0..num_passes {
                HR!(ID3DXEffect_BeginPass(self.fx, i));

                // draw the mirror
                HR!(ID3DXEffect_CommitChanges(self.fx));
                HR!(d3d_device.DrawPrimitive(D3DPT_TRIANGLELIST, 18, 2));

                HR!(ID3DXEffect_EndPass(self.fx));
            }
            HR!(ID3DXEffect_End(self.fx));
        }
    }

    fn draw_teapot(&self, d3d_device: IDirect3DDevice9) {
        unsafe {
            // Cylindrically interpolate texture coordinates.
            HR!(d3d_device.SetRenderState(D3DRS_WRAP0, D3DWRAPCOORD_0 as u32));

            let mut res: D3DXMATRIX = std::mem::zeroed();
            D3DXMatrixMultiply(&mut res, &self.teapot_world, &self.view);
            D3DXMatrixMultiply(&mut res, &res, &self.proj);
            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_wvp, &res));

            let mut world_inverse_transpose: D3DXMATRIX = std::mem::zeroed();
            D3DXMatrixInverse(&mut world_inverse_transpose, 0.0, &self.teapot_world);
            D3DXMatrixTranspose(&mut world_inverse_transpose, &world_inverse_transpose);
            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world_inverse_transpose, &world_inverse_transpose));
            HR!(ID3DXBaseEffect_SetMatrix(self.fx, self.h_world, &self.teapot_world));
            HR!(ID3DXBaseEffect_SetTexture(self.fx, self.h_tex, self.teapot_tex));

            // Begin passes.
            let mut num_passes: u32 = 0;
            HR!(ID3DXEffect_Begin(self.fx, &mut num_passes, 0));
            for i in 0..num_passes {
                HR!(ID3DXEffect_BeginPass(self.fx, i));
                HR!(ID3DXBaseMesh_DrawSubset(self.teapot, 0));
                HR!(ID3DXEffect_EndPass(self.fx));
            }
            HR!(ID3DXEffect_End(self.fx));

            // Disable wrap.
            HR!(d3d_device.SetRenderState(D3DRS_WRAP0, 0));
        }
    }

    fn draw_reflected_teapot(&mut self, d3d_device: IDirect3DDevice9) {
        unsafe {
            HR!(d3d_device.SetRenderState(D3DRS_STENCILENABLE, 1));
            HR!(d3d_device.SetRenderState(D3DRS_STENCILFUNC, D3DCMP_ALWAYS.0 as u32));
            HR!(d3d_device.SetRenderState(D3DRS_STENCILREF, 0x1));
            HR!(d3d_device.SetRenderState(D3DRS_STENCILMASK, 0xffffffff));
            HR!(d3d_device.SetRenderState(D3DRS_STENCILWRITEMASK, 0xffffffff));
            HR!(d3d_device.SetRenderState(D3DRS_STENCILZFAIL, D3DSTENCILOP_KEEP.0));
            HR!(d3d_device.SetRenderState(D3DRS_STENCILFAIL, D3DSTENCILOP_KEEP.0));
            HR!(d3d_device.SetRenderState(D3DRS_STENCILPASS, D3DSTENCILOP_REPLACE.0));

            // Disable writes to the depth and back buffers
            HR!(d3d_device.SetRenderState(D3DRS_ZWRITEENABLE, 0));
            HR!(d3d_device.SetRenderState(D3DRS_ALPHABLENDENABLE, 1));
            HR!(d3d_device.SetRenderState(D3DRS_SRCBLEND, D3DBLEND_ZERO.0));
            HR!(d3d_device.SetRenderState(D3DRS_DESTBLEND, D3DBLEND_ONE.0));

            // Draw mirror to stencil only.
            self.draw_mirror(d3d_device.clone());

            // Re-enable depth writes
            HR!(d3d_device.SetRenderState(D3DRS_ZWRITEENABLE, 1));

            // Only draw reflected teapot to the pixels where the mirror
            // was drawn to.
            HR!(d3d_device.SetRenderState(D3DRS_STENCILFUNC, D3DCMP_EQUAL.0 as u32));
            HR!(d3d_device.SetRenderState(D3DRS_STENCILPASS, D3DSTENCILOP_KEEP.0));

            // Build Reflection transformation.
            let mut r: D3DXMATRIX = std::mem::zeroed();
            let plane = D3DXPLANE { a: 0.0, b: 0.0, c: 1.0, d: 0.0 } ; // xy plane
            D3DXMatrixReflect(&mut r, &plane);

            // Save the original teapot world matrix.
            let old_teapot_world: D3DXMATRIX = self.teapot_world;

            // Add reflection transform.
            let mut res = std::mem::zeroed();
            D3DXMatrixMultiply(&mut res, &self.teapot_world, &r);
            self.teapot_world = res;

            // Reflect light vector also.
            let old_light_vec_w = self.light_vec_w;

            D3DXVec3TransformNormal(&mut self.light_vec_w, &self.light_vec_w, &r);
            HR!(ID3DXBaseEffect_SetValue(self.fx, self.h_light_vec_w, &self.light_vec_w as *const _ as _, std::mem::size_of::<D3DXVECTOR3>() as u32));

            // Disable depth buffer and render the reflected teapot.
            HR!(d3d_device.SetRenderState(D3DRS_ZENABLE, 0));
            HR!(d3d_device.SetRenderState(D3DRS_ALPHABLENDENABLE, 0));

            // Finally, draw the reflected teapot
            HR!(d3d_device.SetRenderState(D3DRS_CULLMODE, D3DCULL_CW.0));
            self.draw_teapot(d3d_device.clone());

            self.teapot_world = old_teapot_world;
            self.light_vec_w = old_light_vec_w;

            // Restore render states.
            HR!(d3d_device.SetRenderState(D3DRS_ZENABLE, 1));
            HR!(d3d_device.SetRenderState(D3DRS_STENCILENABLE, 0));
            HR!(d3d_device.SetRenderState(D3DRS_CULLMODE, D3DCULL_CCW.0));
        }
    }
}