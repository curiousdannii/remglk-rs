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

/** A Glk object that will be returned to the main app */
pub struct GlkObject<T> {
    pub disprock: Option<u32>,
    pub inner: T,
    pub rock: u32,
}

/** Helper functions for Glk arrays */
pub trait GlkArray {
    fn get_u32(&self, index: usize) -> u32;
    fn set_u32(&mut self, index: usize, val: u32);
}

impl GlkArray for [u8] {
    fn get_u32(&self, index: usize) -> u32 {
        self[index] as u32
    }

    fn set_u32(&mut self, index: usize, val: u32) {
        self[index] = if val > MAX_LATIN1 {QUESTION_MARK} else {val} as u8;
    }
}

impl GlkArray for [u32] {
    fn get_u32(&self, index: usize) -> u32 {
        self[index]
    }

    fn set_u32(&mut self, index: usize, val: u32) {
        self[index] = val;
    }
}

/** Handle converting between Glk Latin1 and Unicode arrays */
pub trait SetGlkBuffer<Src=Self>
where
    Src: ?Sized
{
    /** Copy a slice into this slice, must be the same length */
    fn set_buffer(&mut self, src: &Src);
}

impl SetGlkBuffer for [u8] {
    fn set_buffer(&mut self, src: &[u8]) {
        self.copy_from_slice(src);
    }
}

impl SetGlkBuffer<[u32]> for [u8] {
    fn set_buffer(&mut self, src: &[u32]) {
        for (&value, target) in src.iter().zip(self) {
            *target = if value > MAX_LATIN1 {QUESTION_MARK} else {value} as u8;
        }
    }
}

impl SetGlkBuffer for [u32] {
    fn set_buffer(&mut self, src: &[u32]) {
        self.copy_from_slice(&src);
    }
}

impl SetGlkBuffer<[u8]> for [u32] {
    fn set_buffer(&mut self, src: &[u8]) {
        for (&value, target) in src.iter().zip(self) {
            *target = value as u32;
        }
    }
}