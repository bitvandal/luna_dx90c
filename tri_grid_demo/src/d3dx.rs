// Uninspired D3DX9 bindings, made for the sake of running some old book examples.
// Bindings use ANSI, not UNICODE

use libc::*;

use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct3D::*, Win32::Graphics::Direct3D9::*,
};

use crate::D3DCOLOR;

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

// D3DX Core

pub type LPD3DXBUFFER = *mut c_void;

// D3DX Math

pub type D3DXVECTOR3 = D3DVECTOR;
pub type D3DXMATRIX = D3DMATRIX;

pub const D3DX_PI: f32 = 3.141592654;

// Misc

#[allow(non_camel_case_types)]
type D3DX_HRESULT = i32;

// D3DX EFFECT
pub type LPD3DXEFFECT = *mut c_void;

// D3DX SHADER
pub type D3DXHANDLE = *const c_void; // technically it is normally a LPCSTR

// D3DXSHADER flags

pub const D3DXSHADER_DEBUG: u32 = 1 << 0;
pub const D3DXSHADER_SKIPVALIDATION: u32 = 1 << 1;
pub const D3DXSHADER_SKIPOPTIMIZATION: u32 = 1 << 2;
pub const D3DXSHADER_PACKMATRIX_ROWMAJOR: u32 = 1 << 3;
pub const D3DXSHADER_PACKMATRIX_COLUMNMAJOR: u32 = 1 << 4;
pub const D3DXSHADER_PARTIALPRECISION: u32 = 1 << 5;
pub const D3DXSHADER_FORCE_VS_SOFTWARE_NOOPT: u32 = 1 << 6;
pub const D3DXSHADER_FORCE_PS_SOFTWARE_NOOPT: u32 = 1 << 7;
pub const D3DXSHADER_NO_PRESHADER: u32 = 1 << 8;
pub const D3DXSHADER_AVOID_FLOW_CONTROL: u32 = 1 << 9;
pub const D3DXSHADER_PREFER_FLOW_CONTROL: u32 = 1 << 10;
pub const D3DXSHADER_ENABLE_BACKWARDS_COMPATIBILITY: u32 = 1 << 12;
pub const D3DXSHADER_IEEE_STRICTNESS: u32 = 1 << 13;
pub const D3DXSHADER_USE_LEGACY_D3DX9_31_DLL: u32 = 1 << 16;

// D3DX Functions

