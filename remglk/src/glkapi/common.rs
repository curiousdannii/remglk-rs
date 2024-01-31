/*

Common things
=============

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GlkApiError {
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
    #[error("window has pending line input")]
    PendingLineInput,
    #[error("cannot read from write-only stream")]
    ReadFromWriteOnly,
    #[error("splitwin must be null for first window")]
    SplitMustBeNull,
    #[error("cannot write to read-only stream")]
    WriteToReadOnly,
}
pub type GlkResult<'a, T> = Result<T, GlkApiError>;