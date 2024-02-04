/*

RemGlk ported to Rust
=====================

Copyright (c) 2022 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

#![forbid(unsafe_code)]

pub mod glkapi;

use glkapi::*;
use glkapi::constants::*;
use glkapi::protocol::{Event, SystemFileRef, Update};

/** Glk's access to the operating system */
pub trait GlkSystem {
    // Fileref functions
    fn fileref_construct(&mut self, filename: String, filetype: FileType, gameid: Option<String>) -> SystemFileRef;
    fn fileref_delete(&mut self, fileref: &SystemFileRef);
    fn fileref_exists(&mut self, fileref: &SystemFileRef) -> bool;
    fn fileref_read(&mut self, fileref: &SystemFileRef) -> GlkResult<Box<[u8]>>;
    fn fileref_temporary(&mut self, filetype: FileType) -> SystemFileRef;
    fn fileref_write(&mut self, fileref: &SystemFileRef, buf: GlkBuffer) -> GlkResult<()>;

    /** Send an update to GlkOte */
    fn send_glkote_update(&mut self, update: Update);
    /** Get an event from GlkOte */
    fn get_glkote_event(&mut self) -> Event;
}