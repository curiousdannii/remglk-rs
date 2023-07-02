/*

Array helpers
=============

Copyright (c) 2023 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::ops::{Deref, DerefMut};

pub const MAX_LATIN1: u32 = 0xFF;
pub const QUESTION_MARK: u32 = '?' as u32;

/** For when you can take either a slice or a vec */
pub enum DataSource<'a, T> {
    Borrowed(&'a mut [T]),
    Owned(Vec<T>),
}

impl<T> Deref for DataSource<'_, T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        match self {
            DataSource::Borrowed(val) => val,
            DataSource::Owned(val) => val,
        }
    }
}

impl<T> DerefMut for DataSource<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            DataSource::Borrowed(val) => val,
            DataSource::Owned(val) => val,
        }
    }
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