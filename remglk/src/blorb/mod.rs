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

use four_cc::FourCC;
use pb_imgsize as imgsize;
use serde::Deserialize;

use crate::glkapi;
use glkapi::*;

use constants::*;
use iff::*;

pub type BlorbResult<T> = Result<T, u32>;

/** A Blorb map */
pub struct BlorbMap {
    chunks: Vec<BlorbChunk>,
    resources: Vec<ResourceIndex>,
    stream: Option<GlkStreamWeak>,
}

#[derive(Debug, Eq, Hash, PartialEq)]
struct ResourceIndex {
    chunknum: usize,
    number: u32,
    usage: FourCC,
}

struct BlorbChunk {
    /** The FourCC type of the chunk, though for FORM chunks it is the internal type */
    chunktype: FourCC,
    data: Option<Box<[u8]>>,
    description: Option<String>,
    height: u32,
    length: u32,
    /** The offset of the chunk data. Note that for FORM chunks this includes the FORM header */
    offset: u32,
    width: u32,
}

impl BlorbChunk {
    fn new(chunktype: FourCC, length: u32, offset: u32) -> Self {
        Self {
            chunktype,
            data: None,
            description: None,
            height: 0,
            length,
            offset,
            width: 0,
        }
    }
}

#[derive(Deserialize)]
pub struct ResourceMapResource {
    altttext: Option<String>,
    format: String,
    #[serde(default)]
    height: u32,
    id: u32,
    #[serde(default)]
    width: u32,
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
        let mut resources = Vec::new();
        let mut resources_by_offset = Vec::new();
        let mut descriptions = Vec::new();

        // Parse the resource index chunk
        let ridx_chunk =  &iff_chunks[0];
        if ridx_chunk.chunktype != giblorb_ID_RIdx {
            return Err(giblorb_err_Format);
        }
        setpos(&mut str, ridx_chunk.offset_data);
        let count = read4(&mut str);
        for _ in 1..=count {
            let usage = read_four_cc(&mut str);
            let number = read4(&mut str);
            let offset = read4(&mut str);
            resources_by_offset.push((offset, number, usage));
        }

        // Parse other chunks
        for chunk in &iff_chunks {
            if chunk.chunktype == giblorb_ID_RDes {
                setpos(&mut str, chunk.offset_data);
                let count = read4(&mut str);
                for _ in 1..=count {
                    let usage = read_four_cc(&mut str);
                    let number = read4(&mut str);
                    let length = read4(&mut str);
                    let offset = getpos(&mut str);
                    let buf = getbuf(&mut str, offset, length);
                    descriptions.push((number, usage, u8slice_to_string(&buf)));
                }
                break;
            }
        }

        // Loop through one more time to construct the list of chunks
        for (chunknum, iff_chunk) in iff_chunks.iter().enumerate() {
            let mut chunk = BlorbChunk::new(iff_chunk.chunktype, iff_chunk.length, iff_chunk.offset_data);
            // For FORM chunks we set the chunk type to the internal type and the offset and length to the total
            if iff_chunk.chunktype == giblorb_ID_FORM {
                setpos(&mut str, iff_chunk.offset_data);
                chunk.chunktype = read_four_cc(&mut str);
                chunk.length = iff_chunk.length + 8;
                chunk.offset = iff_chunk.offset_header;
            }
            // Check if this chunk is a resource
            for (offset, number, usage) in &resources_by_offset {
                if offset == &iff_chunk.offset_header {
                    resources.push(ResourceIndex {
                        chunknum,
                        number: *number,
                        usage: *usage,
                    });
                    // Look for a description
                    for (desc_number, desc_usage, description) in &descriptions {
                        if desc_number == number && desc_usage == usage {
                            chunk.description = Some(description.clone());
                            break;
                        }
                    }
                    break;
                }
            }
            // Get some metadata
            match iff_chunk.chunktype {
                giblorb_ID_JPEG | giblorb_ID_PNG_ => {
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
            resources,
            stream: Some(str_glkobj.downgrade()),
        })
    }

    /** Make a Blorb handler from Inform 7's new resource map */
    pub fn new_from_resource_map(map: Vec<ResourceMapResource>) -> Self {
        let mut chunks = Vec::new();
        let mut resources = Vec::new();
        for (chunknum, resource) in map.iter().enumerate() {
            let (chunktype, usage) = match resource.format.as_str() {
                "AIFF" => (giblorb_ID_AIFF, giblorb_ID_Snd_),
                "JPEG" => (giblorb_ID_JPEG, giblorb_ID_Pict),
                "MIDI" => (giblorb_ID_MIDI, giblorb_ID_Snd_),
                "MP3" => (giblorb_ID_MP3_, giblorb_ID_Snd_),
                "Ogg Vorbis" => (giblorb_ID_OGGV, giblorb_ID_Snd_),
                "PNG" => (giblorb_ID_PNG_, giblorb_ID_Pict),
                _ => (giblorb_ID_BINA, giblorb_ID_Data),
            };
            chunks.push(BlorbChunk {
                chunktype,
                data: None,
                description: resource.altttext.clone(),
                height: resource.height,
                length: 0,
                offset: 0,
                width: resource.width,
            });
            resources.push(ResourceIndex {
                chunknum,
                number: resource.id,
                usage,
            });
        }
        BlorbMap {
            chunks,
            resources,
            stream: None,
        }
    }

    pub fn count_resources(&self, usage: FourCC) -> BlorbResult<(u32, u32, u32)> {
        let mut count: u32 = 0;
        let mut min: u32 = u32::MAX;
        let mut max: u32 = 0;
        for resource in &self.resources {
            if resource.usage == usage {
                count += 1;
                if resource.number < min {
                    min = resource.number;
                }
                if resource.number > max {
                    max = resource.number;
                }
            }
        }
        Ok((count, min, max))
    }

    pub fn load_chunk_by_number<'a>(&'a mut self, method: u32, chunknum: usize) -> BlorbResult<BlorbChunkResult<'a>> {
        if chunknum > self.chunks.len() {
            return Err(giblorb_err_NotFound);
        }
        if let Some(stream) = &self.stream {
            let chunk = &mut self.chunks[chunknum];
            let data = if method == giblorb_method_Memory {
                let buf = chunk.data.get_or_insert_with(|| {
                    let str_glkobj = Into::<GlkStreamShared>::into(stream);
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
        else {
            Err(giblorb_err_Read)
        }
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
        for resource in &self.resources {
            if resource.usage == usage && resource.number == resnum {
                return self.load_chunk_by_number(method, resource.chunknum);
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
        for resource in &self.resources {
            if resource.usage == giblorb_ID_Pict && resource.number == image {
                let chunk = &self.chunks[resource.chunknum];
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
        for resource in &self.resources {
            if resource.usage == giblorb_ID_Snd_ && resource.number == number {
                return Some(self.chunks[resource.chunknum].chunktype);
            }
        }
        None
    }

    pub fn read_data_resource(&self, number: u32) -> Option<BlorbResourceChunk> {
        if let Some(stream) = &self.stream {
            for resource in &self.resources {
                if resource.usage == giblorb_ID_Data && resource.number == number {
                    let chunk = &self.chunks[resource.chunknum];
                    let str_glkobj = Into::<GlkStreamShared>::into(stream);
                    let mut str = lock!(str_glkobj);
                    let binary = chunk.chunktype != giblorb_ID_TEXT;
                    let data = getbuf(&mut str, chunk.offset, chunk.length);
                    return Some(BlorbResourceChunk {binary, data});
                }
            }
        }
        None
    }
}