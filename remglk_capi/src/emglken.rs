/*

Emglken support code
====================

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::ptr;

use crate::common::*;
use crate::glkapi::*;

#[no_mangle]
pub extern "C" fn emglken_getcwd(buf_ptr: *mut u8, len: usize) -> *const u8 {
    let glkapi = glkapi().lock().unwrap();
    let system_cwd = glkapi.dirs.system_cwd.to_str().unwrap();
    let system_cwd_len = system_cwd.len();
    if system_cwd_len + 1 > len {
        return ptr::null();
    }
    let buf = glk_buffer_mut(buf_ptr, len as u32);
    buf[..system_cwd_len].copy_from_slice(&system_cwd.as_bytes()[..system_cwd_len]);
    buf[system_cwd_len] = 0;
    buf_ptr
}