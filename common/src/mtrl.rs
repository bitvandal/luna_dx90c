use d3dx::D3DXCOLOR;

// Material
#[repr(C)]
pub struct Mtrl {
    pub ambient: D3DXCOLOR,
    pub diffuse: D3DXCOLOR,
    pub spec: D3DXCOLOR,
    pub spec_power: f32,
}