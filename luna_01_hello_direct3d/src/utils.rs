use windows::{
    Win32::Foundation::*, Win32::UI::WindowsAndMessaging::*,
};

use regex::Regex;

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