/*

IFF Parser
==========

Copyright (c) 2026 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use four_cc::FourCC;

use crate::glkapi;
use glkapi::*;
use glkapi::constants::*;
use glkapi::StreamOperation::*;

use super::constants::*;

pub struct IFFChunk {
    pub chunktype: FourCC,
    pub length: u32,
    /** The offset of the chunk data */
    pub offset_data: u32,
    /** The offset of the chunk header */
    pub offset_header: u32,
}

/** Parse an IFF file from a stream */
pub fn parse_iff(str: &mut GlkStream) -> Result<Vec<IFFChunk>, u32> {

    setpos(str, 0);
    if read_four_cc(str) != giblorb_ID_FORM {
        return Err(giblorb_err_Format)
    }
    let _form_length = read4(str);
    _ = read4(str);

    // ADRIFT 5 Developer writes Blorbs whose FORM length undercounts the chunks
    // actually present; walk the physical stream length instead (as AsyncGlk does).
    str.do_operation(SetPosition(SeekMode::End, 0)).unwrap();
    let filelength = getpos(str);
    setpos(str, 12);

    let mut chunks = Vec::new();
    while getpos(str) < filelength {
        let offset_header = getpos(str);
        let chunktype = read_four_cc(str);
        let length = read4(str);
        chunks.push(IFFChunk {
            chunktype,
            length,
            offset_data: offset_header + 8,
            offset_header,
        });
        let newpos = offset_header + 8 + length + (length % 2);
        if newpos > filelength + 1 {
            return Err(giblorb_err_Format);
        }
        setpos(str, newpos);
    }
    
    Ok(chunks)
}

pub fn getbuf(str: &mut GlkStream, offset: u32, length: u32) -> Box<[u8]> {
    setpos(str, offset);
    let mut buf = GlkOwnedBuffer::new(false, length as usize);
    let _ = str.do_operation(GetBuffer(&mut (&mut buf).into()));
    match buf {
        GlkOwnedBuffer::U8(buf) => buf,
        GlkOwnedBuffer::U32(_) => unreachable!(),
    }
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

pub fn read_four_cc(str: &mut GlkStream) -> FourCC {
    FourCC::from(read4(str))
}