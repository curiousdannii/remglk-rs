/*

GlkApi constants
================

Copyright (c) 2022 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

#![allow(non_upper_case_globals)]

use serde::{Deserialize, Serialize};

use super::*;

pub const gestalt_Version: u32 = 0;
pub const gestalt_CharInput: u32 = 1;
pub const gestalt_LineInput: u32 = 2;
pub const gestalt_CharOutput: u32 = 3;
pub const gestalt_CharOutput_CannotPrint: u32 = 0;
pub const gestalt_CharOutput_ApproxPrint: u32 = 1;
pub const gestalt_CharOutput_ExactPrint: u32 = 2;
pub const gestalt_MouseInput: u32 = 4;
pub const gestalt_Timer: u32 = 5;
pub const gestalt_Graphics: u32 = 6;
pub const gestalt_DrawImage: u32 = 7;
pub const gestalt_Sound: u32 = 8;
pub const gestalt_SoundVolume: u32 = 9;
pub const gestalt_SoundNotify: u32 = 10;
pub const gestalt_Hyperlinks: u32 = 11;
pub const gestalt_HyperlinkInput: u32 = 12;
pub const gestalt_SoundMusic: u32 = 13;
pub const gestalt_GraphicsTransparency: u32 = 14;
pub const gestalt_Unicode: u32 = 15;
pub const gestalt_UnicodeNorm: u32 = 16;
pub const gestalt_LineInputEcho: u32 = 17;
pub const gestalt_LineTerminators: u32 = 18;
pub const gestalt_LineTerminatorKey: u32 = 19;
pub const gestalt_DateTime: u32 = 20;
pub const gestalt_Sound2: u32 = 21;
pub const gestalt_ResourceStream: u32 = 22;
pub const gestalt_GraphicsCharInput: u32 = 23;
pub const gestalt_GarglkText: u32 = 0x1100;

pub const keycode_Unknown: u32 = 0xffffffff;
pub const keycode_Left: u32 = 0xfffffffe;
pub const keycode_Right: u32 = 0xfffffffd;
pub const keycode_Up: u32 = 0xfffffffc;
pub const keycode_Down: u32 = 0xfffffffb;
pub const keycode_Return: u32 = 0xfffffffa;
pub const keycode_Delete: u32 = 0xfffffff9;
pub const keycode_Escape: u32 = 0xfffffff8;
pub const keycode_Tab: u32 = 0xfffffff7;
pub const keycode_PageUp: u32 = 0xfffffff6;
pub const keycode_PageDown: u32 = 0xfffffff5;
pub const keycode_Home: u32 = 0xfffffff4;
pub const keycode_End: u32 = 0xfffffff3;
pub const keycode_Func1: u32 = 0xffffffef;
pub const keycode_Func2: u32 = 0xffffffee;
pub const keycode_Func3: u32 = 0xffffffed;
pub const keycode_Func4: u32 = 0xffffffec;
pub const keycode_Func5: u32 = 0xffffffeb;
pub const keycode_Func6: u32 = 0xffffffea;
pub const keycode_Func7: u32 = 0xffffffe9;
pub const keycode_Func8: u32 = 0xffffffe8;
pub const keycode_Func9: u32 = 0xffffffe7;
pub const keycode_Func10: u32 = 0xffffffe6;
pub const keycode_Func11: u32 = 0xffffffe5;
pub const keycode_Func12: u32 = 0xffffffe4;
// The last keycode is always (0x100000000 - keycode_MAXVAL)
pub const keycode_MAXVAL: u32 = 28;
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum TerminatorCode {
    Escape = -8,
    Func1 = -17,
    Func2 = -18,
    Func3 = -19,
    Func4 = -20,
    Func5 = -21,
    Func6 = -22,
    Func7 = -23,
    Func8 = -24,
    Func9 = -25,
    Func10 = -26,
    Func11 = -27,
    Func12 = -28,
}
pub fn key_name_to_code(key: &str) -> u32 {
    match key {
        "delete" => keycode_Delete,
        "down" => keycode_Down,
        "end" => keycode_End,
        "escape" => keycode_Escape,
        "func1" => keycode_Func1,
        "func2" => keycode_Func2,
        "func3" => keycode_Func3,
        "func4" => keycode_Func4,
        "func5" => keycode_Func5,
        "func6" => keycode_Func6,
        "func7" => keycode_Func7,
        "func8" => keycode_Func8,
        "func9" => keycode_Func9,
        "func10" => keycode_Func10,
        "func11" => keycode_Func11,
        "func12" => keycode_Func12,
        "home" => keycode_Home,
        "left" => keycode_Left,
        "pagedown" => keycode_PageDown,
        "pageup" => keycode_PageUp,
        "return" => keycode_Return,
        "right" => keycode_Right,
        "tab" => keycode_Tab,
        "up" => keycode_Up,
        _ => keycode_Unknown,
    }
}

pub const evtype_None: u32 = 0;
pub const evtype_Timer: u32 = 1;
pub const evtype_CharInput: u32 = 2;
pub const evtype_LineInput: u32 = 3;
pub const evtype_MouseInput: u32 = 4;
pub const evtype_Arrange: u32 = 5;
pub const evtype_Redraw: u32 = 6;
pub const evtype_SoundNotify: u32 = 7;
pub const evtype_Hyperlink: u32 = 8;
pub const evtype_VolumeNotify: u32 = 9;
#[derive(Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub enum GlkEventType {
    #[default]
    None = 0,
    Timer,
    Char,
    Line,
    Mouse,
    Arrange,
    Redraw,
    SoundNotify,
    Hyperlink,
    VolumeNotify,
}

pub const style_Normal: u32 = 0;
pub const style_Emphasized: u32 = 1;
pub const style_Preformatted: u32 = 2;
pub const style_Header: u32 = 3;
pub const style_Subheader: u32 = 4;
pub const style_Alert: u32 = 5;
pub const style_Note: u32 = 6;
pub const style_BlockQuote: u32 = 7;
pub const style_Input: u32 = 8;
pub const style_User1: u32 = 9;
pub const style_User2: u32 = 10;
pub const style_NUMSTYLES: u32 = 11;
pub fn style_name(style: u32) -> &'static str {
    match style {
        style_Normal => "normal",
        style_Emphasized => "emphasized",
        style_Preformatted => "preformatted",
        style_Header => "header",
        style_Subheader => "subheader",
        style_Alert => "alert",
        style_Note => "note",
        style_BlockQuote => "blockquote",
        style_Input => "input",
        style_User1 => "user1",
        style_User2 => "user2",
        _ => "normal",
    }
}

pub const wintype_AllTypes: u32 = 0;
pub const wintype_Pair: u32 = 1;
pub const wintype_Blank: u32 = 2;
pub const wintype_TextBuffer: u32 = 3;
pub const wintype_TextGrid: u32 = 4;
pub const wintype_Graphics: u32 = 5;
#[derive(Clone, Copy, Default, PartialEq, Serialize)]
#[repr(C)]
#[serde(rename_all = "lowercase")]
pub enum WindowType {
    All = 0,
    Pair = 1,
    #[default]
    Blank = 2,
    Buffer = 3,
    Graphics = 5,
    Grid = 4,
}

pub const winmethod_Left : u32 = 0x00;
pub const winmethod_Right: u32 = 0x01;
pub const winmethod_Above: u32 = 0x02;
pub const winmethod_Below: u32 = 0x03;
pub const winmethod_DirMask: u32 = 0x0f;

pub const winmethod_Fixed: u32 = 0x10;
pub const winmethod_Proportional: u32 = 0x20;
pub const winmethod_DivisionMask: u32 = 0xf0;

pub const winmethod_Border: u32 = 0x000;
pub const winmethod_NoBorder: u32 = 0x100;
pub const winmethod_BorderMask: u32 = 0x100;

pub fn validate_winmethod(method: u32, wintype: WindowType) -> GlkResult<'static, (u32, u32, u32)> {
    if wintype == WindowType::Pair {
        return Err(SplitCantBePair);
    }
    let division = method & winmethod_DivisionMask;
    let direction = method & winmethod_DirMask;
    if division != winmethod_Fixed && division != winmethod_Proportional {
        return Err(InvalidWindowDivision)
    }
    if division == winmethod_Fixed && wintype == WindowType::Blank {
        return Err(InvalidWindowDivisionBlank)
    }
    if let winmethod_Above | winmethod_Below | winmethod_Left | winmethod_Right = direction {}
    else {
        return Err(InvalidWindowDirection)
    }
    Ok((division, direction, method & winmethod_BorderMask))
}

pub const fileusage_Data: u32 = 0x00;
pub const fileusage_SavedGame: u32 = 0x01;
pub const fileusage_Transcript: u32 = 0x02;
pub const fileusage_InputRecord: u32 = 0x03;
pub const fileusage_TypeMask: u32 = 0x0f;
#[derive(Clone, Copy, Default, Deserialize, PartialEq)]
#[repr(C)]
pub enum FileType {
    #[default]
    Data = 0,
    SavedGame,
    Transcript,
    InputRecord,
}
pub fn file_type(filetype: u32) -> FileType {
    match filetype {
        fileusage_Data => FileType::Data,
        fileusage_SavedGame => FileType::SavedGame,
        fileusage_Transcript => FileType::Transcript,
        fileusage_TypeMask => FileType::InputRecord,
        _ => FileType::Data,
    }
}
pub fn filetype_suffix(filetype: FileType) -> &'static str {
    match filetype {
        FileType::Data => ".glkdata",
        FileType::SavedGame => ".glksave",
        FileType::Transcript | FileType::InputRecord => ".txt",
    }
}

pub const fileusage_TextMode: u32 = 0x100;
pub const fileusage_BinaryMode: u32 = 0x000;

pub const filemode_Write: u32 = 0x01;
pub const filemode_Read: u32 = 0x02;
pub const filemode_ReadWrite: u32 = 0x03;
pub const filemode_WriteAppend: u32 = 0x05;
#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
#[repr(C)]
pub enum FileMode {
    Write = 0x01,
    #[default]
    Read = 0x02,
    ReadWrite = 0x03,
    WriteAppend = 0x05,
}
pub fn file_mode(filemode: u32) -> GlkResult<'static, FileMode> {
    match filemode {
        filemode_Write => Ok(FileMode::Write),
        filemode_Read => Ok(FileMode::Read),
        filemode_ReadWrite => Ok(FileMode::ReadWrite),
        filemode_WriteAppend => Ok(FileMode::WriteAppend),
        _ => Err(IllegalFilemode),
    }
}

pub const seekmode_Start: u32 = 0;
pub const seekmode_Current: u32 = 1;
pub const seekmode_End: u32 = 2;
#[derive(Clone, Copy, PartialEq)]
#[repr(C)]
pub enum SeekMode {
    Current = 1,
    End = 2,
    Start = 0,
}

pub const stylehint_Indentation: u32 = 0;
pub const stylehint_ParaIndentation: u32 = 1;
pub const stylehint_Justification: u32 = 2;
pub const stylehint_Size: u32 = 3;
pub const stylehint_Weight: u32 = 4;
pub const stylehint_Oblique: u32 = 5;
pub const stylehint_Proportional: u32 = 6;
pub const stylehint_TextColor: u32 = 7;
pub const stylehint_BackColor: u32 = 8;
pub const stylehint_ReverseColor: u32 = 9;
pub const stylehint_NUMHINTS: u32 = 10;
pub fn stylehint_name(style: u32) -> &'static str {
    match style {
        stylehint_Indentation => "margin-left",
        stylehint_ParaIndentation => "text-indent",
        stylehint_Justification => "text-align",
        stylehint_Size => "font-size",
        stylehint_Weight => "font-weight",
        stylehint_Oblique => "font-style",
        stylehint_Proportional => "monospace",
        stylehint_TextColor => "color",
        stylehint_BackColor => "background-color",
        stylehint_ReverseColor => "reverse",
        _ => unreachable!(),
    }
}

pub const stylehint_just_LeftFlush: u32 = 0;
pub const stylehint_just_LeftRight: u32 = 1;
pub const stylehint_just_Centered: u32 = 2;
pub const stylehint_just_RightFlush: u32 = 3;
pub fn justification(val: i32) -> &'static str {
    match val {
        0 => "left",
        1 => "justify",
        2 => "center",
        3 => "right",
        _ => "left",
    }
}

pub fn font_weight(val: i32) -> &'static str {
    match val {
        -1 => "lighter",
        0 => "normal",
        1 => "bold",
        _ => "normal",
    }
}

pub const imagealign_InlineUp: u32 = 1;
pub const imagealign_InlineDown: u32 = 2;
pub const imagealign_InlineCenter: u32 = 3;
pub const imagealign_MarginLeft: u32 = 4;
pub const imagealign_MarginRight: u32 = 5;

pub const zcolor_Default: u32 = 0xffffffff;
pub const zcolor_Current: u32 = 0xfffffffe;