/*

Glk Streams
===========

Copyright (c) 2023 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::cmp::min;
use thiserror::Error;

use super::*;
use arrays::*;
use constants::*;
use protocol::FileRef;

const GLK_NULL: u32 = 0;

pub type GlkStream<T, Dest, Src> = GlkObject<dyn Stream<T, Dest, Src>>;

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

pub trait Stream<T, Dest, Src>
where
    [Dest]: GlkArray + SetGlkBuffer<[T]>
{
    fn close(&self) -> StreamResult;
    fn get_buffer(&mut self, buf: &mut [Dest]) -> Result<u32, StreamError>;
    fn get_char(&mut self, uni: bool) -> Result<i32, StreamError>;
    fn get_line(&mut self, buf: &mut [Dest]) -> Result<u32, StreamError>;
    fn get_position(&self) -> u32;
    fn put_buffer(&mut self, buf: &[Src]) -> Result<(), StreamError>;
    fn put_char(&mut self, ch: u32) -> Result<(), StreamError>;
    fn set_position(&mut self, mode: SeekMode, pos: i32);
}

/** A fixed-length stream based on a buffer (slice).
    An ArrayBackedStream are used for memory and resource streams, and are the basis of file streams.
*/
pub struct ArrayBackedStream<'a, T>
where
    [T]: GlkArray,
{
    buf: DataSource<'a, T>,
    close_cb: Option<fn()>,
    /** Whether we need to check if we should expand the active buffer region before writing */
    expandable: bool,
    fmode: FileMode,
    /** The length of the active region of the buffer.
        This can be shorter than the actual length of the buffer in a `filemode_Write` memory stream.
        Expanding filestreams is handled differently, see below.
        See https://github.com/iftechfoundation/ifarchive-if-specs/issues/8
    */
    len: usize,
    pos: usize,
    read_count: usize,
    write_count: usize,
}

