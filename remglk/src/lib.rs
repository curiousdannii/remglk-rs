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
use glkapi::protocol::SystemFileRef;

/** Glk's access to the operating system */
pub trait GlkSystem {
    // Fileref functions
    fn fileref_construct(filename: String, filetype: FileType, gameid: Option<String>) -> SystemFileRef;
    fn fileref_delete(fileref: &SystemFileRef);
    fn fileref_exists(fileref: &SystemFileRef) -> bool;
    fn fileref_read(fileref: &SystemFileRef) -> GlkResult<Box<[u8]>>;
    fn fileref_temporary(&mut self, filetype: FileType) -> SystemFileRef;
    fn fileref_write<'a>(&mut self, fileref: &SystemFileRef, buf: &[u8]) -> GlkResult<'a, ()>;
}