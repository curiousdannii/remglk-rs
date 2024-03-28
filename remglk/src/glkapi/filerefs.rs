/*

Glk FileRefs
============

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::ffi::CString;

use super::*;

pub type GlkFileRef = GlkObject<FileRef>;

#[derive(Default)]
pub struct FileRef {
    pub binary: bool,
    pub path: String,
    pub path_c: CString,
}

impl FileRef {
    pub fn new(path: String, usage: u32) -> Self {
        let path_c = CString::new(&path[..]).unwrap();
        FileRef {
            binary: (usage & fileusage_TextMode) == 0,
            path,
            path_c,
        }
    }
}

impl GlkObjectClass for FileRef {
    fn get_object_class_id() -> u32 {
        2
    }
}