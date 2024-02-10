/*

Blorbs
======

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

#![allow(non_upper_case_globals)]

use std::ffi::c_char;
use std::mem::MaybeUninit;
use std::slice;

const fn giblorb_make_id(c1: char, c2: char, c3: char, c4: char) -> u32 {
    ((c1 as u32) << 24) | ((c2 as u32) << 16) | ((c3 as u32) << 8) | (c4 as u32)
}
const giblorb_ID_BINA: u32 = giblorb_make_id('B', 'I', 'N', 'A');
const giblorb_ID_Data: u32 = giblorb_make_id('D', 'a', 't', 'a');
const giblorb_ID_FORM: u32 = giblorb_make_id('F', 'O', 'R', 'M');
const giblorb_ID_TEXT: u32 = giblorb_make_id('T', 'E', 'X', 'T');

const giblorb_method_Memory: u32 = 1;

/** An opaque struct representing the Blorb map */
#[repr(C)]
struct BlorbMap {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}
type BlorbMapPtr = *const BlorbMap;

/** A Blorb chunk */
#[repr(C)]
struct BlorbChunk {
    chunknum: u32, /* The chunk number (for use in 
        giblorb_unload_chunk(), etc.) */
    data: *const u8,/* A pointer to the data (if you used 
        giblorb_method_Memory) */
    length: u32, /* The length of the data */
    chunktype: u32, /* The type of the chunk. */
}
type BlorbChunkPtr = *mut BlorbChunk;

pub struct ResourceChunk {
    pub binary: bool,
    pub data: &'static [u8],
}

/** Image information */
#[repr(C)]
struct ImageInfoC {
    chunktype: u32,
    width: u32,
    height: u32,
    alttext: *const c_char,
}
type ImageInfoPtr = *mut ImageInfoC;

#[derive(Debug)]
pub struct ImageInfo {
    pub height: u32,
    pub image: u32,
    pub width: u32,
}

extern "C" {
    fn giblorb_get_resource_map() -> BlorbMapPtr;
    fn giblorb_load_image_info(map: BlorbMapPtr, resnum: u32, res: ImageInfoPtr) -> u32;
    fn giblorb_load_resource(map: BlorbMapPtr, method: u32, res: BlorbChunkPtr, usage: u32, resnum: u32) -> u32;
}

pub fn get_blorb_resource_chunk(filenum: u32) -> Option<ResourceChunk> {
    let map = unsafe{giblorb_get_resource_map()};
    if map.is_null() {
        return None;
    }
    let mut chunk = MaybeUninit::uninit();
    let res = unsafe {giblorb_load_resource(map, giblorb_method_Memory, chunk.as_mut_ptr(), giblorb_ID_Data, filenum)};
    if res > 0 {
        return None;
    }
    let chunk = unsafe {chunk.assume_init()};
    let binary = if chunk.chunktype == giblorb_ID_TEXT {
        false
    }
    else if chunk.chunktype == giblorb_ID_BINA || chunk.chunktype == giblorb_ID_FORM {
        true
    }
    else {
        return None;
    };
    Some(ResourceChunk {
        binary,
        data: unsafe {slice::from_raw_parts(chunk.data, chunk.length as usize)},
    })
}

pub fn get_image_info(image: u32) -> Option<ImageInfo> {
    let map = unsafe{giblorb_get_resource_map()};
    if map.is_null() {
        return None;
    }
    let mut info = MaybeUninit::uninit();
    let res = unsafe{giblorb_load_image_info(map, image, info.as_mut_ptr())};
    if res > 0 {
        return None;
    }
    let info = unsafe {info.assume_init()};
    Some(ImageInfo {
        height: info.height,
        image,
        width: info.width,
    })
}