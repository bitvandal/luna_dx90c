// Rewrite of D3D9 functions and macros not provided by windows crate.

use windows::core::*;
use windows::Win32::Graphics::Direct3D9::*;

use libc::c_char;
use crate::*;

// from winerror.h

#[macro_export]
macro_rules! FAILED {
    ($hr:expr) => {
        $hr.is_err()
    }
}

macro_rules! MAKE_HRESULT {
    ($sev:expr, $fac:expr, $code:expr) => {
        HRESULT(($sev as u32) << 31 | ($fac as u32) << 16 | ($code as u32))
    }
}

// from d3d9.h

macro_rules! _FACD3D {
    ($) => { 0x876 }
}

macro_rules! MAKE_D3DHRESULT {
    ($code:expr) => {
        MAKE_HRESULT!(1, _FACD3D, $code)
    }
}

pub const D3DERR_DRIVERINTERNALERROR: HRESULT = MAKE_D3DHRESULT!(2087);
pub const D3DERR_DEVICELOST: HRESULT = MAKE_D3DHRESULT!(2152);
pub const D3DERR_DEVICENOTRESET: HRESULT = MAKE_D3DHRESULT!(2153);

// D3DCOLOR is equivalent to D3DFMT_A8R8G8B8
pub type D3DCOLOR = u32;

// maps unsigned 8 bits/channel to D3DCOLOR

#[macro_export]
macro_rules! D3DCOLOR_ARGB {
    ($a:expr, $r:expr, $g:expr, $b:expr) => {
        (((($a & 0xff) << 24) | (($r & 0xff) << 16) | (($g & 0xff) << 8) | ($b & 0xff)) as D3DCOLOR)
    }
}

#[macro_export]
macro_rules! D3DCOLOR_RGBA {
    ($r:expr, $g:expr, $b:expr, $a:expr) => {
        (((($a & 0xff) << 24) | (($r &0xff) << 16) | (($g & 0xff) << 8) | ($b & 0xff)) as D3DCOLOR)
    }
}

#[macro_export]
macro_rules! D3DCOLOR_XRGB {
    ($r:expr, $g:expr, $b:expr) => {
        D3DCOLOR_ARGB!(0xff, $r, $g, $b)
    }
}

// from d3d9types.h
pub const D3DTS_WORLD: D3DTRANSFORMSTATETYPE = D3DTRANSFORMSTATETYPE(256i32);

#[macro_export]
macro_rules! D3DDECL_END {
    () => {
        D3DVERTEXELEMENT9 {
            Stream: 0xFF,
            Offset: 0,
            Type: D3DDECLTYPE_UNUSED.0 as u8,
            Method: 0,
            Usage: 0,
            UsageIndex: 0,
        }
    }
}

// from DxErr

#[allow(non_snake_case)]
pub fn DXTrace(file: &str, line: u32, hr: HRESULT, str_msg: &str, pop_msg_box: bool) -> HRESULT {
    let msg = format!("File: {}\nLine: {}\nError Code: {}\nCalling: {}",
                      file, line, DXGetErrorString(hr), clean_func_call(str_msg));

    if pop_msg_box {
        message_box(&String::from(msg));
    } else {
        println!("[DXTrace]\n{}", msg);
    }
    hr
}

#[link(name = "legacy_stdio_definitions", kind = "static")]
#[link(name = "DxErr", kind = "static")]
extern {
    pub fn DXGetErrorStringA(hr: u32) -> *const c_char;
}

#[allow(non_snake_case)]
pub fn DXGetErrorString(hr: HRESULT) -> String {
    unsafe {
        let c_str_ptr = DXGetErrorStringA(hr.0);
        let c_str: &CStr = CStr::from_ptr(c_str_ptr);
        let str_slice: &str = c_str.to_str().unwrap_or("<UNKNOWN>");
        return str_slice.to_owned();
    }
}