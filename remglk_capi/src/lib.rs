/*

RemGlk ported to Rust - C API
=============================

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/


mod common;
mod dispatch;
mod glkapi;
mod glkstart;

use std::ffi::{c_char, c_int};

use glkapi::glk_exit;
use glkstart::*;

/** Processed arguments which we give to `glkunix_startup_code` */
#[repr(C)]
struct GlkUnixArguments {
    count: c_int,
    args: *const *const c_char,
}

extern "C" {
    fn glk_main();
    fn glkunix_startup_code(data: &GlkUnixArguments) -> c_int;
}

/** Glk libraries are weird because they define `main`, rather than the eventual app that is linked against them. So control starts here, and then returns to the app when `glk_main` is called. */
#[no_mangle]
extern "C" fn main() {
    // Process the arguments, and optionally display an error/help message
    let processed_args = match glkstart::process_args() {
        ArgProcessingResults::ErrorMsg(msg) => {
            eprint!("{msg}");
            std::process::exit(1);
        },
        ArgProcessingResults::Msg(msg) => {
            print!("{msg}");
            std::process::exit(0);
        },
        ArgProcessingResults::ProcessedArgs(args) => args,
    };

    // We can now hand control over to the app
    if unsafe{glkunix_startup_code(&GlkUnixArguments {
        count: processed_args.len() as c_int,
        args: processed_args.iter().map(|arg| arg.as_ptr()).collect::<Vec<*const c_char>>().as_ptr(),
    })} == 0 {
        glk_exit();
    }

    unsafe{glk_main()};
    glk_exit();
}