/*

Emglken system
==============

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::collections::HashMap;
use std::ptr;
use std::slice;

use super::*;
use remglk::GlkSystem;
use glkapi::protocol::{Event, SystemFileRef, Update};

extern "C" {
    fn emglken_fileref_exists(filename_ptr: *const u8, filename_len: usize) -> bool;
    fn emglken_fileref_read(filename_ptr: *const u8, filename_len: usize, buffer: &mut EmglkenBuffer) -> bool;
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
            let mut buf = EmglkenBuffer {
                ptr: ptr::null_mut(),
                len: 0,
            };
            let result = unsafe {emglken_fileref_read(fileref.filename.as_ptr(), fileref.filename.len(), &mut buf)};
            if result {
                return unsafe {Some(Box::from_raw(slice::from_raw_parts_mut(buf.ptr, buf.len)))};
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
        unimplemented!()
    }

    fn get_glkote_event(&mut self) -> Option<Event> {
        unimplemented!()
    }

    fn send_glkote_update(&mut self, _update: Update) {
        unimplemented!()
    }
}