#[allow(non_snake_case)]
#[link(name = "d3dx9", kind = "static")]
#[link(name = "d3dx9_bindings", kind = "static")]
extern {
    // ULONG IUnknown::Release()
    fn D3DX_Release(obj: *const c_void);

    // FONTS

    // HRESULT D3DXCreateFontIndirect(LPDIRECT3DDEVICE9 pDevice, const D3DXFONT_DESC *pDesc, LPD3DXFONT *ppFont);
    fn D3DX_CreateFontIndirect(pDevice: IDirect3DDevice9, pDesc: *const D3DXFONT_DESC, ppFont: *mut *mut c_void) -> D3DX_HRESULT;

    // HRESULT ID3DXFont::OnLostDevice()
    fn D3DX_ID3DXFont_OnLostDevice(pFont: *const c_void) -> D3DX_HRESULT;

    // HRESULT ID3DXFont::OnResetDevice()
    fn D3DX_ID3DXFont_OnResetDevice(pFont: *const c_void) -> D3DX_HRESULT;

    // INT ID3DXFont::DrawText(LPD3DXSPRITE pSprite, LPCTSTR pString, INT Count, LPRECT pRect, DWORD Format, D3DCOLOR Color);
    fn D3DX_ID3DXFont_DrawText(pFont: *const c_void, pSprite: *const c_void, pString: PSTR, Count: i32,
                               pRect: *const RECT, Format: u32, Color: u32) -> D3DX_HRESULT;

    // SPRITES

    // HRESULT D3DXCreateSprite(LPDIRECT3DDEVICE9 pDevice, LPD3DXSPRITE *ppSprite);
    fn D3DX_CreateSprite(pDevice: IDirect3DDevice9, ppSprite: *mut *mut c_void) -> D3DX_HRESULT;

    // HRESULT D3DXCreateTextureFromFile(LPDIRECT3DDEVICE9 pDevice, LPCTSTR pSrcFile, LPDIRECT3DTEXTURE9 *ppTexture);
    fn D3DX_CreateTextureFromFile(pDevice: IDirect3DDevice9, pSrcFile: PSTR, ppTexture: *mut *mut c_void) -> D3DX_HRESULT;

    // HRESULT ID3DXSprite::Begin(DWORD Flags);
    fn D3DX_ID3DXSprite_Begin(pSprite: *const c_void, flags: u32) -> D3DX_HRESULT;

    // HRESULT ID3DXSprite::Draw(LPDIRECT3DTEXTURE9 pTexture, const RECT *pSrcRect, const D3DXVECTOR3 *pCenter, const D3DXVECTOR3 *pPosition, D3DCOLOR Color);
    fn D3DX_ID3DXSprite_Draw(pSprite: *const c_void, pTexture: *const c_void, pSrcRect: *const RECT,
                             pCenter: *const D3DXVECTOR3, pPosition: *const D3DXVECTOR3, Color: D3DCOLOR) -> D3DX_HRESULT;

    // HRESULT ID3DXSprite::End();
    fn D3DX_ID3DXSprite_End(pSprite: *const c_void) -> D3DX_HRESULT;

    // HRESULT ID3DXSprite::Flush();
    fn D3DX_ID3DXSprite_Flush(pSprite: *const c_void) -> D3DX_HRESULT;

    // HRESULT ID3DXSprite::OnLostDevice()
    fn D3DX_ID3DXSprite_OnLostDevice(pSprite: *const c_void) -> D3DX_HRESULT;

    // HRESULT ID3DXSprite::OnResetDevice()
    fn D3DX_ID3DXSprite_OnResetDevice(pSprite: *const c_void) -> D3DX_HRESULT;

    // HRESULT ID3DXSprite::SetTransform(const D3DXMATRIX *pTransform)
    fn D3DX_ID3DXSprite_SetTransform(pSprite: *const c_void, pTransform: *const D3DXMATRIX) -> D3DX_HRESULT;

    // EFFECTS

    // HRESULT D3DX_ID3DXEffect_Begin(LPD3DXEFFECT self, UINT *pPasses, DWORD Flags) {
    fn D3DX_ID3DXEffect_Begin(pEffect: LPD3DXEFFECT, pPasses: *const u32, Flags: u32) -> D3DX_HRESULT;

    // HRESULT D3DX_ID3DXEffect_BeginPass(LPD3DXEFFECT self, UINT Pass) {
    fn D3DX_ID3DXEffect_BeginPass(pEffect: LPD3DXEFFECT, Pass: u32) -> D3DX_HRESULT;

    // HRESULT D3DX_ID3DXEffect_End(LPD3DXEFFECT self) {
    fn D3DX_ID3DXEffect_End(pEffect: LPD3DXEFFECT) -> D3DX_HRESULT;

    // HRESULT D3DX_ID3DXEffect_EndPass(LPD3DXEFFECT self) {
    fn D3DX_ID3DXEffect_EndPass(pEffect: LPD3DXEFFECT) -> D3DX_HRESULT;

    // D3DXHANDLE ID3DXBaseEffect::GetTechniqueByName(LPCSTR pName)
    fn D3DX_ID3DXBaseEffect_GetTechniqueByName(pEffect: LPD3DXEFFECT, pName: PSTR) -> D3DXHANDLE;

    // D3DXHANDLE ID3DXBaseEffect::GetParameterByName(D3DXHANDLE hParameter, LPCSTR pName)
    fn D3DX_ID3DXBaseEffect_GetParameterByName(pEffect: LPD3DXEFFECT, hParameter: D3DXHANDLE, pName: PSTR) -> D3DXHANDLE;

    // HRESULT ID3DXEffect::OnLostDevice()
    fn D3DX_ID3DXEffect_OnLostDevice(pEffect: *const c_void) -> D3DX_HRESULT;

    // HRESULT ID3DXEffect::OnResetDevice()
    fn D3DX_ID3DXEffect_OnResetDevice(pEffect: *const c_void) -> D3DX_HRESULT;

    // HRESULT SetMatrix(D3DXHANDLE hParameter, const D3DXMATRIX *pMatrix)
    fn D3DX_ID3DXBaseEffect_SetMatrix(pEffect: *const c_void, hParameter: D3DXHANDLE, pMatrix: *const D3DXMATRIX) -> D3DX_HRESULT;

    // HRESULT SetTechnique(D3DXHANDLE hTechnique)
    fn D3DX_ID3DXEffect_SetTechnique(pEffect: *const c_void, h_technique: D3DXHANDLE) -> D3DX_HRESULT;

    // BUFFERS

    // LPVOID D3DX_ID3DXBuffer_GetBufferPointer()
    fn D3DX_ID3DXBuffer_GetBufferPointer(pBuffer: *const c_void) -> *mut c_void;

    // MATH

    // D3DXVECTOR3* D3DX_Vec3Scale(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV, FLOAT s)
    fn D3DX_Vec3Scale(pOut: *mut D3DXVECTOR3, pV: *const D3DXVECTOR3, s: f32) -> *mut D3DXVECTOR3;

    // D3DXVECTOR3* D3DX_Vec3Add(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV1, const D3DXVECTOR3 *pV1)
    fn D3DX_Vec3Add(pOut: *mut D3DXVECTOR3, pV1: *const D3DXVECTOR3, pV2: *const D3DXVECTOR3) -> *mut D3DXVECTOR3;

    // D3DXMATRIX* D3DXMatrixLookAtLH(D3DXMATRIX *pOut, const D3DXVECTOR3 *pEye, const D3DXVECTOR3 *pAt, const D3DXVECTOR3 *pUp)
    fn D3DX_MatrixLookAtLH(pOut: *mut D3DXMATRIX, pEye: *const D3DXVECTOR3, pAt: *const D3DXVECTOR3,
                           pUp: *const D3DXVECTOR3) -> *mut D3DXMATRIX;

    // D3DXMATRIX* D3DXMatrixPerspectiveFovLH(D3DXMATRIX *pOut, FLOAT fovy, FLOAT Aspect, FLOAT zn, FLOAT zf);
    fn D3DX_MatrixPerspectiveFovLH(pOut: *mut D3DXMATRIX, fovy: f32, Aspect: f32, zn: f32, zf: f32) -> *mut D3DXMATRIX;

    // D3DXMATRIX* D3DXMatrixScaling(D3DXMATRIX *pOut, FLOAT sx, FLOAT sy, FLOAT sz);
    fn D3DX_MatrixScaling(pOut: *mut D3DXMATRIX, sx: f32, sy: f32, sz: f32) -> *mut D3DXMATRIX;

    // D3DXMATRIX* D3DXMatrixTranslation (D3DXMATRIX *pOut, FLOAT x, FLOAT y, FLOAT z);
    fn D3DX_MatrixTranslation(pOut: *mut D3DXMATRIX, x: f32, y: f32, z: f32) -> *mut D3DXMATRIX;

    // D3DXMATRIX* D3DX_MatrixMultiply(D3DXMATRIX *pOut, const D3DXMATRIX *pM1, const D3DXMATRIX *pM2)
    fn D3DX_MatrixMultiply(pOut: *mut D3DXMATRIX, pM1: *const D3DXMATRIX, pM2: *const D3DXMATRIX) -> *mut D3DXMATRIX;

    // D3DXMATRIX* D3DX_MatrixRotationZ(D3DXMATRIX *pOut, FLOAT Angle)
    fn D3DX_MatrixRotationZ(pOut: *mut D3DXMATRIX, Angle: f32) -> *mut D3DXMATRIX;

    // D3DXMATRIX* D3DXMatrixIdentity(D3DXMATRIX *pOut)
    fn D3DX_MatrixIdentity(pOut: *mut D3DXMATRIX) -> *mut D3DXMATRIX;

    // D3DXVECTOR3* D3DXVec3TransformCoord(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV, const D3DXMATRIX *pM)
    fn D3DX_Vec3TransformCoord(pOut: *mut D3DXVECTOR3, pV: *const D3DXVECTOR3, pM: *const D3DXMATRIX) -> *mut D3DXVECTOR3;

    // HRESULT D3DXCreateEffectFromFile(LPDIRECT3DDEVICE9 pDevice, LPCTSTR pSrcFile, const D3DXMACRO *pDefines,
    //                                  LPD3DXINCLUDE pInclude, DWORD Flags, LPD3DXEFFECTPOOL pPool,
    //                                  LPD3DXEFFECT *ppEffect, LPD3DXBUFFER *ppCompilationErrors)
    fn D3DX_CreateEffectFromFile(pDevice: IDirect3DDevice9, pSrcFile: PSTR, pDefines: *const c_void,
                                 pInclude: *const c_void, flags: u32, pPool: *const c_void,
                                 ppEffect: *mut LPD3DXEFFECT, ppCompilationErrors: *mut LPD3DXBUFFER) -> D3DX_HRESULT;
}

