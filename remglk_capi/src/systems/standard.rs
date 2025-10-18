/*

Standard system
===============

Copyright (c) 2025 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use jiff::tz::TimeZone;

use super::*;
use remglk::GlkSystem;
use glkapi::protocol::{Event, Update};

pub type GlkApi = glkapi::GlkApi<StandardSystem>;

pub static GLKAPI: LazyLock<Mutex<GlkApi>> = LazyLock::new(|| {
    Mutex::new(GlkApi::new(StandardSystem::default()))
});

#[derive(Default)]
pub struct StandardSystem {
    cache: HashMap<String, Box<[u8]>>,
}

impl GlkSystem for StandardSystem {
    fn file_delete(&mut self, path: &str) {
        self.cache.remove(path);
        let _ = fs::remove_file(Path::new(path));
    }

    fn file_exists(&mut self, path: &str) -> bool {
        self.cache.contains_key(path) || Path::new(path).exists()
    }

    fn file_read(&mut self, path: &str) -> Option<Box<[u8]>> {
        // Check the cache first
        if let Some(buf) = self.cache.get(path) {
            Some(buf.clone())
        }
        else {
            fs::read(path).ok().map(|buf| buf.into_boxed_slice())
        }
    }

    fn file_write_buffer(&mut self, path: &str, buf: Box<[u8]>) {
        self.cache.insert(path.to_string(), buf);
    }

    fn flush_writeable_files(&mut self) {
        for (filename, buf) in self.cache.drain() {
            let _ = fs::write(filename, buf);
        }
        self.cache.shrink_to(4);
    }

    fn get_glkote_event(&mut self) -> Option<Event> {
        // Read a line from stdin
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let data = line.unwrap();
            if data.is_empty() {
                continue;
            }
            let event: Event = serde_json::from_str(&data).unwrap();
            return Some(event);
        }
        None
    }

    fn send_glkote_update(&mut self, update: Update) {
        // Send the update
        let output = serde_json::to_string(&update).unwrap();
        println!("{}", output);
    }

    fn get_directories() -> Directories {
        let cwd = env::current_dir().unwrap();
        Directories {
            storyfile: cwd.clone(),
            system_cwd: cwd.clone(),
            temp: env::temp_dir(),
            working: cwd,
        }
    }

    fn get_local_tz() -> TimeZone {
        TimeZone::system()
    }

    fn set_base_file(dirs: &mut Directories, path: String) {
        let mut path = PathBuf::from(path);
        path.pop();
        dirs.storyfile.clone_from(&path);
        dirs.working = path;
    }
}