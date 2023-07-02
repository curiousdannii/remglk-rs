/*

The Glk API
===========

Copyright (c) 2023 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

pub mod arrays;
pub mod constants;
pub mod filerefs;
pub mod protocol;
pub mod streams;

/** A Glk object that will be returned to the main app */
pub struct GlkObject<T> {
    pub disprock: Option<u32>,
    pub inner: T,
    pub rock: u32,
}