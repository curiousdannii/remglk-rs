/*

Helper functions
================

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use core::slice;
use std::ffi::CStr;
use std::ptr;
use std::sync::{Arc, Mutex};

use widestring::U32CStr;

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
        let obj = unsafe {Arc::from_raw(ptr)};
        GlkObject {
            obj,
        }
    }
}

pub fn to_owned<T>(obj: GlkObject<T>) -> *const Mutex<T> {
    Arc::into_raw(obj.obj)
}

pub fn to_owned_opt<T>(obj: Option<GlkObject<T>>) -> *const Mutex<T> {
    match obj {
        Some(obj) => Arc::into_raw(obj.obj),
        None => ptr::null(),
    }
}

pub fn write_ptr<T>(ptr: *mut T, val: T) {
    if ptr.is_null() {}
    else {
        unsafe {ptr::write(ptr, val);}
    }
}

// Buffer and C string helpers
pub fn glk_buffer<'a, T>(buf: *const T, buflen: u32) -> &'a [T]
where T: Clone {
    unsafe {slice::from_raw_parts(buf, buflen as usize)}
}

pub fn glk_buffer_mut<'a, T>(buf: *mut T, buflen: u32) -> &'a mut [T]
where T: Clone {
    unsafe {slice::from_raw_parts_mut(buf, buflen as usize)}
}

pub fn cstring_u8<'a>(buf: *const i8) -> &'a [u8] {
    unsafe {CStr::from_ptr(buf).to_bytes()}
}

pub fn cstring_u32<'a>(buf: *const u32) -> &'a [u32] {
    unsafe {U32CStr::from_ptr_str(buf).as_slice()}
}