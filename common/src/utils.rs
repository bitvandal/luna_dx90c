// Some utilities

use rand::Rng;
use rand::rngs::ThreadRng;
use regex::Regex;

use windows::{
    Win32::Foundation::*, Win32::UI::WindowsAndMessaging::*,
};
use d3dx::{D3DXVec3Normalize, D3DXVECTOR3};

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

// returns a zero-terminated as a C String
pub fn c_resource_path(base_path: &str, filename: &str) -> String {
    let mut filepath = String::from(base_path);
    filepath.push_str(filename);
    filepath.push_str("\0");
    filepath
}

#[allow(non_snake_case)]
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

pub fn get_random_float(rng: &mut ThreadRng, a: f32, b: f32) -> f32 {
    if a >= b { // bad input
        return a
    }

    // Get random float in [0, 1] interval.
    let rand_int = rng.gen_range(0..32767);
    let f: f32 = (rand_int % 10001) as f32 * 0.0001;
    (f * (b - a)) + a
}

pub fn get_random_vec(rng: &mut ThreadRng, out: &mut D3DXVECTOR3) {
    out.x = get_random_float(rng, -1.0, 1.0);
    out.y = get_random_float(rng, -1.0, 1.0);
    out.z = get_random_float(rng, -1.0, 1.0);

    // Project onto unit sphere.
    D3DXVec3Normalize(out, out);
}