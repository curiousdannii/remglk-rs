/*

Array helpers
=============

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

pub const MAX_LATIN1: u32 = 0xFF;
pub const QUESTION_MARK: u32 = '?' as u32;

/** A Glk array is a u8/u32 owned Vec */
pub enum GlkArray {
    U8(Vec<u8>),
    U32(Vec<u32>),
}

impl GlkArray {
    pub fn get_u32(&self, index: usize) -> u32 {
        match self {
            GlkArray::U8(buf) => buf[index] as u32,
            GlkArray::U32(buf) => buf[index],
        }
    }

    pub fn len(&self) -> usize {
        match self {
            GlkArray::U8(buf) => buf.len(),
            GlkArray::U32(buf) => buf.len(),
        }
    }

    pub fn resize(&mut self, new_len: usize) {
        match self {
            GlkArray::U8(buf) => buf.resize(new_len, 0),
            GlkArray::U32(buf) => buf.resize(new_len, 0),
        }
    }

    /** Copy a slice into this slice, both must be long enough, starting from their respective indices, to contain the length of the slice */
    pub fn set_buffer(&mut self, start: usize, src: &GlkArray, src_start: usize, len: usize) {
        match (self, src) {
            (GlkArray::U8(dest), GlkArray::U8(src)) => dest[start..(start + len)].copy_from_slice(&src[src_start..(src_start + len)]),
            (GlkArray::U8(dest), GlkArray::U32(src)) => {
                for (&value, target) in src[src_start..(src_start + len)].iter().zip(dest[start..(start + len)].iter_mut()) {
                    *target = if value > MAX_LATIN1 {QUESTION_MARK} else {value} as u8;
                }
            },
            (GlkArray::U32(dest), GlkArray::U8(src)) => {
                for (&value, target) in src[src_start..(src_start + len)].iter().zip(dest[start..(start + len)].iter_mut()) {
                    *target = value as u32;
                }
            },
            (GlkArray::U32(dest), GlkArray::U32(src)) => dest[start..(start + len)].copy_from_slice(&src[src_start..(src_start + len)]),
        }
    }

    pub fn set_u32(&mut self, index: usize, val: u32) {
        match self {
            GlkArray::U8(buf) => buf[index] = if val > MAX_LATIN1 {QUESTION_MARK} else {val} as u8,
            GlkArray::U32(buf) => buf[index] = val,
        }
    }
}