/*

GiDispatch functions
====================

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::mem;

use libc::c_void;
use remglk::glkapi::*;
use objects::*;
use std::sync::Mutex;

use super::*;
use common::*;
use glkapi::*;

pub type RegisterCallbackGeneric = extern fn(*const c_void, u32) -> DispatchRock;
pub type UnregisterCallbackGeneric = extern fn(*const c_void, u32, DispatchRock);

#[no_mangle]
pub unsafe extern "C" fn gidispatch_set_object_registry(register_cb: RegisterCallbackGeneric, unregister_cb: UnregisterCallbackGeneric) {
    let mut glkapi = glkapi().lock().unwrap();
    let register = mem::transmute::<RegisterCallbackGeneric, DispatchRegisterCallback<FileRef>>(register_cb);
    let unregister = mem::transmute::<UnregisterCallbackGeneric, DispatchUnregisterCallback<FileRef>>(unregister_cb);
    glkapi.filerefs.set_callbacks(register, unregister);
    let register = mem::transmute::<RegisterCallbackGeneric, DispatchRegisterCallback<Stream>>(register_cb);
    let unregister = mem::transmute::<UnregisterCallbackGeneric, DispatchUnregisterCallback<Stream>>(unregister_cb);
    glkapi.streams.set_callbacks(register, unregister);
    let register = mem::transmute::<RegisterCallbackGeneric, DispatchRegisterCallback<Window>>(register_cb);
    let unregister = mem::transmute::<UnregisterCallbackGeneric, DispatchUnregisterCallback<Window>>(unregister_cb);
    glkapi.windows.set_callbacks(register, unregister);
}

#[no_mangle]
pub unsafe extern "C" fn gidispatch_get_objrock(ptr: *const c_void, objclass: u32) -> *const DispatchRock {
    match objclass {
        0 => {
            let ptr = mem::transmute::<*const c_void, WindowPtr>(ptr);
            obj_ptr_to_disprock(ptr)
        },
        1 => {
            let ptr = mem::transmute::<*const c_void, StreamPtr>(ptr);
            obj_ptr_to_disprock(ptr)
        },
        2 => {
            let ptr = mem::transmute::<*const c_void, FileRefPtr>(ptr);
            obj_ptr_to_disprock(ptr)
        },
        _ => unreachable!(),
    }
}

fn obj_ptr_to_disprock<T>(ptr: *const Mutex<GlkObjectMetadata<T>>) -> *const DispatchRock {
    let obj = from_ptr(ptr);
    let obj = obj.lock().unwrap();
    let disprock = obj.disprock.as_ref().unwrap();
    disprock
}