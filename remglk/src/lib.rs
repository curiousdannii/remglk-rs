/*

RemGlk ported to Rust
=====================

Copyright (c) 2025 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

pub mod blorb;
pub mod glkapi;

use jiff::{Timestamp, tz::TimeZone};

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

    // Unicode functions
    fn buffer_canon_decompose(buf: &mut [u32], initlen: usize) -> usize;
    fn buffer_canon_normalize(buf: &mut [u32], initlen: usize) -> usize;
    fn buffer_to_lower_case(buf: &mut [u32], initlen: usize) -> usize;
    fn buffer_to_title_case(buf: &mut [u32], initlen: usize, lowerrest: bool) -> usize;
    fn buffer_to_upper_case(buf: &mut [u32], initlen: usize) -> usize;

    // Misc system functions
    fn get_directories() -> Directories;
    fn get_local_tz() -> TimeZone;
    fn get_now() -> Timestamp;
    fn set_base_file(dirs: &mut Directories, path: String);
}