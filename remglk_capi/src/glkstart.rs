/*

Glk startup support code
========================

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

#![allow(non_upper_case_globals)]

use std::env;
use std::ffi::{c_char, c_int, CStr, CString};
use std::slice;
use std::str;

use thiserror::Error;

/** Processed arguments which we give to `glkunix_startup_code` */
#[repr(C)]
struct GlkUnixArguments {
    count: c_int,
    args: *const *const c_char,
}

extern "C" {
    fn glkunix_startup_code(data: &GlkUnixArguments);
}

const glkunix_arg_End: i32 = 0;
const glkunix_arg_ValueFollows: i32 = 1;
const glkunix_arg_NoValue: i32 = 2;
const glkunix_arg_ValueCanFollow: i32 = 3;
const glkunix_arg_NumberValue: i32 = 4;

/** Process the command line arguments */
// I didn't really want to reimplement the Zarf's logic, but none of the Rust argument parsing libraries really seem to do what we want.
pub fn process_args() -> Option<Result<String, String>> {
    #[derive(Error, Debug)]
    pub enum ArgError {
        #[error("{0} must be followed by a value")]
        NoValue(String),
        #[error("{0} must be followed by a number")]
        NotNumber(String),
        #[error("unknown argument: {0}")]
        UnknownArg(String),
        #[error("unwanted argument: {0}")]
        UnwantedArg(String),
    }

    enum InnerResult {
        Help,
        NothingToDo,
        Version,
    }

    let args: Vec<String> = env::args().collect();

    fn process_args_inner(args: &Vec<String>, app_arguments: &Vec<GlkUnixArgument>) -> Result<InnerResult, ArgError> {
        let mut processed_args: Vec<CString> = Vec::new();
        let mut push_arg = |arg: &String| {
            processed_args.push(CString::new(arg.as_str()).unwrap());
        };

        // Add the program name
        push_arg(&args[0]);

        let mut args_iter = args.iter().peekable();
        'outer: while let Some(arg) = args_iter.next() {
            // Go through all of the app arguments

            if arg == "-help" {
                return Ok(InnerResult::Help);
            }
            if arg == "-version" {
                return Ok(InnerResult::Version);
            }

            for app_arg in app_arguments {
                // Empty arguments, for example, the storyfile itself
                if app_arg.name.is_empty() && !arg.starts_with('-') {
                    push_arg(arg);
                    continue 'outer;
                }

                if arg == &app_arg.name {
                    push_arg(arg);
                    match app_arg.argtype {
                        glkunix_arg_ValueFollows => {
                            push_arg(args_iter.next().ok_or(ArgError::NoValue(arg.to_string()))?);
                        },
                        glkunix_arg_NoValue => {},
                        glkunix_arg_ValueCanFollow => {
                            if let Some(value) = args_iter.peek() {
                                if !value.starts_with('-') {
                                    push_arg(args_iter.next().unwrap());
                                }
                            }
                        },
                        glkunix_arg_NumberValue => {
                            let value = args_iter.next().ok_or(ArgError::NoValue(arg.to_string()))?;
                            str::parse::<i32>(value).map_err(|_| ArgError::NotNumber(arg.to_string()))?;
                            push_arg(value);
                        },
                        _ => panic!("glkunix_arguments: {arg} has invalid arg type"),
                    };
                    continue 'outer;
                }
            }

            if !arg.starts_with('-') {
                return Err(ArgError::UnwantedArg(arg.to_string()));
            }

            // And now to process the library arguments
            // Except we don't have any! Easy.

            return Err(ArgError::UnknownArg(arg.to_string()));
        }

        unsafe{glkunix_startup_code(&GlkUnixArguments {
            count: processed_args.len() as c_int,
            args: processed_args.iter().map(|arg| arg.as_ptr()).collect::<Vec<*const c_char>>().as_ptr(),
        })};

        Ok(InnerResult::NothingToDo)
    }

    fn print_usage(app_name: &String, app_arguments: &Vec<GlkUnixArgument>) -> String {
        let mut usage = format!("usage: {} [ options ... ]\n", app_name);
        if app_arguments.len() > 0 {
            usage.push_str("app options:\n");
            for app_arg in app_arguments {
                usage.push_str(&if app_arg.name.is_empty() {
                    format!("  {}\n", app_arg.desc)
                }
                else {
                    match app_arg.argtype {
                        glkunix_arg_ValueFollows | glkunix_arg_NumberValue => format!("  {} val: {}\n", app_arg.name, app_arg.desc),
                        glkunix_arg_NoValue => format!("  {}: {}\n", app_arg.name, app_arg.desc),
                        glkunix_arg_ValueCanFollow => format!("  {} [val]: {}\n", app_arg.name, app_arg.desc),
                        _ => unreachable!(),
                    }
                });
            }
        }
        usage
    }

    let app_arguments = unsafe{glkunix_arguments()};
    match process_args_inner(&args, &app_arguments) {
        Ok(InnerResult::Help) => Some(Ok(print_usage(&args[0], &app_arguments))),
        Ok(InnerResult::NothingToDo) => None,
        Ok(InnerResult::Version) => Some(Ok("RemGlk-rs, library version ".to_owned() + env!("CARGO_PKG_VERSION"))),
        Err(err) => Some(Err(err.to_string() + "\n" + &print_usage(&args[0], &app_arguments))),
    }
}

// Solution for extracting GlkUnix arguments by Boiethios
// https://stackoverflow.com/a/58910948/2854284
struct GlkUnixArgument {
    pub name: String,
    pub argtype: i32,
    pub desc: String,
}

/** Turn the C global `glkunix_arguments` into something we can use */
unsafe fn glkunix_arguments() -> Vec<GlkUnixArgument> {
    #[repr(C)]
    struct GlkUnixArgumentC {
        name: *const c_char,
        argtype: c_int,
        desc: *const c_char,
    }

    extern "C" {
        static glkunix_arguments: *const GlkUnixArgumentC;
    }

    // Count how many arguments there are
    let len = (0..)
        .map(|i| glkunix_arguments.offset(i))
        .take_while(|&arg| (*arg).argtype != glkunix_arg_End)
        .count();

    slice::from_raw_parts(glkunix_arguments, len)
        .iter()
        .map(|arg| GlkUnixArgument {
            name: CStr::from_ptr(arg.name).to_str().expect("glkunix_arguments: has a non-UTF-8-safe name").into(),
            argtype: arg.argtype,
            desc: CStr::from_ptr(arg.desc).to_str().expect("glkunix_arguments: has a non-UTF-8-safe description").into(),
        })
        .collect()
}