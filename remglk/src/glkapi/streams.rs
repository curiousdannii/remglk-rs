/*

Glk Streams
===========

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::cmp::{max, min};
use std::ffi::CString;

use enum_dispatch::enum_dispatch;

use super::*;

const GLK_NULL: u32 = 0;

/** A `Stream` wrapped in an GlkObject (`Arc<Mutex>>`) */
pub type GlkStream = GlkObject<Stream>;
pub type GlkStreamWeak = GlkObjectWeak<Stream>;

#[enum_dispatch]
pub enum Stream {
    ArrayBackedU8(ArrayBackedStream<u8>),
    ArrayBackedU32(ArrayBackedStream<u32>),
    FileStreamU8(FileStream<u8>),
    FileStreamU32(FileStream<u32>),
    Null(NullStream),
    Window(WindowStream),
}

impl Default for Stream {
    fn default() -> Self {
        Stream::Null(NullStream::default())
    }
}

impl GlkObjectClass for Stream {
    fn get_object_class_id() -> u32 {
        1
    }
}

#[enum_dispatch(Stream)]
pub trait StreamOperations {
    fn close(&self) -> StreamResultCounts {
        StreamResultCounts {
            read_count: 0,
            write_count: self.write_count() as u32,
        }
    }
    fn file_path(&self) -> GlkResult<&CString> {Err(NotFileStream)}
    fn get_buffer(&mut self, _buf: &mut GlkBufferMut) -> GlkResult<u32> {Ok(0)}
    fn get_char(&mut self, _uni: bool) -> GlkResult<i32> {Ok(-1)}
    fn get_line(&mut self, _buf: &mut GlkBufferMut) -> GlkResult<u32> {Ok(0)}
    fn get_position(&self) -> u32 {0}
    fn put_buffer(&mut self, buf: &GlkBuffer) -> GlkResult<()>;
    fn put_char(&mut self, ch: u32) -> GlkResult<()>;
    fn put_string(&mut self, str: &str, style: Option<u32>) -> GlkResult<()>;
    fn set_css(&self, _name: &str, _val: Option<&CSSValue>) {}
    fn set_hyperlink(&self, _val: u32) {}
    fn set_position(&mut self, _mode: SeekMode, _pos: i32) {}
    fn set_style(&self, _style: u32) {}
    fn write_count(&self) -> usize;
}

/** A fixed-length stream based on a buffer (a boxed slice).
    ArrayBackedStreams are used for memory and resource streams, and are the basis of file streams.
*/
#[derive(Default)]
pub struct ArrayBackedStream<T> {
    buf: Box<[T]>,
    /** Whether we need to check if we should expand the active buffer region before writing */
    expandable: bool,
    fmode: FileMode,
    /** The length of the active region of the buffer.
        This can be shorter than the actual length of the buffer in a `filemode_Write` memory stream.
        Expanding filestreams is handled differently, see below.
        See https://github.com/iftechfoundation/ifarchive-if-specs/issues/8
    */
    len: usize,
    path: Option<CString>,
    pos: usize,
    read_count: usize,
    write_count: usize,
}

