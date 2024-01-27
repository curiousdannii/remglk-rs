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
    #[error("cannot read from write-only stream")]
    ReadFromWriteOnly,
    #[error("cannot write to read-only stream")]
    WriteToReadOnly,
}
pub type GlkResult<'a, T> = Result<T, GlkApiError>;