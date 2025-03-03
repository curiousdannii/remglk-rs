/*

The Glk API
===========

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::ffi::{CStr, c_void};
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
pub type FileRefPtr = *const Mutex<GlkObjectMetadata<FileRef>>;
pub type SchannelPtr = *const Mutex<GlkObjectMetadata<SoundChannel>>;
pub type StreamPtr = *const Mutex<GlkObjectMetadata<Stream>>;
pub type WindowPtr = *const Mutex<GlkObjectMetadata<Window>>;

#[cfg(target_os = "emscripten")]
#[path = "systems/emglken.rs"]
mod system;

#[cfg(not(target_os = "emscripten"))]
#[path = "systems/standard.rs"]
mod system;

pub use system::{glkapi, GlkApi};

// TODO: error handling!

#[no_mangle]
pub extern "C" fn glk_buffer_canon_decompose_uni(buf: BufferMutU32, len: u32, initlen: u32) -> u32 {
    GlkApi::glk_buffer_canon_decompose_uni(glk_buffer_mut(buf, len), initlen as usize) as u32
}

#[no_mangle]
pub extern "C" fn glk_buffer_canon_normalize_uni(buf: BufferMutU32, len: u32, initlen: u32) -> u32 {
    GlkApi::glk_buffer_canon_normalize_uni(glk_buffer_mut(buf, len), initlen as usize) as u32
}

#[no_mangle]
pub extern "C" fn glk_buffer_to_lower_case_uni(buf: BufferMutU32, len: u32, initlen: u32) -> u32 {
    GlkApi::glk_buffer_to_lower_case_uni(glk_buffer_mut(buf, len), initlen as usize) as u32
}

#[no_mangle]
pub extern "C" fn glk_buffer_to_title_case_uni(buf: BufferMutU32, len: u32, initlen: u32, lowerrest: u32) -> u32 {
    GlkApi::glk_buffer_to_title_case_uni(glk_buffer_mut(buf, len), initlen as usize, lowerrest > 0) as u32
}

#[no_mangle]
pub extern "C" fn glk_buffer_to_upper_case_uni(buf: BufferMutU32, len: u32, initlen: u32) -> u32 {
    GlkApi::glk_buffer_to_upper_case_uni(glk_buffer_mut(buf, len), initlen as usize) as u32
}

#[no_mangle]
pub extern "C" fn glk_cancel_char_event(win: WindowPtr) {
    GlkApi::glk_cancel_char_event(&from_ptr(win));
}

#[no_mangle]
pub extern "C" fn glk_cancel_hyperlink_event(win: WindowPtr) {
    GlkApi::glk_cancel_hyperlink_event(&from_ptr(win));
}

#[no_mangle]
pub extern "C" fn glk_cancel_line_event(win: WindowPtr, ev_ptr: *mut GlkEvent) {
    let res: GlkEvent = glkapi().lock().unwrap().glk_cancel_line_event(&from_ptr(win)).unwrap().into();
    write_ptr(ev_ptr, res);
}

#[no_mangle]
pub extern "C" fn glk_cancel_mouse_event(win: WindowPtr) {
    GlkApi::glk_cancel_mouse_event(&from_ptr(win));
}

#[no_mangle]
pub extern "C" fn glk_char_to_lower(val: u32) -> u32 {
    GlkApi::glk_char_to_lower(val)
}

#[no_mangle]
pub extern "C" fn glk_char_to_upper(val: u32) -> u32 {
    GlkApi::glk_char_to_upper(val)
}

#[no_mangle]
pub extern "C" fn glk_current_simple_time(factor: u32) -> i32 {
    GlkApi::glk_current_simple_time(factor)
}

#[no_mangle]
pub extern "C" fn glk_current_time(time_ptr: *mut GlkTime) {
    let time = GlkApi::glk_current_time();
    write_ptr(time_ptr, time);
}

#[no_mangle]
pub extern "C" fn glk_date_to_simple_time_local(date_ptr: *const GlkDate, factor: u32) -> i32 {
    GlkApi::glk_date_to_simple_time_local(unsafe{&(*date_ptr)}, factor)
}

#[no_mangle]
pub extern "C" fn glk_date_to_simple_time_utc(date_ptr: *const GlkDate, factor: u32) -> i32 {
    GlkApi::glk_date_to_simple_time_utc(unsafe{&(*date_ptr)}, factor)
}

#[no_mangle]
pub extern "C" fn glk_date_to_time_local(date_ptr: *const GlkDate, time_ptr: *mut GlkTime) {
    let time = GlkApi::glk_date_to_time_local(unsafe{&(*date_ptr)});
    write_ptr(time_ptr, time);
}

#[no_mangle]
pub extern "C" fn glk_date_to_time_utc(date_ptr: *const GlkDate, time_ptr: *mut GlkTime) {
    let time = GlkApi::glk_date_to_time_utc(unsafe{&(*date_ptr)});
    write_ptr(time_ptr, time);
}

#[no_mangle]
pub extern "C" fn glk_exit() {
    glkapi().lock().unwrap().glk_exit();
    std::process::exit(0);
}

#[no_mangle]
pub extern "C" fn glk_fileref_create_by_name(usage: u32, filename_ptr: *const i8, rock: u32) -> FileRefPtr {
    let filename_cstr = unsafe {CStr::from_ptr(filename_ptr)};
    let filename = filename_cstr.to_string_lossy().to_string();
    let result = glkapi().lock().unwrap().glk_fileref_create_by_name(usage, filename, rock);
    to_owned(result)
}

#[no_mangle]
pub extern "C" fn glk_fileref_create_by_prompt(usage: u32, fmode: FileMode, rock: u32) -> FileRefPtr {
    let result = glkapi().lock().unwrap().glk_fileref_create_by_prompt(usage, fmode, rock);
    to_owned_opt(result.unwrap())
}

#[no_mangle]
pub extern "C" fn glk_fileref_create_from_fileref(usage: u32, fileref: FileRefPtr, rock: u32) -> FileRefPtr {
    let result = glkapi().lock().unwrap().glk_fileref_create_from_fileref(usage, &from_ptr(fileref), rock);
    to_owned(result)
}

#[no_mangle]
pub extern "C" fn glk_fileref_create_temp(usage: u32, rock: u32) -> FileRefPtr {
    let result = glkapi().lock().unwrap().glk_fileref_create_temp(usage, rock);
    to_owned(result)
}

#[no_mangle]
pub extern "C" fn glk_fileref_delete_file(fileref: FileRefPtr) {
    glkapi().lock().unwrap().glk_fileref_delete_file(&from_ptr(fileref));
}

#[no_mangle]
pub extern "C" fn glk_fileref_destroy(fileref: FileRefPtr) {
    glkapi().lock().unwrap().glk_fileref_destroy(reclaim(fileref));
}

#[no_mangle]
pub extern "C" fn glk_fileref_does_file_exist(fileref: FileRefPtr) -> u32 {
    glkapi().lock().unwrap().glk_fileref_does_file_exist(&from_ptr(fileref)) as u32
}

#[no_mangle]
pub extern "C" fn glk_fileref_get_rock(fileref: FileRefPtr) -> u32 {
    GlkApi::glk_fileref_get_rock(&from_ptr(fileref)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_fileref_iterate(fileref: FileRefPtr, rock_ptr: *mut u32) -> FileRefPtr {
    let fileref = from_ptr_opt(fileref);
    let next = glkapi().lock().unwrap().glk_fileref_iterate(fileref.as_ref());
    let (obj, rock) = if let Some(obj) = next {
        let rock = obj.lock().unwrap().rock;
        (Some(obj), rock)
    }
    else {
        (None, 0)
    };
    write_ptr(rock_ptr, rock);
    borrow(obj.as_ref())
}

#[no_mangle]
pub extern "C" fn glk_gestalt(sel: u32, val: u32) -> u32 {
    glkapi().lock().unwrap().glk_gestalt(sel, val)
}

#[no_mangle]
pub extern "C" fn glk_gestalt_ext(sel: u32, val: u32, buf: BufferMutU32, len: u32) -> u32 {
    glkapi().lock().unwrap().glk_gestalt_ext(sel, val, glk_buffer_mut_opt(buf, len))
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
pub extern "C" fn glk_image_draw(win: WindowPtr, image: u32, val1: i32, val2: i32) -> u32 {
    GlkApi::glk_image_draw(&from_ptr(win), image, val1, val2)
}

#[no_mangle]
pub extern "C" fn glk_image_draw_scaled(win: WindowPtr, image: u32, val1: i32, val2: i32, width: u32, height: u32) -> u32 {
    GlkApi::glk_image_draw_scaled(&from_ptr(win), image, val1, val2, width, height)
}

#[no_mangle]
pub extern "C" fn glk_image_get_info(image: u32, width_ptr: *mut u32, height_ptr: *mut u32) -> u32 {
    let res = GlkApi::glk_image_get_info(image);
    if let Some(info) = res {
        write_ptr(height_ptr, info.height);
        write_ptr(width_ptr, info.width);
        return 1;
    }
    0
}

#[no_mangle]
pub extern "C" fn glk_put_buffer(buf: BufferU8, len: u32) {
    glkapi().lock().unwrap().glk_put_buffer(glk_buffer(buf, len)).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_buffer_stream(str: StreamPtr, buf: BufferU8, len: u32) {
    glkapi().lock().unwrap().glk_put_buffer_stream(&from_ptr(str), glk_buffer(buf, len)).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_buffer_stream_uni(str: StreamPtr, buf: BufferU32, len: u32) {
    glkapi().lock().unwrap().glk_put_buffer_stream_uni(&from_ptr(str), glk_buffer(buf, len)).ok();
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
pub extern "C" fn glk_put_string(cstr: CStringU8) {
    glkapi().lock().unwrap().glk_put_buffer(cstring_u8(cstr)).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_string_stream(str: StreamPtr, cstr: CStringU8) {
    glkapi().lock().unwrap().glk_put_buffer_stream(&from_ptr(str), cstring_u8(cstr)).ok();
}

#[no_mangle]
pub extern "C" fn glk_put_string_stream_uni(str: StreamPtr, cstr: CStringU32) {
    glkapi().lock().unwrap().glk_put_buffer_stream_uni(&from_ptr(str), cstring_u32(cstr)).ok();
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
pub extern "C" fn glk_schannel_create(rock: u32) -> SchannelPtr {
    let result = glkapi().lock().unwrap().glk_schannel_create(rock);
    to_owned(result)
}

#[no_mangle]
pub extern "C" fn glk_schannel_create_ext(rock: u32, vol: u32) -> SchannelPtr {
    let result = glkapi().lock().unwrap().glk_schannel_create_ext(rock, vol);
    to_owned(result)
}

#[no_mangle]
pub extern "C" fn glk_schannel_destroy(schannel: SchannelPtr) {
    glkapi().lock().unwrap().glk_schannel_destroy(reclaim(schannel));
}

#[no_mangle]
pub extern "C" fn glk_schannel_get_rock(schannel: SchannelPtr) -> u32 {
    GlkApi::glk_schannel_get_rock(&from_ptr(schannel)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_schannel_iterate(schannel: SchannelPtr, rock_ptr: *mut u32) -> SchannelPtr {
    let schannel = from_ptr_opt(schannel);
    let next = glkapi().lock().unwrap().glk_schannel_iterate(schannel.as_ref());
    let (obj, rock) = if let Some(obj) = next {
        let rock = obj.lock().unwrap().rock;
        (Some(obj), rock)
    }
    else {
        (None, 0)
    };
    write_ptr(rock_ptr, rock);
    borrow(obj.as_ref())
}

#[no_mangle]
pub extern "C" fn glk_schannel_pause(schannel: SchannelPtr) {
    glkapi().lock().unwrap().glk_schannel_pause(&from_ptr(schannel));
}

#[no_mangle]
pub extern "C" fn glk_schannel_play(schannel: SchannelPtr, snd: u32) -> u32 {
    glkapi().lock().unwrap().glk_schannel_play(&from_ptr(schannel), snd)
}

#[no_mangle]
pub extern "C" fn glk_schannel_play_ext(schannel: SchannelPtr, snd: u32, repeats: u32, notify: u32) -> u32 {
    glkapi().lock().unwrap().glk_schannel_play_ext(&from_ptr(schannel), snd, repeats, notify)
}

#[no_mangle]
pub extern "C" fn glk_schannel_play_multi(schannels: *const SchannelPtr, schannels_len: u32, sounds: BufferU32, sounds_len: u32, notify: u32) -> u32 {
    glkapi().lock().unwrap().glk_schannel_play_multi(from_ptr_array(schannels, schannels_len), glk_buffer(sounds, sounds_len), notify)
}

#[no_mangle]
pub extern "C" fn glk_schannel_set_volume(schannel: SchannelPtr, vol: u32) {
    glkapi().lock().unwrap().glk_schannel_set_volume(&from_ptr(schannel), vol);
}

#[no_mangle]
pub extern "C" fn glk_schannel_set_volume_ext(schannel: SchannelPtr, vol: u32, duration: u32, notify: u32) {
    glkapi().lock().unwrap().glk_schannel_set_volume_ext(&from_ptr(schannel), vol, duration, notify);
}

#[no_mangle]
pub extern "C" fn glk_schannel_stop(schannel: SchannelPtr) {
    glkapi().lock().unwrap().glk_schannel_stop(&from_ptr(schannel));
}

#[no_mangle]
pub extern "C" fn glk_schannel_unpause(schannel: SchannelPtr) {
    glkapi().lock().unwrap().glk_schannel_unpause(&from_ptr(schannel));
}

#[no_mangle]
pub extern "C" fn glk_select(ev_ptr: *mut GlkEvent) {
    let res = glkapi().lock().unwrap().glk_select().unwrap().into();
    write_ptr(ev_ptr, res);
}

#[no_mangle]
pub extern "C" fn glk_select_poll(ev_ptr: *mut GlkEvent) {
    let res: GlkEvent = glkapi().lock().unwrap().glk_select_poll().into();
    write_ptr(ev_ptr, res);
}

#[no_mangle]
pub extern "C" fn glk_set_echo_line_event(win: WindowPtr, val: u32) {
    GlkApi::glk_set_echo_line_event(&from_ptr(win), val);
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
pub extern "C" fn glk_set_interrupt_handler(_func: *const c_void) {}

#[no_mangle]
pub extern "C" fn glk_set_style(val: u32) {
    glkapi().lock().unwrap().glk_set_style(val).ok();
}

#[no_mangle]
pub extern "C" fn glk_set_style_stream(str: StreamPtr, val: u32) {
    GlkApi::glk_set_style_stream(&from_ptr(str), val);
}

#[no_mangle]
pub extern "C" fn glk_set_terminators_line_event(win: WindowPtr, keycodes_ptr: *mut TerminatorCode, count: u32) {
    let terminators = glk_buffer(keycodes_ptr, count);
    GlkApi::glk_set_terminators_line_event(&from_ptr(win), terminators.to_vec());
}

#[no_mangle]
pub extern "C" fn glk_set_window(win: WindowPtr) {
    glkapi().lock().unwrap().glk_set_window(from_ptr_opt(win).as_ref())
}

#[no_mangle]
pub extern "C" fn glk_simple_time_to_date_local(time: i32, factor: u32, date_ptr: *mut GlkDate) {
    let date = GlkApi::glk_simple_time_to_date_local(time, factor);
    write_ptr(date_ptr, date);
}

#[no_mangle]
pub extern "C" fn glk_simple_time_to_date_utc(time: i32, factor: u32, date_ptr: *mut GlkDate) {
    let date = GlkApi::glk_simple_time_to_date_utc(time, factor);
    write_ptr(date_ptr, date);
}

#[no_mangle]
pub extern "C" fn glk_sound_load_hint(_snd: u32, _flag: u32) {}

#[no_mangle]
pub extern "C" fn glk_stream_close(str: StreamPtr, result_ptr: *mut StreamResultCounts) {
    let res = glkapi().lock().unwrap().glk_stream_close(reclaim(str)).unwrap();
    write_ptr(result_ptr, res);
}

#[no_mangle]
pub extern "C" fn glk_stream_get_current() -> StreamPtr {
    let result = glkapi().lock().unwrap().glk_stream_get_current();
    borrow(result.as_ref())
}

#[no_mangle]
pub extern "C" fn glk_stream_get_position(str: StreamPtr) -> u32 {
    GlkApi::glk_stream_get_position(&from_ptr(str))
}

#[no_mangle]
pub extern "C" fn glk_stream_get_rock(str: StreamPtr) -> u32 {
    GlkApi::glk_stream_get_rock(&from_ptr(str)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_stream_iterate(str: StreamPtr, rock_ptr: *mut u32) -> StreamPtr {
    let str = from_ptr_opt(str);
    let next = glkapi().lock().unwrap().glk_stream_iterate(str.as_ref());
    let (obj, rock) = if let Some(obj) = next {
        let rock = obj.lock().unwrap().rock;
        (Some(obj), rock)
    }
    else {
        (None, 0)
    };
    write_ptr(rock_ptr, rock);
    borrow(obj.as_ref())
}

#[no_mangle]
pub extern "C" fn glk_stream_open_file(fileref: FileRefPtr, mode: FileMode, rock: u32) -> StreamPtr {
    let result = glkapi().lock().unwrap().glk_stream_open_file(&from_ptr(fileref), mode, rock);
    to_owned_opt(result.unwrap())
}

#[no_mangle]
pub extern "C" fn glk_stream_open_file_uni(fileref: FileRefPtr, mode: FileMode, rock: u32) -> StreamPtr {
    let result = glkapi().lock().unwrap().glk_stream_open_file_uni(&from_ptr(fileref), mode, rock);
    to_owned_opt(result.unwrap())
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
pub extern "C" fn glk_stream_open_resource(filenum: u32, rock: u32) -> StreamPtr {
    let result = glkapi().lock().unwrap().glk_stream_open_resource(filenum, rock);
    to_owned_opt(result.unwrap())
}

#[no_mangle]
pub extern "C" fn glk_stream_open_resource_uni(filenum: u32, rock: u32) -> StreamPtr {
    let result = glkapi().lock().unwrap().glk_stream_open_resource_uni(filenum, rock);
    to_owned_opt(result.unwrap())
}

#[no_mangle]
pub extern "C" fn glk_stream_set_current(str: StreamPtr) {
    glkapi().lock().unwrap().glk_stream_set_current(from_ptr_opt(str).as_ref())
}

#[no_mangle]
pub extern "C" fn glk_stream_set_position(str: StreamPtr, pos: i32, mode: SeekMode) {
    GlkApi::glk_stream_set_position(&from_ptr(str), pos, mode);
}

#[no_mangle]
pub extern "C" fn glk_style_distinguish(_win: WindowPtr, _style1: u32, _style2: u32) -> u32 {
    0
}

#[no_mangle]
pub extern "C" fn glk_style_measure(_win: WindowPtr, _style: u32, _hint: u32, result_ptr: *mut u32) -> u32 {
    write_ptr(result_ptr, 0);
    0
}

#[no_mangle]
pub extern "C" fn glk_stylehint_clear(wintype: WindowType, style: u32, hint: u32) {
    glkapi().lock().unwrap().glk_stylehint_clear(wintype, style, hint);
}

#[no_mangle]
pub extern "C" fn glk_stylehint_set(wintype: WindowType, style: u32, hint: u32, value: i32) {
    glkapi().lock().unwrap().glk_stylehint_set(wintype, style, hint, value);
}

#[no_mangle]
pub extern "C" fn glk_tick() {}

#[no_mangle]
pub extern "C" fn glk_time_to_date_local(time_ptr: *const GlkTime, date_ptr: *mut GlkDate) {
    let date = GlkApi::glk_time_to_date_local(unsafe{&(*time_ptr)});
    write_ptr(date_ptr, date);
}

#[no_mangle]
pub extern "C" fn glk_time_to_date_utc(time_ptr: *const GlkTime, date_ptr: *mut GlkDate) {
    let date = GlkApi::glk_time_to_date_utc(unsafe{&(*time_ptr)});
    write_ptr(date_ptr, date);
}

#[no_mangle]
pub extern "C" fn glk_window_clear(win: WindowPtr) {
    glkapi().lock().unwrap().glk_window_clear(&from_ptr(win));
}

#[no_mangle]
pub extern "C" fn glk_window_close(win: WindowPtr, result_ptr: *mut StreamResultCounts) {
    let result = glkapi().lock().unwrap().glk_window_close(reclaim(win)).unwrap();
    write_ptr(result_ptr, result);
}

#[no_mangle]
pub extern "C" fn glk_window_erase_rect(win: WindowPtr, left: i32, top: i32, width: u32, height: u32) {
    GlkApi::glk_window_erase_rect(&from_ptr(win), left, top, width, height).unwrap();
}

#[no_mangle]
pub extern "C" fn glk_window_fill_rect(win: WindowPtr, colour: u32, left: i32, top: i32, width: u32, height: u32) {
    GlkApi::glk_window_fill_rect(&from_ptr(win), colour, left, top, width, height).unwrap();
}

#[no_mangle]
pub extern "C" fn glk_window_flow_break(win: WindowPtr) {
    GlkApi::glk_window_flow_break(&from_ptr(win));
}

#[no_mangle]
pub extern "C" fn glk_window_get_arrangement(win: WindowPtr, method_ptr: *mut u32, size_ptr: *mut u32, keywin_ptr: *mut WindowPtr) {
    let (method, size, keywin) = GlkApi::glk_window_get_arrangement(&from_ptr(win)).unwrap();
    write_ptr(method_ptr, method);
    write_ptr(size_ptr, size);
    write_ptr(keywin_ptr, borrow(Some(&keywin)));
}

#[no_mangle]
pub extern "C" fn glk_window_get_echo_stream(win: WindowPtr) -> StreamPtr {
    let result = GlkApi::glk_window_get_echo_stream(&from_ptr(win));
    borrow(result.as_ref())
}

#[no_mangle]
pub extern "C" fn glk_window_get_parent(win: WindowPtr) -> WindowPtr {
    let result = GlkApi::glk_window_get_parent(&from_ptr(win));
    borrow(result.as_ref())
}

#[no_mangle]
pub extern "C" fn glk_window_get_rock(win: WindowPtr) -> u32 {
    GlkApi::glk_window_get_rock(&from_ptr(win)).unwrap()
}

#[no_mangle]
pub extern "C" fn glk_window_get_root() -> WindowPtr {
    let result = glkapi().lock().unwrap().glk_window_get_root();
    borrow(result.as_ref())
}

#[no_mangle]
pub extern "C" fn glk_window_get_sibling(win: WindowPtr) -> WindowPtr {
    let result = GlkApi::glk_window_get_sibling(&from_ptr(win)).unwrap();
    borrow(result.as_ref())
}

#[no_mangle]
pub extern "C" fn glk_window_get_size(win: WindowPtr, width_ptr: *mut u32, height_ptr: *mut u32) {
    let (height, width) = glkapi().lock().unwrap().glk_window_get_size(&from_ptr(win));
    write_ptr(height_ptr, height as u32);
    write_ptr(width_ptr, width as u32);
}

#[no_mangle]
pub extern "C" fn glk_window_get_stream(win: WindowPtr) -> StreamPtr {
    let result = GlkApi::glk_window_get_stream(&from_ptr(win));
    borrow(Some(&result))
}

#[no_mangle]
pub extern "C" fn glk_window_get_type(win: WindowPtr) -> WindowType {
    GlkApi::glk_window_get_type(&from_ptr(win))
}

#[no_mangle]
pub extern "C" fn glk_window_iterate(win: WindowPtr, rock_ptr: *mut u32) -> WindowPtr {
    let win: Option<GlkObject<Window>> = from_ptr_opt(win);
    let next = glkapi().lock().unwrap().glk_window_iterate(win.as_ref());
    let (obj, rock) = if let Some(obj) = next {
        let rock = obj.lock().unwrap().rock;
        (Some(obj), rock)
    }
    else {
        (None, 0)
    };
    write_ptr(rock_ptr, rock);
    borrow(obj.as_ref())
}

#[no_mangle]
pub extern "C" fn glk_window_move_cursor(win: WindowPtr, xpos: u32, ypos: u32) {
    GlkApi::glk_window_move_cursor(&from_ptr(win), xpos as usize, ypos as usize).unwrap();
}

#[no_mangle]
pub extern "C" fn glk_window_open(splitwin: WindowPtr, method: u32, size: u32, wintype: WindowType, rock: u32) -> WindowPtr {
    let result = glkapi().lock().unwrap().glk_window_open(from_ptr_opt(splitwin).as_ref(), method, size, wintype, rock);
    to_owned(result.unwrap())
}

#[no_mangle]
pub extern "C" fn glk_window_set_arrangement(win: WindowPtr, method: u32, size: u32, keywin: WindowPtr) {
    glkapi().lock().unwrap().glk_window_set_arrangement(&from_ptr(win), method, size, from_ptr_opt(keywin).as_ref()).unwrap();
}

#[no_mangle]
pub extern "C" fn glk_window_set_background_color(win: WindowPtr, colour: u32) {
    GlkApi::glk_window_set_background_color(&from_ptr(win), colour).unwrap();
}

#[no_mangle]
pub extern "C" fn glk_window_set_echo_stream(win: WindowPtr, str: StreamPtr) {
    GlkApi::glk_window_set_echo_stream(&from_ptr(win), from_ptr_opt(str).as_ref())
}

// Extensions

#[no_mangle]
pub extern "C" fn garglk_set_reversevideo(val: u32) {
    glkapi().lock().unwrap().garglk_set_reversevideo(val).ok();
}

#[no_mangle]
pub extern "C" fn garglk_set_reversevideo_stream(str: StreamPtr, val: u32) {
    GlkApi::garglk_set_reversevideo_stream(&from_ptr(str), val);
}

#[no_mangle]
pub extern "C" fn garglk_set_zcolors(fg: u32, bg: u32) {
    glkapi().lock().unwrap().garglk_set_zcolors(fg, bg).ok();
}

#[no_mangle]
pub extern "C" fn garglk_set_zcolors_stream(str: StreamPtr, fg: u32, bg: u32) {
    GlkApi::garglk_set_zcolors_stream(&from_ptr(str), fg, bg);
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