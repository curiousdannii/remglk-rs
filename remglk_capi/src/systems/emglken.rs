/*

Emglken system
==============

Copyright (c) 2025 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::path::PathBuf;
use std::slice;
use std::sync::LazyLock;

use jiff::tz::{Offset, TimeZone};
use serde::Deserialize;
use serde::de::DeserializeOwned;

use super::*;
use remglk::GlkSystem;
use glkapi::protocol::{Event, Update};

extern "C" {
    fn emglken_buffer_canon_decompose(buffer_ptr: *mut u32, buffer_len: usize, initlen: usize) -> usize;
    fn emglken_buffer_canon_normalize(buffer_ptr: *mut u32, buffer_len: usize, initlen: usize) -> usize;
    fn emglken_buffer_to_lower_case(buffer_ptr: *mut u32, buffer_len: usize, initlen: usize) -> usize;
    fn emglken_buffer_to_title_case(buffer_ptr: *mut u32, buffer_len: usize, initlen: usize, lowerrest: bool) -> usize;
    fn emglken_buffer_to_upper_case(buffer_ptr: *mut u32, buffer_len: usize, initlen: usize) -> usize;
    fn emglken_file_delete(path_ptr: *const u8, path_len: usize);
    fn emglken_file_exists(path_ptr: *const u8, path_len: usize) -> bool;
    fn emglken_file_flush();
    fn emglken_file_read(path_ptr: *const u8, path_len: usize, buffer: *mut EmglkenBuffer) -> bool;
    fn emglken_file_write_buffer(path_ptr: *const u8, path_len: usize, buf_ptr: *const u8, buf_len: usize);
    fn emglken_get_dirs(buffer: *mut EmglkenBuffer);
    fn emglken_get_glkote_event(buffer: *mut EmglkenBuffer);
    fn emglken_get_local_tz() -> i32;
    fn emglken_send_glkote_update(update_ptr: *const u8, update_len: usize);
    fn emglken_set_storyfile_dir(path_ptr: *const u8, path_len: usize, buffer: *mut EmglkenBuffer);
}

pub type GlkApi = glkapi::GlkApi<EmglkenSystem>;

pub static GLKAPI: LazyLock<Mutex<GlkApi>> = LazyLock::new(|| {
    Mutex::new(GlkApi::new(EmglkenSystem::default()))
});

#[derive(Default)]
pub struct EmglkenSystem {
    cache: HashMap<String, Box<[u8]>>,
}

impl GlkSystem for EmglkenSystem {
    fn file_delete(&mut self, path: &str) {
        self.cache.remove(path);
        unsafe {emglken_file_delete(path.as_ptr(), path.len())};
    }

    fn file_exists(&mut self, path: &str) -> bool {
        self.cache.contains_key(path) || {
            unsafe {emglken_file_exists(path.as_ptr(), path.len())}
        }
    }

    fn file_read(&mut self, path: &str) -> Option<Box<[u8]>> {
        // Check the cache first
        if let Some(buf) = self.cache.get(path) {
            Some(buf.clone())
        }
        else {
            let mut buf: MaybeUninit<EmglkenBuffer> = MaybeUninit::uninit();
            let result = unsafe {emglken_file_read(path.as_ptr(), path.len(), buf.as_mut_ptr())};
            if result {
                return Some(buffer_to_boxed_slice(buf));
            }
            None
        }
    }

    fn file_write_buffer(&mut self, path: &str, buf: Box<[u8]>) {
        self.cache.insert(path.to_string(), buf);
    }

    fn flush_writeable_files(&mut self) {
        for (path, buf) in &self.cache {
            unsafe {emglken_file_write_buffer(path.as_ptr(), path.len(), buf.as_ptr(), buf.len())};
        }
        // Signal we've written all the files
        unsafe {emglken_file_flush()};
        self.cache.clear();
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

    fn buffer_canon_decompose(buf: &mut [u32], initlen: usize) -> usize {
        unsafe {emglken_buffer_canon_decompose(buf.as_mut_ptr(), buf.len(), initlen)}
    }

    fn buffer_canon_normalize(buf: &mut [u32], initlen: usize) -> usize {
        unsafe {emglken_buffer_canon_normalize(buf.as_mut_ptr(), buf.len(), initlen)}
    }

    fn buffer_to_lower_case(buf: &mut [u32], initlen: usize) -> usize {
        unsafe {emglken_buffer_to_lower_case(buf.as_mut_ptr(), buf.len(), initlen)}
    }

    fn buffer_to_title_case(buf: &mut [u32], initlen: usize, lowerrest: bool) -> usize {
        unsafe {emglken_buffer_to_title_case(buf.as_mut_ptr(), buf.len(), initlen, lowerrest)}
    }

    fn buffer_to_upper_case(buf: &mut [u32], initlen: usize) -> usize {
        unsafe {emglken_buffer_to_upper_case(buf.as_mut_ptr(), buf.len(), initlen)}
    }

    fn get_directories() -> Directories {
        let mut buf: MaybeUninit<EmglkenBuffer> = MaybeUninit::uninit();
        unsafe {emglken_get_dirs(buf.as_mut_ptr())};
        let dirs: EmglkenDirectories = buffer_to_protocol_struct(buf);
        Directories {
            storyfile: PathBuf::from(dirs.storyfile),
            system_cwd: PathBuf::from(dirs.system_cwd),
            temp: PathBuf::from(dirs.temp),
            working: PathBuf::from(dirs.working),
        }
    }

    fn get_local_tz() -> TimeZone {
        let offset = Offset::from_seconds(unsafe {emglken_get_local_tz()}).unwrap();
        TimeZone::fixed(offset)
    }

    fn set_base_file(dirs: &mut Directories, path: String) {
        let mut path = PathBuf::from(path);
        path.pop();
        let path = path.to_str().unwrap();
        let mut buf: MaybeUninit<EmglkenBuffer> = MaybeUninit::uninit();
        unsafe {emglken_set_storyfile_dir(path.as_ptr(), path.len(), buf.as_mut_ptr())};
        let emglken_dirs: EmglkenSetBaseFileDirectories = buffer_to_protocol_struct(buf);
        if let Some(path) = emglken_dirs.storyfile {
            dirs.storyfile = PathBuf::from(path);
        }
        if let Some(path) = emglken_dirs.working {
            dirs.working = PathBuf::from(path);
        }
    }
}

#[repr(C)]
pub struct EmglkenBuffer {
    pub ptr: *mut u8,
    pub len: usize,
}

#[derive(Deserialize)]
struct EmglkenDirectories {
    pub storyfile: String,
    pub system_cwd: String,
    pub temp: String,
    pub working: String,
}

#[derive(Deserialize)]
struct EmglkenSetBaseFileDirectories {
    pub storyfile: Option<String>,
    pub working: Option<String>,
}

fn buffer_to_boxed_slice(buffer: MaybeUninit<EmglkenBuffer>) -> Box<[u8]> {
    let buffer = unsafe {buffer.assume_init()};
    unsafe {Box::from_raw(slice::from_raw_parts_mut(buffer.ptr, buffer.len))}
}

fn buffer_to_protocol_struct<T: DeserializeOwned>(buffer: MaybeUninit<EmglkenBuffer>) -> T {
    let data = buffer_to_boxed_slice(buffer);
    serde_json::from_slice(&data).unwrap()
}