/*

Helper functions
================

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use core::slice;

pub fn glk_buffer<'a, T>(buf: *mut T, buflen: u32) -> &'a [T]
where T: Clone {
    unsafe {slice::from_raw_parts(buf, buflen as usize)}
}

pub fn glk_buffer_mut<'a, T>(buf: *mut T, buflen: u32) -> &'a mut [T]
where T: Clone {
    unsafe {slice::from_raw_parts_mut(buf, buflen as usize)}
}