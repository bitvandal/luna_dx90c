// Uninspired D3DX9 bindings, made for the sake of running some old book examples.
// Bindings use ANSI, not UNICODE

use libc::*;

use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct3D::*, Win32::Graphics::Direct3D9::*,
};

// D3DCOLOR is equivalent to D3DFMT_A8R8G8B8
type D3DCOLOR = u32;

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

// D3DX Material
#[allow(non_snake_case)]
#[repr(C)]
pub struct D3DXMATERIAL {
    pub MatD3D: D3DMATERIAL9,
    pub pTextureFilename: PSTR
}

// D3DX Core

pub type LPD3DXBUFFER = *mut c_void;

// D3DX Math

pub type D3DXVECTOR3 = D3DVECTOR;
pub type D3DXMATRIX = D3DMATRIX;

pub type LPD3DXVECTOR3 = *mut c_void;

#[derive(Clone)]
#[repr(C)]
pub struct D3DXVECTOR2 {
    pub x: f32,
    pub y: f32,
}

impl D3DXVECTOR2 {
    pub fn add_vec(&self, other: D3DXVECTOR2) -> D3DXVECTOR2 {
        D3DXVECTOR2 { x: self.x + other.x, y: self.y + other.y }
    }
}

#[repr(C)]
pub struct D3DXVECTOR4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

pub const D3DX_PI: f32 = std::f32::consts::PI;

#[repr(C)]
pub struct D3DXCOLOR {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl D3DXCOLOR {
    pub fn mult(&self, k: f32) -> D3DXCOLOR {
        D3DXCOLOR { r: k * self.r, g: k * self.g, b: k * self.b, a: self.a }
    }
}

// isomorphic types
impl From<D3DCOLORVALUE> for D3DXCOLOR {
    fn from(color: D3DCOLORVALUE) -> Self {
        D3DXCOLOR {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a
        }
    }
}

// Planes

#[repr(C)]
pub struct D3DXPLANE {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
}

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

// D3DX Mesh
pub type LPD3DXMESH = *mut c_void;
pub const MAX_FVF_DECL_SIZE: u32 = MAXD3DDECLLENGTH + 1;

// enum D3DXMESH
pub const D3DXMESH_32BIT: u32 = 0x001;
pub const D3DXMESH_SYSTEMMEM: u32 = 0x110;
pub const D3DXMESH_MANAGED: u32 = 0x220;
pub const D3DXMESH_WRITEONLY: u32 = 0x440;

// enum D3DXMESHOPT
pub const D3DXMESHOPT_COMPACT: u32 = 0x01000000;
pub const D3DXMESHOPT_ATTRSORT: u32 = 0x02000000;
pub const D3DXMESHOPT_VERTEXCACHE: u32 = 0x04000000;


// D3DX Functions

#[allow(non_snake_case)]
#[link(name = "../dependencies/d3dx9", kind = "static")]
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

    // HRESULT D3DXCreateEffectFromFile(LPDIRECT3DDEVICE9 pDevice, LPCTSTR pSrcFile, const D3DXMACRO *pDefines,
    //                                  LPD3DXINCLUDE pInclude, DWORD Flags, LPD3DXEFFECTPOOL pPool,
    //                                  LPD3DXEFFECT *ppEffect, LPD3DXBUFFER *ppCompilationErrors)
    fn D3DX_CreateEffectFromFile(pDevice: IDirect3DDevice9, pSrcFile: PSTR, pDefines: *const c_void,
                                 pInclude: *const c_void, flags: u32, pPool: *const c_void,
                                 ppEffect: *mut LPD3DXEFFECT, ppCompilationErrors: *mut LPD3DXBUFFER) -> D3DX_HRESULT;

    // HRESULT ID3DXEffect::Begin(LPD3DXEFFECT self, UINT *pPasses, DWORD Flags) {
    fn D3DX_ID3DXEffect_Begin(pEffect: LPD3DXEFFECT, pPasses: *const u32, Flags: u32) -> D3DX_HRESULT;

    // HRESULT ID3DXEffect::BeginPass(LPD3DXEFFECT self, UINT Pass) {
    fn D3DX_ID3DXEffect_BeginPass(pEffect: LPD3DXEFFECT, Pass: u32) -> D3DX_HRESULT;

    // HRESULT ID3DXEffect::End(LPD3DXEFFECT self) {
    fn D3DX_ID3DXEffect_End(pEffect: LPD3DXEFFECT) -> D3DX_HRESULT;

    // HRESULT ID3DXEffect::EndPass(LPD3DXEFFECT self) {
    fn D3DX_ID3DXEffect_EndPass(pEffect: LPD3DXEFFECT) -> D3DX_HRESULT;

    // D3DXHANDLE ID3DXBaseEffect::GetTechniqueByName(LPCSTR pName)
    fn D3DX_ID3DXBaseEffect_GetTechniqueByName(pEffect: LPD3DXEFFECT, pName: PSTR) -> D3DXHANDLE;

