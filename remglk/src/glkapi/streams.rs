/*

Glk Streams
===========

Copyright (c) 2025 Dannii Willis
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
    ArrayBacked(ArrayBackedStream),
    FileStream(FileStream),
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

/** A stream operation */
pub enum StreamOperation<'a> {
    GetBuffer(&'a mut GlkBufferMut<'a>),
    GetChar(bool),
    GetLine(&'a mut GlkBufferMut<'a>),
    GetPosition,
    PutBuffer(&'a GlkBuffer<'a>),
    PutChar(u32),
    PutString(&'a str, Option<u32>),
    SetPosition(SeekMode, i32),
}
use StreamOperation::*;

#[enum_dispatch(Stream)]
pub trait StreamOperations {
    fn close(&self) -> StreamResultCounts {
        StreamResultCounts {
            read_count: 0,
            write_count: self.write_count() as u32,
        }
    }
    fn do_operation(&mut self, op: StreamOperation) -> GlkResult<'_, i32>;
    fn file_path(&self) -> GlkResult<'_, &CString> {Err(NotFileStream)}
    fn write_count(&self) -> usize;
}

/** A fixed-length stream based on a buffer (a boxed slice).
    ArrayBackedStreams are used for memory and resource streams, and are the basis of file streams.
*/
pub struct ArrayBackedStream {
    buf: GlkOwnedBuffer,
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

impl ArrayBackedStream {
    pub fn new(buf: GlkOwnedBuffer, fmode: FileMode, fileref: Option<&GlkFileRef>) -> ArrayBackedStream {
        let buf_len = buf.len();
        let path = fileref.map(|fileref| {
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

    pub fn take_buffer(&mut self) -> GlkOwnedBuffer {
        mem::take(&mut self.buf)
    }
}

impl StreamOperations for ArrayBackedStream {
    fn close(&self) -> StreamResultCounts {
        StreamResultCounts {
            read_count: self.read_count as u32,
            write_count: self.write_count as u32,
        }
    }

    fn do_operation(&mut self, op: StreamOperation) -> GlkResult<'_, i32> {
        // Check file mode first
        match &op {
            GetBuffer(_) | GetChar(_) | GetLine(_) => {
                if let FileMode::Write | FileMode::WriteAppend = self.fmode {
                    return Err(ReadFromWriteOnly);
                }
            },
            PutBuffer(_) | PutChar(_) | PutString(_, _) => {
                if let FileMode::Read = self.fmode {
                    return Err(WriteToReadOnly);
                }
            },
            _ => {},
        };

        match op {
            GetBuffer(buf) => {
                let read_length = min(buf.len(), self.len - self.pos);
                if read_length == 0 {
                    return Ok(0);
                }
                self.buf.copy_to_buffer(self.pos, buf, 0, read_length);
                self.pos += read_length;
                self.read_count += read_length;
                Ok(read_length as i32)
            },
            GetChar(uni) =>{
                if self.pos < self.len {
                    let ch = self.buf.get_u32(self.pos);
                    self.pos += 1;
                    self.read_count += 1;
                    return Ok(if !uni && ch > MAX_LATIN1 {QUESTION_MARK} else {ch} as i32);
                }
                Ok(-1)
            },
            GetLine(buf) => {
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
                Ok(i as i32)
            },
            GetPosition => {
                Ok(self.pos as i32)
            },
            PutBuffer(buf) => {
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
                Ok(0)
            },
            PutChar(ch) => {
                self.write_count += 1;
                if self.pos == self.len && self.expandable {
                    self.expand(1);
                }
                if self.pos < self.len {
                    self.buf.set_u32(self.pos, ch);
                    self.pos += 1;
                }
                Ok(0)
            },
            PutString(str, _style) => {
                let buf: GlkOwnedBuffer = Into::<GlkOwnedBuffer>::into(str);
                self.do_operation(PutBuffer(&(&buf).into()))
            },
            SetPosition(mode, pos) => {
                let new_pos: i32 = match mode {
                    SeekMode::Current => self.pos as i32 + pos,
                    SeekMode::End => self.len as i32 + pos,
                    SeekMode::Start => pos,
                };
                self.pos = new_pos.clamp(0, self.len as i32) as usize;
                Ok(0)
            },
        }
    }

    fn file_path(&self) -> GlkResult<'_, &CString> {
        self.path.as_ref().ok_or(NotFileStream)
    }

    fn write_count(&self) -> usize {
        self.write_count
    }
}

/** Writable FileStreams are based on array backed streams, but can grow in length.
    Read-only file streams just use an ArrayBackedStream directly.
*/
pub struct FileStream {
    binary: bool,
    pub changed: bool,
    pub path: String,
    str: ArrayBackedStream,
}

impl FileStream {
    pub fn new(fileref: &GlkFileRef, buf: GlkOwnedBuffer, fmode: FileMode) -> FileStream {
        debug_assert!(fmode != FileMode::Read);
        let str = ArrayBackedStream::new(buf, fmode, Some(fileref));
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
            self.str.buf.resize(max_len);
        }
        self.str.expand(increase);
    }