fn to_result(code: D3DX_HRESULT) -> Result<()> {
    HRESULT(code as u32).ok()
}

// D3DXSPRITE flags

pub const D3DXSPRITE_DONOTSAVESTATE: u32 = 1 << 0;
pub const D3DXSPRITE_DONOTMODIFY_RENDERSTATE: u32 = 1 << 1;
pub const D3DXSPRITE_OBJECTSPACE: u32 = 1 << 2;
pub const D3DXSPRITE_BILLBOARD: u32 = 1 << 3;
pub const D3DXSPRITE_ALPHABLEND: u32 = 1 << 4;
pub const D3DXSPRITE_SORT_TEXTURE: u32 = 1 << 5;
pub const D3DXSPRITE_SORT_DEPTH_FRONTTOBACK: u32 = 1 << 6;
pub const D3DXSPRITE_SORT_DEPTH_BACKTOFRONT: u32 = 1 << 7;
pub const D3DXSPRITE_DO_NOT_ADDREF_TEXTURE: u32 = 1 << 8;

// Function bindings

#[allow(non_snake_case)]
pub fn ReleaseCOM(com_obj: *const c_void) {
    unsafe { D3DX_Release(com_obj); }
}

#[allow(non_snake_case)]
pub fn D3DXCreateFontIndirect(pDevice: IDirect3DDevice9, font_desc: D3DXFONT_DESC, ppFont: &mut *mut c_void) -> Result<()> {
    unsafe { to_result(D3DX_CreateFontIndirect(pDevice, &font_desc, ppFont)) }
}

