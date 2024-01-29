/*

RemGlk-rs build script
=============================

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

fn main() {
    cc::Build::new()
        .file("src/glk/gi_blorb.c")
        .file("src/glk/gi_debug.c")
        .file("src/glk/gi_dispa.c")
        .warnings(false)
        .compile("miniglk");
    println!("cargo:rerun-if-changed=src/");
}