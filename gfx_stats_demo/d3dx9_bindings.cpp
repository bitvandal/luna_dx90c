#include <stdio.h>
#include <stdbool.h>
#include <tchar.h>
#include <d3d9.h>
#include <d3dx9.h>

extern "C" void D3DX_Release(IUnknown *self) {
    if (self) {
        ULONG refcount = self->Release();
    }
}

// ID3DXFont

extern "C" HRESULT D3DX_CreateFontIndirect(LPDIRECT3DDEVICE9 pDevice, const D3DXFONT_DESC *pDesc, LPD3DXFONT *ppFont) {
    return D3DXCreateFontIndirect(pDevice, pDesc, ppFont);
}

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