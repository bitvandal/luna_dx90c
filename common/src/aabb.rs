use d3dx::*;

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

    pub fn extent(&self) -> D3DXVECTOR3 {
        let mut res: D3DXVECTOR3 = D3DXVECTOR3::default();
        D3DXVec3Subtract(&mut res, &self.max_pt, &self.min_pt);
        D3DXVec3Scale(&mut res, &res, 0.5);
        res
    }

    pub fn xform(&self, m: &D3DXMATRIX, out: &mut AABB) {
        // Convert to center/extent representation.
        let mut c: D3DXVECTOR3 = self.center();
        let mut e: D3DXVECTOR3 = self.extent();

        // Transform center in usual way.
        D3DXVec3TransformCoord(&mut c, &c, m);

        // Transform extent.
        let mut abs_m: D3DXMATRIX = D3DXMATRIX::default();
        D3DXMatrixIdentity(&mut abs_m);

        unsafe {
            abs_m.Anonymous.m[0] = m.Anonymous.m[0].abs();
            abs_m.Anonymous.m[1] = m.Anonymous.m[1].abs();
            abs_m.Anonymous.m[2] = m.Anonymous.m[2].abs();
            abs_m.Anonymous.m[3] = m.Anonymous.m[3].abs();
            abs_m.Anonymous.m[4] = m.Anonymous.m[4].abs();
            abs_m.Anonymous.m[5] = m.Anonymous.m[5].abs();
            abs_m.Anonymous.m[6] = m.Anonymous.m[6].abs();
            abs_m.Anonymous.m[7] = m.Anonymous.m[7].abs();
            abs_m.Anonymous.m[8] = m.Anonymous.m[8].abs();
        }

        D3DXVec3TransformNormal(&mut e, &e, &abs_m);

        // Convert back to AABB representation.
        D3DXVec3Subtract(&mut out.min_pt, &c, &e);
        D3DXVec3Add(&mut out.max_pt, &c, &e);
    }
}