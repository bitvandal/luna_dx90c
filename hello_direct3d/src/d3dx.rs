// Uninspired D3DX9 bindings, made for the sake of running some old book examples.
// Bindings use ANSI, not UNICODE

use libc::*;

use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct3D9::*,
};

// struct D3DXFONT_DESCA (ANSI)
#[allow(non_snake_case)]
#[repr(C)]
pub struct D3DXFONT_DESC {
    pub Height: c_int,
    pub Width: c_uint,
    pub Weight: c_uint,
    pub MipLevels: c_uint,
    pub Italic: bool,
    pub CharSet: c_uchar,
    pub OutputPrecision: c_uchar,
    pub Quality: c_uchar,
    pub PitchAndFamily: c_uchar,
    pub FaceName: PSTR
}

#[allow(non_snake_case)]
#[link(name = "d3dx9", kind = "static")]
#[link(name = "d3dx9_bindings", kind = "static")]
extern {
    // ULONG IUnknown::Release()
    fn D3DX_Release(obj: *const c_void);

    // HRESULT D3DXCreateFontIndirect(LPDIRECT3DDEVICE9 pDevice, const D3DXFONT_DESC *pDesc, LPD3DXFONT *ppFont);
    fn D3DX_CreateFontIndirect(pDevice: IDirect3DDevice9, pDesc: *const D3DXFONT_DESC, ppFont: *mut *mut c_void) -> i32;

    // HRESULT ID3DXFont::OnLostDevice()
    fn D3DX_ID3DXFont_OnLostDevice(pFont: *const c_void) -> i32;

    // HRESULT ID3DXFont::OnResetDevice()
    fn D3DX_ID3DXFont_OnResetDevice(pFont: *const c_void) -> i32;

    // INT DrawText(LPD3DXSPRITE pSprite, LPCTSTR pString, INT Count, LPRECT pRect, DWORD Format, D3DCOLOR Color);
    fn D3DX_ID3DXFont_DrawText(pFont: *const c_void, pSprite: *const c_void, pString: PSTR, Count: i32,
                               pRect: *const RECT, Format: u32, Color: u32) -> i32;
}

fn to_result(code: i32) -> Result<()> {
    HRESULT(code as u32).ok()
}

#[allow(non_snake_case)]
pub fn ReleaseCOM(com_obj: *mut c_void) {
    unsafe { D3DX_Release(com_obj); }
}

#[allow(non_snake_case)]
pub fn D3DXCreateFontIndirect(pDevice: IDirect3DDevice9, font_desc: D3DXFONT_DESC, ppFont: &mut *mut c_void) -> Result<()> {
    unsafe { to_result(D3DX_CreateFontIndirect(pDevice, &font_desc, ppFont)) }
}

#[allow(non_snake_case)]
pub fn ID3DXFont_DrawText(pFont: *mut c_void, pSprite: *mut c_void, pString: PSTR, Count: i32,
                      pRect: &mut RECT, Format: u32, Color: u32) -> i32 {
    unsafe { D3DX_ID3DXFont_DrawText(pFont, pSprite, pString, Count, pRect, Format, Color) }
}

#[allow(non_snake_case)]
pub fn ID3DXFont_OnLostDevice(pFont: *mut c_void) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXFont_OnLostDevice(pFont)) }
}

#[allow(non_snake_case)]
pub fn ID3DXFont_OnResetDevice(pFont: *mut c_void) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXFont_OnResetDevice(pFont)) }
}