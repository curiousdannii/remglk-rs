/*

Standard system
===============

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::collections::HashMap;
use std::env::temp_dir;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

use super::*;
use remglk::GlkSystem;
use glkapi::protocol::{Event, SystemFileRef, Update};

#[derive(Default)]
pub struct StandardSystem {
    cache: HashMap<String, Box<[u8]>>,
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
        self.cache.remove(&fileref.filename);
        let _ = fs::remove_file(Path::new(&fileref.filename));
    }

    fn fileref_exists(&mut self, fileref: &SystemFileRef) -> bool {
        self.cache.contains_key(&fileref.filename) || Path::new(&fileref.filename).exists()
    }

    fn fileref_read(&mut self, fileref: &SystemFileRef) -> Option<Box<[u8]>> {
        // Check the cache first
        if let Some(buf) = self.cache.get(&fileref.filename) {
            Some(buf.clone())
        }
        else {
            fs::read(&fileref.filename).ok().map(|buf| buf.into_boxed_slice())
        }
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

    fn fileref_write_buffer(&mut self, fileref: &SystemFileRef, buf: Box<[u8]>) {
        self.cache.insert(fileref.filename.clone(), buf);
    }

    fn flush_writeable_files(&mut self) {
        for (filename, buf) in self.cache.drain() {
            let _ = fs::write(filename, buf);
        }
        self.cache.shrink_to(4);
    }

    fn get_glkote_event(&mut self) -> Event {
        // Read a line from stdin
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let data = line.unwrap();
            if data.is_empty() {
                continue;
            }
            let event: Event = serde_json::from_str(&data).unwrap();
            return event;
        }
        unreachable!()
    }

    fn send_glkote_update(&mut self, update: Update) {
        // Send the update
        let output = serde_json::to_string(&update).unwrap();
        println!("{}", output);
    }
}