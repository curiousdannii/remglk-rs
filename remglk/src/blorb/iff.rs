/*

IFF Parser
==========

Copyright (c) 2026 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use crate::glkapi;
use glkapi::*;
use glkapi::constants::*;
use glkapi::StreamOperation::*;

use super::constants::*;

pub struct IFFChunk {
    pub chunktype: u32,
    pub length: u32,
    pub offset: u32,
}

/** Parse an IFF file from a stream */
pub fn parse_iff(str: &mut GlkStream) -> Result<Vec<IFFChunk>, u32> {

    setpos(str, 0);
    if read4(str) != giblorb_ID_FORM {
        return Err(giblorb_err_Format)
    }
    let length = read4(str);
    _ = read4(str);

    let mut chunks = Vec::new();
    while getpos(str) <= length {
        let offset = getpos(str);
        let chunktype = read4(str);
        let length = read4(str);
        chunks.push(IFFChunk {
            chunktype,
            length,
            offset,
        });
        let newpos = offset + length + (if length % 2 > 0 {1} else {0});
        setpos(str, newpos);
    }
    
    Ok(chunks)
}

pub fn getbuf(str: &mut GlkStream, offset: u32, length: u32) -> GlkOwnedBuffer {
    setpos(str, offset);
    let mut buf = GlkOwnedBuffer::new(false, length as usize);
    let _ = str.do_operation(GetBuffer(&mut (&mut buf).into()));
    buf
}

pub fn getpos(str: &mut GlkStream) -> u32 {
    str.do_operation(GetPosition).unwrap() as u32
}

pub fn setpos(str: &mut GlkStream, pos: u32) {
    str.do_operation(SetPosition(SeekMode::Start, pos as i32)).unwrap();
}

pub fn read4(str: &mut GlkStream) -> u32 {
    ((str.do_operation(GetChar(false)).unwrap() as u32) << 24)
        | ((str.do_operation(GetChar(false)).unwrap() as u32) << 16)
        | ((str.do_operation(GetChar(false)).unwrap() as u32) << 8)
        | (str.do_operation(GetChar(false)).unwrap() as u32)
}