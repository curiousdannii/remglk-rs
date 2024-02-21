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

// The C function `gidispatch_get_objrock` takes a generic pointer, which we can't really deal with here in Rust, so support.c will handle calling the appropriate function
// The WASM ABI means that we can't return a DispatchRock, so it must be set through an out parameter
#[no_mangle]
pub extern "C" fn gidispatch_get_objrock_fileref(ptr: FileRefPtr, dispatchrock_ptr: *mut DispatchRock) {
    let obj = from_ptr(ptr);
    let obj = obj.lock().unwrap();
    write_ptr(dispatchrock_ptr, obj.disprock.unwrap());
}

#[no_mangle]
pub extern "C" fn gidispatch_get_objrock_stream(ptr: StreamPtr, dispatchrock_ptr: *mut DispatchRock) {
    let obj = from_ptr(ptr);
    let obj = obj.lock().unwrap();
    write_ptr(dispatchrock_ptr, obj.disprock.unwrap());
}

#[no_mangle]
pub extern "C" fn gidispatch_get_objrock_window(ptr: WindowPtr, dispatchrock_ptr: *mut DispatchRock) {
    let obj = from_ptr(ptr);
    let obj = obj.lock().unwrap();
    write_ptr(dispatchrock_ptr, obj.disprock.unwrap());
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