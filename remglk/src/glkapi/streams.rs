/*

Glk Streams
===========

Copyright (c) 2023 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::cmp::{max, min};
use thiserror::Error;

use super::*;
use constants::*;
use filerefs::FileRef;

const GLK_NULL: u32 = 0;

pub type GlkStream = GlkObject<dyn Stream>;

#[derive(Error, Debug)]
pub enum StreamError {
    #[error("cannot read from write-only stream")]
    ReadFromWriteOnly,
    #[error("cannot write to read-only stream")]
    WriteToReadOnly,
}

/** Final read/write character counts of a stream */
pub struct StreamResult {
    pub read_count: u32,
    pub write_count: u32,
}

pub trait Stream {
    fn close(&self) -> StreamResult;
    fn get_buffer(&mut self, buf: &mut GlkArray) -> Result<u32, StreamError>;
    fn get_char(&mut self, uni: bool) -> Result<i32, StreamError>;
    fn get_line(&mut self, buf: &mut GlkArray) -> Result<u32, StreamError>;
    fn get_position(&self) -> u32;
    fn put_buffer(&mut self, buf: &GlkArray) -> Result<(), StreamError>;
    fn put_char(&mut self, ch: u32) -> Result<(), StreamError>;
    fn set_position(&mut self, mode: SeekMode, pos: i32);
}

/** A fixed-length TypedArray backed stream */
pub struct ArrayBackedStream<'a> {
    buf: &'a mut GlkArray<'a>,
    close_cb: Option<fn()>,
    fmode: FileMode,
    /** The length of the active region of the buffer.
        This can be shorter than the actual length of the buffer in two situations: a file stream, or a `filemode_Write` memory stream.
        See https://github.com/iftechfoundation/ifarchive-if-specs/issues/8
    */
    len: usize,
    pos: usize,
    read_count: usize,
    write_count: usize,
}

impl<'a> ArrayBackedStream<'a> {
    pub fn new(buf: &'a mut GlkArray<'a>, fmode: FileMode, close_cb: Option<fn()>) -> ArrayBackedStream<'a> {
        let buf_len = buf.len();
        ArrayBackedStream {
            buf,
            close_cb,
            fmode,
            len: match fmode {
                FileMode::Write => 0,
                _ => buf_len,
            },
            pos: 0,
            read_count: 0,
            write_count: 0,
        }
    }
}

/*** An ArrayBackedStream is the basis of memory and file streams */
impl Stream for ArrayBackedStream<'_> {
    fn close(&self) -> StreamResult {
        if let Some(cb) = self.close_cb {
            cb();
        }
        StreamResult {
            read_count: self.read_count as u32,
            write_count: self.write_count as u32,
        }
    }

    fn get_buffer(&mut self, buf: &mut GlkArray) -> Result<u32, StreamError> {
        if let FileMode::Write | FileMode::WriteAppend = self.fmode {
            return Err(StreamError::ReadFromWriteOnly);
        }
        let read_length = min(buf.len(), self.len - self.pos);
        if read_length == 0 {
            return Ok(0);
        }
        buf.set_slice(self.buf, self.pos, 0, read_length);
        self.pos += read_length;
        self.read_count += read_length;
        Ok(read_length as u32)
    }

    fn get_char(&mut self, uni: bool) -> Result<i32, StreamError> {
        if let FileMode::Write | FileMode::WriteAppend = self.fmode {
            return Err(StreamError::ReadFromWriteOnly);
        }
        self.read_count += 1;
        if self.pos < self.len {
            let ch = self.buf.get_u32(self.pos);
            self.pos += 1;
            return Ok(if !uni && ch > MAX_LATIN1 {QUESTION_MARK} else {ch} as i32);
        }
        Ok(-1)
    }

    fn get_line(&mut self, buf: &mut GlkArray) -> Result<u32, StreamError> {
        if let FileMode::Write | FileMode::WriteAppend = self.fmode {
            return Err(StreamError::ReadFromWriteOnly);
        }
        let read_length: isize = min(buf.len() as isize - 1, (self.len - self.pos) as isize);
        if read_length < 0 {
            return Ok(0);
        }
        let mut i: usize = 0;
        while i < read_length as usize {
            let ch = self.buf.get_u32(self.pos);
            self.pos += 1;
            buf.set_u32(i, ch);
            i += 1;
            if ch == 10 {
                break;
            }
        }
        buf.set_u32(i, GLK_NULL);
        self.read_count += i;
        Ok(i as u32)
    }

    fn get_position(&self) -> u32 {
        self.pos as u32
    }

    fn put_buffer(&mut self, buf: &GlkArray) -> Result<(), StreamError> {
        if let FileMode::Read = self.fmode {
            return Err(StreamError::WriteToReadOnly);
        }
        let buf_length = buf.len();
        let write_length = min(buf_length, self.len - self.pos);
        if write_length > 0 {
            self.buf.set_slice(buf, 0, self.pos, write_length);
            self.pos += write_length;
        }
        self.write_count += buf_length;
        Ok(())
    }

    fn put_char(&mut self, ch: u32) -> Result<(), StreamError> {
        if let FileMode::Read = self.fmode {
            return Err(StreamError::WriteToReadOnly);
        }
        if self.pos < self.len {
            self.buf.set_u32(self.pos, ch);
            self.pos += 1;
        }
        self.write_count += 1;
        Ok(())
    }

    fn set_position(&mut self, mode: SeekMode, pos: i32) {
        let new_pos: i32 = match mode {
            SeekMode::Current => self.pos as i32 + pos,
            SeekMode::End => self.len as i32 + pos,
            SeekMode::Start => pos,
        };
        self.pos = new_pos.clamp(0, self.len as i32) as usize;
    }
}

