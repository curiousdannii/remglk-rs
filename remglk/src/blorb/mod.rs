/*

RemGlk-rs Blorblib
==================

Copyright (c) 2026 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

pub mod constants;
mod iff;

use std::collections::HashMap;

use crate::glkapi;
use glkapi::*;
use StreamOperation::*;

use constants::*;
use iff::*;

/** A Blorb map */
pub struct BlorbMap {
    chunks: Vec<BlorbChunk>,
    stream: GlkStreamShared,
}

#[derive(Default)]
struct BlorbChunk {
    chunktype: u32,
    description: Option<String>,
    length: u32,
    number: u32,
    offset: u32,
    usage: u32,
}

/** The result struct for loading a Blorb chunk */
pub struct BlorbResult<'a> {
    pub chunknum: u32,
    pub data: BlorbResultData<'a>,
    pub length: u32,
    pub chunktype: u32,
}
pub enum BlorbResultData<'a> {
    Data(&'a [u8]),
    Startpos(u32),
}

/** Image information */
pub struct ImageInfo {
    chunktype: u32,
    width: u32,
    height: u32,
    alttext: String,
}

impl BlorbMap {
    /** Process a Blorb read from a stream */
    pub fn new(str_glkobj: GlkStreamShared) -> Result<Self, u32> {
        let mut str = lock!(str_glkobj);

        let iff_chunks = parse_iff(&mut str)?;
        let mut chunks = Vec::new();
        let mut resources = HashMap::new();
        let mut descriptions = HashMap::new();

        // Parse the resource index chunk
        let ridx_chunk =  &iff_chunks[0];
        if ridx_chunk.chunktype != giblorb_ID_RIdx {
            return Err(giblorb_err_Format);
        }
        setpos(&mut str, ridx_chunk.offset + 8);
        let count = read4(&mut str);
        for _ in 1..=count {
            let usage = read4(&mut str);
            let number = read4(&mut str);
            let start = read4(&mut str);
            resources.insert(start, (usage, number));
        }

        // Parse other chunks
        for chunk in &iff_chunks {
            if chunk.chunktype == giblorb_ID_RDes {
                setpos(&mut str, chunk.offset + 8);
                let count = read4(&mut str);
                for _ in 1..=count {
                    let usage = read4(&mut str);
                    let number = read4(&mut str);
                    let length = read4(&mut str) as usize;
                    let mut buf = GlkOwnedBuffer::new(false, length);
                    str.do_operation(GetBuffer(&mut (&mut buf).into()));
                    descriptions.insert((usage, number), buf.to_string(length));
                }
                break;
            }
        }

        // Loop through one more time to construct the list of chunks
        for iff_chunk in &iff_chunks {
            let mut chunk = BlorbChunk::default();
            if iff_chunk.chunktype == giblorb_ID_FORM {
                setpos(&mut str, chunk.offset + 8);
                chunk.chunktype = read4(&mut str);
                chunk.length = iff_chunk.length;
                chunk.offset = iff_chunk.offset;
            }
            else {
                chunk.chunktype = iff_chunk.chunktype;
                chunk.length = iff_chunk.length - 8;
                chunk.offset = iff_chunk.offset + 8;
            }
            if let Some((usage, number)) = resources.remove(&iff_chunk.offset) {
                chunk.number = number;
                chunk.usage = usage;
            }
            chunk.description = descriptions.remove(&(chunk.usage, chunk.number));
            chunks.push(chunk);
        }

        Ok(BlorbMap {
            chunks,
            stream: str_glkobj.clone(),
        })
    }
}