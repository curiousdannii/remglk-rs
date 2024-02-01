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

type BufferU8 = *const u8;
type BufferU32 = *const u32;
type BufferMutU8 = *mut u8;
type BufferMutU32 = *mut u32;
type CStringU8 = *const i8;
type CStringU32 = *const u32;
type StreamPtr = *const Mutex<Stream>;
type WindowPtr = *const Mutex<Window>;

fn glkapi() -> &'static Mutex<GlkApi> {
    static GLKAPI: OnceLock<Mutex<GlkApi>> = OnceLock::new();
    GLKAPI.get_or_init(|| Mutex::new(GlkApi::default()))
}

// TODO: error handling!

#[no_mangle]
pub extern "C" fn glk_cancel_char_event(win: WindowPtr) {
    GlkApi::glk_cancel_char_event(&from_ptr(win));
}

#[no_mangle]
pub extern "C" fn glk_cancel_hyperlink_event(win: WindowPtr) {
    GlkApi::glk_cancel_hyperlink_event(&from_ptr(win));
}

#[no_mangle]
pub extern "C" fn glk_cancel_line_event(win: WindowPtr, ev: &mut GlkEvent) {
    let res: GlkEvent = glkapi().lock().unwrap().glk_cancel_line_event(&from_ptr(win)).unwrap().into();
    *ev = res;
}

#[no_mangle]
pub extern "C" fn glk_cancel_mouse_event(win: WindowPtr) {
    GlkApi::glk_cancel_mouse_event(&from_ptr(win));
}

