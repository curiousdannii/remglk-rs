/*

Blorb constants
===============

Copyright (c) 2026 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

#![allow(non_upper_case_globals)]

use four_cc::FourCC;

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

pub const giblorb_ID_AIFF: FourCC = FourCC(*b"AIFF");
pub const giblorb_ID_BINA: FourCC = FourCC(*b"BINA");
pub const giblorb_ID_Data: FourCC = FourCC(*b"Data");
pub const giblorb_ID_Exec: FourCC = FourCC(*b"Exec");
pub const giblorb_ID_FORM: FourCC = FourCC(*b"FORM");
pub const giblorb_ID_JPEG: FourCC = FourCC(*b"JPEG");
pub const giblorb_ID_OGGV: FourCC = FourCC(*b"OGGV");
pub const giblorb_ID_Pict: FourCC = FourCC(*b"Pict");
pub const giblorb_ID_PNG_: FourCC = FourCC(*b"PNG ");
pub const giblorb_ID_RDes: FourCC = FourCC(*b"RDes");
pub const giblorb_ID_RIdx: FourCC = FourCC(*b"RIdx");
pub const giblorb_ID_Snd_: FourCC = FourCC(*b"Snd ");
pub const giblorb_ID_TEXT: FourCC = FourCC(*b"TEXT");