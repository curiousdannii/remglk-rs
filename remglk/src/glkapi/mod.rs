/*

The Glk API
===========

Copyright (c) 2023 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

pub mod constants;
pub mod filerefs;
pub mod protocol;
pub mod streams;

pub const MAX_LATIN1: u32 = 0xFF;
pub const QUESTION_MARK: u32 = '?' as u32;

/** A Glk object that will be return to the Glk user */
pub struct GlkObject<T> {
    pub disprock: Option<u32>,
    pub inner: T,
    pub rock: u32,
}

pub trait GlkInt {}
impl GlkInt for u8 {}
impl GlkInt for u32 {}

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

    pub fn is_uni(&self) -> bool {
        match self {
            U8(_) => false,
            U32(_) => true,
        }
    }

    /** Copy an array into this array */
    pub fn set_slice(&mut self, src: &GlkArray, src_start: usize, dest_start: usize, len: usize) {
        let src_end = src_start + len;
        let dest_end = dest_start + len;
        match (src, self) {
            (U8(src), U8(dest)) => dest[dest_start..dest_end].copy_from_slice(&src[src_start..src_end]),
            (U32(src), U32(dest)) => dest[dest_start..dest_end].copy_from_slice(&src[src_start..src_end]),
            (U8(src), U32(dest)) => {
                for (&value, target) in src[src_start..src_end].iter().zip(&mut dest[dest_start..dest_end]) {
                    *target = value as u32;
                }
            },
            // When a unicode array is read into a Latin-1 array we must catch non-latin1 characters
            (U32(src), U8(dest)) => {
                for (&value, target) in src[src_start..src_end].iter().zip(&mut dest[dest_start..dest_end]) {
                    *target = if value > MAX_LATIN1 {QUESTION_MARK} else {value} as u8;
                }
            },
        }
    }

    pub fn set_u32(&mut self, index: usize, ch: u32) {
        match self {
            U8(arr) => arr[index] = if ch > MAX_LATIN1 {QUESTION_MARK} else {ch} as u8,
            U32(arr) => arr[index] = ch,
        };
    }
}

// /** An owned Glk array */
/*pub enum GlkArrayBuf {
    U8(Vec<u8>),
    U32(Vec<u32>),
}

impl Deref for GlkArrayBuf {
    type Target = GlkArray<'a>;
    #[inline]
    fn deref(&'a self) -> &GlkArray {
        Path::new(&self.inner)
    }
}*/