/*

Glk FileRefs
============

Copyright (c) 2023 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use super::*;

pub struct FileRef {}

impl FileRef {
    pub fn delete_file() {}

    pub fn exists() -> bool {
        false
    }

    pub fn read() -> Option<Vec<u8>> {
        None
    }

    pub fn write(_: &GlkArray) {}
}