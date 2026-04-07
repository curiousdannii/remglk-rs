/*

RemGlk-rs Blorblib
==================

Copyright (c) 2026 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

#![allow(non_upper_case_globals)]

pub mod constants;
mod iff;

use std::collections::HashMap;

use four_cc::FourCC;
use pb_imgsize as imgsize;

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

struct BlorbChunk {
    chunktype: FourCC,
    data: Option<Box<[u8]>>,
    description: Option<String>,
    height: u32,
    length: u32,
    number: u32,
    offset: u32,
    usage: FourCC,
    width: u32,
}

impl BlorbChunk {
    fn new() -> Self {
        Self {
            chunktype: FourCC::from(0),
            data: None,
            description: None,
            height: 0,
            length: 0,
            number: 0,
            offset: 0,
            usage: FourCC::from(0),
            width: 0,
        }
    }
}

/** The result struct for loading a Blorb chunk */
pub struct BlorbChunkResult<'a> {
    pub chunknum: u32,
    pub chunktype: FourCC,
    pub data: BlorbResultData<'a>,
    pub length: u32,
}
pub enum BlorbResultData<'a> {
    Data(&'a [u8]),
    Startpos(u32),
}

pub struct BlorbResourceChunk {
    pub binary: bool,
    pub data: Box<[u8]>,
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
            let usage = read_four_cc(&mut str);
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
                    let usage = read_four_cc(&mut str);
                    let number = read4(&mut str);
                    let length = read4(&mut str);
                    let offset = getpos(&mut str);
                    let buf = getbuf(&mut str, offset, length);
                    descriptions.insert((usage, number), u8slice_to_string(&buf));
                }
                break;
            }
        }

        // Loop through one more time to construct the list of chunks
        for iff_chunk in &iff_chunks {
            let mut chunk = BlorbChunk::new();
            // For FORM chunks we set the chunk type to the internal type, but the offset and length to the total
            if iff_chunk.chunktype == giblorb_ID_FORM {
                setpos(&mut str, chunk.offset + 8);
                chunk.chunktype = read_four_cc(&mut str);
                chunk.length = iff_chunk.length + 8;
                chunk.offset = iff_chunk.offset;
            }
            // For regular chunks we set the offset and length to where the data begins
            else {
                chunk.chunktype = iff_chunk.chunktype;
                chunk.length = iff_chunk.length;
                chunk.offset = iff_chunk.offset + 8;
            }
            if let Some((usage, number)) = resources.remove(&iff_chunk.offset) {
                chunk.number = number;
                chunk.usage = usage;
            }
            chunk.description = descriptions.remove(&(chunk.usage, chunk.number));
            // Get some metadata
            match iff_chunk.chunktype {
                giblorb_ID_JPEG | giblorb_ID_PNG_ => {
                    setpos(&mut str, chunk.offset);
                    let data = getbuf(&mut str, chunk.offset, chunk.length);
                    if let Ok(meta) = imgsize::read_bytes(&data) {
                        chunk.height = meta.height;
                        chunk.width = meta.width;
                    }
                },
                _ => {},
            };
            chunks.push(chunk);
        }

        Ok(BlorbMap {
            chunks,
            stream: str_glkobj.downgrade(),
        })
    }

    pub fn count_resources(&self, usage: FourCC) -> BlorbResult<(u32, u32, u32)> {
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
                getbuf(&mut str, chunk.offset, chunk.length)
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

    pub fn load_chunk_by_type<'a>(&'a mut self, method: u32, chunktype: FourCC, mut count: u32) -> BlorbResult<BlorbChunkResult<'a>> {
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

    pub fn load_resource<'a>(&'a mut self, method: u32, usage: FourCC, resnum: u32) -> BlorbResult<BlorbChunkResult<'a>> {
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

    pub fn get_image_info(&self, image: u32) -> Option<ImageInfo> {
        for chunk in &self.chunks {
            if chunk.usage == giblorb_ID_Pict && chunk.number == image {
                return Some(ImageInfo {
                    alttext: chunk.description.clone(),
                    height: chunk.height,
                    image,
                    width: chunk.width,
                });
            }
        }
        None
    }

    pub fn get_sound_format(&self, number: u32) -> Option<FourCC> {
        for chunk in &self.chunks {
            if chunk.usage == giblorb_ID_Snd_ && chunk.number == number {
                return Some(chunk.chunktype);
            }
        }
        None
    }

    pub fn read_data_resource(&self, number: u32) -> Option<BlorbResourceChunk> {
        for chunk in &self.chunks {
            if chunk.usage == giblorb_ID_Data && chunk.number == number {
                let str_glkobj = Into::<GlkStreamShared>::into(&self.stream);
                let mut str = lock!(str_glkobj);
                let binary = chunk.chunktype != giblorb_ID_TEXT;
                let data = getbuf(&mut str, chunk.offset, chunk.length);
                return Some(BlorbResourceChunk {binary, data});
            }
        }
        None
    }
}