#[allow(non_snake_case)]
pub fn ID3DXFont_DrawText(pFont: *const c_void, pSprite: *const c_void, pString: PSTR, Count: i32,
                      pRect: &RECT, Format: u32, Color: D3DCOLOR) -> i32 {
    unsafe { D3DX_ID3DXFont_DrawText(pFont, pSprite, pString, Count, pRect, Format, Color) }
}

#[allow(non_snake_case)]
pub fn ID3DXFont_OnLostDevice(pFont: *const c_void) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXFont_OnLostDevice(pFont)) }
}

#[allow(non_snake_case)]
pub fn ID3DXFont_OnResetDevice(pFont: *const c_void) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXFont_OnResetDevice(pFont)) }
}

#[allow(non_snake_case)]
pub fn D3DXCreateSprite(pDevice: IDirect3DDevice9, ppSprite: &mut *mut c_void) -> Result<()> {
    unsafe { to_result(D3DX_CreateSprite(pDevice, ppSprite)) }
}

#[allow(non_snake_case)]
pub fn D3DXCreateTextureFromFile(pDevice: IDirect3DDevice9, pSrcFile: PSTR, ppTexture: &mut *mut c_void) -> Result<()> {
    unsafe { to_result(D3DX_CreateTextureFromFile(pDevice, pSrcFile, ppTexture)) }
}

