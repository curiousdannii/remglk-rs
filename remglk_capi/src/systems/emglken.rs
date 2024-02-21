/*

Emglken system
==============

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::collections::HashMap;

//use emscripten_em_js::em_js;

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
    _cache: HashMap<String, Box<[u8]>>,
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

    fn fileref_exists(&mut self, _fileref: &SystemFileRef) -> bool {
        unimplemented!()
    }

    fn fileref_read(&mut self, _fileref: &SystemFileRef) -> Option<Box<[u8]>> {
        unimplemented!()
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