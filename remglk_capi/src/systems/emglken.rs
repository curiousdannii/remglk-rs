/*

Emglken system
==============

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::slice;

use serde::de::DeserializeOwned;

use super::*;
use remglk::GlkSystem;
use glkapi::protocol::{Event, SystemFileRef, Update};

extern "C" {
    fn emglken_fileref_delete(fref_ptr: *const u8, fref_len: usize);
    fn emglken_fileref_exists(fref_ptr: *const u8, fref_len: usize) -> bool;
    fn emglken_fileref_read(fref_ptr: *const u8, fref_len: usize, buffer: *mut EmglkenBuffer) -> bool;
    fn emglken_fileref_temporary(filetype: FileType, buffer: *mut EmglkenBuffer);
    fn emglken_fileref_write_buffer(fref_ptr: *const u8, fref_len: usize, buf_ptr: *const u8, buf_len: usize);
    fn emglken_get_glkote_event(buffer: *mut EmglkenBuffer);
    fn emglken_send_glkote_update(update_ptr: *const u8, update_len: usize);
}

#[repr(C)]
pub struct EmglkenBuffer {
    pub ptr: *mut u8,
    pub len: usize,
}

pub type GlkApi = glkapi::GlkApi<EmglkenSystem>;

pub fn glkapi() -> &'static Mutex<GlkApi> {
    static GLKAPI: OnceLock<Mutex<GlkApi>> = OnceLock::new();
    GLKAPI.get_or_init(|| {
        Mutex::new(GlkApi::new(EmglkenSystem::default()))
    })
}

#[derive(Default)]
pub struct EmglkenSystem {
    cache: HashMap<SystemFileRef, Box<[u8]>>,
}

impl GlkSystem for EmglkenSystem {
    fn fileref_construct(&mut self, filename: String, filetype: FileType, gameid: Option<String>) -> SystemFileRef {
        SystemFileRef {
            filename,
            gameid,
            usage: Some(filetype),
            ..Default::default()
        }
    }

    fn fileref_delete(&mut self, fileref: &SystemFileRef) {
        self.cache.remove(&fileref);
        // TODO: cache the json inside the SystemFileRef?
        let json = serde_json::to_string(&fileref).unwrap();
        unsafe {emglken_fileref_delete(json.as_ptr(), json.len())};
    }

    fn fileref_exists(&mut self, fileref: &SystemFileRef) -> bool {
        self.cache.contains_key(&fileref) || {
            let json = serde_json::to_string(&fileref).unwrap();
            unsafe {emglken_fileref_exists(json.as_ptr(), json.len())}
        }
    }

    fn fileref_read(&mut self, fileref: &SystemFileRef) -> Option<Box<[u8]>> {
        // Check the cache first
        if let Some(buf) = self.cache.get(&fileref) {
            Some(buf.clone())
        }
        else {
            let mut buf: MaybeUninit<EmglkenBuffer> = MaybeUninit::uninit();
            let json = serde_json::to_string(&fileref).unwrap();
            let result = unsafe {emglken_fileref_read(json.as_ptr(), json.len(), buf.as_mut_ptr())};
            if result {
                return Some(buffer_to_boxed_slice(buf));
            }
            None
        }
    }

    fn fileref_temporary(&mut self, filetype: FileType) -> SystemFileRef {
        let mut buf: MaybeUninit<EmglkenBuffer> = MaybeUninit::uninit();
        unsafe {emglken_fileref_temporary(filetype, buf.as_mut_ptr())};
        buffer_to_protocol_struct(buf)
    }

    fn fileref_write_buffer(&mut self, fileref: &SystemFileRef, buf: Box<[u8]>) {
        self.cache.insert(fileref.clone(), buf);
    }

    fn flush_writeable_files(&mut self) {
        for (fileref, buf) in self.cache.drain() {
            let json = serde_json::to_string(&fileref).unwrap();
            unsafe {emglken_fileref_write_buffer(json.as_ptr(), json.len(), buf.as_ptr(), buf.len())};
        }
        self.cache.shrink_to(4);
    }

    fn get_glkote_event(&mut self) -> Option<Event> {
        let mut buf: MaybeUninit<EmglkenBuffer> = MaybeUninit::uninit();
        unsafe {emglken_get_glkote_event(buf.as_mut_ptr())};
        Some(buffer_to_protocol_struct(buf))
    }

    fn send_glkote_update(&mut self, update: Update) {
        // Send the update
        let json = serde_json::to_string(&update).unwrap();
        unsafe {emglken_send_glkote_update(json.as_ptr(), json.len())};
    }
}

fn buffer_to_boxed_slice(buffer: MaybeUninit<EmglkenBuffer>) -> Box<[u8]> {
    let buffer = unsafe {buffer.assume_init()};
    unsafe {Box::from_raw(slice::from_raw_parts_mut(buffer.ptr, buffer.len))}
}

fn buffer_to_protocol_struct<T: DeserializeOwned>(buffer: MaybeUninit<EmglkenBuffer>) -> T {
    let data = buffer_to_boxed_slice(buffer);
    serde_json::from_slice(&data).unwrap()
}