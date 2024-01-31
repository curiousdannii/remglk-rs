/*

The Glk API
===========

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::sync::{Mutex, OnceLock};

use remglk::glkapi;
use glkapi::*;
use glkapi::constants::*;

use crate::common::*;

type StreamPtr = *const Mutex<Stream>;
type WindowPtr = *const Mutex<Window>;

fn glkapi() -> &'static Mutex<GlkApi> {
    static GLKAPI: OnceLock<Mutex<GlkApi>> = OnceLock::new();
    GLKAPI.get_or_init(|| Mutex::new(GlkApi::default()))
}

// TODO: error handling!

#[no_mangle]
pub extern "C" fn glk_get_buffer_stream(str: StreamPtr, buf: *mut u8, len: u32) -> u32 {
    glkapi().lock().unwrap().glk_get_buffer_stream(&from_ptr(str), glk_buffer_mut(buf, len)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_get_buffer_stream_uni(str: StreamPtr, buf: *mut u32, len: u32) -> u32 {
    glkapi().lock().unwrap().glk_get_buffer_stream_uni(&from_ptr(str), glk_buffer_mut(buf, len)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_get_char_stream_uni(str: StreamPtr) -> i32 {
    glkapi().lock().unwrap().glk_get_char_stream_uni(&from_ptr(str)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_get_char_stream(str: StreamPtr) -> i32 {
    glkapi().lock().unwrap().glk_get_char_stream(&from_ptr(str)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_get_line_stream(str: StreamPtr, buf: *mut u8, len: u32) -> u32 {
    glkapi().lock().unwrap().glk_get_line_stream(&from_ptr(str), glk_buffer_mut(buf, len)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_get_line_stream_uni(str: StreamPtr, buf: *mut u32, len: u32) -> u32 {
    glkapi().lock().unwrap().glk_get_line_stream_uni(&from_ptr(str), glk_buffer_mut(buf, len)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_put_buffer(buf: *mut u8, len: u32) {
    glkapi().lock().unwrap().glk_put_buffer(glk_buffer(buf, len)).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_buffer_stream(str: StreamPtr, buf: *mut u8, len: u32) {
    glkapi().lock().unwrap().glk_put_buffer_stream(&from_ptr(str), glk_buffer(buf, len)).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_buffer_stream_uni(str: StreamPtr, buf: *mut u32, len: u32) {
    glkapi().lock().unwrap().glk_put_buffer_stream_uni(&from_ptr(str), glk_buffer(buf, len)).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_buffer_uni(buf: *mut u32, len: u32) {
    glkapi().lock().unwrap().glk_put_buffer_uni(glk_buffer(buf, len)).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_char(ch: u8) {
    glkapi().lock().unwrap().glk_put_char(ch).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_char_stream(str: StreamPtr, ch: u8) {
    glkapi().lock().unwrap().glk_put_char_stream(&from_ptr(str), ch).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_char_stream_uni(str: StreamPtr, ch: u32) {
    glkapi().lock().unwrap().glk_put_char_stream_uni(&from_ptr(str), ch).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_char_uni(ch: u32) {
    glkapi().lock().unwrap().glk_put_char_uni(ch).ok();
}

#[no_mangle]
pub extern "C" fn glk_stream_close(str: StreamPtr, result: &mut Option<StreamResultCounts>) {
    let res = glkapi().lock().unwrap().glk_stream_close(reclaim(str)).unwrap();
    if let Some(result) = result {
        *result = res;
    }
}

#[no_mangle]
pub extern "C" fn glk_stream_get_current() -> StreamPtr {
    let glk = glkapi().lock().unwrap();
    let result = glk.glk_stream_get_current();
    borrow(result)
}

#[no_mangle]
pub extern "C" fn glk_stream_get_position(str: StreamPtr) -> u32 {
    glkapi().lock().unwrap().glk_stream_get_position(&from_ptr(str)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_stream_get_rock(str: StreamPtr) -> u32 {
    glkapi().lock().unwrap().glk_stream_get_rock(&from_ptr(str)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_stream_iterate(str: StreamPtr, rock: &mut u32) -> StreamPtr {
    let glk = glkapi().lock().unwrap();
    let str = from_ptr_opt(str);
    let res = glk.glk_stream_iterate(str.as_ref());
    let res = match res {
        Some(res) => {
            *rock = res.rock;
            Some(res.obj)
        },
        None => {
            *rock = 0;
            None
        }
    };
    borrow(res)
}

#[no_mangle]
pub extern "C" fn glk_stream_open_memory(buf: *mut u8, len: u32, fmode: FileMode, rock: u32) -> StreamPtr {
    let buf = unsafe{Box::from_raw(glk_buffer_mut(buf, len))};
    let result = glkapi().lock().unwrap().glk_stream_open_memory(buf, fmode, rock);
    result.unwrap().to_owned()
}

#[no_mangle]
pub extern "C" fn glk_stream_open_memory_uni(buf: *mut u32, len: u32, fmode: FileMode, rock: u32) -> StreamPtr {
    let buf = unsafe{Box::from_raw(glk_buffer_mut(buf, len))};
    let result = glkapi().lock().unwrap().glk_stream_open_memory_uni(buf, fmode, rock);
    result.unwrap().to_owned()
}

#[no_mangle]
pub extern "C" fn glk_stream_set_current(str: StreamPtr) {
    glkapi().lock().unwrap().glk_stream_set_current(from_ptr_opt(str).as_ref())
}

#[no_mangle]
pub extern "C" fn glk_stream_set_position(str: StreamPtr, mode: SeekMode, pos: i32) {
    glkapi().lock().unwrap().glk_stream_set_position(&from_ptr(str), mode, pos).ok();
}

/*#[no_mangle]
pub extern "C" fn glk_window_get_parent(win: &GlkWindow) -> GlkWindow {
    let result = glkapi().lock().unwrap().glk_window_get_parent(win).unwrap();
    result.unwrap().to_owned()
}*/

#[no_mangle]
pub extern "C" fn glk_window_get_rock(win: WindowPtr) -> u32 {
    glkapi().lock().unwrap().glk_window_get_rock(&from_ptr(win)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_window_get_root() -> WindowPtr {
    let glk = glkapi().lock().unwrap();
    let result = glk.glk_window_get_root();
    borrow(result)
}

#[no_mangle]
pub extern "C" fn glk_window_get_type(win: WindowPtr) -> WindowType {
    glkapi().lock().unwrap().glk_window_get_type(&from_ptr(win)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_window_iterate(win: WindowPtr, rock: &mut u32) -> WindowPtr {
    let glk = glkapi().lock().unwrap();
    let res = glk.glk_window_iterate(from_ptr_opt(win).as_ref());
    let res = match res {
        Some(res) => {
            *rock = res.rock;
            Some(res.obj)
        },
        None => {
            *rock = 0;
            None
        }
    };
    borrow(res)
}

#[no_mangle]
pub extern "C" fn glk_window_open(splitwin: WindowPtr, method: u32, size: u32, wintype: WindowType, rock: u32) -> WindowPtr {
    let result = glkapi().lock().unwrap().glk_window_open(from_ptr_opt(splitwin).as_ref(), method, size, wintype, rock);
    result.unwrap().to_owned()
}