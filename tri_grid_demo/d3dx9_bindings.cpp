#include <stdio.h>
#include <stdbool.h>
#include <tchar.h>
#include <d3d9.h>
#include <d3dx9.h>

extern "C" void D3DX_Release(IUnknown *self) {
    if (self) {
        ULONG _refcount = self->Release();
    }
}

// D3DX Functions

extern "C" HRESULT D3DX_CreateFontIndirect(LPDIRECT3DDEVICE9 pDevice, const D3DXFONT_DESC *pDesc, LPD3DXFONT *ppFont) {
    return D3DXCreateFontIndirect(pDevice, pDesc, ppFont);
}

extern "C" HRESULT D3DX_CreateSprite(LPDIRECT3DDEVICE9 pDevice, LPD3DXSPRITE *ppSprite) {
    return D3DXCreateSprite(pDevice, ppSprite);
}

extern "C" HRESULT D3DX_CreateTextureFromFile(LPDIRECT3DDEVICE9 pDevice, LPCTSTR pSrcFile, LPDIRECT3DTEXTURE9 *ppTexture) {
    return D3DXCreateTextureFromFile(pDevice, pSrcFile, ppTexture);
}

extern "C" HRESULT D3DX_CreateEffectFromFile(LPDIRECT3DDEVICE9 pDevice, LPCTSTR pSrcFile, const D3DXMACRO *pDefines,
                                            LPD3DXINCLUDE pInclude, DWORD Flags, LPD3DXEFFECTPOOL pPool,
                                            LPD3DXEFFECT *ppEffect, LPD3DXBUFFER *ppCompilationErrors) {
    return D3DXCreateEffectFromFile(pDevice, pSrcFile, pDefines, pInclude, Flags, pPool,
                                            ppEffect, ppCompilationErrors);
}

// D3DX Interfaces

// ID3DXFont

extern "C" INT D3DX_ID3DXFont_DrawText(LPD3DXFONT self,
        LPD3DXSPRITE pSprite, LPCTSTR pString, INT Count,
        LPRECT pRect, DWORD Format, D3DCOLOR Color) {
    return self->DrawText(pSprite, pString, Count, pRect, Format, Color);
}

extern "C" HRESULT D3DX_ID3DXFont_OnLostDevice(LPD3DXFONT self) {
    return self->OnLostDevice();
}

extern "C" HRESULT D3DX_ID3DXFont_OnResetDevice(LPD3DXFONT self) {
    return self->OnResetDevice();
}

// ID3DXSprite

extern "C" HRESULT D3DX_ID3DXSprite_Begin(LPD3DXSPRITE self, DWORD flags) {
    return self->Begin(flags);
}

extern "C" HRESULT D3DX_ID3DXSprite_Draw(LPD3DXSPRITE self, LPDIRECT3DTEXTURE9 pTexture, const RECT *pSrcRect,
        const D3DXVECTOR3 *pCenter, const D3DXVECTOR3 *pPosition, D3DCOLOR Color) {
    return self->Draw(pTexture, pSrcRect, pCenter, pPosition, Color);
}

extern "C" HRESULT D3DX_ID3DXSprite_End(LPD3DXSPRITE self) {
    return self->End();
}

extern "C" HRESULT D3DX_ID3DXSprite_Flush(LPD3DXSPRITE self) {
    return self->Flush();
}

extern "C" HRESULT D3DX_ID3DXSprite_OnLostDevice(LPD3DXSPRITE self) {
    return self->OnLostDevice();
}

extern "C" HRESULT D3DX_ID3DXSprite_OnResetDevice(LPD3DXSPRITE self) {
    return self->OnResetDevice();
}

extern "C" HRESULT D3DX_ID3DXSprite_SetTransform(LPD3DXSPRITE self, const D3DXMATRIX *pTransform) {
    return self->SetTransform(pTransform);
}

// ID3DXEffect

extern "C" HRESULT D3DX_ID3DXEffect_Begin(LPD3DXEFFECT self, UINT *pPasses, DWORD Flags) {
    return self->Begin(pPasses, Flags);
}

extern "C" HRESULT D3DX_ID3DXEffect_BeginPass(LPD3DXEFFECT self, UINT Pass) {
    return self->BeginPass(Pass);
}

