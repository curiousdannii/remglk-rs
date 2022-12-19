/*

The Glk API
===========

Copyright (c) 2022 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::cmp::min;

pub mod constants;
pub mod protocol;
pub mod streams;

pub const MAX_LATIN1: u32 = 0xFF;
pub const QUESTION_MARK: u32 = '?' as u32;

/** A Glk array (actually a slice) */
pub enum GlkArray<'a> {
    U8(&'a mut [u8]),
    U32(&'a mut [u32]),
}
use GlkArray::*;

impl GlkArray<'_> {
    pub fn len(&self) -> usize {
        match self {
            U8(arr) => arr.len(),
            U32(arr) => arr.len(),
        }
    }

    pub fn get_u32(&self, i: usize) -> u32 {
        match self {
            U8(arr) => arr[i] as u32,
            U32(arr) => arr[i],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /** Copy an array into this array */
    pub fn set_slice(&mut self, src: &GlkArray, src_start: usize, dest_start: usize, len: usize) {
        let src_end = src_start + len;
        let dest_end = dest_start + len;
        match (src, self) {
            (U8(src), U8(dest)) => dest[dest_start..dest_end].copy_from_slice(&src[src_start..src_end]),
            (U32(src), U32(dest)) => dest[dest_start..dest_end].copy_from_slice(&src[src_start..src_end]),
            (U8(src), U32(dest)) => {
                for (i, &ch) in src[src_start..src_end].iter().enumerate() {
                    dest[dest_start + i] = ch as u32;
                }
            },
            // When a unicode array is read into a Latin-1 array we must catch non-latin1 characters
            (U32(src), U8(dest)) => {
                for (i, &ch) in src[src_start..src_end].iter().enumerate() {
                    dest[dest_start + i] = if ch > MAX_LATIN1 {QUESTION_MARK} else {ch} as u8;
                }
            },
        }
    }

    pub fn set_str(&mut self, start: usize, str: &str) -> (usize, usize) {
        let chars: Vec<char> = str.chars().collect();
        let str_length = chars.len();
        let write_length = min(str_length, self.len() - start);
        match self {
            U8(arr) => {
                for (i, &ch) in chars[..write_length].iter().enumerate() {
                    arr[start + i] = if ch as u32 > MAX_LATIN1 {QUESTION_MARK} else {ch as u32} as u8;
                }
            },
            U32(arr) => {
                for (i, &ch) in chars[..write_length].iter().enumerate() {
                    arr[start + i] = ch as u32;
                }
            }
        }
        (str_length, write_length)
    }

    pub fn set_u32(&mut self, index: usize, ch: u32) {
        match self {
            U8(arr) => arr[index] = if ch > MAX_LATIN1 {QUESTION_MARK} else {ch} as u8,
            U32(arr) => arr[index] = ch,
        };
    }

    pub fn uni(&self) -> bool {
        match self {
            U8(_) => false,
            U32(_) => true,
        }
    }
}

/** Final read/write character counts of a stream */
pub struct StreamResult {
    pub read_count: u32,
    pub write_count: u32,
}