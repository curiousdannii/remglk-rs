/*

Common things
=============

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::io;
use std::str;

use byteorder::{BigEndian, ReadBytesExt};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GlkApiError {
    #[error("cannot change window split direction")]
    CannotChangeWindowSplitDirection,
    #[error("cannot close window stream")]
    CannotCloseWindowStream,
    #[error("illegal filemode")]
    IllegalFilemode,
    #[error("invalid reference")]
    InvalidReference,
    #[error("invalid splitwin")]
    InvalidSplitwin,
    #[error("invalid method: bad direction")]
    InvalidWindowDirection,
    #[error("invalid method: must be fixed or proportional")]
    InvalidWindowDivision,
    #[error("invalid method: blank windows cannot be only be split proportionally")]
    InvalidWindowDivisionBlank,
    #[error("invalid wintype")]
    InvalidWindowType,
    #[error("invalid keywin: can't be a pair window")]
    KeywinCantBePair,
    #[error("keywin must be a descendant")]
    KeywinMustBeDescendant,
    #[error("no current stream")]
    NoCurrentStream,
    #[error("invalid window: not a grid window")]
    NotGridWindow,
    #[error("invalid window: not a pair window")]
    NotPairWindow,
    #[error("window already has keyboard request")]
    PendingKeyboardRequest,
    #[error("window has pending line input")]
    PendingLineInput,
    #[error("cannot read from write-only stream")]
    ReadFromWriteOnly,
    #[error("splitwin must be null for first window")]
    SplitMustBeNull,
    #[error("window doesn't support character input")]
    WindowDoesntSupportCharInput,
    #[error("cannot write to read-only stream")]
    WriteToReadOnly,
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Utf8(#[from] str::Utf8Error),
}
pub type GlkResult<'a, T> = Result<T, GlkApiError>;

macro_rules! current_stream {
    ($self: expr) => {
        $self.current_stream.as_ref().map(|str| Into::<GlkStream>::into(str)).as_ref().ok_or(NoCurrentStream)?
    };
}
pub(crate) use current_stream;

macro_rules! lock {
    ($str: expr) => {
        $str.lock().unwrap()
    }
}
pub(crate) use lock;

macro_rules! write_stream {
    ($self: expr, $str: expr) => {
        match $str.deref() {
            Stream::FileStreamU8(str) => $self.system.fileref_write(&str.fileref, GlkBuffer::U8(str.get_buf()))?,
            Stream::FileStreamU32(str) => $self.system.fileref_write(&str.fileref, GlkBuffer::U32(str.get_buf()))?,
            _ => {},
        };
    };
}
pub(crate) use write_stream;

// Array conversions
pub fn u8slice_to_u32vec(buf: &[u8]) -> Vec<u32> {
    let mut curs = io::Cursor::new(buf);
    let mut dest: Vec<u32> = vec![];
    let _ = curs.read_u32_into::<BigEndian>(&mut dest);
    dest
}

// From https://codereview.stackexchange.com/a/250318/52143
pub fn u32slice_to_u8vec(buf: &[u32]) -> Vec<u8> {
    let mut dest = Vec::with_capacity(buf.len() * 4);
    for val in buf {
        dest.extend(&val.to_be_bytes());
    }
    dest
}