#[allow(non_snake_case)]
pub fn ID3DXSprite_Begin(pSprite: *const c_void, flags: u32) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXSprite_Begin(pSprite, flags)) }
}

#[allow(non_snake_case)]
pub fn ID3DXSprite_Draw(pSprite: *const c_void, pTexture: *const c_void, pSrcRect: *const RECT,
                        pCenter: *const D3DXVECTOR3, pPosition: *const D3DXVECTOR3, Color: D3DCOLOR) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXSprite_Draw(pSprite, pTexture, pSrcRect, pCenter, pPosition, Color)) }
}

#[allow(non_snake_case)]
pub fn ID3DXSprite_End(pSprite: *const c_void) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXSprite_End(pSprite)) }
}

#[allow(non_snake_case)]
pub fn ID3DXSprite_Flush(pSprite: *const c_void) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXSprite_Flush(pSprite)) }
}

#[allow(non_snake_case)]
pub fn ID3DXSprite_OnLostDevice(pSprite: *const c_void) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXSprite_OnLostDevice(pSprite)) }
}

#[allow(non_snake_case)]
pub fn ID3DXSprite_OnResetDevice(pSprite: *const c_void) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXSprite_OnResetDevice(pSprite)) }
}

#[allow(non_snake_case)]
pub fn ID3DXSprite_SetTransform(pSprite: *const c_void, pTransform: *const D3DXMATRIX) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXSprite_SetTransform(pSprite, pTransform)) }
}

#[allow(non_snake_case)]
pub fn D3DXCreateEffectFromFile(pDevice: IDirect3DDevice9, pSrcFile: PSTR, pDefines: *const c_void,
                                pInclude: *const c_void, flags: u32, pPool: *const c_void,
                                ppEffect: &mut LPD3DXEFFECT, ppCompilationErrors: &mut LPD3DXBUFFER) -> Result<()> {
    unsafe {
        to_result(D3DX_CreateEffectFromFile(pDevice, pSrcFile, pDefines, pInclude, flags, pPool, ppEffect, ppCompilationErrors))
    }
}

#[allow(non_snake_case)]
pub fn ID3DXEffect_Begin(pEffect: LPD3DXEFFECT, pPasses: *const u32, Flags: u32) -> Result<()> {
    unsafe {
        to_result(D3DX_ID3DXEffect_Begin(pEffect, pPasses, Flags))
    }
}

#[allow(non_snake_case)]
pub fn ID3DXEffect_BeginPass(pEffect: LPD3DXEFFECT, Pass: u32) -> Result<()> {
    unsafe {
        to_result(D3DX_ID3DXEffect_BeginPass(pEffect, Pass))
    }
}

#[allow(non_snake_case)]
pub fn ID3DXEffect_End(pEffect: LPD3DXEFFECT) -> Result<()> {
    unsafe {
        to_result(D3DX_ID3DXEffect_End(pEffect))
    }
}

#[allow(non_snake_case)]
pub fn ID3DXEffect_EndPass(pEffect: LPD3DXEFFECT) -> Result<()> {
    unsafe {
        to_result(D3DX_ID3DXEffect_EndPass(pEffect))
    }
}

#[allow(non_snake_case)]
pub fn ID3DXBaseEffect_GetTechniqueByName(pEffect: LPD3DXEFFECT, pName: PSTR) -> D3DXHANDLE {
    unsafe { D3DX_ID3DXBaseEffect_GetTechniqueByName(pEffect, pName) }
}

#[allow(non_snake_case)]
pub fn ID3DXBaseEffect_GetParameterByName(pEffect: LPD3DXEFFECT, hParameter: D3DXHANDLE, pName: PSTR) -> D3DXHANDLE {
    unsafe { D3DX_ID3DXBaseEffect_GetParameterByName(pEffect, hParameter, pName) }
}

#[allow(non_snake_case)]
pub fn ID3DXEffect_OnLostDevice(pEffect: *const c_void) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXEffect_OnLostDevice(pEffect)) }
}

