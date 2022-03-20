use d3dx::D3DXVECTOR3;

// Bounding Volumes
#[derive(Clone)]
#[repr(C)]
pub struct AABB {
    pub min_pt: D3DXVECTOR3,
    pub max_pt: D3DXVECTOR3,
}

impl Default for AABB {
    fn default() -> Self {
        AABB {
            min_pt: D3DXVECTOR3 {
                x: f32::MAX,
                y: f32::MAX,
                z: f32::MAX
            },
            max_pt: D3DXVECTOR3 {
                x: f32::MIN,
                y: f32::MIN,
                z: f32::MIN
            }
        }
    }
}

impl AABB {
    pub fn center(&self) -> D3DXVECTOR3 {
        D3DXVECTOR3 {
            x: 0.5 * (self.min_pt.x + self.max_pt.x),
            y: 0.5 * (self.min_pt.y + self.max_pt.y),
            z: 0.5 * (self.min_pt.z + self.max_pt.z)
        }
    }
}