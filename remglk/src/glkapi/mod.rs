/*

The Glk API
===========

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

pub mod arrays;
pub mod constants;
pub mod filerefs;
pub mod objects;
pub mod protocol;
pub mod streams;

use thiserror::Error;

use arrays::*;
use constants::*;
use objects::*;
use streams::*;

#[derive(Error, Debug)]
pub enum GlkApiError {
    #[error("illegal filemode")]
    IllegalFilemode,
}
pub type GlkResult<'a, T> = Result<T, GlkApiError>;

pub struct GlkApi<'fref> {
    streams: GlkObjectStore<Stream<'fref>>,
    current_stream: Option<u32>,
}

impl GlkApi<'_> {
    pub fn glk_stream_get_current(&self) -> Option<u32> {
        self.current_stream
    }

    pub fn glk_stream_get_rock(&self, id: u32) -> Option<u32> {
        self.streams.get_rock(id)
    }

    pub fn glk_stream_iterate(&self, id: Option<u32>) -> Option<IterationResult> {
        self.streams.iterate(id)
    }

    pub fn glk_stream_open_memory<'buf>(&mut self, buf: Vec<u8>, fmode: FileMode, rock: u32) -> GlkResult<u32> {
        self.create_memory_stream(GlkArray::U8(buf), fmode, rock)
    }

    pub fn glk_stream_open_memory_uni<'buf>(&mut self, buf: Vec<u32>, fmode: FileMode, rock: u32) -> GlkResult<u32> {
        self.create_memory_stream(GlkArray::U32(buf), fmode, rock)
    }

    pub fn glk_stream_set_current(&mut self, str: u32) {
        self.current_stream = Some(str);
    }

    fn create_memory_stream<'buf>(&mut self, buf: GlkArray, fmode: FileMode, rock: u32) -> GlkResult<'buf, u32> {
        if fmode == FileMode::WriteAppend {
            return Err(GlkApiError::IllegalFilemode);
        }
        let str: Stream = if buf.len() == 0 {
            Stream::Null(NullStream::new())
        }
        else {
            Stream::Array(ArrayBackedStream::new(buf, fmode, None))
        };
        Ok(self.streams.register(str, rock))
    }
}