    // D3DXHANDLE ID3DXBaseEffect::GetParameterByName(D3DXHANDLE hParameter, LPCSTR pName)
    fn D3DX_ID3DXBaseEffect_GetParameterByName(pEffect: LPD3DXEFFECT, hParameter: D3DXHANDLE, pName: PSTR) -> D3DXHANDLE;

    // HRESULT ID3DXEffect::OnLostDevice()
    fn D3DX_ID3DXEffect_OnLostDevice(pEffect: *const c_void) -> D3DX_HRESULT;

    // HRESULT ID3DXEffect::OnResetDevice()
    fn D3DX_ID3DXEffect_OnResetDevice(pEffect: *const c_void) -> D3DX_HRESULT;

    // HRESULT ID3DXBaseEffect::SetMatrix(D3DXHANDLE hParameter, const D3DXMATRIX *pMatrix)
    fn D3DX_ID3DXBaseEffect_SetMatrix(pEffect: *const c_void, hParameter: D3DXHANDLE, pMatrix: *const D3DXMATRIX) -> D3DX_HRESULT;

    // HRESULT ID3DXBaseEffect::SetFloat(D3DXHANDLE hParameter, FLOAT f)
    fn D3DX_ID3DXBaseEffect_SetFloat(pEffect: *const c_void, hParameter: D3DXHANDLE, f: f32) -> D3DX_HRESULT;

    // HRESULT SetTexture(D3DXHANDLE hParameter, LPDIRECT3DBASETEXTURE9 pTexture);
    fn D3DX_ID3DXBaseEffect_SetTexture(pEffect: *const c_void, hParameter: D3DXHANDLE, pTexture: *const c_void) -> D3DX_HRESULT;

    // HRESULT SetValue(D3DXHANDLE hParameter, LPCVOID pData, UINT Bytes)
    fn D3DX_ID3DXBaseEffect_SetValue(pEffect: *const c_void, hParameter: D3DXHANDLE, pData: *const c_void, bytes: u32) -> D3DX_HRESULT;

    // HRESULT ID3DXEffect::SetTechnique(D3DXHANDLE hTechnique)
    fn D3DX_ID3DXEffect_SetTechnique(pEffect: *const c_void, h_technique: D3DXHANDLE) -> D3DX_HRESULT;

    // HRESULT ID3DXEffect::CommitChanges(LPD3DXEFFECT self)
    fn D3DX_ID3DXEffect_CommitChanges(pEffect: *const c_void) -> D3DX_HRESULT;

    // BUFFERS

    // LPVOID ID3DXBuffer::GetBufferPointer()
    fn D3DX_ID3DXBuffer_GetBufferPointer(pBuffer: *const c_void) -> *mut c_void;

    // MESHES

    // HRESULT D3DXCreateCylinder(LPDIRECT3DDEVICE9 pDevice, FLOAT Radius1, FLOAT Radius2, FLOAT Length,
    //                            UINT Slices, UINT Stacks, LPD3DXMESH *ppMesh, LPD3DXBUFFER *ppAdjacency)
    fn D3DX_CreateCylinder(pDevice: IDirect3DDevice9, Radius1: f32, Radius2: f32, Length: f32,
                           Slices: u32, Stacks: u32, ppMesh: *mut LPD3DXMESH,
                           ppAdjacency: *mut LPD3DXBUFFER) -> D3DX_HRESULT;

    // HRESULT D3DXCreateSphere(LPDIRECT3DDEVICE9 pDevice, FLOAT Radius, UINT Slices, UINT Stacks,
    //                          LPD3DXMESH *ppMesh, LPD3DXBUFFER *ppAdjacency)
    fn D3DX_CreateSphere(pDevice: IDirect3DDevice9, Radius: f32, Slices: u32, Stacks: u32,
                         ppMesh: *mut LPD3DXMESH, ppAdjacency: *mut LPD3DXBUFFER) -> D3DX_HRESULT;

    // HRESULT D3DXCreateTeapot(LPDIRECT3DDEVICE9 pDevice, LPD3DXMESH *ppMesh,
    //                          LPD3DXBUFFER *ppAdjacency);
    fn D3DX_CreateTeapot(pDevice: IDirect3DDevice9, ppMesh: *mut LPD3DXMESH,
                         ppAdjacency: *mut LPD3DXBUFFER) -> D3DX_HRESULT;

    // HRESULT D3DXCreateBox(LPDIRECT3DDEVICE9 pDevice, FLOAT Width, FLOAT Height, FLOAT Depth,
    //                       LPD3DXMESH *ppMesh, LPD3DXBUFFER *ppAdjacency);
    fn D3DX_CreateBox(pDevice: IDirect3DDevice9, Width: f32, Height: f32, Depth: f32,
                     ppMesh: *mut LPD3DXMESH, ppAdjacency: *mut LPD3DXBUFFER) -> D3DX_HRESULT;

