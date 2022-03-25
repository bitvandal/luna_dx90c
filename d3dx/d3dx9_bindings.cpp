#include <stdio.h>
#include <stdbool.h>
#include <tchar.h>
#include <d3d9.h>
#include <d3dx9.h>

extern "C" void D3DX_Release(IUnknown *self) {
    if (self) {
        /*ULONG *refcount = */self->Release();
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

extern "C" HRESULT D3DX_ID3DXBaseEffect_SetFloat(LPD3DXBASEEFFECT self, D3DXHANDLE hParameter, FLOAT f) {
    return self->SetFloat(hParameter, f);
}

extern "C" HRESULT D3DX_ID3DXBaseEffect_SetInt(LPD3DXBASEEFFECT self, D3DXHANDLE hParameter, INT n) {
    return self->SetInt(hParameter, n);
}

extern "C" HRESULT D3DX_ID3DXBaseEffect_SetTexture(LPD3DXBASEEFFECT self, D3DXHANDLE hParameter, LPDIRECT3DBASETEXTURE9 pTexture) {
    return self->SetTexture(hParameter, pTexture);
}

extern "C" HRESULT D3DX_ID3DXBaseEffect_SetValue(LPD3DXBASEEFFECT self, D3DXHANDLE hParameter, LPCVOID pData, UINT Bytes) {
    return self->SetValue(hParameter, pData, Bytes);
}

extern "C" HRESULT D3DX_ID3DXEffect_SetTechnique(LPD3DXEFFECT self, D3DXHANDLE hTechnique) {
    return self->SetTechnique(hTechnique);
}

extern "C" HRESULT D3DX_ID3DXEffect_CommitChanges(LPD3DXEFFECT self) {
    return self->CommitChanges();
}

// ID3DXBuffer

extern "C" LPVOID D3DX_ID3DXBuffer_GetBufferPointer(LPD3DXBUFFER self) {
    return self->GetBufferPointer();
}

// ID3DXMesh

extern "C" HRESULT D3DX_CreateCylinder(LPDIRECT3DDEVICE9 pDevice, FLOAT Radius1, FLOAT Radius2, FLOAT Length,
                                      UINT Slices, UINT Stacks, LPD3DXMESH *ppMesh, LPD3DXBUFFER *ppAdjacency) {
    return D3DXCreateCylinder(pDevice, Radius1, Radius2, Length, Slices, Stacks, ppMesh, ppAdjacency);
}

extern "C" HRESULT D3DX_CreateSphere(LPDIRECT3DDEVICE9 pDevice, FLOAT Radius, UINT Slices, UINT Stacks,
                                    LPD3DXMESH *ppMesh, LPD3DXBUFFER *ppAdjacency) {
    return D3DXCreateSphere(pDevice, Radius, Slices, Stacks, ppMesh, ppAdjacency);
}

extern "C" HRESULT D3DX_CreateTeapot(LPDIRECT3DDEVICE9 pDevice, LPD3DXMESH *ppMesh, LPD3DXBUFFER *ppAdjacency) {
    return D3DXCreateTeapot(pDevice, ppMesh, ppAdjacency);
}

extern "C" HRESULT D3DX_CreateBox(LPDIRECT3DDEVICE9 pDevice, FLOAT Width, FLOAT Height, FLOAT Depth,
                                  LPD3DXMESH *ppMesh, LPD3DXBUFFER *ppAdjacency) {
    return D3DXCreateBox(pDevice, Width, Height, Depth, ppMesh, ppAdjacency);
}

extern "C" DWORD D3DX_ID3DXBaseMesh_GetNumVertices(LPD3DXMESH self) {
    return self->GetNumVertices();
}

extern "C" DWORD D3DX_ID3DXBaseMesh_GetNumFaces(LPD3DXMESH self) {
    return self->GetNumFaces();
}

extern "C" DWORD D3DX_ID3DXBaseMesh_GetNumBytesPerVertex(LPD3DXMESH self) {
    return self->GetNumBytesPerVertex();
}

extern "C" HRESULT D3DX_ID3DXBaseMesh_DrawSubset(LPD3DXMESH self, DWORD AttribId) {
    return self->DrawSubset(AttribId);
}

extern "C" HRESULT D3DX_ID3DXBaseMesh_CloneMesh(LPD3DXMESH self, DWORD Options,
        const D3DVERTEXELEMENT9 *pDeclaration, LPDIRECT3DDEVICE9 pDevice, LPD3DXMESH *ppCloneMesh) {
    return self->CloneMesh(Options, pDeclaration, pDevice, ppCloneMesh);
}

extern "C" HRESULT D3DX_ID3DXBaseMesh_LockVertexBuffer(LPD3DXMESH self, DWORD Flags, LPVOID *ppData) {
    return self->LockVertexBuffer(Flags, ppData);
}

extern "C" HRESULT D3DX_ID3DXBaseMesh_UnlockVertexBuffer(LPD3DXMESH self) {
    return self->UnlockVertexBuffer();
}

extern "C" HRESULT D3DX_ID3DXBaseMesh_LockIndexBuffer(LPD3DXMESH self, DWORD Flags, LPVOID *ppData) {
    return self->LockIndexBuffer(Flags, ppData);
}

extern "C" HRESULT D3DX_ID3DXBaseMesh_UnlockIndexBuffer(LPD3DXMESH self) {
    return self->UnlockIndexBuffer();
}

extern "C" HRESULT D3DX_ID3DXBaseMesh_GetDeclaration(LPD3DXMESH self, D3DVERTEXELEMENT9 *Declaration) {
    return self->GetDeclaration(Declaration);
}

extern "C" HRESULT D3DX_ID3DXBaseMesh_GenerateAdjacency(LPD3DXMESH self, FLOAT Epsilon, DWORD *pAdjacency) {
    return self->GenerateAdjacency(Epsilon, pAdjacency);
}

extern "C" HRESULT D3DX_ID3DXMesh_Optimize(LPD3DXMESH self, DWORD Flags, const DWORD *pAdjacencyIn,
                                           DWORD *pAdjacencyOut, DWORD *pFaceRemap, LPD3DXBUFFER *ppVertexRemap,
                                           LPD3DXMESH *ppOptMesh) {
    return self->Optimize(Flags, pAdjacencyIn, pAdjacencyOut, pFaceRemap, ppVertexRemap, ppOptMesh);
}

extern "C" HRESULT D3DX_ID3DXMesh_OptimizeInPlace(LPD3DXMESH self, DWORD Flags, const DWORD *pAdjacencyIn,
                                                  DWORD *pAdjacencyOut, DWORD *pFaceRemap, LPD3DXBUFFER *ppVertexRemap) {
    return self->OptimizeInplace(Flags, pAdjacencyIn, pAdjacencyOut, pFaceRemap, ppVertexRemap);
}

extern "C" HRESULT D3DX_ID3DXMesh_LockAttributeBuffer(LPD3DXMESH self, DWORD Flags, DWORD **ppData) {
    return self->LockAttributeBuffer(Flags, ppData);
}

extern "C" HRESULT D3DX_ID3DXMesh_UnlockAttributeBuffer(LPD3DXMESH self) {
    return self->UnlockAttributeBuffer();
}

extern "C" HRESULT D3DX_LoadMeshFromX(LPCTSTR pFilename, DWORD Options, LPDIRECT3DDEVICE9 pD3DDevice,
        LPD3DXBUFFER *ppAdjacency, LPD3DXBUFFER *ppMaterials, LPD3DXBUFFER *ppEffectInstances,
        DWORD *pNumMaterials, LPD3DXMESH *ppMesh) {
    return D3DXLoadMeshFromX(pFilename, Options, pD3DDevice, ppAdjacency, ppMaterials,
        ppEffectInstances, pNumMaterials, ppMesh);
}

extern "C" HRESULT D3DX_ComputeBoundingBox(const D3DXVECTOR3 *pFirstPosition, DWORD NumVertices,
                                           DWORD dwStride, D3DXVECTOR3 *pMin, D3DXVECTOR3 *pMax) {
    return D3DXComputeBoundingBox(pFirstPosition, NumVertices, dwStride, pMin, pMax);
}

extern "C" HRESULT D3DX_ComputeNormals(LPD3DXBASEMESH pMesh, const DWORD *pAdjacency) {
    return D3DXComputeNormals(pMesh, pAdjacency);
}

extern "C" HRESULT D3DX_CreateMesh(DWORD NumFaces, DWORD NumVertices, DWORD Options,
                                   const D3DVERTEXELEMENT9 *pDeclaration, LPDIRECT3DDEVICE9 pD3DDevice,
                                   LPD3DXMESH *ppMesh) {
    return D3DXCreateMesh(NumFaces, NumVertices, Options, pDeclaration, pD3DDevice, ppMesh);
}

// MATH

extern "C" D3DXVECTOR3* D3DX_Vec3Scale(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV, FLOAT s) {
    return D3DXVec3Scale(pOut, pV, s);
}

extern "C" D3DXVECTOR3* D3DX_Vec3Add(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV1, const D3DXVECTOR3 *pV2) {
    return D3DXVec3Add(pOut, pV1, pV2);
}

extern "C" D3DXVECTOR3* D3DX_Vec3Subtract(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV1, const D3DXVECTOR3 *pV2) {
    return D3DXVec3Subtract(pOut, pV1, pV2);
}

extern "C" FLOAT D3DX_Vec3LengthSq(const D3DXVECTOR3 *pV) {
    return D3DXVec3LengthSq(pV);
}

extern "C" D3DXVECTOR4* D3DX_Vec4Add(D3DXVECTOR4 *pOut, const D3DXVECTOR4 *pV1, const D3DXVECTOR4 *pV2) {
    return D3DXVec4Add(pOut, pV1, pV2);
}

extern "C" D3DXVECTOR4* D3DX_Vec4Subtract(D3DXVECTOR4 *pOut, const D3DXVECTOR4 *pV1, const D3DXVECTOR4 *pV2) {
    return D3DXVec4Subtract(pOut, pV1, pV2);
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

extern "C" D3DXMATRIX* D3DX_MatrixRotationX(D3DXMATRIX *pOut, FLOAT Angle) {
    return D3DXMatrixRotationX(pOut, Angle);
}

extern "C" D3DXMATRIX* D3DX_MatrixRotationY(D3DXMATRIX *pOut, FLOAT Angle) {
    return D3DXMatrixRotationY(pOut, Angle);
}

extern "C" D3DXMATRIX* D3DX_MatrixRotationZ(D3DXMATRIX *pOut, FLOAT Angle) {
    return D3DXMatrixRotationZ(pOut, Angle);
}

extern "C" D3DXMATRIX* D3DX_MatrixRotationAxis(D3DXMATRIX *pOut, const D3DXVECTOR3 *pV, FLOAT Angle) {
    return D3DXMatrixRotationAxis(pOut, pV, Angle);
}

extern "C" D3DXMATRIX* D3DX_MatrixIdentity(D3DXMATRIX *pOut) {
    return D3DXMatrixIdentity(pOut);
}

extern "C" D3DXMATRIX* D3DX_MatrixInverse(D3DXMATRIX *pOut, FLOAT *pDeterminant, const D3DXMATRIX *pM) {
    return D3DXMatrixInverse(pOut, pDeterminant, pM);
}

extern "C" D3DXMATRIX* D3DX_MatrixTranspose(D3DXMATRIX *pOut, const D3DXMATRIX *pM) {
    return D3DXMatrixTranspose(pOut, pM);
}

extern "C" D3DXMATRIX* D3DX_MatrixReflect(D3DXMATRIX *pOut, const D3DXPLANE *pPlane) {
    return D3DXMatrixReflect(pOut, pPlane);
}

extern "C" D3DXMATRIX* D3DX_MatrixShadow(D3DXMATRIX *pOut, const D3DXVECTOR4 *pLight, const D3DXPLANE *pPlane) {
    return D3DXMatrixShadow(pOut, pLight, pPlane);
}

extern "C" FLOAT D3DX_PlaneDotCoord(const D3DXPLANE *pP, const D3DXVECTOR3 *pV) {
    return D3DXPlaneDotCoord(pP, pV);
}

extern "C" D3DXPLANE* D3DX_PlaneNormalize(D3DXPLANE *pOut, const D3DXPLANE *pP) {
    return D3DXPlaneNormalize(pOut, pP);
}

extern "C" D3DXVECTOR3* D3DX_Vec3TransformCoord(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV, const D3DXMATRIX *pM) {
    return D3DXVec3TransformCoord(pOut, pV, pM);
}

extern "C" D3DXVECTOR3* D3DX_Vec3TransformNormal(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV, const D3DXMATRIX *pM) {
    return D3DXVec3TransformNormal(pOut, pV, pM);
}

extern "C" D3DXVECTOR3* D3DX_Vec3Normalize(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV) {
    return D3DXVec3Normalize(pOut, pV);
}

extern "C" D3DXVECTOR3* D3DX_Vec3Maximize(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV1, const D3DXVECTOR3 *pV2) {
    return D3DXVec3Maximize(pOut, pV1, pV2);
}

extern "C" D3DXVECTOR3* D3DX_Vec3Minimize(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV1, const D3DXVECTOR3 *pV2) {
    return D3DXVec3Minimize(pOut, pV1, pV2);
}

extern "C" D3DXVECTOR3* D3DX_Vec3Cross(D3DXVECTOR3 *pOut, const D3DXVECTOR3 *pV1, const D3DXVECTOR3 *pV2) {
    return D3DXVec3Cross(pOut, pV1, pV2);
}

extern "C" FLOAT D3DX_Vec3Dot(const D3DXVECTOR3 *pV1, const D3DXVECTOR3 *pV2) {
    return D3DXVec3Dot(pV1, pV2);
}