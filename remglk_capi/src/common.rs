/*

Helper functions
================

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use core::slice;

pub fn glk_buffer_to_vec<T>(buf: *mut T, buflen: u32) -> Vec<T>
where T: Clone
{
    let buf: &[T] = unsafe {slice::from_raw_parts(buf, buflen as usize)};
    let mut vec: Vec<T> = Vec::new();
    vec.extend_from_slice(buf);
    vec
}