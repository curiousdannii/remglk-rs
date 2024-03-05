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

use super::*;
use remglk::GlkSystem;
use glkapi::protocol::{Event, SystemFileRef, Update};

extern "C" {
    fn emglken_fileref_exists(filename_ptr: *const u8, filename_len: usize) -> bool;
    fn emglken_fileref_read(filename_ptr: *const u8, filename_len: usize, buffer: *mut EmglkenBuffer) -> bool;
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
    cache: HashMap<String, Box<[u8]>>,
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

    fn fileref_delete(&mut self, _fileref: &SystemFileRef) {
        unimplemented!()
    }

    fn fileref_exists(&mut self, fileref: &SystemFileRef) -> bool {
        self.cache.contains_key(&fileref.filename) || unsafe {emglken_fileref_exists(fileref.filename.as_ptr(), fileref.filename.len())}
    }

    fn fileref_read(&mut self, fileref: &SystemFileRef) -> Option<Box<[u8]>> {
        // Check the cache first
        if let Some(buf) = self.cache.get(&fileref.filename) {
            Some(buf.clone())
        }
        else {
            let mut buf: MaybeUninit<EmglkenBuffer> = MaybeUninit::uninit();
            let result = unsafe {emglken_fileref_read(fileref.filename.as_ptr(), fileref.filename.len(), buf.as_mut_ptr())};
            if result {
                return Some(buffer_to_boxed_slice(&unsafe {buf.assume_init()}));
            }
            None
        }
    }

    fn fileref_temporary(&mut self, _filetype: FileType) -> SystemFileRef {
        unimplemented!()
    }

    fn fileref_write_buffer(&mut self, _fileref: &SystemFileRef, _buf: Box<[u8]>) {
        unimplemented!()
    }

    fn flush_writeable_files(&mut self) {
        for (_filename, _buf) in self.cache.drain() {
            unimplemented!()
        }
        self.cache.shrink_to(4);
    }

    fn get_glkote_event(&mut self) -> Option<Event> {
        let mut buf: MaybeUninit<EmglkenBuffer> = MaybeUninit::uninit();
        unsafe {emglken_get_glkote_event(buf.as_mut_ptr())};
        let data = buffer_to_boxed_slice(&unsafe {buf.assume_init()});
        let event: Event = serde_json::from_slice(&data).unwrap();
        return Some(event);
    }

    fn send_glkote_update(&mut self, update: Update) {
        // Send the update
        let output = serde_json::to_string(&update).unwrap();
        unsafe {emglken_send_glkote_update(output.as_ptr(), output.len())};
    }
}

fn buffer_to_boxed_slice(buffer: &EmglkenBuffer) -> Box<[u8]> {
    unsafe {Box::from_raw(slice::from_raw_parts_mut(buffer.ptr, buffer.len))}
}