    // DWORD ID3DXBaseMesh::GetNumVertices();
    fn D3DX_ID3DXBaseMesh_GetNumVertices(pMesh: *const c_void) -> u32;

    // DWORD ID3DXBaseMesh::GetNumFaces();
    fn D3DX_ID3DXBaseMesh_GetNumFaces(pMesh: *const c_void) -> u32;

    // HRESULT ID3DXBaseMesh::DrawSubset(LPD3DXMESH self, DWORD AttribId)
    fn D3DX_ID3DXBaseMesh_DrawSubset(pMesh: *const c_void, AttribId: u32) -> D3DX_HRESULT;

    // HRESULT CloneMesh(DWORD Options, const D3DVERTEXELEMENT9 *pDeclaration, LPDIRECT3DDEVICE9 pDevice,
    //                   LPD3DXMESH *ppCloneMesh)
    fn D3DX_ID3DXBaseMesh_CloneMesh(pMesh: *const c_void, Options: u32, pDeclaration: *const D3DVERTEXELEMENT9,
                                    pDevice: IDirect3DDevice9, ppCloneMesh: *mut LPD3DXMESH) -> D3DX_HRESULT;

    // HRESULT LockVertexBuffer(DWORD Flags, LPVOID *ppData)
    fn D3DX_ID3DXBaseMesh_LockVertexBuffer(pMesh: *const c_void, Flags: u32, ppData: &mut *mut c_void) -> D3DX_HRESULT;

    // HRESULT UnlockVertexBuffer()
    fn D3DX_ID3DXBaseMesh_UnlockVertexBuffer(pMesh: *const c_void) -> D3DX_HRESULT;

    // HRESULT LockIndexBuffer(DWORD Flags, LPVOID *ppData)
    fn D3DX_ID3DXBaseMesh_LockIndexBuffer(pMesh: *const c_void, Flags: u32, ppData: &mut *mut c_void) -> D3DX_HRESULT;

    // HRESULT UnlockIndexBuffer()
    fn D3DX_ID3DXBaseMesh_UnlockIndexBuffer(pMesh: *const c_void) -> D3DX_HRESULT;

    // HRESULT GetDeclaration(D3DVERTEXELEMENT9 Declaration);
    fn D3DX_ID3DXBaseMesh_GetDeclaration(pMesh: *const c_void, Declaration: *const D3DVERTEXELEMENT9) -> D3DX_HRESULT;

    // HRESULT GenerateAdjacency(FLOAT Epsilon, DWORD *pAdjacency);
    fn D3DX_ID3DXBaseMesh_GenerateAdjacency(pMesh: *const c_void, Epsilon: f32, pAdjacency: *mut u32) -> D3DX_HRESULT;

    // HRESULT Optimize(DWORD Flags, const DWORD *pAdjacencyIn, DWORD *pAdjacencyOut, DWORD *pFaceRemap,
    //                  LPD3DXBUFFER *ppVertexRemap, LPD3DXMESH *ppOptMesh);
    fn D3DX_ID3DXMesh_Optimize(pMesh: *const c_void, Flags: u32, pAdjacencyIn: *const u32,
                          pAdjacencyOut: *const u32, pFaceRemap: *const u32,
                          ppVertexRemap: *mut LPD3DXBUFFER, ppOptMesh: *mut LPD3DXMESH) -> D3DX_HRESULT;

    // HRESULT OptimizeInplace(DWORD Flags, const DWORD *pAdjacencyIn, DWORD *pAdjacencyOut,
    //                         DWORD *pFaceRemap, LPD3DXBUFFER *ppVertexRemap);
    fn D3DX_ID3DXMesh_OptimizeInPlace(pMesh: *const c_void, Flags: u32, pAdjacencyIn: *const u32,
                               pAdjacencyOut: *const u32, pFaceRemap: *const u32,
                               ppVertexRemap: *mut LPD3DXBUFFER) -> D3DX_HRESULT;

    // HRESULT LockAttributeBuffer(DWORD Flags, DWORD **ppData);
    fn D3DX_ID3DXMesh_LockAttributeBuffer(pMesh: *const c_void, Flags: u32, ppData: *mut *mut u32) -> D3DX_HRESULT;

    // HRESULT UnlockAttributeBuffer();
    fn D3DX_ID3DXMesh_UnlockAttributeBuffer(pMesh: *const c_void) -> D3DX_HRESULT;