/** FileStreams are based on array backed streams, but can grow in length */
pub struct FileStream<'buf, 'fref> {
    fref: &'fref FileRef,
    str: ArrayBackedStream<'buf>,
}

impl<'buf, 'fref> FileStream<'buf, 'fref> {
    pub fn new(fref: &'fref FileRef, buf: &'buf mut GlkArray<'buf>, fmode: FileMode, ) -> FileStream<'buf, 'fref> {
        FileStream {
            fref,
            str: ArrayBackedStream::new(buf, fmode, None),
        }
    }

    fn expand(&mut self, increase: usize) {
        let end_pos = self.str.pos + increase;
        self.str.len = end_pos;
        let mut max_len = self.str.buf.len();
        if end_pos > max_len {
            // Always expand by at least 100
            max_len += max(end_pos - max_len, 100);

        }
    }
}

impl Stream for FileStream<'_, '_> {
    fn close(&self) -> StreamResult {
        StreamResult {
            read_count: self.str.read_count as u32,
            write_count: self.str.write_count as u32,
        }
    }

    fn get_buffer(&mut self, buf: &mut GlkArray) -> Result<u32, StreamError> {
        self.str.get_buffer(buf)
    }

    fn get_char(&mut self, uni: bool) -> Result<i32, StreamError> {
        self.str.get_char(uni)
    }

    fn get_line(&mut self, buf: &mut GlkArray) -> Result<u32, StreamError> {
        self.str.get_line(buf)
    }

    fn get_position(&self) -> u32 {
        self.str.get_position()
    }

    fn put_buffer(&mut self, buf: &GlkArray) -> Result<(), StreamError> {
        self.str.put_buffer(buf)
    }

    fn put_char(&mut self, ch: u32) -> Result<(), StreamError> {
        self.str.put_char(ch)
    }

    fn set_position(&mut self, mode: SeekMode, pos: i32) {
        self.str.set_position(mode, pos)
    }
}

/*** A NullStream is only used for a memory stream with no buffer */
pub struct NullStream {
    write_count: usize,
}

impl NullStream {
    pub fn new() -> NullStream {
        NullStream {
            write_count: 0,
        }
    }
}

impl Stream for NullStream {
    fn close(&self) -> StreamResult {
        StreamResult {
            read_count: 0,
            write_count: self.write_count as u32,
        }
    }

    fn get_buffer(&mut self, _: &mut GlkArray) -> Result<u32, StreamError> {
        Ok(0)
    }

    fn get_char(&mut self, _: bool) -> Result<i32, StreamError> {
        Ok(-1)
    }

    fn get_line(&mut self, _: &mut GlkArray) -> Result<u32, StreamError> {
        Ok(0)
    }

    fn get_position(&self) -> u32 {
        0
    }

    fn put_buffer(&mut self, buf: &GlkArray) -> Result<(), StreamError> {
        self.write_count += buf.len();
        Ok(())
    }

    fn put_char(&mut self, _: u32) -> Result<(), StreamError> {
        self.write_count += 1;
        Ok(())
    }

    fn set_position(&mut self, _: SeekMode, _: i32) {}
}