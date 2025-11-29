/*

Glk FileRefs
============

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::ffi::CString;

use super::*;

pub type GlkFileRefShared = GlkObject<GlkFileRef>;
pub type GlkFileRefMetadata = GlkObjectMetadata<GlkFileRef>;

#[derive(Default)]
pub struct GlkFileRef {
    pub binary: bool,
    pub path: String,
    pub path_c: CString,
}

impl GlkFileRef {
    pub fn new(path: String, usage: u32) -> Self {
        let path_c = CString::new(&path[..]).unwrap();
        GlkFileRef {
            binary: (usage & fileusage_TextMode) == 0,
            path,
            path_c,
        }
    }
}

impl GlkObjectClass for GlkFileRef {
    fn get_object_class_id() -> u32 {
        2
    }
}