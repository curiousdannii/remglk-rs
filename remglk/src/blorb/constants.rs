/*

Blorb constants
===============

Copyright (c) 2026 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

#![allow(non_upper_case_globals)]

/* Error type and error codes */
pub const giblorb_err_None: u32 = 0;
pub const giblorb_err_CompileTime: u32 = 1;
pub const giblorb_err_Alloc: u32 = 2;
pub const giblorb_err_Read: u32 = 3;
pub const giblorb_err_NotAMap: u32 = 4;
pub const giblorb_err_Format: u32 = 5;
pub const giblorb_err_NotFound: u32 = 6;

/* Methods for loading a chunk */
pub const giblorb_method_DontLoad: u32 = 0;
pub const giblorb_method_Memory: u32 = 1;
pub const giblorb_method_FilePos: u32 = 2;

const fn giblorb_make_id(c1: char, c2: char, c3: char, c4: char) -> u32 {
    ((c1 as u32) << 24) | ((c2 as u32) << 16) | ((c3 as u32) << 8) | (c4 as u32)
}
pub const giblorb_ID_BINA: u32 = giblorb_make_id('B', 'I', 'N', 'A');
pub const giblorb_ID_Data: u32 = giblorb_make_id('D', 'a', 't', 'a');
pub const giblorb_ID_Exec: u32 = giblorb_make_id('E', 'x', 'e', 'c');
pub const giblorb_ID_FORM: u32 = giblorb_make_id('F', 'O', 'R', 'M');
pub const giblorb_ID_Pict: u32 = giblorb_make_id('P', 'i', 'c', 't');
pub const giblorb_ID_RDes: u32 = giblorb_make_id('R', 'D', 'e', 's');
pub const giblorb_ID_RIdx: u32 = giblorb_make_id('R', 'I', 'd', 'x');
pub const giblorb_ID_Snd: u32 =  giblorb_make_id('S', 'n', 'd', ' ');
pub const giblorb_ID_TEXT: u32 = giblorb_make_id('T', 'E', 'X', 'T');