    // HRESULT D3DXLoadMeshFromX(LPCTSTR pFilename, DWORD Options, LPDIRECT3DDEVICE9 pD3DDevice,
    //      LPD3DXBUFFER *ppAdjacency, LPD3DXBUFFER *ppMaterials, LPD3DXBUFFER *ppEffectInstances,
    //      DWORD *pNumMaterials, LPD3DXMESH *ppMesh)
    fn D3DX_LoadMeshFromX(pFilename: PSTR, Options: u32, pDevice: IDirect3DDevice9,
                          ppAdjacency: *mut LPD3DXBUFFER, ppMaterials: *mut LPD3DXBUFFER,
                          ppEffectInstances: *mut LPD3DXBUFFER, pNumMaterials: *mut u32,
                          ppMesh: *mut LPD3DXMESH) -> D3DX_HRESULT;

    // HRESULT D3DXComputeBoundingBox(const D3DXVECTOR3 *pFirstPosition, DWORD NumVertices,
    //                                DWORD dwStride, D3DXVECTOR3 *pMin, D3DXVECTOR3 *pMax);
    fn D3DX_ComputeBoundingBox(pFirstPosition: *const D3DXVECTOR3, NumVertices: u32,
                               dwStride: u32, pMin: *mut D3DXVECTOR3, pMax: *mut D3DXVECTOR3) -> D3DX_HRESULT;

    // HRESULT D3DXComputeNormals(LPD3DXBASEMESH pMesh, const DWORD *pAdjacency);
    fn D3DX_ComputeNormals(pMesh: LPD3DXMESH, pAdjacency: *const u32) -> D3DX_HRESULT;

    // HRESULT D3DXCreateMesh(DWORD NumFaces, DWORD NumVertices, DWORD Options,
    //                        const D3DVERTEXELEMENT9 *pDeclaration, LPDIRECT3DDEVICE9 pD3DDevice,
    //                        LPD3DXMESH *ppMesh);
    fn D3DX_CreateMesh(NumFaces: u32, NumVertices: u32, Options: u32, pDeclaration: *const D3DVERTEXELEMENT9,
                       pD3DDevice: IDirect3DDevice9, ppMesh: *mut LPD3DXMESH) -> D3DX_HRESULT;

    // MATH

    // D3DXVECTOR3* D3DXVec3Scale(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV, FLOAT s)
    fn D3DX_Vec3Scale(pOut: *mut D3DXVECTOR3, pV: *const D3DXVECTOR3, s: f32) -> *mut D3DXVECTOR3;

    // D3DXVECTOR3* D3DXVec3Add(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV1, const D3DXVECTOR3 *pV1)
    fn D3DX_Vec3Add(pOut: *mut D3DXVECTOR3, pV1: *const D3DXVECTOR3, pV2: *const D3DXVECTOR3) -> *mut D3DXVECTOR3;

    // D3DXVECTOR3* D3DXVec3Subtract(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV1, const D3DXVECTOR3 *pV1)
    fn D3DX_Vec3Subtract(pOut: *mut D3DXVECTOR3, pV1: *const D3DXVECTOR3, pV2: *const D3DXVECTOR3) -> *mut D3DXVECTOR3;

    // D3DXMATRIX* D3DXMatrixLookAtLH(D3DXMATRIX *pOut, const D3DXVECTOR3 *pEye, const D3DXVECTOR3 *pAt, const D3DXVECTOR3 *pUp)
    fn D3DX_MatrixLookAtLH(pOut: *mut D3DXMATRIX, pEye: *const D3DXVECTOR3, pAt: *const D3DXVECTOR3,
                           pUp: *const D3DXVECTOR3) -> *mut D3DXMATRIX;

    // D3DXMATRIX* D3DXMatrixPerspectiveFovLH(D3DXMATRIX *pOut, FLOAT fovy, FLOAT Aspect, FLOAT zn, FLOAT zf);
    fn D3DX_MatrixPerspectiveFovLH(pOut: *mut D3DXMATRIX, fovy: f32, Aspect: f32, zn: f32, zf: f32) -> *mut D3DXMATRIX;

    // D3DXMATRIX* D3DXMatrixScaling(D3DXMATRIX *pOut, FLOAT sx, FLOAT sy, FLOAT sz);
    fn D3DX_MatrixScaling(pOut: *mut D3DXMATRIX, sx: f32, sy: f32, sz: f32) -> *mut D3DXMATRIX;

    // D3DXMATRIX* D3DXMatrixTranslation (D3DXMATRIX *pOut, FLOAT x, FLOAT y, FLOAT z);
    fn D3DX_MatrixTranslation(pOut: *mut D3DXMATRIX, x: f32, y: f32, z: f32) -> *mut D3DXMATRIX;

    // D3DXMATRIX* D3DXMatrixMultiply(D3DXMATRIX *pOut, const D3DXMATRIX *pM1, const D3DXMATRIX *pM2)
    fn D3DX_MatrixMultiply(pOut: *mut D3DXMATRIX, pM1: *const D3DXMATRIX, pM2: *const D3DXMATRIX) -> *mut D3DXMATRIX;

    // D3DXMATRIX* D3DXMatrixRotationX(D3DXMATRIX *pOut, FLOAT Angle)
    fn D3DX_MatrixRotationX(pOut: *mut D3DXMATRIX, Angle: f32) -> *mut D3DXMATRIX;

