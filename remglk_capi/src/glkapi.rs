/*

The Glk API
===========

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::num::NonZeroU32;
use std::sync::{Mutex, OnceLock};

use remglk::glkapi;
use glkapi::*;
use glkapi::constants::*;

use crate::common::*;

fn glkapi() -> &'static Mutex<GlkApi> {
    static GLKAPI: OnceLock<Mutex<GlkApi>> = OnceLock::new();
    GLKAPI.get_or_init(|| Mutex::new(GlkApi::new()))
}

// TODO: error handling!

#[no_mangle]
pub extern "C" fn glk_get_buffer_stream(str_id: Option<NonZeroU32>, buf: *mut u8, len: u32) -> u32 {
    glkapi().lock().unwrap().glk_get_buffer_stream(str_id, glk_buffer_mut(buf, len)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_get_buffer_stream_uni(str_id: Option<NonZeroU32>, buf: *mut u32, len: u32) -> u32 {
    glkapi().lock().unwrap().glk_get_buffer_stream_uni(str_id, glk_buffer_mut(buf, len)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_get_char_stream_uni(str_id: Option<NonZeroU32>) -> i32 {
    glkapi().lock().unwrap().glk_get_char_stream_uni(str_id).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_get_char_stream(str_id: Option<NonZeroU32>) -> i32 {
    glkapi().lock().unwrap().glk_get_char_stream(str_id).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_get_line_stream(str_id: Option<NonZeroU32>, buf: *mut u8, len: u32) -> u32 {
    glkapi().lock().unwrap().glk_get_line_stream(str_id, glk_buffer_mut(buf, len)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_get_line_stream_uni(str_id: Option<NonZeroU32>, buf: *mut u32, len: u32) -> u32 {
    glkapi().lock().unwrap().glk_get_line_stream_uni(str_id, glk_buffer_mut(buf, len)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_put_buffer(buf: *mut u8, len: u32) {
    glkapi().lock().unwrap().glk_put_buffer(glk_buffer(buf, len)).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_buffer_stream(str_id: Option<NonZeroU32>, buf: *mut u8, len: u32) {
    glkapi().lock().unwrap().glk_put_buffer_stream(str_id, glk_buffer(buf, len)).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_buffer_stream_uni(str_id: Option<NonZeroU32>, buf: *mut u32, len: u32) {
    glkapi().lock().unwrap().glk_put_buffer_stream_uni(str_id, glk_buffer(buf, len)).ok();
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
pub extern "C" fn glk_put_char_stream(str_id: Option<NonZeroU32>, ch: u8) {
    glkapi().lock().unwrap().glk_put_char_stream(str_id, ch).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_char_stream_uni(str_id: Option<NonZeroU32>, ch: u32) {
    glkapi().lock().unwrap().glk_put_char_stream_uni(str_id, ch).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_char_uni(ch: u32) {
    glkapi().lock().unwrap().glk_put_char_uni(ch).ok();
}

#[no_mangle]
pub extern "C" fn glk_stream_close(str_id: Option<NonZeroU32>, result: &mut Option<StreamResultCounts>) {
    let res = glkapi().lock().unwrap().glk_stream_close(str_id).unwrap();
    if let Some(result) = result {
        *result = res;
    }
}

#[no_mangle]
pub extern "C" fn glk_stream_get_current() -> Option<NonZeroU32> {
    glkapi().lock().unwrap().glk_stream_get_current()
}

#[no_mangle]
pub extern "C" fn glk_stream_get_position(str_id: Option<NonZeroU32>) -> u32 {
    glkapi().lock().unwrap().glk_stream_get_position(str_id).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_stream_get_rock(str_id: Option<NonZeroU32>) -> u32 {
    glkapi().lock().unwrap().glk_stream_get_rock(str_id).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_stream_iterate(str_id: Option<NonZeroU32>, rock: &mut u32) -> Option<NonZeroU32> {
    let res = glkapi().lock().unwrap().glk_stream_iterate(str_id);
    match res {
        Some(res) => {
            *rock = res.rock;
            Some(res.id)
        },
        None => {
            *rock = 0;
            None
        }
    }
}

#[no_mangle]
pub extern "C" fn glk_stream_open_memory(buf: *mut u8, len: u32, fmode: FileMode, rock: u32) -> Option<NonZeroU32> {
    let buf = unsafe{Box::from_raw(glk_buffer_mut(buf, len))};
    let result = glkapi().lock().unwrap().glk_stream_open_memory(buf, fmode, rock);
    result.ok()
}

#[no_mangle]
pub extern "C" fn glk_stream_open_memory_uni(buf: *mut u32, len: u32, fmode: FileMode, rock: u32) -> Option<NonZeroU32> {
    let buf = unsafe{Box::from_raw(glk_buffer_mut(buf, len))};
    let result = glkapi().lock().unwrap().glk_stream_open_memory_uni(buf, fmode, rock);
    result.ok()
}

#[no_mangle]
pub extern "C" fn glk_stream_set_current(str_id: Option<NonZeroU32>) {
    glkapi().lock().unwrap().glk_stream_set_current(str_id)
}

#[no_mangle]
pub extern "C" fn glk_stream_set_position(str_id: Option<NonZeroU32>, mode: SeekMode, pos: i32) {
    glkapi().lock().unwrap().glk_stream_set_position(str_id, mode, pos).ok();
}