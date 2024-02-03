/*

Standard system
===============

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::env::temp_dir;
use std::fs;
use std::path::Path;

use super::*;
use remglk::GlkSystem;
use glkapi::protocol::{Event, SystemFileRef, Update};

#[derive(Default)]
pub struct StandardSystem {
    tempfile_counter: u32,
}

impl GlkSystem for StandardSystem {
    fn fileref_construct(&mut self, filename: String, filetype: FileType, gameid: Option<String>) -> SystemFileRef {
        SystemFileRef {
            filename,
            gameid,
            usage: Some(filetype),
            ..Default::default()
        }
    }

    fn fileref_delete(&mut self, fileref: &SystemFileRef) {
        let _ = fs::remove_file(Path::new(&fileref.filename));
    }

    fn fileref_exists(&mut self, fileref: &SystemFileRef) -> bool {
        Path::new(&fileref.filename).exists()
    }

    fn fileref_read(&mut self, fileref: &SystemFileRef) -> GlkResult<Box<[u8]>> {
        Ok(fs::read(&fileref.filename)?.into_boxed_slice())
    }

    fn fileref_temporary(&mut self, filetype: FileType) -> SystemFileRef {
        let filename = format!("remglktempfile-{}", self.tempfile_counter);
        self.tempfile_counter += 1;
        let path = temp_dir().join(filename);
        SystemFileRef {
            filename: path.to_str().unwrap().to_string(),
            usage: Some(filetype),
            ..Default::default()
        }
    }

    fn fileref_write(&mut self, fileref: &SystemFileRef, buf: GlkBuffer) -> GlkResult<()> {
        // TODO: caching
        match buf {
            GlkBuffer::U8(buf) => Ok(fs::write(&fileref.filename, buf)?),
            GlkBuffer::U32(buf) => Ok(fs::write(&fileref.filename, u32slice_to_u8vec(buf))?)
        }
    }

    fn send_glkote_update(&mut self, _update: Update) -> Event {
        unimplemented!()
    }
}