    pub fn to_file_buffer(&self) -> Box<[u8]> {
        self.str.buf.to_file_buffer(self.binary, self.str.len)
    }
}

impl StreamOperations for FileStream {
    fn close(&self) -> StreamResultCounts {
        self.str.close()
    }

    fn do_operation(&mut self, op: StreamOperation) -> GlkResult<'_, i32> {
        match op {
            PutBuffer(buf) => {
                self.changed = true;
                if self.str.pos + buf.len() > self.str.len {
                    self.expand(buf.len());
                }
            },
            PutChar(_) => {
                self.changed = true;
                if self.str.pos == self.str.len {
                    self.expand(1);
                }
            },
            PutString(str, _) => {
                let buf: GlkOwnedBuffer = Into::<GlkOwnedBuffer>::into(str);
                return self.do_operation(PutBuffer(&(&buf).into()))
            },
            SetPosition(mode, pos) => {
                // Despite the Glk spec saying that it's illegal to specify a position after the end of a file (5.4) this is needed by Bocfel, and seemingly supported by all libc based Glk interpreters, so we might need to expand first
                // See https://github.com/iftechfoundation/ifarchive-if-specs/issues/17
                let new_pos = match mode {
                    SeekMode::Current => self.str.pos as i32 + pos,
                    SeekMode::End => self.str.len as i32 + pos,
                    SeekMode::Start => pos,
                } as usize;
                if new_pos > self.str.len {
                    self.expand(new_pos - self.str.len);
                }
            },
            _ => {},
        }
        self.str.do_operation(op)
    }

    fn file_path(&self) -> GlkResult<'_, &CString> {
        self.str.file_path()
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
    fn do_operation(&mut self, op: StreamOperation) -> GlkResult<'_, i32> {
        match op {
            PutBuffer(buf) => self.write_count += buf.len(),
            PutChar(_) => self.write_count += 1,
            PutString(str, _) => self.write_count += str.chars().count(),
            _ => {},
        };
        Ok(if let GetChar(_) = op {-1} else {0})
    }

    fn write_count(&self) -> usize {
        self.write_count
    }
}

/** A window stream */
#[derive(Default)]
pub struct WindowStream {
    pub win: GlkWindowWeak,
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
    fn do_operation(&mut self, op: StreamOperation) -> GlkResult<'_, i32> {
        if let GetChar(_) = &op {
            return Ok(-1)
        }
        if let PutBuffer(_) | PutChar(_) | PutString(_, _) = op {
            let win: GlkWindow = (&self.win).into();
            let mut win = win.lock().unwrap();
            if let Some(TextInputType::Line) = win.input.text_input_type {
                return Err(PendingLineInput);
            }
            match op {
                PutBuffer(buf) => {
                    self.write_count += buf.len();
                    win.put_string(&buf.to_string(), None);
                },
                PutChar(ch) => {
                    self.write_count += 1;
                    win.put_string(&char::from_u32(ch).unwrap().to_string(), None);
                },
                PutString(str, style) => {
                    self.write_count += str.chars().count();
                    win.put_string(str, style);
                },
                _ => {},
            };
            if let Some(str) = &win.echostr {
                let str: GlkStream = str.into();
                str.lock().unwrap().do_operation(op)?;
            }
        }
        Ok(0)
    }

    fn write_count(&self) -> usize {
        self.write_count
    }
}