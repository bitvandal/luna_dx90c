use libc::c_void;
use windows::Win32::Graphics::Direct3D9::*;
use common::*;
use d3dx::*;
use crate::*;

pub struct DrawableTex2D {
    tex: *mut c_void,       // IDirect3DTexture9
    rts: *mut c_void,       // ID3DXRenderToSurface
    top_surf: *mut c_void,  // IDirect3DSurface9

    width: u32,
    height: u32,
    mip_levels: u32,
    tex_format: D3DFORMAT,
    use_depth_buffer: bool,
    depth_format: D3DFORMAT,
    viewport: D3DVIEWPORT9,
    auto_gen_mips: bool,
}

impl DrawableTex2D {
    pub fn new(width: u32, height: u32, mip_levels: u32, tex_format: D3DFORMAT, use_depth_buffer: bool,
               depth_format: D3DFORMAT, viewport: &D3DVIEWPORT9, auto_gen_mips: bool) -> DrawableTex2D {
        DrawableTex2D {
            tex: std::ptr::null_mut(),
            rts: std::ptr::null_mut(),
            top_surf: std::ptr::null_mut(),
            width,
            height,
            mip_levels,
            tex_format,
            use_depth_buffer,
            depth_format,
            viewport: viewport.clone(),
            auto_gen_mips,
        }
    }

    pub fn release_com_objects(&self) {
        self.on_lost_device();
    }

    pub fn d3d_tex(&self) -> *const c_void {
        self.tex
    }

    pub fn on_lost_device(&self) {
        ReleaseCOM(self.tex);
        ReleaseCOM(self.rts);
        ReleaseCOM(self.top_surf);
    }

    pub fn on_reset_device(&mut self) {
        unsafe {
            let mut usage = D3DUSAGE_RENDERTARGET;
            if self.auto_gen_mips {
                usage |= D3DUSAGE_AUTOGENMIPMAP;
            }

            if let Some(d3d_device) = &D3D_DEVICE {
                HR!(D3DXCreateTexture(d3d_device.clone(), self.width, self.height, self.mip_levels,
                    usage as u32, self.tex_format, D3DPOOL_DEFAULT, &mut self.tex));

                let use_depth_buffer: i32 = if self.use_depth_buffer { 1 } else { 0 };
                HR!(D3DXCreateRenderToSurface(d3d_device.clone(), self.width, self.height,
                    self.tex_format, use_depth_buffer, self.depth_format, &mut self.rts));

                HR!(IDirect3DTexture9_GetSurfaceLevel(self.tex, 0, &mut self.top_surf));
            }
        }
    }

    pub fn begin_scene(&self) {
        HR!(ID3DXRenderToSurface_BeginScene(self.rts, self.top_surf, &self.viewport));
    }

    pub fn end_scene(&self) {
        HR!(ID3DXRenderToSurface_EndScene(self.rts, D3DX_FILTER_NONE));
    }
}