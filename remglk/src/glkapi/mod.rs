/*

The Glk API
===========

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

mod arrays;
mod common;
pub mod constants;
mod filerefs;
mod macros;
mod objects;
mod protocol;
mod streams;
mod windows;

use std::num::NonZeroU32;

use arrays::*;
use common::*;
use GlkApiError::*;
use constants::*;
use macros::*;
use objects::*;
use streams::*;
use windows::*;

pub struct GlkApi {
    streams: GlkObjectStore<Stream>,
    current_stream: Option<NonZeroU32>,
    windows: GlkObjectStore<Window>,
}

impl GlkApi {
    pub fn new() -> Self {
        GlkApi {
            streams: GlkObjectStore::new(),
            current_stream: None,
            windows: GlkObjectStore::new(),
        }
    }

    pub fn glk_get_buffer_stream(&mut self, str_id: Option<NonZeroU32>, buf: &mut [u8]) -> GlkResult<u32> {
        stream_op!(self, str_id, |str: &mut Stream| str.get_buffer(&mut GlkBufferMut::U8(buf)))
    }

    pub fn glk_get_buffer_stream_uni(&mut self, str_id: Option<NonZeroU32>, buf: &mut [u32]) -> GlkResult<u32> {
        stream_op!(self, str_id, |str: &mut Stream| str.get_buffer(&mut GlkBufferMut::U32(buf)))
    }

    pub fn glk_get_char_stream(&mut self, str_id: Option<NonZeroU32>) -> GlkResult<i32> {
        stream_op!(self, str_id, |str: &mut Stream| str.get_char(false))
    }

    pub fn glk_get_char_stream_uni(&mut self, str_id: Option<NonZeroU32>) -> GlkResult<i32> {
        stream_op!(self, str_id, |str: &mut Stream| str.get_char(true))
    }

    pub fn glk_get_line_stream(&mut self, str_id: Option<NonZeroU32>, buf: &mut [u8]) -> GlkResult<u32> {
        stream_op!(self, str_id, |str: &mut Stream| str.get_line(&mut GlkBufferMut::U8(buf)))
    }

    pub fn glk_get_line_stream_uni(&mut self, str_id: Option<NonZeroU32>, buf: &mut [u32]) -> GlkResult<u32> {
        stream_op!(self, str_id, |str: &mut Stream| str.get_line(&mut GlkBufferMut::U32(buf)))
    }

    pub fn glk_put_buffer(&mut self, buf: &[u8]) -> GlkResult<()> {
        stream_op!(self, self.current_stream, |str: &mut Stream| str.put_buffer(&GlkBuffer::U8(buf)))
    }

    pub fn glk_put_buffer_stream(&mut self, str_id: Option<NonZeroU32>, buf: &[u8]) -> GlkResult<()> {
        stream_op!(self, str_id, |str: &mut Stream| str.put_buffer(&GlkBuffer::U8(buf)))
    }

    pub fn glk_put_buffer_stream_uni(&mut self, str_id: Option<NonZeroU32>, buf: &[u32]) -> GlkResult<()> {
        stream_op!(self, str_id, |str: &mut Stream| str.put_buffer(&GlkBuffer::U32(buf)))
    }

    pub fn glk_put_buffer_uni(&mut self, buf: &[u32]) -> GlkResult<()> {
        stream_op!(self, self.current_stream, |str: &mut Stream| str.put_buffer(&GlkBuffer::U32(buf)))
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

    pub fn glk_window_clear(&mut self, win_id: Option<NonZeroU32>) -> GlkResult<()> {
        Ok(win_mut!(self, win_id).data.clear())
    }

    pub fn glk_window_get_type(&mut self, win_id: Option<NonZeroU32>) -> GlkResult<WindowType> {
        Ok(win!(self, win_id).wintype)
    }

    pub fn glk_stream_close(&mut self, str_id: Option<NonZeroU32>) -> GlkResult<StreamResultCounts> {
        let res = stream_op!(self, str_id, |str: &mut Stream| str.close());
        self.streams.unregister(str_id.unwrap());
        res
    }

    pub fn glk_stream_get_current(&self) -> Option<NonZeroU32> {
        self.current_stream
    }

    pub fn glk_stream_get_position(&self, str_id: Option<NonZeroU32>) -> GlkResult<u32> {
        Ok(str!(self, str_id).get_position())
    }

    pub fn glk_stream_get_rock(&self, str_id: Option<NonZeroU32>) -> GlkResult<u32> {
        self.streams.get_rock(str_id).ok_or(InvalidReference)
    }

    pub fn glk_stream_iterate(&self, str_id: Option<NonZeroU32>) -> Option<IterationResult> {
        self.streams.iterate(str_id)
    }

    pub fn glk_stream_open_memory(&mut self, buf: Box<[u8]>, fmode: FileMode, rock: u32) -> GlkResult<NonZeroU32> {
        self.create_memory_stream(buf, fmode, rock)
    }

    pub fn glk_stream_open_memory_uni(&mut self, buf: Box<[u32]>, fmode: FileMode, rock: u32) -> GlkResult<NonZeroU32> {
        self.create_memory_stream(buf, fmode, rock)
    }

    pub fn glk_stream_set_current(&mut self, str_id: Option<NonZeroU32>) {
        self.current_stream = str_id;
    }

    pub fn glk_stream_set_position(&mut self, str_id: Option<NonZeroU32>, mode: SeekMode, pos: i32) -> GlkResult<()> {
        Ok(str_mut!(self, str_id).set_position(mode, pos))
    }

    fn create_memory_stream<T>(&mut self, buf: Box<[T]>, fmode: FileMode, rock: u32) -> GlkResult<NonZeroU32>
    where Stream: From<ArrayBackedStream<T>> {
        if fmode == FileMode::WriteAppend {
            return Err(IllegalFilemode);
        }
        let str: Stream = if buf.len() == 0 {
            NullStream::default().into()
        }
        else {
            ArrayBackedStream::<T>::new(buf, fmode, None).into()
        };
        Ok(self.streams.register(str, rock))
    }
}

/** Final read/write character counts of a stream */
#[repr(C)]
pub struct StreamResultCounts {
    pub read_count: u32,
    pub write_count: u32,
}