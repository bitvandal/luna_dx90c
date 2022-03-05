use libc::*;

use windows::{
    Win32::Foundation::*, Win32::Graphics::Direct3D9::*, Win32::Graphics::Gdi::*,
};

use crate::d3d9_extra::*;
use crate::*;

pub struct GfxStats {
    font: *mut c_void,
    fps: f32,
    millisec_per_frame: f32,
    num_tris: u32,
    num_vertices: u32,
}

impl GfxStats {
    pub fn new(d3d_device: IDirect3DDevice9) -> Option<GfxStats> {
        let mut gfx_stats = GfxStats {
            font: std::ptr::null_mut(),
            fps: 0.0,
            millisec_per_frame: 0.0,
            num_tris: 0,
            num_vertices: 0,
        };

        let font_desc = D3DXFONT_DESC {
            Height: 18,
            Width: 0,
            Weight: 0,
            MipLevels: 1,
            Italic: false,
            CharSet: DEFAULT_CHARSET as c_uchar,
            OutputPrecision: OUT_DEFAULT_PRECIS.0 as c_uchar,
            Quality: DEFAULT_QUALITY.0 as c_uchar,
            PitchAndFamily: (DEFAULT_PITCH | FF_DONTCARE.0) as c_uchar,
            FaceName: PSTR(b"Times New Roman\0".as_ptr() as _),
        };

        HR!(D3DXCreateFontIndirect(d3d_device, font_desc, &mut gfx_stats.font));

        Some(gfx_stats)
    }

    pub fn on_lost_device(&self) {
        HR!(ID3DXFont_OnLostDevice(self.font));
    }

    pub fn on_reset_device(&self) {
        HR!(ID3DXFont_OnResetDevice(self.font));
    }

    #[allow(unused)]
    fn add_vertices(&mut self, n: u32) {
        self.num_vertices += n;
    }

    #[allow(unused)]
    fn sub_vertices(&mut self, n: u32) {
        self.num_vertices -= n;
    }

    #[allow(unused)]
    fn add_triangles(&mut self, n: u32) {
        self.num_tris += n;
    }

    #[allow(unused)]
    fn sub_triangles(&mut self, n: u32) {
        self.num_tris -= n;
    }

    #[allow(unused)]
    pub fn set_tri_count(&mut self, n: u32) {
        self.num_tris = n;
    }

    #[allow(unused)]
    pub fn set_vertex_count(&mut self, n: u32) {
        self.num_vertices = n;
    }

    pub fn update(&mut self, dt: f32) {
        unsafe {
            // Make static so that their values persist across function calls.
            static mut NUM_FRAMES: f32 = 0.0;
            static mut TIME_ELAPSED: f32 = 0.0;

            // Increment the frame count.
            NUM_FRAMES += 1.0;

            // Accumulate how much time has passed.
            TIME_ELAPSED += dt;

            // Has one second passed?--we compute the frame statistics once
            // per second.  Note that the time between frames can vary so
            // these stats are averages over a second.
            if TIME_ELAPSED >= 1.0 {
                // Frames Per Second = numFrames / timeElapsed,
                // but timeElapsed approx. equals 1.0, so
                // frames per second = numFrames.
                self.fps = NUM_FRAMES;

                // Average time, in milliseconds, it took to render a single frame.
                self.millisec_per_frame = 1000.0 / self.fps;

                // Reset time counter and frame count to prepare for computing
                // the average stats over the next second.
                TIME_ELAPSED = 0.0;
                NUM_FRAMES = 0.0;
            }
        }
    }

    pub fn display(&self) {
        // (Maybe) an improvement can be to reserve some fixed size memory in the heap
        // and use it instead of creating a new String on every frame. This is what the
        // book does anyway.
        let buffer: String = format!("Frames Per Second = {:.2}\n \
                Milliseconds Per Frame = {:.4}\n \
                Triangle Count = {}\n \
                Vertex Count = {}\0",
                         self.fps, self.millisec_per_frame, self.num_tris, self.num_vertices);

        let r = RECT { left: 5, top: 5, right: 0, bottom: 0 };

        ID3DXFont_DrawText(self.font, std::ptr::null_mut(),
                           PSTR(buffer.as_ptr() as _),
                           -1,
                           &r,
                           DT_NOCLIP.0,
                           D3DCOLOR_XRGB!(255, 255, 255));
    }

    pub fn release_com_objects(&self) {
        ReleaseCOM(self.font);
    }
}