#[allow(non_snake_case)]
pub fn ID3DXEffect_OnResetDevice(pEffect: *const c_void) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXEffect_OnResetDevice(pEffect)) }
}

#[allow(non_snake_case)]
pub fn ID3DXBaseEffect_SetMatrix(pEffect: *const c_void, hParameter: D3DXHANDLE, pMatrix: *const D3DXMATRIX) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXBaseEffect_SetMatrix(pEffect, hParameter, pMatrix)) }
}

#[allow(non_snake_case)]
pub fn ID3DXEffect_SetTechnique(pEffect: *const c_void, hTechnique: D3DXHANDLE) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXEffect_SetTechnique(pEffect, hTechnique)) }
}

#[allow(non_snake_case)]
pub fn ID3DXBuffer_GetBufferPointer(pBuffer: *const c_void) -> *mut c_void {
    unsafe { D3DX_ID3DXBuffer_GetBufferPointer(pBuffer) }
}

#[allow(non_snake_case)]
pub fn D3DXVec3Add(pOut: *mut D3DXVECTOR3, pV1: *const D3DXVECTOR3, pV2: *const D3DXVECTOR3) -> *mut D3DXVECTOR3 {
    unsafe { D3DX_Vec3Add(pOut, pV1, pV2) }
}

#[allow(non_snake_case)]
pub fn D3DXVec3Scale(pOut: *mut D3DXVECTOR3, pV: *const D3DXVECTOR3, s: f32) -> *mut D3DXVECTOR3 {
    unsafe { D3DX_Vec3Scale(pOut, pV, s) }
}

#[allow(non_snake_case)]
pub fn D3DXMatrixLookAtLH(pOut: *mut D3DXMATRIX, pEye: *const D3DXVECTOR3, pAt: *const D3DXVECTOR3,
                       pUp: *const D3DXVECTOR3) -> *const D3DXMATRIX {
    unsafe { D3DX_MatrixLookAtLH(pOut, pEye, pAt, pUp) }
}

#[allow(non_snake_case)]
pub fn D3DXMatrixPerspectiveFovLH(pOut: *mut D3DXMATRIX, fovy: f32, Aspect: f32,
                              zn: f32, zf: f32) -> *mut D3DXMATRIX {
    unsafe { D3DX_MatrixPerspectiveFovLH(pOut, fovy, Aspect, zn, zf) }
}

#[allow(non_snake_case)]
pub fn D3DXMatrixScaling(pOut: *mut D3DXMATRIX, sx: f32, sy: f32, sz: f32) -> *mut D3DXMATRIX {
    unsafe { D3DX_MatrixScaling(pOut, sx, sy, sz) }
}

#[allow(non_snake_case)]
pub fn D3DXMatrixTranslation(pOut: *mut D3DXMATRIX, x: f32, y: f32, z: f32) -> *mut D3DXMATRIX {
    unsafe { D3DX_MatrixTranslation(pOut, x, y, z) }
}

#[allow(non_snake_case)]
pub fn D3DXMatrixMultiply(pOut: *mut D3DXMATRIX, pM1: *const D3DXMATRIX, pM2: *const D3DXMATRIX) -> *mut D3DXMATRIX {
    unsafe { D3DX_MatrixMultiply(pOut, pM1, pM2) }
}

#[allow(non_snake_case)]
pub fn D3DXMatrixRotationZ(pOut: *mut D3DXMATRIX, Angle: f32) -> *mut D3DXMATRIX {
    unsafe { D3DX_MatrixRotationZ(pOut, Angle) }
}

#[allow(non_snake_case)]
pub fn D3DXMatrixIdentity(pOut: *mut D3DXMATRIX) -> *mut D3DXMATRIX {
    unsafe { D3DX_MatrixIdentity(pOut) }
}

#[allow(non_snake_case)]
pub fn D3DXVec3TransformCoord(pOut: *mut D3DXVECTOR3 , pV: *const D3DXVECTOR3,
                              pM: *const D3DXMATRIX) -> *mut D3DXVECTOR3 {
    unsafe { D3DX_Vec3TransformCoord(pOut, pV, pM) }
}