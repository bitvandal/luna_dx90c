use windows::Win32::Graphics::Direct3D9::*;
use d3dx::*;
use crate::*;

pub struct VertexPos {
    pub pos: D3DXVECTOR3,
}

pub struct VertexCol {
    pub pos: D3DXVECTOR3,
    pub col: D3DCOLOR,
}

pub struct VertexPN {
    pub pos: D3DXVECTOR3,
    pub normal: D3DXVECTOR3,
}

pub struct VertexPNT {
    pub pos: D3DXVECTOR3,
    pub normal: D3DXVECTOR3,
    pub tex0: D3DXVECTOR2,
}

pub struct VertexPT {
    pub pos: D3DXVECTOR3,
    pub tex0: D3DXVECTOR2,
}

// Vertex declarations

pub static mut VERTEX_POS_DECL: Option<IDirect3DVertexDeclaration9> = None;
pub static mut VERTEX_COL_DECL: Option<IDirect3DVertexDeclaration9> = None;
pub static mut VERTEX_PN_DECL: Option<IDirect3DVertexDeclaration9> = None;
pub static mut VERTEX_PNT_DECL: Option<IDirect3DVertexDeclaration9> = None;
pub static mut VERTEX_PT_DECL: Option<IDirect3DVertexDeclaration9> = None;

pub fn init_all_vertex_declarations(d3d_device: IDirect3DDevice9) {
    unsafe {
        let vertex_pos_elements: [D3DVERTEXELEMENT9; 2] = [
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

        VERTEX_POS_DECL = Some(d3d_device.CreateVertexDeclaration(vertex_pos_elements.as_ptr()).unwrap());

        let vertex_col_elements: [D3DVERTEXELEMENT9; 3] = [
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 0,
                Type: D3DDECLTYPE_FLOAT3.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_POSITION.0 as u8,
                UsageIndex: 0
            },
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 12,
                Type: D3DDECLTYPE_D3DCOLOR.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_COLOR.0 as u8,
                UsageIndex: 0
            },
            D3DDECL_END!()
        ];

        VERTEX_COL_DECL = Some(d3d_device.CreateVertexDeclaration(vertex_col_elements.as_ptr()).unwrap());

        let vertex_pn_elements: [D3DVERTEXELEMENT9; 3] = [
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 0,
                Type: D3DDECLTYPE_FLOAT3.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_POSITION.0 as u8,
                UsageIndex: 0
            },
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 12,
                Type: D3DDECLTYPE_FLOAT3.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_NORMAL.0 as u8,
                UsageIndex: 0
            },
            D3DDECL_END!()
        ];

        VERTEX_PN_DECL = Some(d3d_device.CreateVertexDeclaration(vertex_pn_elements.as_ptr()).unwrap());

        let vertex_pnt_elements: [D3DVERTEXELEMENT9; 4] = [
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 0,
                Type: D3DDECLTYPE_FLOAT3.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_POSITION.0 as u8,
                UsageIndex: 0
            },
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 12,
                Type: D3DDECLTYPE_FLOAT3.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_NORMAL.0 as u8,
                UsageIndex: 0
            },
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 24,
                Type: D3DDECLTYPE_FLOAT2.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_TEXCOORD.0 as u8,
                UsageIndex: 0
            },
            D3DDECL_END!()
        ];

        VERTEX_PNT_DECL = Some(d3d_device.CreateVertexDeclaration(vertex_pnt_elements.as_ptr()).unwrap());


        let vertex_pt_elements: [D3DVERTEXELEMENT9; 3] = [
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 0,
                Type: D3DDECLTYPE_FLOAT3.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_POSITION.0 as u8,
                UsageIndex: 0
            },
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 12,
                Type: D3DDECLTYPE_FLOAT2.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_TEXCOORD.0 as u8,
                UsageIndex: 0
            },
            D3DDECL_END!()
        ];

        VERTEX_PT_DECL = Some(d3d_device.CreateVertexDeclaration(vertex_pt_elements.as_ptr()).unwrap());
    }
}

pub fn destroy_all_vertex_declarations() {
    // not really needed
    // drop(vertex_pos_decl);
    // drop(vertex_col_decl);
    // drop(vertex_pn_decl);
    // drop(vertex_pnt_decl);
    // drop(vertex_pt_decl);
}