extern "C" HRESULT D3DX_ID3DXEffect_End(LPD3DXEFFECT self) {
    return self->End();
}

extern "C" HRESULT D3DX_ID3DXEffect_EndPass(LPD3DXEFFECT self) {
    return self->EndPass();
}

extern "C" D3DXHANDLE D3DX_ID3DXBaseEffect_GetTechniqueByName(LPD3DXBASEEFFECT self, LPCSTR pName) {
    return self->GetTechniqueByName(pName);
}

extern "C" D3DXHANDLE D3DX_ID3DXBaseEffect_GetParameterByName(LPD3DXBASEEFFECT self, D3DXHANDLE hParameter, LPCSTR pName) {
    return self->GetParameterByName(hParameter, pName);
}

extern "C" HRESULT D3DX_ID3DXEffect_OnLostDevice(LPD3DXEFFECT self) {
    return self->OnLostDevice();
}

extern "C" HRESULT D3DX_ID3DXEffect_OnResetDevice(LPD3DXEFFECT self) {
    return self->OnResetDevice();
}

extern "C" HRESULT D3DX_ID3DXBaseEffect_SetMatrix(LPD3DXBASEEFFECT self, D3DXHANDLE hParameter, const D3DXMATRIX *pMatrix) {
    return self->SetMatrix(hParameter, pMatrix);
}

extern "C" HRESULT D3DX_ID3DXEffect_SetTechnique(LPD3DXEFFECT self, D3DXHANDLE hTechnique) {
    return self->SetTechnique(hTechnique);
}

// ID3DXBuffer

extern "C" LPVOID D3DX_ID3DXBuffer_GetBufferPointer(LPD3DXBUFFER self) {
    return self->GetBufferPointer();
}

// MATH

extern "C" D3DXVECTOR3* D3DX_Vec3Scale(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV, FLOAT s) {
    return D3DXVec3Scale(pOut, pV, s);
}

extern "C" D3DXVECTOR3* D3DX_Vec3Add(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV1, const D3DXVECTOR3 *pV2) {
    return D3DXVec3Add(pOut, pV1, pV2);
}

extern "C" D3DXMATRIX* D3DX_MatrixLookAtLH(D3DXMATRIX *pOut, const D3DXVECTOR3 *pEye, const D3DXVECTOR3 *pAt, const D3DXVECTOR3 *pUp) {
    return D3DXMatrixLookAtLH(pOut, pEye, pAt, pUp);
}

extern "C" D3DXMATRIX* D3DX_MatrixPerspectiveFovLH(D3DXMATRIX *pOut, FLOAT fovy, FLOAT Aspect, FLOAT zn, FLOAT zf) {
    return D3DXMatrixPerspectiveFovLH(pOut, fovy, Aspect, zn, zf);
}

extern "C" D3DXMATRIX* D3DX_MatrixScaling(D3DXMATRIX *pOut, FLOAT sx, FLOAT sy, FLOAT sz) {
    return D3DXMatrixScaling(pOut, sx, sy, sz);
}

extern "C" D3DXMATRIX* D3DX_MatrixTranslation(D3DXMATRIX *pOut, FLOAT x, FLOAT y, FLOAT z) {
    return D3DXMatrixTranslation(pOut, x, y, z);
}

extern "C" D3DXMATRIX* D3DX_MatrixMultiply(D3DXMATRIX *pOut, const D3DXMATRIX *pM1, const D3DXMATRIX *pM2) {
    return D3DXMatrixMultiply(pOut, pM1, pM2);
}

extern "C" D3DXMATRIX* D3DX_MatrixRotationZ(D3DXMATRIX *pOut, FLOAT Angle) {
    return D3DXMatrixRotationZ(pOut, Angle);
}

extern "C" D3DXMATRIX* D3DX_MatrixIdentity(D3DXMATRIX *pOut) {
    return D3DXMatrixIdentity(pOut);
}

extern "C" D3DXVECTOR3* D3DX_Vec3TransformCoord(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV, const D3DXMATRIX *pM) {
    return D3DXVec3TransformCoord(pOut, pV, pM);
}