impl<'a, T> ArrayBackedStream<'a, T>
where
    [T]: GlkArray,
{
    pub fn new(buf: DataSource<'a, T>, fmode: FileMode, close_cb: Option<fn()>) -> ArrayBackedStream<'a, T> {
        let buf_len = buf.len();
        ArrayBackedStream {
            buf,
            close_cb,
            expandable: fmode == FileMode::Write,
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

    fn expand(&mut self, increase: usize) {
        self.len = min(self.pos + increase, self.buf.len());
        if self.len == self.buf.len() {
            self.expandable = false;
        }
    }
}

impl<T, Dest, Src> Stream<T, Dest, Src> for ArrayBackedStream<'_, T>
where
    [Dest]: GlkArray + SetGlkBuffer<[T]>,
    [T]: GlkArray + SetGlkBuffer<[Src]>,
{
    fn close(&self) -> StreamResult {
        if let Some(cb) = self.close_cb {
            cb();
        }
        StreamResult {
            read_count: self.read_count as u32,
            write_count: self.write_count as u32,
        }
    }

    fn get_buffer(&mut self, buf: &mut [Dest]) -> Result<u32, StreamError> {
        if let FileMode::Write | FileMode::WriteAppend = self.fmode {
            return Err(StreamError::ReadFromWriteOnly);
        }
        let read_length = min(buf.len(), self.len - self.pos);
        if read_length == 0 {
            return Ok(0);
        }
        buf[..read_length].set_buffer(&self.buf[self.pos..(self.pos + read_length)]);
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

    fn get_line(&mut self, buf: &mut [Dest]) -> Result<u32, StreamError> {
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

    fn put_buffer(&mut self, buf: &[Src]) -> Result<(), StreamError> {
        if let FileMode::Read = self.fmode {
            return Err(StreamError::WriteToReadOnly);
        }
        self.write_count += buf.len();
        if self.pos + buf.len() > self.len && self.expandable {
            self.expand(buf.len());
        }
        let write_length = min(buf.len(), self.len - self.pos);
        if write_length > 0 {
            self.buf[self.pos..(self.pos + write_length)].set_buffer(&buf[..write_length]);
            self.pos += write_length;
        }
        Ok(())
    }

    fn put_char(&mut self, ch: u32) -> Result<(), StreamError> {
        if let FileMode::Read = self.fmode {
            return Err(StreamError::WriteToReadOnly);
        }
        self.write_count += 1;
        if self.pos == self.len && self.expandable {
            self.expand(1);
        }
        if self.pos < self.len {
            self.buf.set_u32(self.pos, ch);
            self.pos += 1;
        }
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

/** Writable FileStreams are based on array backed streams, but can grow in length.
    Read-only file streams just use an ArrayBackedStream directly.
*/
pub struct FileStream<'buf, 'fref, T>
where
    T: Clone + Default,
    [T]: GlkArray,
{
    fref: &'fref FileRef,
    str: ArrayBackedStream<'buf, T>,
}

impl<'buf, 'fref, T> FileStream<'buf, 'fref, T>
where
    T: Clone + Default,
    [T]: GlkArray,
{
    pub fn new(fref: &'fref FileRef, fmode: FileMode) -> FileStream<'buf, 'fref, T> {
        assert!(fmode != FileMode::Read);
        FileStream {
            fref,
            str: ArrayBackedStream::new(DataSource::Owned(Vec::<T>::new()), fmode, None),
        }
    }

    fn expand(&mut self, increase: usize) {
        let end_pos = self.str.pos + increase;
        if end_pos > self.str.buf.len() {
            match self.str.buf {
                DataSource::Owned(ref mut v) => {
                    v.resize(end_pos, T::default());
                },
                _ => unreachable!(),
            };
            self.str.len = end_pos;
        }
    }
}

impl<T, Dest, Src> Stream<T, Dest, Src> for FileStream<'_, '_, T>
where
    [Dest]: GlkArray + SetGlkBuffer<[T]>,
    T: Clone + Default,
    [T]: GlkArray + SetGlkBuffer<[Src]>,
{
    fn close(&self) -> StreamResult {
        StreamResult {
            read_count: self.str.read_count as u32,
            write_count: self.str.write_count as u32,
        }
    }

    fn get_buffer(&mut self, buf: &mut [Dest]) -> Result<u32, StreamError> {
        self.str.get_buffer(buf)
    }

    fn get_char(&mut self, uni: bool) -> Result<i32, StreamError> {
        self.str.get_char(uni)
    }

    fn get_line(&mut self, buf: &mut [Dest]) -> Result<u32, StreamError> {
        self.str.get_line(buf)
    }

    fn get_position(&self) -> u32 {
        self.str.get_position()
    }

    fn put_buffer(&mut self, buf: &[Src]) -> Result<(), StreamError> {
        if self.str.pos + buf.len() > self.str.len {
            self.expand(buf.len());
        }
        self.str.put_buffer(buf)
    }

    fn put_char(&mut self, ch: u32) -> Result<(), StreamError> {
        if self.str.pos == self.str.len {
            self.expand(1);
        }
        self.str.put_char(ch)
    }

    fn set_position(&mut self, mode: SeekMode, pos: i32) {
        self.str.set_position(mode, pos)
    }
}

/** A NullStream is only used for a memory stream with no buffer */
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

impl<T, Dest, Src> Stream<T, Dest, Src> for NullStream
where
    [Dest]: GlkArray + SetGlkBuffer<[T]>,
{
    fn close(&self) -> StreamResult {
        StreamResult {
            read_count: 0,
            write_count: self.write_count as u32,
        }
    }

    fn get_buffer(&mut self, _: &mut [Dest]) -> Result<u32, StreamError> {
        Ok(0)
    }

    fn get_char(&mut self, _: bool) -> Result<i32, StreamError> {
        Ok(-1)
    }

    fn get_line(&mut self, _: &mut [Dest]) -> Result<u32, StreamError> {
        Ok(0)
    }

    fn get_position(&self) -> u32 {
        0
    }

    fn put_buffer(&mut self, buf: &[Src]) -> Result<(), StreamError> {
        self.write_count += buf.len();
        Ok(())
    }

    fn put_char(&mut self, _: u32) -> Result<(), StreamError> {
        self.write_count += 1;
        Ok(())
    }

    fn set_position(&mut self, _: SeekMode, _: i32) {}
}