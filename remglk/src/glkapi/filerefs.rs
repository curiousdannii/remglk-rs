/*

Glk FileRefs
============

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use super::*;

pub type GlkFileRef = GlkObject<FileRef>;

#[derive(Default)]
pub struct FileRef {
    pub binary: bool,
    pub path: String,
}

impl FileRef {
    pub fn new(path: String, usage: u32) -> Self {
        FileRef {
            binary: (usage & fileusage_TextMode) == 0,
            path,
        }
    }
}

impl GlkObjectClass for FileRef {
    fn get_object_class_id() -> u32 {
        2
    }
}