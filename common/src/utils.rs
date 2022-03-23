// Some utilities

use regex::Regex;

use windows::{
    Win32::Foundation::*, Win32::UI::WindowsAndMessaging::*,
};

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

// FIXME quick and dirty adapted way to calculate a random float based on a source rand int
pub fn get_random_float(base_rand_int: i32, a: f32, b: f32) -> f32 {
    if a >= b { // bad input
        return a
    }

    // Get random float in [0, 1] interval.
    let f: f32 = (base_rand_int % 10001) as f32 * 0.0001;

    (f * (b - a)) + a
}