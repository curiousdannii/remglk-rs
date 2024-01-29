/*

RemGlk ported to Rust - C API
=============================

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

mod common;
mod glkapi;
mod glkstart;

/** Glk libraries are weird because they define `main`, rather than the eventual app that is linked against them. So control starts here, and then returns to the app when `glk_main` is called. */
#[no_mangle]
extern "C" fn main() {
    // Process the arguments, and optionally display an error/help message
    if let Some(msg) = glkstart::process_args() {
        match msg {
            Ok(msg) => {
                print!("{msg}");
                std::process::exit(0);
            },
            Err(msg) => {
                eprint!("{msg}");
                std::process::exit(1);
            },
        };
    }
}