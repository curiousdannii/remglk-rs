/*

The Glk API
===========

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

pub mod arrays;
pub mod common;
pub mod constants;
pub mod filerefs;
pub mod macros;
pub mod objects;
pub mod protocol;
pub mod streams;

use std::num::NonZeroU32;

use arrays::*;
use common::*;
use GlkApiError::*;
use constants::*;
use macros::*;
use objects::*;
use streams::*;

pub struct GlkApi {
    streams: GlkObjectStore<Stream>,
    current_stream: Option<NonZeroU32>,
}

impl GlkApi {
    pub fn new() -> Self {
        GlkApi {
            streams: GlkObjectStore::new(),
            current_stream: None,
        }
    }

    pub fn glk_get_buffer_stream(&mut self, str_id: Option<NonZeroU32>, buf: Vec<u8>) -> GlkResult<u32> {
        stream_op!(self, str_id, |str: &mut Stream| str.get_buffer(&mut GlkArray::U8(buf)))
    }

    pub fn glk_get_buffer_stream_uni(&mut self, str_id: Option<NonZeroU32>, buf: Vec<u32>) -> GlkResult<u32> {
        stream_op!(self, str_id, |str: &mut Stream| str.get_buffer(&mut GlkArray::U32(buf)))
    }

    pub fn glk_get_char_stream(&mut self, str_id: Option<NonZeroU32>) -> GlkResult<i32> {
        stream_op!(self, str_id, |str: &mut Stream| str.get_char(false))
    }

    pub fn glk_get_char_stream_uni(&mut self, str_id: Option<NonZeroU32>) -> GlkResult<i32> {
        stream_op!(self, str_id, |str: &mut Stream| str.get_char(true))
    }

    pub fn glk_get_line_stream(&mut self, str_id: Option<NonZeroU32>, buf: Vec<u8>) -> GlkResult<u32> {
        stream_op!(self, str_id, |str: &mut Stream| str.get_line(&mut GlkArray::U8(buf)))
    }

    pub fn glk_get_line_stream_uni(&mut self, str_id: Option<NonZeroU32>, buf: Vec<u32>) -> GlkResult<u32> {
        stream_op!(self, str_id, |str: &mut Stream| str.get_line(&mut GlkArray::U32(buf)))
    }

    pub fn glk_put_buffer(&mut self, buf: Vec<u8>) -> GlkResult<()> {
        stream_op!(self, self.current_stream, |str: &mut Stream| str.put_buffer(&GlkArray::U8(buf)))
    }

    pub fn glk_put_buffer_stream(&mut self, str_id: Option<NonZeroU32>, buf: Vec<u8>) -> GlkResult<()> {
        stream_op!(self, str_id, |str: &mut Stream| str.put_buffer(&GlkArray::U8(buf)))
    }

    pub fn glk_put_buffer_stream_uni(&mut self, str_id: Option<NonZeroU32>, buf: Vec<u32>) -> GlkResult<()> {
        stream_op!(self, str_id, |str: &mut Stream| str.put_buffer(&GlkArray::U32(buf)))
    }

    pub fn glk_put_buffer_uni(&mut self, buf: Vec<u32>) -> GlkResult<()> {
        stream_op!(self, self.current_stream, |str: &mut Stream| str.put_buffer(&GlkArray::U32(buf)))
    }

    pub fn glk_put_char(&mut self, ch: u8) -> GlkResult<()> {
        stream_op!(self, self.current_stream, |str: &mut Stream| str.put_char(ch as u32))
    }

    pub fn glk_put_char_stream(&mut self, str_id: Option<NonZeroU32>, ch: u8) -> GlkResult<()> {
        stream_op!(self, str_id, |str: &mut Stream| str.put_char(ch as u32))
    }

    pub fn glk_put_char_stream_uni(&mut self, str_id: Option<NonZeroU32>, ch: u32) -> GlkResult<()> {
        stream_op!(self, str_id, |str: &mut Stream| str.put_char(ch))
    }

    pub fn glk_put_char_uni(&mut self, ch: u32) -> GlkResult<()> {
        stream_op!(self, self.current_stream, |str: &mut Stream| str.put_char(ch))
    }

    pub fn glk_stream_close(&mut self, str_id: Option<NonZeroU32>) -> GlkResult<StreamResultCounts> {
        stream_op!(self, str_id, |str: &mut Stream| str.close())
    }

    pub fn glk_stream_get_current(&self) -> Option<NonZeroU32> {
        self.current_stream
    }

    pub fn glk_stream_get_position(&self, str_id: Option<NonZeroU32>) -> GlkResult<u32> {
        let str = self.streams.get(str_id)
            .ok_or(InvalidReference)?;
        Ok(str.get_position())
    }

    pub fn glk_stream_get_rock(&self, str_id: Option<NonZeroU32>) -> GlkResult<u32> {
        self.streams.get_rock(str_id).ok_or(InvalidReference)
    }

    pub fn glk_stream_iterate(&self, str_id: Option<NonZeroU32>) -> Option<IterationResult> {
        self.streams.iterate(str_id)
    }

    pub fn glk_stream_open_memory(&mut self, buf: Vec<u8>, fmode: FileMode, rock: u32) -> GlkResult<NonZeroU32> {
        self.create_memory_stream(GlkArray::U8(buf), fmode, rock)
    }

    pub fn glk_stream_open_memory_uni(&mut self, buf: Vec<u32>, fmode: FileMode, rock: u32) -> GlkResult<NonZeroU32> {
        self.create_memory_stream(GlkArray::U32(buf), fmode, rock)
    }

    pub fn glk_stream_set_current(&mut self, str_id: Option<NonZeroU32>) {
        self.current_stream = str_id;
    }

    pub fn glk_stream_set_position(&mut self, str_id: Option<NonZeroU32>, mode: SeekMode, pos: i32) -> GlkResult<()> {
        let str = self.streams.get_mut(str_id)
            .ok_or(InvalidReference)?;
        Ok(str.set_position(mode, pos))
    }

    fn create_memory_stream(&mut self, buf: GlkArray, fmode: FileMode, rock: u32) -> GlkResult<NonZeroU32> {
        if fmode == FileMode::WriteAppend {
            return Err(IllegalFilemode);
        }
        let str: Stream = if buf.len() == 0 {
            NullStream::new().into()
        }
        else {
            ArrayBackedStream::new(buf, fmode, None).into()
        };
        Ok(self.streams.register(str, rock))
    }
}