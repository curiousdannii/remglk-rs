/*

The Blorb API
=============

Copyright (c) 2026 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use remglk::blorb::*;
use remglk::blorb::constants::*;

use super::*;
use crate::common::*;
use glkapi::*;

type BlorbMapPtr = *mut BlorbMap;


#[repr(C)]
pub struct BlorbChunkResultC {
    pub chunknum: u32,
    pub data: BlorbResultDataC,
    pub length: u32,
    pub chunktype: u32,
}
#[repr(C)]
pub union BlorbResultDataC {
    data: *const u8,
    startpos: u32,
}

fn map_blorb_res(res: BlorbResult<()>) -> u32 {
    match res {
        Ok(_) => giblorb_err_None,
        Err(err) => err,
    }
}

fn map_blorb_chunk_result(res: BlorbChunkResult) -> BlorbChunkResultC {
    BlorbChunkResultC {
        chunknum: res.chunknum,
        chunktype: res.chunktype,
        data: match res.data {
            BlorbResultData::Data(data) => BlorbResultDataC {data: data.as_ptr()},
            BlorbResultData::Startpos(startpos) => BlorbResultDataC {startpos},
        },
        length: res.length,
    }
}

#[no_mangle]
pub extern "C" fn giblorb_count_resources(map: BlorbMapPtr, usage: u32, num_ptr: *mut u32, min_ptr: *mut u32, max_ptr: *mut u32) -> u32 {
    let map = unsafe {&mut (*map)};
    match map.count_resources(usage) {
        Ok((num, min, max)) => {
            write_ptr(num_ptr, num);
            write_ptr(min_ptr, min);
            write_ptr(max_ptr, max);
            giblorb_err_None
        }
        Err(err) => err,
    }
}

#[no_mangle]
pub extern "C" fn giblorb_create_map(str: StreamPtr, newmap: BlorbMapPtr) -> u32 {
    match GlkApi::giblorb_create_map(from_ptr(str)) {
        Ok(map) => {
            write_ptr(newmap, map);
            giblorb_err_None
        },
        Err(err) => err,
    }
}

#[no_mangle]
pub extern "C" fn giblorb_destroy_map(map: BlorbMapPtr) -> u32 {
    let map = unsafe{Box::from_raw(map)};
    drop(map);
    return giblorb_err_None;
}

#[no_mangle]
pub extern "C" fn giblorb_get_resource_map() -> BlorbMapPtr {
    borrow_any_opt(GLKAPI.lock().unwrap().giblorb_get_resource_map())
}

#[no_mangle]
pub extern "C" fn giblorb_load_chunk_by_number(map: BlorbMapPtr, method: u32, resptr: *mut BlorbChunkResultC, chunknum: u32) -> u32 {
    let map = unsafe {&mut (*map)};
    match map.load_chunk_by_number(method, chunknum as usize) {
        Ok(res) => {
            write_ptr(resptr, map_blorb_chunk_result(res));
            giblorb_err_None
        }
        Err(err) => err,
    }
}

#[no_mangle]
pub extern "C" fn giblorb_load_chunk_by_type(map: BlorbMapPtr, method: u32, resptr: *mut BlorbChunkResultC, chunktype: u32, count: u32) -> u32 {
    let map = unsafe {&mut (*map)};
    match map.load_chunk_by_type(method, chunktype, count) {
        Ok(res) => {
            write_ptr(resptr, map_blorb_chunk_result(res));
            giblorb_err_None
        }
        Err(err) => err,
    }
}

#[no_mangle]
pub extern "C" fn giblorb_load_resource(map: BlorbMapPtr, method: u32, resptr: *mut BlorbChunkResultC, usage: u32, resnum: u32) -> u32 {
    let map = unsafe {&mut (*map)};
    match map.load_resource(method, usage, resnum) {
        Ok(res) => {
            write_ptr(resptr, map_blorb_chunk_result(res));
            giblorb_err_None
        }
        Err(err) => err,
    }
}

#[no_mangle]
pub extern "C" fn giblorb_set_resource_map(str: StreamPtr) -> u32 {
    map_blorb_res(GLKAPI.lock().unwrap().giblorb_set_resource_map(from_ptr(str)))
}

#[no_mangle]
pub extern "C" fn giblorb_unload_chunk(map: BlorbMapPtr, chunknum: u32) -> u32 {
    let map = unsafe {&mut (*map)};
    map_blorb_res(map.unload_chunk(chunknum as usize))
}

#[no_mangle]
pub extern "C" fn giblorb_unset_resource_map() -> u32 {
    map_blorb_res(GLKAPI.lock().unwrap().giblorb_unset_resource_map())
}