    // D3DXMATRIX* D3DXMatrixRotationY(D3DXMATRIX *pOut, FLOAT Angle)
    fn D3DX_MatrixRotationY(pOut: *mut D3DXMATRIX, Angle: f32) -> *mut D3DXMATRIX;

    // D3DXMATRIX* D3DXMatrixRotationZ(D3DXMATRIX *pOut, FLOAT Angle)
    fn D3DX_MatrixRotationZ(pOut: *mut D3DXMATRIX, Angle: f32) -> *mut D3DXMATRIX;

    // D3DXMATRIX* D3DXMatrixRotationAxis(D3DXMATRIX *pOut, const D3DXVECTOR3 *pV, FLOAT Angle);
    fn D3DX_MatrixRotationAxis(pOut: *mut D3DXMATRIX, pV: *const D3DXVECTOR3, Angle: f32) -> *mut D3DXMATRIX;

    // D3DXMATRIX* D3DXMatrixIdentity(D3DXMATRIX *pOut)
    fn D3DX_MatrixIdentity(pOut: *mut D3DXMATRIX) -> *mut D3DXMATRIX;

    // D3DXMATRIX* D3DXMatrixInverse(D3DXMATRIX *pOut, FLOAT *pDeterminant, const D3DXMATRIX *pM);
    fn D3DX_MatrixInverse(pOut: *mut D3DXMATRIX, pDeterminant: f32, pM: *const D3DXMATRIX) -> *mut D3DXMATRIX;

    // D3DXMATRIX* D3DXMatrixTranspose(D3DXMATRIX *pOut, const D3DXMATRIX *pM);
    fn D3DX_MatrixTranspose(pOut: *mut D3DXMATRIX, pM: *const D3DXMATRIX) -> *mut D3DXMATRIX;

    // D3DXMATRIX* D3DXMatrixReflect(D3DXMATRIX *pOut, const D3DXPLANE *pPlane);
    fn D3DX_MatrixReflect(pOut: *mut D3DXMATRIX, pPlane: *const D3DXPLANE) -> *mut D3DXMATRIX;

    // D3DXMATRIX* D3DXMatrixShadow(D3DXMATRIX *pOut, const D3DXVECTOR4 *pLight, const D3DXPLANE *pPlane);
    fn D3DX_MatrixShadow(pOut: *mut D3DXMATRIX, pLight: *const D3DXVECTOR4, pPlane: *const D3DXPLANE) -> *mut D3DXMATRIX;

    // D3DXVECTOR3* D3DXVec3TransformCoord(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV, const D3DXMATRIX *pM)
    fn D3DX_Vec3TransformCoord(pOut: *mut D3DXVECTOR3, pV: *const D3DXVECTOR3, pM: *const D3DXMATRIX) -> *mut D3DXVECTOR3;

    // D3DXVECTOR3* D3DXVec3TransformNormal(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV, const D3DXMATRIX *pM)
    fn D3DX_Vec3TransformNormal(pOut: *mut D3DXVECTOR3, pV: *const D3DXVECTOR3, pM: *const D3DXMATRIX) -> *mut D3DXVECTOR3;

    // D3DXVECTOR3* D3DXVec3Normalize(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV)
    fn D3DX_Vec3Normalize(pOut: *mut D3DXVECTOR3, pV: *const D3DXVECTOR3) -> *mut D3DXVECTOR3;

    // D3DXVECTOR3* D3DXVec3Maximize(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV1, const D3DXVECTOR3 *pV2)
    fn D3DX_Vec3Maximize(pOut: *mut D3DXVECTOR3, pV1: *const D3DXVECTOR3, pV2: *const D3DXVECTOR3) -> *mut D3DXVECTOR3;

    // D3DXVECTOR3* D3DXVec3Maximize(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV1, const D3DXVECTOR3 *pV2)
    fn D3DX_Vec3Minimize(pOut: *mut D3DXVECTOR3, pV1: *const D3DXVECTOR3, pV2: *const D3DXVECTOR3) -> *mut D3DXVECTOR3;

    // D3DXVECTOR3* D3DXVec3Cross(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV1, const D3DXVECTOR3 *pV2)
    fn D3DX_Vec3Cross(pOut: *mut D3DXVECTOR3, pV1: *const D3DXVECTOR3, pV2: *const D3DXVECTOR3) -> *mut D3DXVECTOR3;

    // FLOAT D3DXVec3Dot(const D3DXVECTOR3 *pV1, const D3DXVECTOR3 *pV2)
    fn D3DX_Vec3Dot(pV1: *const D3DXVECTOR3, pV2: *const D3DXVECTOR3) -> f32;
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
pub fn ID3DXBaseEffect_SetFloat(pEffect: *const c_void, hParameter: D3DXHANDLE, f: f32) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXBaseEffect_SetFloat(pEffect, hParameter, f)) }
}

