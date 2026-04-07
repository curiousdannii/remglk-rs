/*

RemGlk-rs Blorblib
==================

Copyright (c) 2026 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

pub mod constants;
mod iff;

use std::{collections::HashMap, u32};

use crate::glkapi;
use glkapi::*;

use constants::*;
use iff::*;

pub type BlorbResult<T> = Result<T, u32>;

/** A Blorb map */
pub struct BlorbMap {
    chunks: Vec<BlorbChunk>,
    stream: GlkStreamWeak,
}

#[derive(Default)]
struct BlorbChunk {
    chunktype: u32,
    data: Option<Box<[u8]>>,
    description: Option<String>,
    length: u32,
    number: u32,
    offset: u32,
    usage: u32,
}

/** The result struct for loading a Blorb chunk */
pub struct BlorbChunkResult<'a> {
    pub chunknum: u32,
    pub chunktype: u32,
    pub data: BlorbResultData<'a>,
    pub length: u32,
}
pub enum BlorbResultData<'a> {
    Data(&'a [u8]),
    Startpos(u32),
}

pub struct BlorbResourceChunk {
    pub binary: bool,
    pub data: &'static [u8],
}

/** Image information */
pub struct ImageInfo {
    pub alttext: Option<String>,
    pub height: u32,
    pub image: u32,
    pub width: u32,
}

impl BlorbMap {
    /** Process a Blorb read from a stream */
    pub fn new(str_glkobj: GlkStreamShared) -> BlorbResult<Self> {
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
                    let length = read4(&mut str);
                    let offset = getpos(&mut str);
                    let buf = getbuf(&mut str, offset, length);
                    descriptions.insert((usage, number), buf.to_string(length as usize));
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
            stream: str_glkobj.downgrade(),
        })
    }

    pub fn count_resources(&self, usage: u32) -> BlorbResult<(u32, u32, u32)> {
        let mut count: u32 = 0;
        let mut min: u32 = u32::MAX;
        let mut max: u32 = 0;
        for chunk in &self.chunks {
            if chunk.usage == usage {
                count += 1;
                if chunk.number < min {
                    min = chunk.number;
                }
                if chunk.number > max {
                    max = chunk.number;
                }
            }
        }
        Ok((count, min, max))
    }

    pub fn load_chunk_by_number<'a>(&'a mut self, method: u32, chunknum: usize) -> BlorbResult<BlorbChunkResult<'a>> {
        if chunknum > self.chunks.len() {
            return Err(giblorb_err_NotFound);
        }
        let chunk = &mut self.chunks[chunknum];
        let data = if method == giblorb_method_Memory {
            let buf = chunk.data.get_or_insert_with(|| {
                let str_glkobj = Into::<GlkStreamShared>::into(&self.stream);
                let mut str = lock!(str_glkobj);
                let buf = getbuf(&mut str, chunk.offset, chunk.length);
                match buf {
                    GlkOwnedBuffer::U8(buf) => buf,
                    GlkOwnedBuffer::U32(_) => unreachable!(),
                }
            });
            BlorbResultData::Data(buf)
        }
        else {
            BlorbResultData::Startpos(chunk.offset)
        };
        Ok(BlorbChunkResult {
            chunknum: chunknum as u32,
            data,
            length: chunk.length,
            chunktype: chunk.chunktype,
        })
    }

    pub fn load_chunk_by_type<'a>(&'a mut self, method: u32, chunktype: u32, mut count: u32) -> BlorbResult<BlorbChunkResult<'a>> {
        for (i, chunk) in self.chunks.iter().enumerate() {
            if chunk.chunktype != chunktype {
                continue;
            }
            if count > 0 {
                count -= 1;
                continue;
            }
            return self.load_chunk_by_number(method, i);
        }
        Err(giblorb_err_NotFound)
    }

    pub fn load_resource<'a>(&'a mut self, method: u32, usage: u32, resnum: u32) -> BlorbResult<BlorbChunkResult<'a>> {
        for (i, chunk) in self.chunks.iter().enumerate() {
            if chunk.usage == usage && chunk.number == resnum {
                return self.load_chunk_by_number(method, i);
            }
        }
        Err(giblorb_err_NotFound)
    }

    pub fn unload_chunk(&mut self, chunknum: usize) -> BlorbResult<()> {
        if chunknum > self.chunks.len() {
            return Err(giblorb_err_NotFound);
        }
        let chunk = &mut self.chunks[chunknum];
        chunk.data = None;
        Ok(())
    }

    // And some helper functions

    pub fn get_blorb_resource(&self, usage: u32, resnum: u32) -> Option<&'static [u8]> {
        todo!()
    }

    pub fn get_data_resource(&self, resnum: u32) -> Option<BlorbResourceChunk> {
        todo!()
    }

    pub fn get_image_info(&self, image: u32) -> Option<ImageInfo> {
        todo!()
    }
}