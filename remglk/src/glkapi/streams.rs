/*

Glk Streams
===========

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::cmp::min;

use enum_dispatch::enum_dispatch;

use super::*;
use arrays::*;
use common::*;
use GlkApiError::*;
use constants::*;
use protocol::FileRef;

const GLK_NULL: u32 = 0;

#[enum_dispatch]
pub enum Stream {
    ArrayBackedStreamU8(ArrayBackedStream<u8>),
    ArrayBackedStreamU32(ArrayBackedStream<u32>),
    //FileStream,
    NullStream,
}

#[enum_dispatch(Stream)]
pub trait StreamOperations {
    fn close(&self) -> GlkResult<StreamResultCounts>;
    fn get_buffer(&mut self, buf: &mut GlkBufferMut) -> GlkResult<u32>;
    fn get_char(&mut self, uni: bool) -> GlkResult<i32>;
    fn get_line(&mut self, buf: &mut GlkBufferMut) -> GlkResult<u32>;
    fn get_position(&self) -> u32;
    fn put_buffer(&mut self, buf: &GlkBuffer) -> GlkResult<()>;
    fn put_char(&mut self, ch: u32) -> GlkResult<()>;
    fn set_position(&mut self, mode: SeekMode, pos: i32);
}

/** Final read/write character counts of a stream */
#[repr(C)]
pub struct StreamResultCounts {
    pub read_count: u32,
    pub write_count: u32,
}

/** A fixed-length stream based on a buffer (a boxed slice).
    ArrayBackedStreams are used for memory and resource streams, and are the basis of file streams.
*/
pub struct ArrayBackedStream<T> {
    buf: Box<[T]>,
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

impl<T> ArrayBackedStream<T> {
    pub fn new(buf: Box<[T]>, fmode: FileMode, close_cb: Option<fn()>) -> ArrayBackedStream<T> {
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

impl<T> StreamOperations for ArrayBackedStream<T>
where Box<[T]>: GlkArray {
    fn close(&self) -> GlkResult<StreamResultCounts> {
        if let Some(cb) = self.close_cb {
            cb();
        }
        Ok(StreamResultCounts {
            read_count: self.read_count as u32,
            write_count: self.write_count as u32,
        })
    }

    fn get_buffer(&mut self, buf: &mut GlkBufferMut) -> GlkResult<u32> {
        if let FileMode::Write | FileMode::WriteAppend = self.fmode {
            return Err(ReadFromWriteOnly);
        }
        let read_length = min(buf.len(), self.len - self.pos);
        if read_length == 0 {
            return Ok(0);
        }
        self.buf.copy_to_buffer(self.pos, buf, 0, read_length);
        self.pos += read_length;
        self.read_count += read_length;
        Ok(read_length as u32)
    }

    fn get_char(&mut self, uni: bool) -> GlkResult<i32> {
        if let FileMode::Write | FileMode::WriteAppend = self.fmode {
            return Err(ReadFromWriteOnly);
        }
        self.read_count += 1;
        if self.pos < self.len {
            let ch = self.buf.get_u32(self.pos);
            self.pos += 1;
            return Ok(if !uni && ch > MAX_LATIN1 {QUESTION_MARK} else {ch} as i32);
        }
        Ok(-1)
    }

    fn get_line(&mut self, buf: &mut GlkBufferMut) -> GlkResult<u32> {
        if let FileMode::Write | FileMode::WriteAppend = self.fmode {
            return Err(ReadFromWriteOnly);
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

    fn put_buffer(&mut self, buf: &GlkBuffer) -> GlkResult<()> {
        if let FileMode::Read = self.fmode {
            return Err(WriteToReadOnly);
        }
        let buf_len = buf.len();
        self.write_count += buf_len;
        if self.pos + buf_len > self.len && self.expandable {
            self.expand(buf_len);
        }
        let write_length = min(buf_len, self.len - self.pos);
        if write_length > 0 {
            self.buf.copy_from_buffer(self.pos, buf, 0, write_length);
            self.pos += write_length;
        }
        Ok(())
    }

    fn put_char(&mut self, ch: u32) -> GlkResult<()> {
        if let FileMode::Read = self.fmode {
            return Err(WriteToReadOnly);
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

/* /** Writable FileStreams are based on array backed streams, but can grow in length.
    Read-only file streams just use an ArrayBackedStream directly.
*/
pub struct FileStream {
    fref: FileRef,
    str: ArrayBackedStream,
}

impl FileStream {
    pub fn new(fref: FileRef, buf: GlkArray, fmode: FileMode) -> FileStream {
        assert!(fmode != FileMode::Read);
        FileStream {
            fref,
            str: ArrayBackedStream::new(buf, fmode, None),
        }
    }

    fn expand(&mut self, increase: usize) {
        let end_pos = self.str.pos + increase;
        if end_pos > self.str.buf.len() {
            self.str.buf.resize(end_pos);
            self.str.len = end_pos;
        }
    }
}

impl StreamOperations for FileStream {
    fn close(&self) -> GlkResult<StreamResultCounts> {
        Ok(StreamResultCounts {
            read_count: self.str.read_count as u32,
            write_count: self.str.write_count as u32,
        })
    }

    fn get_buffer(&mut self, buf: &mut GlkArray) -> GlkResult<u32> {
        self.str.get_buffer(buf)
    }

    fn get_char(&mut self, uni: bool) -> GlkResult<i32> {
        self.str.get_char(uni)
    }

    fn get_line(&mut self, buf: &mut GlkArray) -> GlkResult<u32> {
        self.str.get_line(buf)
    }

    fn get_position(&self) -> u32 {
        self.str.get_position()
    }

    fn put_buffer(&mut self, buf: &GlkArray) -> GlkResult<()> {
        if self.str.pos + buf.len() > self.str.len {
            self.expand(buf.len());
        }
        self.str.put_buffer(buf)
    }

    fn put_char(&mut self, ch: u32) -> GlkResult<()> {
        if self.str.pos == self.str.len {
            self.expand(1);
        }
        self.str.put_char(ch)
    }

    fn set_position(&mut self, mode: SeekMode, pos: i32) {
        self.str.set_position(mode, pos)
    }
} */

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

impl StreamOperations for NullStream {
    fn close(&self) -> GlkResult<StreamResultCounts> {
        Ok(StreamResultCounts {
            read_count: 0,
            write_count: self.write_count as u32,
        })
    }

    fn get_buffer(&mut self, _: &mut GlkBufferMut) -> GlkResult<u32> {
        Ok(0)
    }

    fn get_char(&mut self, _: bool) -> GlkResult<i32> {
        Ok(-1)
    }

    fn get_line(&mut self, _: &mut GlkBufferMut) -> GlkResult<u32> {
        Ok(0)
    }

    fn get_position(&self) -> u32 {
        0
    }

    fn put_buffer(&mut self, buf: &GlkBuffer) -> GlkResult<()> {
        self.write_count += buf.len();
        Ok(())
    }

    fn put_char(&mut self, _: u32) -> GlkResult<()> {
        self.write_count += 1;
        Ok(())
    }

    fn set_position(&mut self, _: SeekMode, _: i32) {}
}