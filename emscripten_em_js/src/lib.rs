/*

Rust versions of the Emscripten EM_JS and EM_ASYNC_JS macros
============================================================

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

//! em_js!{} declares a Javascript function. It is largely similar to the Emscripten macro `EM_JS`.
//! 
//! ```c
//! EM_JS(int, add, (int x, int y), {
//!     return x + y;
//! })
//! ```
//! 
//! But instead of separate parameters, it takes a whole Rust function declaration, formatted as usual.
//! The Javascript code must be included as a string. If your JS code uses double quotes, you can use a
//! raw string.
//! 
//! ```
//! em_js!{fn add(x: i32, y: i32) -> i32 { r#"
//!     return x + y;
//! "# }}
//! ```
//! 
//! You may also declare async functions. Unlike in Emscripten where you would use the `EM_ASYNC_JS`
//! macro, these use the same macro, just declare the function as `async`:
//! 
//! ```
//! em_js!{async fn add(x: i32, y: i32) -> i32 { r#"
//!     return x + y;
//! "# }}
//! ```
//! 
//! Supported types:
//! 
//! | Type    | Input | Output |
//! |---------|-------|--------|
//! | pointer | Y     | ?      |
//! | [f64]   | Y     | Y      |
//! | [i32]   | Y     | Y      |
//! | [usize] | Y     | Y      |

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Block, Expr, FnArg, ItemFn, Lit, Pat, Stmt, Type};
use syn::punctuated::Punctuated;
use syn::token::Comma;

/** em_js!{} declares a Javascript function. It is largely similar to the Emscripten macro `EM_JS`.
 * 
 * For examples, and supported types, see [the module documentation](crate).
*/
#[proc_macro]
pub fn em_js(input: TokenStream) -> TokenStream {
    let parsed = syn::parse::<ItemFn>(input).unwrap();
    let name = parsed.sig.ident;
    let link_name = name.to_string();
    let js_name = format_ident!("__em_js__{}{}", if parsed.sig.asyncness.is_some() {"__asyncjs__"} else {""}, name);
    let inputs = parsed.sig.inputs;
    let output = parsed.sig.output;
    let body = format!("({})<::>{{{}}}\0", rust_args_to_c(&inputs), get_body_str(parsed.block.as_ref()));
    let body = body.as_bytes();
    let body_len = body.len();
    
    let result = quote! {
        extern "C" {
            #[link_name = #link_name]
            pub fn #name(#inputs) #output;
        }

        #[link_section = "em_js"]
        #[no_mangle]
        #[used]
        static #js_name: [u8; #body_len] = [#(#body),*];
    };

    result.into()
}

fn get_body_str(block: &Block) -> String {
    let body = &block.stmts[0];
    if let Stmt::Expr(Expr::Lit(lit), _) = body {
        if let Lit::Str(body) = &lit.lit {
            return body.value().to_owned();
        }
    }
    panic!("em_js body was not string");
}

fn rust_args_to_c(args: &Punctuated<FnArg, Comma>) -> String {
    let mut results: Vec<String> = vec![];
    for arg in args.iter() {
        let c_type = if let FnArg::Typed(arg) = arg {
            let name = if let Pat::Ident(name) = arg.pat.as_ref() {
                &name.ident
            }
            else {
                unreachable!("name: as_ref()");
            };
            let rust_type = match arg.ty.as_ref() {
                Type::Path(path) => path.path.segments.first().unwrap().ident.to_string(),
                Type::Ptr(_) => "*".to_owned(),
                _ => panic!("unsupported rust_type: as_ref(), {:?}", arg.ty.as_ref()),
            };
            let c_type = match rust_type.as_str() {
                "*" => "int",
                "f64" => "double",
                "i32" => "int",
                "usize" => "int",
                other => panic!("unsupported argument type: {}", other),
            };
            format!("{} {}", c_type, name)
        }
        else {
            panic!("self arg in em_js");
        };
        results.push(c_type);
    }
    results.join(", ")
}