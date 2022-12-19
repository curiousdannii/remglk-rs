/*

Glk Streams
===========

Copyright (c) 2022 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::cmp::min;

use super::*;
use constants::*;

const GLK_NULL: u32 = 0;

pub trait Stream {
    fn close(&self) -> StreamResult;
    fn disprock(&self) -> Option<u32>;
    fn get_buffer(&mut self, buf: &mut GlkArray) -> u32;
    fn get_char(&mut self, uni: bool) -> i32;
    fn get_line(&mut self, buf: &mut GlkArray) -> u32;
    fn get_position(&self) -> u32;
    fn put_buffer(&mut self, buf: &GlkArray);
    fn put_char(&mut self, ch: u32);
    fn put_string(&mut self, str: &str, style: Option<&str>);
    fn rock(&self) -> u32;
    fn set_position(&mut self, mode: SeekMode, pos: i32);
}

/** A fixed-length TypedArray backed stream */
pub struct ArrayBackedStream<'a> {
    buf: &'a mut GlkArray<'a>,
    close_cb: Option<fn()>,
    disprock: Option<u32>,
    fmode: FileMode,
    len: usize,
    pos: usize,
    read_count: usize,
    rock: u32,
    write_count: usize,
}

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

    fn disprock(&self) -> Option<u32> {
        self.disprock
    }

    fn get_buffer(&mut self, buf: &mut GlkArray) -> u32 {
        if let FileMode::Write | FileMode::WriteAppend = self.fmode {
            panic!("Cannot read from write-only stream")
        }
        let read_length = min(buf.len(), self.len - self.pos);
        buf.set_slice(self.buf, self.pos, 0, read_length);
        self.pos += read_length;
        self.read_count += read_length;
        read_length as u32
    }

    fn get_char(&mut self, uni: bool) -> i32 {
        if let FileMode::Write | FileMode::WriteAppend = self.fmode {
            panic!("Cannot read from write-only stream")
        }
        self.read_count += 1;
        if self.pos < self.len {
            let ch = self.buf.get_u32(self.pos);
            self.pos += 1;
            return if !uni && ch > MAX_LATIN1 {QUESTION_MARK} else {ch} as i32;
        }
        -1
    }

    fn get_line(&mut self, buf: &mut GlkArray) -> u32 {
        if let FileMode::Write | FileMode::WriteAppend = self.fmode {
            panic!("Cannot read from write-only stream")
        }
        let read_length: isize = min(buf.len() as isize - 1, (self.len - self.pos) as isize);
        if read_length < 0 {
            return 0;
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
        i as u32
    }

    fn get_position(&self) -> u32 {
        self.pos as u32
    }

    fn put_buffer(&mut self, buf: &GlkArray) {
        if let FileMode::Read = self.fmode {
            panic!("Cannot write to read-only stream")
        }
        let buf_length = buf.len();
        let write_length = min(buf_length, self.len - self.pos);
        self.buf.set_slice(buf, 0, self.pos, write_length);
        self.pos += write_length;
        self.write_count += buf_length;
    }

    fn put_char(&mut self, ch: u32) {
        if let FileMode::Read = self.fmode {
            panic!("Cannot write to read-only stream")
        }
        if self.pos < self.len {
            self.buf.set_u32(self.pos, ch);
            self.pos += 1;
        }
        self.write_count += 1;
    }

    fn put_string(&mut self, str: &str, _style: Option<&str>) {
        if let FileMode::Read = self.fmode {
            panic!("Cannot write to read-only stream")
        }
        let (str_length, write_length) = self.buf.set_str(self.pos, str);
        self.pos += write_length;
        self.write_count += str_length;
    }

    fn rock(&self) -> u32 {
        self.rock
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