impl<T> ArrayBackedStream<T> {
    pub fn new(buf: Box<[T]>, fmode: FileMode, fileref: Option<&GlkFileRef>) -> ArrayBackedStream<T> {
        let buf_len = buf.len();
        let path = fileref.map(|fileref| {
            let fileref = lock!(fileref);
            CString::new(&fileref.path[..]).unwrap()
        });
        ArrayBackedStream {
            buf,
            expandable: fmode == FileMode::Write,
            fmode,
            len: match fmode {
                FileMode::Write => 0,
                _ => buf_len,
            },
            path,
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

    pub fn take_buffer(&mut self) -> Box<[T]> {
        mem::take(&mut self.buf)
    }
}

impl<T> StreamOperations for ArrayBackedStream<T>
where Box<[T]>: GlkArray {
    fn close(&self) -> StreamResultCounts {
        StreamResultCounts {
            read_count: self.read_count as u32,
            write_count: self.write_count as u32,
        }
    }

    fn file_path(&self) -> GlkResult<&CString> {
        self.path.as_ref().ok_or(NotFileStream)
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
        if self.pos < self.len {
            self.read_count += 1;
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

    fn put_string(&mut self, str: &str, _style: Option<u32>) -> GlkResult<()> {
        let buf: GlkOwnedBuffer = str.into();
        self.put_buffer(&(&buf).into())
    }

    fn set_position(&mut self, mode: SeekMode, pos: i32) {
        let new_pos: i32 = match mode {
            SeekMode::Current => self.pos as i32 + pos,
            SeekMode::End => self.len as i32 + pos,
            SeekMode::Start => pos,
        };
        self.pos = new_pos.clamp(0, self.len as i32) as usize;
    }

    fn write_count(&self) -> usize {
        self.write_count
    }
}

/** Writable FileStreams are based on array backed streams, but can grow in length.
    Read-only file streams just use an ArrayBackedStream directly.
*/
#[derive(Default)]
pub struct FileStream<T> {
    binary: bool,
    pub changed: bool,
    pub path: String,
    str: ArrayBackedStream<T>,
}

impl<T> FileStream<T>
where T: Clone + Default, Box<[T]>: GlkArray {
    pub fn new(fileref_glkobj: &GlkFileRef, buf: Box<[T]>, fmode: FileMode) -> FileStream<T> {
        debug_assert!(fmode != FileMode::Read);
        let str = ArrayBackedStream::new(buf, fmode, Some(fileref_glkobj));
        let fileref = lock!(fileref_glkobj);
        FileStream {
            binary: fileref.binary,
            changed: false,
            path: fileref.path.clone(),
            str,
        }
    }

    fn expand(&mut self, increase: usize) {
        let end_pos = self.str.pos + increase;
        let mut max_len = self.str.buf.len();
        if end_pos > max_len {
            // Expand the vec by at least 100
            max_len += max(end_pos - max_len, 100);
            let mut buf = mem::take(&mut self.str.buf).into_vec();
            buf.resize(max_len, T::default());
            self.str.buf = buf.into_boxed_slice();
        }
        self.str.expand(increase);
    }

    pub fn to_file_buffer(&self) -> Box<[u8]> {
        self.str.buf.to_file_buffer(self.binary, self.str.len)
    }
}

impl<T> StreamOperations for FileStream<T>
where T: Clone + Default, Box<[T]>: GlkArray {
    fn close(&self) -> StreamResultCounts {
        self.str.close()
    }

    fn file_path(&self) -> GlkResult<&CString> {
        self.str.file_path()
    }

    fn get_buffer(&mut self, buf: &mut GlkBufferMut) -> GlkResult<u32> {
        self.str.get_buffer(buf)
    }

    fn get_char(&mut self, uni: bool) -> GlkResult<i32> {
        self.str.get_char(uni)
    }

    fn get_line(&mut self, buf: &mut GlkBufferMut) -> GlkResult<u32> {
        self.str.get_line(buf)
    }

    fn get_position(&self) -> u32 {
        self.str.get_position()
    }

    fn put_buffer(&mut self, buf: &GlkBuffer) -> GlkResult<()> {
        self.changed = true;
        if self.str.pos + buf.len() > self.str.len {
            self.expand(buf.len());
        }
        self.str.put_buffer(buf)
    }

    fn put_char(&mut self, ch: u32) -> GlkResult<()> {
        self.changed = true;
        if self.str.pos == self.str.len {
            self.expand(1);
        }
        self.str.put_char(ch)
    }

    fn put_string(&mut self, str: &str, _style: Option<u32>) -> GlkResult<()> {
        let buf: GlkOwnedBuffer = str.into();
        self.put_buffer(&(&buf).into())
    }

    fn set_position(&mut self, mode: SeekMode, pos: i32) {
        self.str.set_position(mode, pos);
    }

    fn write_count(&self) -> usize {
        self.str.write_count
    }
}

/** A NullStream is only used for a memory stream with no buffer */
#[derive(Default)]
pub struct NullStream {
    write_count: usize,
}

impl StreamOperations for NullStream {
    fn put_buffer(&mut self, buf: &GlkBuffer) -> GlkResult<()> {
        self.write_count += buf.len();
        Ok(())
    }

    fn put_char(&mut self, _: u32) -> GlkResult<()> {
        self.write_count += 1;
        Ok(())
    }

    fn put_string(&mut self, str: &str, _style: Option<u32>) -> GlkResult<()> {
        self.write_count += str.chars().count();
        Ok(())
    }

    fn write_count(&self) -> usize {
        self.write_count
    }
}

/** A window stream */
#[derive(Default)]
pub struct WindowStream {
    win: GlkWindowWeak,
    write_count: usize,
}

impl WindowStream {
    pub fn new(win: &GlkWindow) -> Self {
        WindowStream {
            win: win.downgrade(),
            ..Default::default()
        }
    }
}

impl StreamOperations for WindowStream {
    fn put_buffer(&mut self, buf: &GlkBuffer) -> GlkResult<()> {
        let win: GlkWindow = (&self.win).into();
        let mut win = win.lock().unwrap();
        if let Some(TextInputType::Line) = win.input.text_input_type {
            return Err(PendingLineInput);
        }
        self.write_count += buf.len();
        win.put_string(&buf.to_string(), None);
        if let Some(str) = &win.echostr {
            let str: GlkStream = str.into();
            str.lock().unwrap().put_buffer(buf)?;
        }
        Ok(())
    }

    fn put_char(&mut self, ch: u32) -> GlkResult<()> {
        let win: GlkWindow = (&self.win).into();
        let mut win = win.lock().unwrap();
        if let Some(TextInputType::Line) = win.input.text_input_type {
            return Err(PendingLineInput);
        }
        self.write_count += 1;
        win.put_string(&char::from_u32(ch).unwrap().to_string(), None);
        if let Some(str) = &win.echostr {
            let str: GlkStream = str.into();
            str.lock().unwrap().put_char(ch)?;
        }
        Ok(())
    }

    fn put_string(&mut self, str: &str, style: Option<u32>) -> GlkResult<()> {
        let win: GlkWindow = (&self.win).into();
        let mut win = win.lock().unwrap();
        if let Some(TextInputType::Line) = win.input.text_input_type {
            return Err(PendingLineInput);
        }
        self.write_count += str.chars().count();
        win.put_string(str, style);
        if let Some(echostr) = &win.echostr {
            let echostr: GlkStream = echostr.into();
            echostr.lock().unwrap().put_string(str, style)?;
        }
        Ok(())
    }

    fn set_css(&self, name: &str, val: Option<&CSSValue>) {
        let win: GlkWindow = (&self.win).into();
        let mut win = win.lock().unwrap();
        win.set_css(name, val);
        if let Some(str) = &win.echostr {
            let str: GlkStream = str.into();
            str.lock().unwrap().set_css(name, val);
        }
    }

    fn set_hyperlink(&self, val: u32) {
        let win: GlkWindow = (&self.win).into();
        let mut win = win.lock().unwrap();
        win.set_hyperlink(val);
        if let Some(str) = &win.echostr {
            let str: GlkStream = str.into();
            str.lock().unwrap().set_hyperlink(val);
        }
    }

    fn set_style(&self, val: u32) {
        let win: GlkWindow = (&self.win).into();
        let mut win = win.lock().unwrap();
        win.set_style(val);
        if let Some(str) = &win.echostr {
            let str: GlkStream = str.into();
            str.lock().unwrap().set_style(val);
        }
    }

    fn write_count(&self) -> usize {
        self.write_count
    }
}