use windows::core::*;

use crate::*;

pub struct VertexPos {
    pub pos: D3DXVECTOR3,
}

// temporary until I have to add additional vertex decl
pub fn init_all_vertex_declarations(d3d_device: IDirect3DDevice9) -> Result<IDirect3DVertexDeclaration9> {
    #[allow(non_snake_case)]
    let VertexPosElements: [D3DVERTEXELEMENT9; 2] = [
        D3DVERTEXELEMENT9 {
            Stream: 0,
            Offset: 0,
            Type: D3DDECLTYPE_FLOAT3.0 as u8,
            Method: D3DDECLMETHOD_DEFAULT.0 as u8,
            Usage: D3DDECLUSAGE_POSITION.0 as u8,
            UsageIndex: 0
        },
        D3DDECL_END!()
    ];

    unsafe { d3d_device.CreateVertexDeclaration(VertexPosElements.as_ptr()) }
}

pub fn destroy_all_vertex_declarations(_decl: &IDirect3DVertexDeclaration9) {
    // not really needed
    // drop(_decl);
}