#[allow(non_snake_case)]
pub fn ID3DXBaseEffect_SetTexture(pEffect: *const c_void, hParameter: D3DXHANDLE, pTexture: *const c_void) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXBaseEffect_SetTexture(pEffect, hParameter, pTexture)) }
}

#[allow(non_snake_case)]
pub fn ID3DXBaseEffect_SetValue(pEffect: *const c_void, hParameter: D3DXHANDLE, pData: *const c_void, bytes: u32) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXBaseEffect_SetValue(pEffect, hParameter, pData, bytes)) }
}

#[allow(non_snake_case)]
pub fn ID3DXEffect_SetTechnique(pEffect: *const c_void, hTechnique: D3DXHANDLE) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXEffect_SetTechnique(pEffect, hTechnique)) }
}

#[allow(non_snake_case)]
pub fn ID3DXEffect_CommitChanges(pEffect: *const c_void) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXEffect_CommitChanges(pEffect)) }
}

#[allow(non_snake_case)]
pub fn ID3DXBuffer_GetBufferPointer(pBuffer: *const c_void) -> *mut c_void {
    unsafe { D3DX_ID3DXBuffer_GetBufferPointer(pBuffer) }
}

#[allow(non_snake_case)]
pub fn D3DXCreateCylinder(pDevice: IDirect3DDevice9, Radius1: f32, Radius2: f32, Length: f32,
                          Slices: u32, Stacks: u32, ppMesh: *mut LPD3DXMESH,
                          ppAdjacency: *mut LPD3DXBUFFER) -> Result<()> {
    unsafe { to_result(D3DX_CreateCylinder(pDevice, Radius1, Radius2, Length, Slices,
                                           Stacks, ppMesh, ppAdjacency)) }
}

#[allow(non_snake_case)]
pub fn D3DXCreateSphere(pDevice: IDirect3DDevice9, Radius: f32, Slices: u32, Stacks: u32,
                        ppMesh: *mut LPD3DXMESH, ppAdjacency: *mut LPD3DXBUFFER) -> Result<()> {
    unsafe { to_result(D3DX_CreateSphere(pDevice, Radius, Slices, Stacks, ppMesh, ppAdjacency)) }
}

#[allow(non_snake_case)]
pub fn D3DXCreateTeapot(pDevice: IDirect3DDevice9, ppMesh: *mut LPD3DXMESH,
                        ppAdjacency: *mut LPD3DXBUFFER) -> Result<()> {
    unsafe { to_result(D3DX_CreateTeapot(pDevice, ppMesh, ppAdjacency)) }
}

#[allow(non_snake_case)]
pub fn D3DXCreateBox(pDevice: IDirect3DDevice9, Width: f32, Height: f32, Depth: f32,
                      ppMesh: *mut LPD3DXMESH, ppAdjacency: *mut LPD3DXBUFFER) -> Result<()> {
    unsafe { to_result(D3DX_CreateBox(pDevice, Width, Height, Depth, ppMesh, ppAdjacency)) }
}

#[allow(non_snake_case)]
pub fn ID3DXBaseMesh_GetNumVertices(pMesh: *const c_void) -> u32 {
    unsafe { D3DX_ID3DXBaseMesh_GetNumVertices(pMesh) }
}

#[allow(non_snake_case)]
pub fn ID3DXBaseMesh_GetNumFaces(pMesh: *const c_void) -> u32 {
    unsafe { D3DX_ID3DXBaseMesh_GetNumFaces(pMesh) }
}

#[allow(non_snake_case)]
pub fn ID3DXBaseMesh_DrawSubset(pMesh: *const c_void, AttribId: u32) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXBaseMesh_DrawSubset(pMesh, AttribId)) }
}

#[allow(non_snake_case)]
pub fn D3DXVec3Add(pOut: *mut D3DXVECTOR3, pV1: *const D3DXVECTOR3, pV2: *const D3DXVECTOR3) -> *mut D3DXVECTOR3 {
    unsafe { D3DX_Vec3Add(pOut, pV1, pV2) }
}

#[allow(non_snake_case)]
pub fn ID3DXBaseMesh_CloneMesh(pMesh: *const c_void, Options: u32, pDeclaration: *const D3DVERTEXELEMENT9,
                               pDevice: IDirect3DDevice9, ppCloneMesh: *mut LPD3DXMESH) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXBaseMesh_CloneMesh(pMesh, Options, pDeclaration, pDevice, ppCloneMesh)) }
}

#[allow(non_snake_case)]
pub fn ID3DXBaseMesh_LockVertexBuffer(pMesh: *const c_void, Flags: u32, ppData: &mut *mut c_void) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXBaseMesh_LockVertexBuffer(pMesh, Flags, ppData)) }
}

