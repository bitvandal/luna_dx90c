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

#[derive(Clone)]
pub struct VertexPNT {
    pub pos: D3DXVECTOR3,
    pub normal: D3DXVECTOR3,
    pub tex0: D3DXVECTOR2,
}

pub struct VertexPT {
    pub pos: D3DXVECTOR3,
    pub tex0: D3DXVECTOR2,
}

pub struct VertexGrass {
    pub pos: D3DXVECTOR3,
    pub quad_pos: D3DXVECTOR3,
    pub tex0: D3DXVECTOR2,
    pub amplitude: f32, // for wind oscillation.
    pub color_offset: D3DCOLOR,
}

#[derive(Default, Clone, Debug)]
pub struct Particle {
    pub initial_pos: D3DXVECTOR3,
    pub initial_velocity: D3DXVECTOR3,
    pub initial_size: f32, // In pixels.
    pub initial_time: f32,
    pub life_time: f32,
    pub mass: f32,
    pub initial_color: D3DCOLOR,
}

pub struct NMapVertex {
    pub pos: D3DXVECTOR3,
    pub tangent: D3DXVECTOR3,
    pub binormal: D3DXVECTOR3,
    pub normal: D3DXVECTOR3,
    pub tex0: D3DXVECTOR2,
}

pub struct WaterDMapVertex {
    pub pos: D3DXVECTOR3,
    pub scaled_tex_c: D3DXVECTOR2,      // [a, b]
    pub normalized_tex_c: D3DXVECTOR2,  // [0, 1]
}

// Vertex declarations

pub static mut VERTEX_POS_DECL: Option<IDirect3DVertexDeclaration9> = None;
pub static mut VERTEX_COL_DECL: Option<IDirect3DVertexDeclaration9> = None;
pub static mut VERTEX_PN_DECL: Option<IDirect3DVertexDeclaration9> = None;
pub static mut VERTEX_PNT_DECL: Option<IDirect3DVertexDeclaration9> = None;
pub static mut VERTEX_PT_DECL: Option<IDirect3DVertexDeclaration9> = None;
pub static mut VERTEX_GRASS: Option<IDirect3DVertexDeclaration9> = None;
pub static mut PARTICLE_DECL: Option<IDirect3DVertexDeclaration9> = None;
pub static mut NMAP_VERTEX_DECL: Option<IDirect3DVertexDeclaration9> = None;
pub static mut WATER_DMAP_VERTEX_DECL: Option<IDirect3DVertexDeclaration9> = None;

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

        let vertex_grass_elements: [D3DVERTEXELEMENT9; 6] = [
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
                Usage: D3DDECLUSAGE_TEXCOORD.0 as u8,
                UsageIndex: 0
            },
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 24,
                Type: D3DDECLTYPE_FLOAT2.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_TEXCOORD.0 as u8,
                UsageIndex: 1
            },
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 32,
                Type: D3DDECLTYPE_FLOAT1.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_TEXCOORD.0 as u8,
                UsageIndex: 2
            },
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 36,
                Type: D3DDECLTYPE_D3DCOLOR.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_COLOR.0 as u8,
                UsageIndex: 0
            },
            D3DDECL_END!()
        ];

        VERTEX_GRASS = Some(d3d_device.CreateVertexDeclaration(vertex_grass_elements.as_ptr()).unwrap());

        let particle_elements: [D3DVERTEXELEMENT9; 8] = [
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
                Usage: D3DDECLUSAGE_TEXCOORD.0 as u8,
                UsageIndex: 0
            },
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 24,
                Type: D3DDECLTYPE_FLOAT1.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_TEXCOORD.0 as u8,
                UsageIndex: 1
            },
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 28,
                Type: D3DDECLTYPE_FLOAT1.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_TEXCOORD.0 as u8,
                UsageIndex: 2
            },
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 32,
                Type: D3DDECLTYPE_FLOAT1.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_TEXCOORD.0 as u8,
                UsageIndex: 3
            },
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 36,
                Type: D3DDECLTYPE_FLOAT1.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_TEXCOORD.0 as u8,
                UsageIndex: 4
            },
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 40,
                Type: D3DDECLTYPE_D3DCOLOR.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_COLOR.0 as u8,
                UsageIndex: 0
            },
            D3DDECL_END!()
        ];

        PARTICLE_DECL = Some(d3d_device.CreateVertexDeclaration(particle_elements.as_ptr()).unwrap());

        let nmap_vertex_elements: [D3DVERTEXELEMENT9; 6] = [
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
                Usage: D3DDECLUSAGE_TANGENT.0 as u8,
                UsageIndex: 0
            },
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 24,
                Type: D3DDECLTYPE_FLOAT3.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_BINORMAL.0 as u8,
                UsageIndex: 0
            },
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 36,
                Type: D3DDECLTYPE_FLOAT3.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_NORMAL.0 as u8,
                UsageIndex: 0
            },
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 48,
                Type: D3DDECLTYPE_FLOAT2.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_TEXCOORD.0 as u8,
                UsageIndex: 0
            },
            D3DDECL_END!()
        ];

        NMAP_VERTEX_DECL = Some(d3d_device.CreateVertexDeclaration(nmap_vertex_elements.as_ptr()).unwrap());

        let water_dmap_vertex_elements: [D3DVERTEXELEMENT9; 4] = [
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
            D3DVERTEXELEMENT9 {
                Stream: 0,
                Offset: 20,
                Type: D3DDECLTYPE_FLOAT2.0 as u8,
                Method: D3DDECLMETHOD_DEFAULT.0 as u8,
                Usage: D3DDECLUSAGE_TEXCOORD.0 as u8,
                UsageIndex: 1
            },
            D3DDECL_END!()
        ];

        WATER_DMAP_VERTEX_DECL = Some(d3d_device.CreateVertexDeclaration(water_dmap_vertex_elements.as_ptr()).unwrap());

    }
}

pub fn destroy_all_vertex_declarations() {
    // not really needed
    // drop(vertex_pos_decl);
    // drop(vertex_col_decl);
    // drop(vertex_pn_decl);
    // drop(vertex_pnt_decl);
    // drop(vertex_pt_decl);
    // drop(vertex_grass_decl);
    // drop(particle_decl);
    // drop(nmap_vertex_decl);
    // drop(water_dmap_vertex_decl);
}
