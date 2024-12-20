/*

Common things
=============

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::io;
use std::str;

use byteorder::{BigEndian, ByteOrder};
use thiserror::Error;
use widestring::Utf32String;

use super::*;

pub const MAX_LATIN1: u32 = 0xFF;
pub const QUESTION_MARK: u32 = '?' as u32;

#[derive(Debug, Error)]
pub enum GlkApiError {
    #[error("cannot change window split direction")]
    CannotChangeWindowSplitDirection,
    #[error("cannot close window stream")]
    CannotCloseWindowStream,
    #[error("event not supported")]
    EventNotSupported,
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
    #[error("invalid stream: not a file stream")]
    NotFileStream,
    #[error("invalid window: not a graphics window")]
    NotGraphicsWindow,
    #[error("invalid window: not a grid window")]
    NotGridWindow,
    #[error("invalid window: not a pair window")]
    NotPairWindow,
    #[error("outspacing must be zero")]
    OutspacingMustBeZero,
    #[error("window already has keyboard request")]
    PendingKeyboardRequest,
    #[error("window has pending line input")]
    PendingLineInput,
    #[error("cannot read from write-only stream")]
    ReadFromWriteOnly,
    #[error("splitwin must be null for first window")]
    SplitMustBeNull,
    #[error("invalid splitwin: split window's parent isn't a pair window")]
    SplitParentIsntPair,
    #[error("window doesn't support character input")]
    WindowDoesntSupportCharInput,
    #[error("window doesn't support line input")]
    WindowDoesntSupportLineInput,
    #[error("cannot write to read-only stream")]
    WriteToReadOnly,
    #[error("event has wrong generation number: expected {0}, received {1}")]
    WrongGeneration(u32, u32),
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
        // We don't want to actually wait for a Mutex to be unlocked, so call try_lock
        $str.try_lock().unwrap()
    }
}
pub(crate) use lock;

pub fn write_common_buffer(src: &[u32], dest: &mut [u32]) -> usize {
    let len = src.len();
    let act_len = min(len, dest.len());
    dest[..act_len].copy_from_slice(&src[..act_len]);
    len
}

// Array & string conversions

pub fn str_to_u32vec(str: &str) -> Vec<u32> {
    let str = Utf32String::from_str(str);
    str.into_vec()
}

pub fn u8slice_to_string(buf: &[u8]) -> String {
    buf.iter().map(|&c| c as char).collect()
}

pub fn u8slice_to_u32vec(buf: &[u8]) -> Vec<u32> {
    assert!(buf.len() % 4 == 0, "buffer length not multiple of 4");
    let mut dest = Vec::with_capacity(buf.len() / 4);
    for i in (0..buf.len()).step_by(4) {
        dest.push(BigEndian::read_u32(&buf[i..]));
    }
    dest
}

pub fn u32slice_to_string(buf: &[u32]) -> String {
    buf.iter().map(|&c| char::from_u32(c).unwrap()).collect()
}

// From https://codereview.stackexchange.com/a/250318/52143
pub fn u32slice_to_u8vec(buf: &[u32]) -> Vec<u8> {
    let mut dest = Vec::with_capacity(buf.len() * 4);
    for val in buf {
        dest.extend(&val.to_be_bytes());
    }
    dest
}