#[allow(non_snake_case)]
pub fn ID3DXBaseMesh_UnlockVertexBuffer(pMesh: *const c_void) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXBaseMesh_UnlockVertexBuffer(pMesh)) }
}

#[allow(non_snake_case)]
pub fn ID3DXBaseMesh_LockIndexBuffer(pMesh: *const c_void, Flags: u32, ppData: &mut *mut c_void) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXBaseMesh_LockIndexBuffer(pMesh, Flags, ppData)) }
}

#[allow(non_snake_case)]
pub fn ID3DXBaseMesh_UnlockIndexBuffer(pMesh: *const c_void) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXBaseMesh_UnlockIndexBuffer(pMesh)) }
}

#[allow(non_snake_case)]
pub fn ID3DXBaseMesh_GetDeclaration(pMesh: *const c_void, Declaration: *const D3DVERTEXELEMENT9) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXBaseMesh_GetDeclaration(pMesh, Declaration)) }
}

#[allow(non_snake_case)]
pub fn ID3DXBaseMesh_GenerateAdjacency(pMesh: *const c_void, Epsilon: f32, pAdjacency: *mut u32) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXBaseMesh_GenerateAdjacency(pMesh, Epsilon, pAdjacency)) }
}

#[allow(non_snake_case)]
pub fn ID3DXMesh_Optimize(pMesh: *const c_void, Flags: u32, pAdjacencyIn: *const u32,
                          pAdjacencyOut: *const u32, pFaceRemap: *const u32,
                          ppVertexRemap: *mut LPD3DXBUFFER, ppOptMesh: *mut LPD3DXMESH) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXMesh_Optimize(pMesh, Flags, pAdjacencyIn, pAdjacencyOut,
                                                    pFaceRemap, ppVertexRemap, ppOptMesh)) }
}

#[allow(non_snake_case)]
pub fn ID3DXMesh_OptimizeInPlace(pMesh: *const c_void, Flags: u32, pAdjacencyIn: *const u32,
                          pAdjacencyOut: *const u32, pFaceRemap: *const u32,
                          ppVertexRemap: *mut LPD3DXBUFFER) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXMesh_OptimizeInPlace(pMesh, Flags, pAdjacencyIn, pAdjacencyOut,
                                                           pFaceRemap, ppVertexRemap)) }
}

#[allow(non_snake_case)]
pub fn ID3DXMesh_LockAttributeBuffer(pMesh: *const c_void, Flags: u32, ppData: *mut *mut u32) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXMesh_LockAttributeBuffer(pMesh, Flags, ppData)) }
}

#[allow(non_snake_case)]
pub fn ID3DXMesh_UnlockAttributeBuffer(pMesh: *const c_void) -> Result<()> {
    unsafe { to_result(D3DX_ID3DXMesh_UnlockAttributeBuffer(pMesh)) }
}

#[allow(non_snake_case)]
pub fn D3DXLoadMeshFromX(pFilename: PSTR, Options: u32, pDevice: IDirect3DDevice9,
                      ppAdjacency: *mut LPD3DXBUFFER, ppMaterials: *mut LPD3DXBUFFER,
                      ppEffectInstances: *mut LPD3DXBUFFER, pNumMaterials: *mut u32,
                      ppMesh: *mut LPD3DXMESH) -> Result<()> {
    unsafe { to_result(D3DX_LoadMeshFromX(pFilename, Options, pDevice, ppAdjacency,
                                          ppMaterials, ppEffectInstances, pNumMaterials,
                                          ppMesh)) }
}

#[allow(non_snake_case)]
pub fn D3DXComputeBoundingBox(pFirstPosition: *const D3DXVECTOR3, NumVertices: u32,
                           dwStride: u32, pMin: *mut D3DXVECTOR3, pMax: *mut D3DXVECTOR3) -> Result<()> {
    unsafe { to_result(D3DX_ComputeBoundingBox(pFirstPosition, NumVertices, dwStride, pMin, pMax)) }
}

#[allow(non_snake_case)]
pub fn D3DXComputeNormals(pMesh: LPD3DXMESH, pAdjacency: *const u32) -> Result<()> {
    unsafe { to_result(D3DX_ComputeNormals(pMesh, pAdjacency)) }
}

#[allow(non_snake_case)]
pub fn D3DXCreateMesh(NumFaces: u32, NumVertices: u32, Options: u32, pDeclaration: *const D3DVERTEXELEMENT9,
                      pD3DDevice: IDirect3DDevice9, ppMesh: *mut LPD3DXMESH) -> Result<()> {
    unsafe { to_result(D3DX_CreateMesh(NumFaces, NumVertices, Options, pDeclaration, pD3DDevice, ppMesh)) }
}

