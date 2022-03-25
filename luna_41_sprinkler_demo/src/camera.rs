use windows::Win32::Devices::HumanInterfaceDevice::DIK_W;
use d3dx::*;
use crate::*;
use crate::terrain::Terrain;

#[derive(Copy, Clone)]
pub struct Camera {
    // Save camera related matrices.
    view: D3DXMATRIX,
    proj: D3DXMATRIX,
    view_proj: D3DXMATRIX,

    // Relative to world space.
    pos_w: D3DXVECTOR3,
    right_w: D3DXVECTOR3,
    up_w: D3DXVECTOR3,
    look_w: D3DXVECTOR3,

    // Camera speed.
    speed: f32,

    // Frustum Planes
    // [0] = near
    // [1] = far
    // [2] = left
    // [3] = right
    // [4] = top
    // [5] = bottom
    frustum_planes: [D3DXPLANE; 6]
}

impl Camera {
    pub fn new() -> Camera {
        unsafe {
            let mut view: D3DXMATRIX = std::mem::zeroed();
            D3DXMatrixIdentity(&mut view);

            let mut proj: D3DXMATRIX = std::mem::zeroed();
            D3DXMatrixIdentity(&mut proj);

            let mut view_proj: D3DXMATRIX = std::mem::zeroed();
            D3DXMatrixIdentity(&mut view_proj);

            let pos_w = D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 };
            let right_w = D3DXVECTOR3 { x: 1.0, y: 0.0, z: 0.0 };
            let up_w = D3DXVECTOR3 { x: 0.0, y: 1.0, z: 0.0 };
            let look_w = D3DXVECTOR3 { x: 0.0, y: 0.0, z: 1.0 };

            // Client should adjust to a value that makes sense for application's
            // unit scale, and the object the camera is attached to--e.g., car, jet,
            // human walking, etc.
            let speed = 50.0;

            Camera {
                view,
                proj,
                view_proj,
                pos_w,
                right_w,
                up_w,
                look_w,
                speed,
                frustum_planes: [D3DXPLANE::default(); 6],
            }
        }
    }

    pub fn set_pos(&mut self, pos: D3DXVECTOR3) {
        self.pos_w = pos;
    }

    pub fn set_speed(&mut self, s: f32) {
        self.speed = s;
    }

    pub fn get_pos(&self) -> D3DXVECTOR3 {
        self.pos_w.clone()
    }

    pub fn get_view_proj(&self) -> &D3DXMATRIX {
        &self.view_proj
    }

    pub fn look_at(&mut self, pos: &D3DXVECTOR3, target: &D3DXVECTOR3, up: &D3DXVECTOR3) {
        let mut l: D3DXVECTOR3 = unsafe { std::mem::zeroed() };
        D3DXVec3Subtract(&mut l, target, pos);
        D3DXVec3Normalize(&mut l, &l);

        let mut r: D3DXVECTOR3 = unsafe { std::mem::zeroed() };
        D3DXVec3Cross(&mut r, up, &l);
        D3DXVec3Normalize(&mut r, &r);

        let mut u: D3DXVECTOR3 = unsafe { std::mem::zeroed() };
        D3DXVec3Cross(&mut u, &l, &r);
        D3DXVec3Normalize(&mut u, &u);

        self.pos_w = pos.clone();
        self.right_w = r;
        self.up_w = u;
        self.look_w = l;

        self.build_view();
        self.build_world_frustum_planes();

        self.view_proj = unsafe { std::mem::zeroed() };
        D3DXMatrixMultiply(&mut self.view_proj, &self.view, &self.proj);
    }

    pub fn set_lens(&mut self, fov: f32, aspect: f32, near_z: f32, far_z: f32) {
        D3DXMatrixPerspectiveFovLH(&mut self.proj, fov, aspect, near_z, far_z);

        self.build_world_frustum_planes();

        self.view_proj = unsafe { std::mem::zeroed() };
        D3DXMatrixMultiply(&mut self.view_proj, &self.view, &self.proj);
    }

    // Box coordinates should be relative to world space.
    pub fn is_visible(&self, bounding_box: &AABB) -> bool {
        // Test assumes frustum planes face inward.

        let mut p = D3DXVECTOR3::default();
        let mut q = D3DXVECTOR3::default();

        //      N  *Q                    *P
        //      | /                     /
        //      |/                     /
        // -----/----- Plane     -----/----- Plane
        //     /                     / |
        //    /                     /  |
        //   *P                    *Q  N
        //
        // PQ forms diagonal most closely aligned with plane normal.

        // For each frustum plane, find the box diagonal (there are four main
        // diagonals that intersect the box center point) that points in the
        // same direction as the normal along each axis (i.e., the diagonal
        // that is most aligned with the plane normal).  Then test if the box
        // is in front of the plane or not.
        for i in 0..6 {
            // For each coordinate axis x, y, z...
            // Make PQ point in the same direction as the plane normal on this axis.
            if self.frustum_planes[i].a >= 0.0 {
                p.x = (&bounding_box.min_pt).x;
                q.x = (&bounding_box.max_pt).x;
            } else  {
                p.x = (&bounding_box.max_pt).x;
                q.x = (&bounding_box.min_pt).x;
            }

            if self.frustum_planes[i].b >= 0.0 {
                p.y = (&bounding_box.min_pt).y;
                q.y = (&bounding_box.max_pt).y;
            } else {
                p.y = (&bounding_box.max_pt).y;
                q.y = (&bounding_box.min_pt).y;
            }

            if self.frustum_planes[i].c >= 0.0 {
                p.z = (&bounding_box.min_pt).z;
                q.z = (&bounding_box.max_pt).z;
            } else {
                p.z = (&bounding_box.max_pt).z;
                q.z = (&bounding_box.min_pt).z;
            }

            // If box is in negative half space, it is behind the plane, and thus, completely
            // outside the frustum.  Note that because PQ points roughly in the direction of the
            // plane normal, we can deduce that if Q is outside then P is also outside--thus we
            // only need to test Q.
            if D3DXPlaneDotCoord(&self.frustum_planes[i], &q) < 0.0 {
                return false   // outside
            }
        }

        true
    }

    fn build_world_frustum_planes(&mut self) {
        unsafe {
            // Note: Extract the frustum planes in world space.

            let mut vp: D3DXMATRIX = std::mem::zeroed();
            D3DXMatrixMultiply(&mut vp, &self.view, &self.proj);

            let col0 = D3DXVECTOR4 { x: vp.Anonymous.m[0], y: vp.Anonymous.m[4], z: vp.Anonymous.m[8],  w: vp.Anonymous.m[12] };
            let col1 = D3DXVECTOR4 { x: vp.Anonymous.m[1], y: vp.Anonymous.m[5], z: vp.Anonymous.m[9],  w: vp.Anonymous.m[13] };
            let col2 = D3DXVECTOR4 { x: vp.Anonymous.m[2], y: vp.Anonymous.m[6], z: vp.Anonymous.m[10], w: vp.Anonymous.m[14] };
            let col3 = D3DXVECTOR4 { x: vp.Anonymous.m[3], y: vp.Anonymous.m[7], z: vp.Anonymous.m[11], w: vp.Anonymous.m[15] };

            // Planes face inward.
            self.frustum_planes[0] = Camera::plane_from_vec4(&col2);                 // near
            self.frustum_planes[1] = Camera::plane_from_vec4(&col3.sub(&col2)); // far
            self.frustum_planes[2] = Camera::plane_from_vec4(&col3.add(&col0)); // left
            self.frustum_planes[3] = Camera::plane_from_vec4(&col3.sub(&col0)); // right
            self.frustum_planes[4] = Camera::plane_from_vec4(&col3.sub(&col1)); // top
            self.frustum_planes[5] = Camera::plane_from_vec4(&col3.add(&col1)); // bottom

            for i in 0..6 as usize {
                D3DXPlaneNormalize(&mut self.frustum_planes[i], &self.frustum_planes[i]);
            }
        }
    }

    pub fn update(&mut self, dt: f32, terrain: Option<&Terrain>, offset_height: f32) {
        unsafe {
            if let Some(dinput) = &mut DIRECT_INPUT {
                // Find the net direction the camera is traveling in (since the
                // camera could be running and strafing).
                let mut dir = D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 };

                if dinput.key_down(DIK_W as usize) {
                    D3DXVec3Add(&mut dir, &dir, &self.look_w);
                }

                if dinput.key_down(DIK_S as usize) {
                    D3DXVec3Subtract(&mut dir, &dir, &self.look_w);
                }

                if dinput.key_down(DIK_D as usize) {
                    D3DXVec3Add(&mut dir, &dir, &self.right_w);
                }

                if dinput.key_down(DIK_A as usize) {
                    D3DXVec3Subtract(&mut dir, &dir, &self.right_w);
                }

                // Move at mSpeed along net direction.
                D3DXVec3Normalize(&mut dir, &dir);

                let mut new_pos: D3DXVECTOR3 = std::mem::zeroed();
                D3DXVec3Scale(&mut new_pos, &dir, self.speed * dt);
                D3DXVec3Add(&mut new_pos, &self.pos_w, &new_pos);

                if let Some(terrain) = terrain {
                    // New position might not be on terrain, so project the
                    // point onto the terrain.
                    new_pos.y = terrain.get_height(new_pos.x, new_pos.z) + offset_height;

                    // Now the difference of the new position and old (current)
                    // position approximates a tangent vector on the terrain.
                    let mut tangent: D3DXVECTOR3 = std::mem::zeroed();
                    D3DXVec3Subtract(&mut tangent, &new_pos, &self.pos_w);
                    D3DXVec3Normalize(&mut tangent, &tangent);

                    // Now move camera along tangent vector.
                    let mut res: D3DXVECTOR3 = std::mem::zeroed();
                    D3DXVec3Scale(&mut res, &tangent, self.speed * dt);
                    D3DXVec3Add(&mut self.pos_w, &self.pos_w, &res);

                    // After update, there may be errors in the camera height since our
                    // tangent is only an approximation.  So force camera to correct height,
                    // and offset by the specified amount so that camera does not sit
                    // exactly on terrain, but instead, slightly above it.
                    self.pos_w.y = terrain.get_height(self.pos_w.x, self.pos_w.z) + offset_height;
                } else {
                    self.pos_w = new_pos;
                }

                // We rotate at a fixed speed.
                let pitch: f32 = dinput.mouse_dy() / 150.0;
                let y_angle: f32 = dinput.mouse_dx() / 150.0;

                // Rotate camera's look and up vectors around the camera's right vector.
                let mut r: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixRotationAxis(&mut r, &self.right_w, pitch);
                D3DXVec3TransformCoord(&mut self.look_w, &self.look_w, &r);
                D3DXVec3TransformCoord(&mut self.up_w, &self.up_w, &r);

                // Rotate camera axes about the world's y-axis.
                D3DXMatrixRotationY(&mut r, y_angle);
                D3DXVec3TransformCoord(&mut self.right_w, &self.right_w, &r);
                D3DXVec3TransformCoord(&mut self.up_w, &self.up_w, &r);
                D3DXVec3TransformCoord(&mut self.look_w, &self.look_w, &r);

                // Rebuild the view matrix to reflect changes.
                self.build_view();
                self.build_world_frustum_planes();

                self.view_proj = std::mem::zeroed();
                D3DXMatrixMultiply(&mut self.view_proj, &self.view, &self.proj);
            }
        }
    }

    fn build_view(&mut self) {
        // Keep camera's axes orthogonal to each other and of unit length.
        D3DXVec3Normalize(&mut self.look_w, &self.look_w);

        D3DXVec3Cross(&mut self.up_w, &self.look_w, &self.right_w);
        D3DXVec3Normalize(&mut self.up_w, &self.up_w);

        D3DXVec3Cross(&mut self.right_w, &self.up_w, &self.look_w);
        D3DXVec3Normalize(&mut self.right_w, &self.right_w);

        // Fill in the view matrix entries.

        let x: f32 = -D3DXVec3Dot(&self.pos_w, &self.right_w);
        let y: f32 = -D3DXVec3Dot(&self.pos_w, &self.up_w);
        let z: f32 = -D3DXVec3Dot(&self.pos_w, &self.look_w);

        unsafe {
            self.view.Anonymous.m[0] = self.right_w.x;
            self.view.Anonymous.m[4] = self.right_w.y;
            self.view.Anonymous.m[8] = self.right_w.z;
            self.view.Anonymous.m[12] = x;

            self.view.Anonymous.m[1] = self.up_w.x;
            self.view.Anonymous.m[5] = self.up_w.y;
            self.view.Anonymous.m[9] = self.up_w.z;
            self.view.Anonymous.m[13] = y;

            self.view.Anonymous.m[2] = self.look_w.x;
            self.view.Anonymous.m[6] = self.look_w.y;
            self.view.Anonymous.m[10] = self.look_w.z;
            self.view.Anonymous.m[14] = z;

            self.view.Anonymous.m[3] = 0.0;
            self.view.Anonymous.m[7] = 0.0;
            self.view.Anonymous.m[11] = 0.0;
            self.view.Anonymous.m[15] = 1.0;
        }
    }

    fn plane_from_vec4(vec: &D3DXVECTOR4) -> D3DXPLANE {
        D3DXPLANE { a: vec.x, b: vec.y, c: vec.z, d: vec.w }
    }
}