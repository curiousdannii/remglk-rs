/*

RemGlk ported to Rust
=====================

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

pub mod blorb;
pub mod glkapi;

use glkapi::Directories;
use glkapi::protocol::{Event, Update};

/** Glk's access to the operating system */
pub trait GlkSystem {
    // File functions
    fn file_delete(&mut self, path: &str);
    fn file_exists(&mut self, path: &str) -> bool;
    fn file_read(&mut self, path: &str) -> Option<Box<[u8]>>;
    fn file_write_buffer(&mut self, path: &str, buf: Box<[u8]>);
    fn flush_writeable_files(&mut self);

    /** Send an update to GlkOte */
    fn send_glkote_update(&mut self, update: Update);
    /** Get an event from GlkOte */
    fn get_glkote_event(&mut self) -> Option<Event>;

    fn get_directories() -> Directories;
    fn set_base_file(dirs: &mut Directories, path: String);
}