#[allow(non_snake_case)]
pub fn D3DXVec3Subtract(pOut: *mut D3DXVECTOR3, pV1: *const D3DXVECTOR3, pV2: *const D3DXVECTOR3) -> *mut D3DXVECTOR3 {
    unsafe { D3DX_Vec3Subtract(pOut, pV1, pV2) }
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
pub fn D3DXMatrixRotationX(pOut: *mut D3DXMATRIX, Angle: f32) -> *mut D3DXMATRIX {
    unsafe { D3DX_MatrixRotationX(pOut, Angle) }
}

#[allow(non_snake_case)]
pub fn D3DXMatrixRotationY(pOut: *mut D3DXMATRIX, Angle: f32) -> *mut D3DXMATRIX {
    unsafe { D3DX_MatrixRotationY(pOut, Angle) }
}

#[allow(non_snake_case)]
pub fn D3DXMatrixRotationZ(pOut: *mut D3DXMATRIX, Angle: f32) -> *mut D3DXMATRIX {
    unsafe { D3DX_MatrixRotationZ(pOut, Angle) }
}

#[allow(non_snake_case)]
pub fn D3DXMatrixRotationAxis(pOut: *mut D3DXMATRIX, pV: *const D3DXVECTOR3, Angle: f32) -> *mut D3DXMATRIX {
    unsafe { D3DX_MatrixRotationAxis(pOut, pV, Angle) }
}

#[allow(non_snake_case)]
pub fn D3DXMatrixIdentity(pOut: *mut D3DXMATRIX) -> *mut D3DXMATRIX {
    unsafe { D3DX_MatrixIdentity(pOut) }
}

#[allow(non_snake_case)]
pub fn D3DXMatrixInverse(pOut: *mut D3DXMATRIX, pDeterminant: f32, pM: *const D3DXMATRIX) -> *mut D3DXMATRIX {
    unsafe { D3DX_MatrixInverse(pOut, pDeterminant, pM) }
}

#[allow(non_snake_case)]
pub fn D3DXMatrixTranspose(pOut: *mut D3DXMATRIX, pM: *const D3DXMATRIX) -> *mut D3DXMATRIX {
    unsafe { D3DX_MatrixTranspose(pOut, pM) }
}

#[allow(non_snake_case)]
pub fn D3DXMatrixReflect(pOut: *mut D3DXMATRIX, pPlane: *const D3DXPLANE) -> *mut D3DXMATRIX {
    unsafe { D3DX_MatrixReflect(pOut, pPlane) }
}

#[allow(non_snake_case)]
pub fn D3DXMatrixShadow(pOut: *mut D3DXMATRIX, pLight: *const D3DXVECTOR4, pPlane: *const D3DXPLANE) -> *mut D3DXMATRIX {
    unsafe { D3DX_MatrixShadow(pOut, pLight, pPlane) }
}

#[allow(non_snake_case)]
pub fn D3DXVec3TransformCoord(pOut: *mut D3DXVECTOR3, pV: *const D3DXVECTOR3,
                              pM: *const D3DXMATRIX) -> *mut D3DXVECTOR3 {
    unsafe { D3DX_Vec3TransformCoord(pOut, pV, pM) }
}

#[allow(non_snake_case)]
pub fn D3DXVec3TransformNormal(pOut: *mut D3DXVECTOR3, pV: *const D3DXVECTOR3,
                              pM: *const D3DXMATRIX) -> *mut D3DXVECTOR3 {
    unsafe { D3DX_Vec3TransformNormal(pOut, pV, pM) }
}

#[allow(non_snake_case)]
pub fn D3DXVec3Normalize(pOut: *mut D3DXVECTOR3 , pV: *const D3DXVECTOR3) -> *mut D3DXVECTOR3 {
    unsafe { D3DX_Vec3Normalize(pOut, pV) }
}

#[allow(non_snake_case)]
pub fn D3DXVec3Maximize(pOut: *mut D3DXVECTOR3, pV1: *const D3DXVECTOR3, pV2: *const D3DXVECTOR3) -> *mut D3DXVECTOR3 {
    unsafe { D3DX_Vec3Maximize(pOut, pV1, pV2) }
}

#[allow(non_snake_case)]
pub fn D3DXVec3Minimize(pOut: *mut D3DXVECTOR3, pV1: *const D3DXVECTOR3, pV2: *const D3DXVECTOR3) -> *mut D3DXVECTOR3 {
    unsafe { D3DX_Vec3Minimize(pOut, pV1, pV2) }
}

#[allow(non_snake_case)]
pub fn D3DXVec3Cross(pOut: *mut D3DXVECTOR3, pV1: *const D3DXVECTOR3, pV2: *const D3DXVECTOR3) -> *mut D3DXVECTOR3 {
    unsafe { D3DX_Vec3Cross(pOut, pV1, pV2) }
}

#[allow(non_snake_case)]
pub fn D3DXVec3Dot(pV1: *const D3DXVECTOR3, pV2: *const D3DXVECTOR3) -> f32 {
    unsafe { D3DX_Vec3Dot(pV1, pV2) }
}