/*

Array helpers
=============

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use super::*;

pub const MAX_LATIN1: u32 = 0xFF;
pub const QUESTION_MARK: u32 = '?' as u32;

/** A Glk buffer argument */
pub enum GlkBuffer<'a> {
    U8(&'a [u8]),
    U32(&'a [u32]),
}

/** A Glk mutable buffer argument */
pub enum GlkBufferMut<'a> {
    U8(&'a mut [u8]),
    U32(&'a mut [u32]),
}

/** An owned buffer that won't be known when the owning object is created, such as window text input buffers */
pub enum GlkOwnedBuffer {
    U8(Box<[u8]>),
    U32(Box<[u32]>),
}

impl GlkBuffer<'_> {
    pub fn len(&self) -> usize {
        match self {
            GlkBuffer::U8(buf) => buf.len(),
            GlkBuffer::U32(buf) => buf.len(),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            GlkBuffer::U8(buf) => u8slice_to_string(buf),
            GlkBuffer::U32(buf) => u32slice_to_string(buf),
        }
    }
}

impl GlkBufferMut<'_> {
    pub fn len(&self) -> usize {
        match self {
            GlkBufferMut::U8(buf) => buf.len(),
            GlkBufferMut::U32(buf) => buf.len(),
        }
    }

    pub fn set_u32(&mut self, index: usize, val: u32) {
        match self {
            GlkBufferMut::U8(buf) => buf[index] = if val > MAX_LATIN1 {QUESTION_MARK} else {val} as u8,
            GlkBufferMut::U32(buf) => buf[index] = val,
        }
    }
}

impl GlkOwnedBuffer {
    pub fn len(&self) -> usize {
        match self {
            GlkOwnedBuffer::U8(buf) => buf.len(),
            GlkOwnedBuffer::U32(buf) => buf.len(),
        }
    }

    pub fn to_string(&self, len: usize) -> String {
        match self {
            GlkOwnedBuffer::U8(buf) => u8slice_to_string(&buf[..len]),
            GlkOwnedBuffer::U32(buf) => u32slice_to_string(&buf[..len]),
        }
    }
}

impl From<&str> for GlkOwnedBuffer {
    fn from(value: &str) -> Self {
        GlkOwnedBuffer::U32(str_to_u32vec(value).into_boxed_slice())
    }
}

impl<'a> From<&'a GlkOwnedBuffer> for GlkBuffer<'a> {
    fn from(value: &'a GlkOwnedBuffer) -> Self {
        match value {
            GlkOwnedBuffer::U8(ref buf) => GlkBuffer::U8(buf.as_ref()),
            GlkOwnedBuffer::U32(ref buf) => GlkBuffer::U32(buf.as_ref()),
        }
    }
}

impl<'a> From<&'a mut GlkOwnedBuffer> for GlkBufferMut<'a> {
    fn from(value: &'a mut GlkOwnedBuffer) -> Self {
        match value {
            GlkOwnedBuffer::U8(ref mut buf) => GlkBufferMut::U8(buf.as_mut()),
            GlkOwnedBuffer::U32(ref mut buf) => GlkBufferMut::U32(buf.as_mut()),
        }
    }
}

/** Copy a slice into this slice, both must be long enough, starting from their respective indices, to contain the length of the slice */
pub fn set_buffer(src: &GlkBuffer, src_offset: usize, dest: &mut GlkBufferMut, dest_offset: usize, len: usize) {
    match (src, dest) {
        (GlkBuffer::U8(src), GlkBufferMut::U8(dest)) => dest[dest_offset..(dest_offset + len)].copy_from_slice(&src[src_offset..(src_offset + len)]),
        (GlkBuffer::U8(src), GlkBufferMut::U32(dest)) => {
            for (&value, target) in src[src_offset..(src_offset + len)].iter().zip(dest[dest_offset..(dest_offset + len)].iter_mut()) {
                *target = value as u32;
            }
        },
        (GlkBuffer::U32(src), GlkBufferMut::U8(dest)) => {
            for (&value, target) in src[src_offset..(src_offset + len)].iter().zip(dest[dest_offset..(dest_offset + len)].iter_mut()) {
                *target = if value > MAX_LATIN1 {QUESTION_MARK} else {value} as u8;
            }
        },
        (GlkBuffer::U32(src), GlkBufferMut::U32(dest)) => dest[dest_offset..(dest_offset + len)].copy_from_slice(&src[src_offset..(src_offset + len)]),
    }
}

/** Helper functions for Glk arrays (meaning boxed slices) */
pub trait GlkArray {
    fn copy_from_buffer(&mut self, self_offset: usize, buf: &GlkBuffer, buf_offset: usize, len: usize);
    fn copy_to_buffer(&self, self_offset: usize, buf: &mut GlkBufferMut, buf_offset: usize, len: usize);
    fn get_u32(&self, index: usize) -> u32;
    fn set_u32(&mut self, index: usize, val: u32);
}

impl GlkArray for Box<[u8]> {
    fn copy_from_buffer(&mut self, self_offset: usize, buf: &GlkBuffer, buf_offset: usize, len: usize) {
        set_buffer(buf, buf_offset, &mut GlkBufferMut::U8(self), self_offset, len);
    }

    fn copy_to_buffer(&self, self_offset: usize, buf: &mut GlkBufferMut, buf_offset: usize, len: usize) {
        set_buffer(&GlkBuffer::U8(self), self_offset, buf, buf_offset, len);
    }

    fn get_u32(&self, index: usize) -> u32 {
        self[index] as u32
    }

    fn set_u32(&mut self, index: usize, val: u32) {
        self[index] = if val > MAX_LATIN1 {QUESTION_MARK} else {val} as u8;
    }
}

impl GlkArray for Box<[u32]> {
    fn copy_from_buffer(&mut self, self_offset: usize, buf: &GlkBuffer, buf_offset: usize, len: usize) {
        set_buffer(buf, buf_offset, &mut GlkBufferMut::U32(self), self_offset, len);
    }

    fn copy_to_buffer(&self, self_offset: usize, buf: &mut GlkBufferMut, buf_offset: usize, len: usize) {
        set_buffer(&GlkBuffer::U32(self), self_offset, buf, buf_offset, len);
    }

    fn get_u32(&self, index: usize) -> u32 {
        self[index]
    }

    fn set_u32(&mut self, index: usize, val: u32) {
        self[index] = val;
    }
}