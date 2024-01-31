/*

Helper functions
================

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use core::slice;
use std::ptr;
use std::sync::{Arc, Mutex};

use remglk::glkapi::GlkObject;

// Functions for sharing our Glk objects across the FFI barrier
pub fn borrow<T>(obj: Option<&GlkObject<T>>) -> *const Mutex<T> {
    if let Some(obj) = obj {
        Arc::as_ptr(obj)
    }
    else {
        ptr::null()
    }
}

pub fn from_ptr<T>(ptr: *const Mutex<T>) -> GlkObject<T> {
    unsafe {Arc::increment_strong_count(ptr);}
    reclaim(ptr)
}

pub fn from_ptr_opt<T>(ptr: *const Mutex<T>) -> Option<GlkObject<T>> {
    if ptr.is_null() {
        None
    }
    else {
        Some(from_ptr(ptr))
    }
}

pub fn reclaim<T>(ptr: *const Mutex<T>) -> GlkObject<T> {
    if ptr.is_null() {
        panic!("Invalid (null) reference!")
    }
    else {
        unsafe{
            let obj = Arc::from_raw(ptr);
            GlkObject {
                obj,
            }
        }
    }
}

pub fn to_owned<T>(obj: GlkObject<T>) -> *const Mutex<T> {
    Arc::into_raw(obj.obj)
}

// Buffer helpers
pub fn glk_buffer<'a, T>(buf: *mut T, buflen: u32) -> &'a [T]
where T: Clone {
    unsafe {slice::from_raw_parts(buf, buflen as usize)}
}

pub fn glk_buffer_mut<'a, T>(buf: *mut T, buflen: u32) -> &'a mut [T]
where T: Clone {
    unsafe {slice::from_raw_parts_mut(buf, buflen as usize)}
}