#[no_mangle]
pub extern "C" fn glk_get_buffer_stream(str: StreamPtr, buf: BufferMutU8, len: u32) -> u32 {
    GlkApi::glk_get_buffer_stream(&from_ptr(str), glk_buffer_mut(buf, len)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_get_buffer_stream_uni(str: StreamPtr, buf: BufferMutU32, len: u32) -> u32 {
    GlkApi::glk_get_buffer_stream_uni(&from_ptr(str), glk_buffer_mut(buf, len)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_get_char_stream_uni(str: StreamPtr) -> i32 {
    GlkApi::glk_get_char_stream_uni(&from_ptr(str)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_get_char_stream(str: StreamPtr) -> i32 {
    GlkApi::glk_get_char_stream(&from_ptr(str)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_get_line_stream(str: StreamPtr, buf: BufferMutU8, len: u32) -> u32 {
    GlkApi::glk_get_line_stream(&from_ptr(str), glk_buffer_mut(buf, len)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_get_line_stream_uni(str: StreamPtr, buf: BufferMutU32, len: u32) -> u32 {
    GlkApi::glk_get_line_stream_uni(&from_ptr(str), glk_buffer_mut(buf, len)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_put_buffer(buf: BufferU8, len: u32) {
    glkapi().lock().unwrap().glk_put_buffer(glk_buffer(buf, len)).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_buffer_stream(str: StreamPtr, buf: BufferU8, len: u32) {
    GlkApi::glk_put_buffer_stream(&from_ptr(str), glk_buffer(buf, len)).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_buffer_stream_uni(str: StreamPtr, buf: BufferU32, len: u32) {
    GlkApi::glk_put_buffer_stream_uni(&from_ptr(str), glk_buffer(buf, len)).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_buffer_uni(buf: BufferU32, len: u32) {
    glkapi().lock().unwrap().glk_put_buffer_uni(glk_buffer(buf, len)).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_char(ch: u8) {
    glkapi().lock().unwrap().glk_put_char(ch).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_char_stream(str: StreamPtr, ch: u8) {
    GlkApi::glk_put_char_stream(&from_ptr(str), ch).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_char_stream_uni(str: StreamPtr, ch: u32) {
    GlkApi::glk_put_char_stream_uni(&from_ptr(str), ch).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_char_uni(ch: u32) {
    glkapi().lock().unwrap().glk_put_char_uni(ch).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_string(cstr: CStringU8) {
    glkapi().lock().unwrap().glk_put_buffer(cstring_u8(cstr)).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_string_stream(str: StreamPtr, cstr: CStringU8) {
    GlkApi::glk_put_buffer_stream(&from_ptr(str), cstring_u8(cstr)).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_string_stream_uni(str: StreamPtr, cstr: CStringU32) {
    GlkApi::glk_put_buffer_stream_uni(&from_ptr(str), cstring_u32(cstr)).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_string_uni(cstr: CStringU32) {
    glkapi().lock().unwrap().glk_put_buffer_uni(cstring_u32(cstr)).ok();
}

#[no_mangle]
pub extern "C" fn glk_request_char_event(win: WindowPtr) {
    glkapi().lock().unwrap().glk_request_char_event(&from_ptr(win)).unwrap();
}

#[no_mangle]
pub extern "C" fn glk_request_char_event_uni(win: WindowPtr) {
    glkapi().lock().unwrap().glk_request_char_event_uni(&from_ptr(win)).unwrap();
}

#[no_mangle]
pub extern "C" fn glk_request_hyperlink_event(win: WindowPtr) {
    GlkApi::glk_request_hyperlink_event(&from_ptr(win));
}

#[no_mangle]
pub extern "C" fn glk_request_line_event(win: WindowPtr, buf: BufferMutU8, len: u32, initlen: u32) {
    let buf = unsafe{Box::from_raw(glk_buffer_mut(buf, len))};
    glkapi().lock().unwrap().glk_request_line_event(&from_ptr(win), buf, initlen).unwrap();
}

#[no_mangle]
pub extern "C" fn glk_request_line_event_uni(win: WindowPtr, buf: BufferMutU32, len: u32, initlen: u32) {
    let buf = unsafe{Box::from_raw(glk_buffer_mut(buf, len))};
    glkapi().lock().unwrap().glk_request_line_event_uni(&from_ptr(win), buf, initlen).unwrap();
}

#[no_mangle]
pub extern "C" fn glk_request_mouse_event(win: WindowPtr) {
    GlkApi::glk_request_mouse_event(&from_ptr(win));
}

#[no_mangle]
pub extern "C" fn glk_request_timer_events(msecs: u32) {
    glkapi().lock().unwrap().glk_request_timer_events(msecs);
}

#[no_mangle]
pub extern "C" fn glk_set_hyperlink(val: u32) {
    glkapi().lock().unwrap().glk_set_hyperlink(val).ok();
}

#[no_mangle]
pub extern "C" fn glk_set_hyperlink_stream(str: StreamPtr, val: u32) {
    GlkApi::glk_set_hyperlink_stream(&from_ptr(str), val);
}

#[no_mangle]
pub extern "C" fn glk_set_style(val: u32) {
    glkapi().lock().unwrap().glk_set_style(val).ok();
}

#[no_mangle]
pub extern "C" fn glk_set_style_stream(str: StreamPtr, val: u32) {
    GlkApi::glk_set_style_stream(&from_ptr(str), val);
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
    GlkApi::glk_stream_get_position(&from_ptr(str))
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
    borrow(res.as_ref())
}

#[no_mangle]
pub extern "C" fn glk_stream_open_memory(buf: BufferMutU8, len: u32, fmode: FileMode, rock: u32) -> StreamPtr {
    let buf = unsafe{Box::from_raw(glk_buffer_mut(buf, len))};
    let result = glkapi().lock().unwrap().glk_stream_open_memory(buf, fmode, rock);
    to_owned(result.unwrap())
}

#[no_mangle]
pub extern "C" fn glk_stream_open_memory_uni(buf: BufferMutU32, len: u32, fmode: FileMode, rock: u32) -> StreamPtr {
    let buf = unsafe{Box::from_raw(glk_buffer_mut(buf, len))};
    let result = glkapi().lock().unwrap().glk_stream_open_memory_uni(buf, fmode, rock);
    to_owned(result.unwrap())
}

#[no_mangle]
pub extern "C" fn glk_stream_set_current(str: StreamPtr) {
    glkapi().lock().unwrap().glk_stream_set_current(from_ptr_opt(str).as_ref())
}

#[no_mangle]
pub extern "C" fn glk_stream_set_position(str: StreamPtr, mode: SeekMode, pos: i32) {
    GlkApi::glk_stream_set_position(&from_ptr(str), mode, pos);
}

#[no_mangle]
pub extern "C" fn glk_window_clear(win: WindowPtr) {
    GlkApi::glk_window_clear(&from_ptr(win));
}

/*#[no_mangle]
pub extern "C" fn glk_window_get_parent(win: &GlkWindow) -> GlkWindow {
    let result = glkapi().lock().unwrap().glk_window_get_parent(win).unwrap();
    to_owned(result.unwrap())
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
    GlkApi::glk_window_get_type(&from_ptr(win))
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
    borrow(res.as_ref())
}

#[no_mangle]
pub extern "C" fn glk_window_open(splitwin: WindowPtr, method: u32, size: u32, wintype: WindowType, rock: u32) -> WindowPtr {
    let result = glkapi().lock().unwrap().glk_window_open(from_ptr_opt(splitwin).as_ref(), method, size, wintype, rock);
    to_owned(result.unwrap())
}

/** A Glk event */
#[derive(Clone, Copy)]
#[repr(C)]
pub struct GlkEvent {
    pub evtype: GlkEventType,
    pub win: WindowPtr,
    pub val1: u32,
    pub val2: u32,
}

impl From<glkapi::GlkEvent> for GlkEvent {
    fn from(ev: glkapi::GlkEvent) -> Self {
        GlkEvent {
            evtype: ev.evtype,
            win: borrow(ev.win.as_ref()),
            val1: ev.val1,
            val2: ev.val2,
        }
    }
}