/*

GiDispatch functions
====================

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::ffi::c_void;
use std::mem;

use remglk::glkapi::*;
use objects::*;
use std::sync::Mutex;

use super::*;
use common::*;
use glkapi::*;

type RegisterCallbackGeneric = extern fn(*const c_void, u32) -> DispatchRock;
type UnregisterCallbackGeneric = extern fn(*const c_void, u32, DispatchRock);

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

type RetainArrayCallbackGeneric = extern fn(*const c_void, u32, *const c_char) -> DispatchRock;
type UnretainArrayCallbackGeneric = extern fn(*const c_void, u32, *const c_char, DispatchRock);

#[no_mangle]
pub unsafe extern "C" fn gidispatch_set_retained_registry(register_cb: RetainArrayCallbackGeneric, unregister_cb: UnretainArrayCallbackGeneric) {
    let mut glkapi = glkapi().lock().unwrap();
    let retain = mem::transmute::<RetainArrayCallbackGeneric, RetainArrayCallback<u8>>(register_cb);
    let unretain = mem::transmute::<UnretainArrayCallbackGeneric, UnretainArrayCallback<u8>>(unregister_cb);
    glkapi.retain_array_callbacks_u8 = Some(RetainArrayCallbacks {
        retain,
        unretain,
    });
    let retain = mem::transmute::<RetainArrayCallbackGeneric, RetainArrayCallback<u32>>(register_cb);
    let unretain = mem::transmute::<UnretainArrayCallbackGeneric, UnretainArrayCallback<u32>>(unregister_cb);
    glkapi.retain_array_callbacks_u32 = Some(RetainArrayCallbacks {
        retain,
        unretain,
    });
}