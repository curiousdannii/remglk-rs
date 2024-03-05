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

use std::ffi::{c_char, c_int, CStr};

use remglk::glkapi::protocol::{Event, EventData, InitEvent, Metrics};

use glkapi::{glk_exit, glkapi};
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

/// Glk libraries are weird because they define `main`, rather than the eventual app that is linked against them. So control starts here, and then returns to the app when `glk_main` is called.
///
/// We must manually process the args instead of using `env::args`, because of this limitation in WASM: https://github.com/rust-lang/rust/issues/121883
#[no_mangle]
extern "C" fn main(argc: c_int, argv: *const *const c_char) -> c_int {
    // Process the arguments, and optionally display an error/help message
    let args: Vec<String> = (0..argc)
        .map(|i| unsafe {CStr::from_ptr(*argv.add(i as usize))}.to_str().unwrap().to_owned())
        .collect();

    let (processed_args, library_args) = match glkstart::process_args(args) {
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
        return 0;
    }

    // Wait for the initial event with the metrics
    if library_args.autoinit {
        glkapi().lock().unwrap().handle_event(Event {
            data: EventData::Init(InitEvent {
                metrics: Metrics {
                    buffercharheight: Some(1.0),
                    buffercharwidth: Some(1.0),
                    gridcharheight: Some(1.0),
                    gridcharwidth: Some(1.0),
                    height: 50.0,
                    width: 80.0,
                    ..Default::default()
                },
                support: vec![
                    "garglktext".to_string(),
                    "graphics".to_string(),
                    "graphicswin".to_string(),
                    "hyperlinks".to_string(),
                    "timer".to_string(),
                ],
            }),
            gen: 0,
            partial: None,
        }).unwrap();
    }
    else {
        glkapi().lock().unwrap().get_glkote_init();
    }

    unsafe{glk_main()};
    glk_exit();
    0
}