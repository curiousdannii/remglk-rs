/*

Common things
=============

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::io;
use std::str;

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
    ($str: expr) => {
        lock!($str.current_stream.as_ref().map(|str| Into::<GlkStream>::into(str)).as_ref().ok_or(NoCurrentStream)?)
    };
}
pub(crate) use current_stream;

macro_rules! lock {
    ($str: expr) => {
        $str.lock().unwrap()
    }
}
pub(crate) use lock;