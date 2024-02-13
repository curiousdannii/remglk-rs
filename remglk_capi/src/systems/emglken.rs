/*

Emglken system
==============

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::collections::HashMap;
use std::env::temp_dir;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

use emscripten_em_js::em_js;

use super::*;
use remglk::GlkSystem;
use glkapi::protocol::{Event, SystemFileRef, Update};

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
    tempfile_counter: u32,
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
        unimplemented!()
    }

    fn fileref_exists(&mut self, fileref: &SystemFileRef) -> bool {
        unimplemented!()
    }

    fn fileref_read(&mut self, fileref: &SystemFileRef) -> Option<Box<[u8]>> {
        unimplemented!()
    }

    fn fileref_temporary(&mut self, filetype: FileType) -> SystemFileRef {
        unimplemented!()
    }

    fn fileref_write_buffer(&mut self, fileref: &SystemFileRef, buf: Box<[u8]>) {
        unimplemented!()
    }

    fn flush_writeable_files(&mut self) {
        unimplemented!()
    }

    fn get_glkote_event(&mut self) -> Event {
        unimplemented!()
    }

    fn send_glkote_update(&mut self, update: Update) {
        unimplemented!()
    }
}