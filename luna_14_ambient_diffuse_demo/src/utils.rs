// Some utilities

use regex::Regex;

use windows::{
    Win32::Foundation::*, Win32::UI::WindowsAndMessaging::*,
};

use crate::*;

pub fn gen_tri_grid(num_vert_rows: i32, num_vert_cols: i32, dx: f32, dz: f32,
                    center: D3DXVECTOR3, verts: &mut Vec<D3DXVECTOR3>, indices: &mut Vec<u16>) {
    let num_vertices = num_vert_rows * num_vert_cols;
    let num_cell_rows = num_vert_rows - 1;
    let num_cell_cols = num_vert_cols - 1;

    let num_tris = num_cell_rows * num_cell_cols * 2;

    let width: f32 = num_cell_cols as f32 * dx;
    let depth: f32 = num_cell_rows as f32 * dz;

    //===========================================
    // Build vertices.

    // We first build the grid geometry centered about the origin and on
    // the xz-plane, row-by-row and in a top-down fashion.  We then translate
    // the grid vertices so that they are centered about the specified
    // parameter 'center'.

    verts.resize(num_vertices as usize, D3DXVECTOR3 { x: 0.0, y: 0.0, z: 0.0 });

    // Offsets to translate grid from quadrant 4 to center of
    // coordinate system.
    let x_offset: f32 = -width * 0.5;
    let z_offset: f32 =  depth * 0.5;

    let mut k = 0;
    for i in 0..num_vert_rows {
        for j in 0..num_vert_cols {
            // Negate the depth coordinate to put in quadrant four.
            // Then offset to center about coordinate system.
            verts[k].x = j as f32 * dx + x_offset;
            verts[k].z = -i as f32 * dz + z_offset;
            verts[k].y = 0.0;

            // Translate so that the center of the grid is at the
            // specified 'center' parameter.

            unsafe {
                let mut t: D3DXMATRIX = std::mem::zeroed();
                D3DXMatrixTranslation(&mut t, center.x, center.y, center.z);
                D3DXVec3TransformCoord(&mut verts[k], &verts[k], &t);
            }

            // Next vertex
            k += 1;
        }
    }

    //===========================================
    // Build indices.

    indices.resize((num_tris * 3) as usize, 0);

    // Generate indices for each quad.
    k = 0;
    for i in 0..num_cell_rows {
        for j in 0..num_cell_cols {
            indices[k]     = (i * num_vert_cols + j) as u16;
            indices[k + 1] = (i * num_vert_cols + j + 1) as u16;
            indices[k + 2] = ((i + 1) * num_vert_cols + j) as u16;

            indices[k + 3] = ((i + 1) * num_vert_cols + j) as u16;
            indices[k + 4] = (i * num_vert_cols + j + 1) as u16;
            indices[k + 5] = ((i + 1) * num_vert_cols + j + 1) as u16;

            // next quad
            k += 6;
        }
    }
}

pub fn message_box(err_msg: &str) {
    unsafe {
        let mut msg = String::from(err_msg);
        msg.push(0 as char);

        MessageBoxA(
            None,
            PSTR(msg.as_ptr() as _),
            PSTR(std::ptr::null_mut()),
            MB_OK);
    }
}

pub fn display_error_then_quit(err_msg: &str) {
    unsafe {
        message_box(err_msg);
        PostQuitMessage(0);
    }
}

#[macro_export]
macro_rules! HR {
    ($func_call:expr) => {
        {
            if let Err(err) = $func_call {
                if FAILED!(err.code()) {
                    let _hr = DXTrace(file!(), line!(), err.code(), stringify!($func_call), true);
                }
            }
        }
    };
}

// Cleans up function calls to pretty print in error traces
pub fn clean_func_call(s: &str) -> String {
    let tmp = Regex::new(r"\s+").unwrap().replace_all(s, " ");
    tmp.chars().filter(|c| *c != '\n' && *c != '\r').collect()
}