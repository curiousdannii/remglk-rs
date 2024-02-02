/*

The Glk API
===========

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

mod arrays;
mod common;
pub mod constants;
mod filerefs;
mod objects;
pub mod protocol;
mod streams;
mod windows;

use std::cmp::min;
use std::mem;
use std::ops::Deref;
use std::str;
use std::time::SystemTime;

use super::*;
pub use arrays::*;
pub use common::*;
pub use GlkApiError::*;
use constants::*;
use filerefs::*;
use objects::*;
use protocol::*;
use streams::*;
use windows::*;

// Expose for so they can be turned into pointers
pub use filerefs::FileRef;
pub use objects::GlkObject;
pub use streams::Stream;
pub use windows::Window;

#[derive(Default)]
pub struct GlkApi<S>
where S: Default + GlkSystem {
    current_stream: Option<GlkStreamWeak>,
    exited: bool,
    filerefs: GlkObjectStore<FileRef>,
    gen: u32,
    metrics: NormalisedMetrics,
    partial_inputs: PartialInputs,
    root_window: Option<GlkWindowWeak>,
    streams: GlkObjectStore<Stream>,
    stylehints_buffer: WindowStyles,
    stylehints_grid: WindowStyles,
    support: SupportedFeatures,
    system: S,
    timer: TimerData,
    windows: GlkObjectStore<Window>,
    windows_changed: bool,
}

impl<S> GlkApi<S>
where S: Default + GlkSystem {
    pub fn new(system: S) -> Self {
        GlkApi {
            system,
            ..Default::default()
        }
    }

    // The Glk API

    pub fn glk_buffer_to_lower_case_uni(buf: &mut [u32], initlen: usize) -> usize {
        let res = str_to_u32vec(&u32slice_to_string(&buf[..initlen]).to_lowercase());
        let len = res.len();
        let act_len = min(len, buf.len());
        buf[..act_len].copy_from_slice(&res[..act_len]);
        len
    }

    pub fn glk_buffer_to_title_case_uni(buf: &mut [u32], initlen: usize, lowerrest: bool) -> usize {
        let mut res = str_to_u32vec(&u32slice_to_string(&buf[0..1]).to_uppercase());
        if lowerrest {
            res.append(&mut str_to_u32vec(&u32slice_to_string(&buf[1..initlen]).to_lowercase()));
        }
        else {
            res.extend_from_slice(&buf[1..initlen]);
        }
        let len = res.len();
        let act_len = min(len, buf.len());
        buf[..act_len].copy_from_slice(&res[..act_len]);
        len
    }

    pub fn glk_buffer_to_upper_case_uni(buf: &mut [u32], initlen: usize) -> usize {
        let res = str_to_u32vec(&u32slice_to_string(&buf[..initlen]).to_uppercase());
        let len = res.len();
        let act_len = min(len, buf.len());
        buf[..act_len].copy_from_slice(&res[..act_len]);
        len
    }

    pub fn glk_cancel_char_event(win: &GlkWindow) {
        lock!(win).input.text_input_type = None;
    }

    pub fn glk_cancel_hyperlink_event(win: &GlkWindow) {
        lock!(win).input.hyperlink = None;
    }

    pub fn glk_cancel_line_event(&mut self, win: &GlkWindow) -> GlkResult<GlkEvent> {
        let disprock = self.windows.get_disprock(win).unwrap();
        let mut win_locked = lock!(win);
        if let Some(TextInputType::Line) = win_locked.input.text_input_type {
            let partial = self.partial_inputs.as_mut().map(|partials| partials.remove(&disprock)).flatten().unwrap_or("".to_string());
            // Steal the data temporarily so that we're not double borrowing the window
            let mut data = mem::take(&mut win_locked.data);
            let mut res = match &mut data {
                WindowData::Buffer(data) => self.handle_line_input(&mut win_locked, data, &partial, None)?,
                WindowData::Grid(data) => self.handle_line_input(&mut win_locked, data, &partial, None)?,
                _ => unreachable!(),
            };
            win_locked.data = data;
            res.win = Some(win.clone());
            Ok(res)
        }
        else {
            Ok(GlkEvent::default())
        }
    }

    pub fn glk_cancel_mouse_event(win: &GlkWindow) {
        lock!(win).input.mouse = None;
    }

    pub fn glk_char_to_lower(val: u32) -> u32 {
        match val {
            0x41..=0x5A => val + 0x20,
            0xC0..=0xD6 | 0xD8..=0xDE => val + 0x20,
            _ => val,
        }
    }

    pub fn glk_char_to_upper(val: u32) -> u32 {
        match val {
            0x61..=0x7A => val - 0x20,
            0xE0..=0xE6 | 0xF8..=0xFE => val - 0x20,
            _ => val,
        }
    }

    pub fn glk_exit(&mut self) {
        self.exited = true;
    }

    pub fn glk_fileref_create_by_name(&mut self, usage: u32, filename: String, rock: u32) -> GlkFileRef {
        let filetype = file_type(usage & fileusage_TypeMask);
        // Clean the filename
        let mut fixed_filename = String::default();
        for char in filename.chars() {
            match char {
                '"' | '\\' | '/' | '>' | '<' | ':' | '?' | '*' | char::REPLACEMENT_CHARACTER => {},
                _ => fixed_filename.push(char),
            };
        }
        if fixed_filename.is_empty() {
            fixed_filename = "null".to_string();
        }
        fixed_filename.push_str(filetype_suffix(filetype));
        self.create_fileref(fixed_filename, rock, usage, None)
    }

    // For glkunix_stream_open_pathname
    pub fn glk_fileref_create_by_name_uncleaned(&mut self, usage: u32, filename: String, rock: u32) -> GlkFileRef {
        self.create_fileref(filename, rock, usage, None)
    }

    pub fn glk_fileref_create_from_fileref(&mut self, usage: u32, fileref: &GlkFileRef, rock: u32) -> GlkFileRef {
        let fileref = lock!(fileref);
        self.create_fileref(fileref.system_fileref.filename.clone(), rock, usage, None)
    }

    pub fn glk_fileref_create_temp(&mut self, usage: u32, rock: u32) -> GlkFileRef {
        let filetype = file_type(usage & fileusage_TypeMask);
        let system_fileref = self.system.fileref_temporary(filetype);
        self.create_fileref(system_fileref.filename.clone(), rock, usage, Some(system_fileref))
    }

    pub fn glk_fileref_delete_file(fileref: &GlkFileRef) {
        let fileref = lock!(fileref);
        S::fileref_delete(&fileref.system_fileref);
    }

    pub fn glk_fileref_destroy(&mut self, fileref: GlkFileRef) {
        self.filerefs.unregister(fileref);
    }

    pub fn glk_fileref_does_file_exist(fileref: &GlkFileRef) -> bool {
        let fileref = lock!(fileref);
        S::fileref_exists(&fileref.system_fileref)
    }

    pub fn glk_fileref_get_rock(&self, fileref: &GlkFileRef) -> GlkResult<u32> {
        self.filerefs.get_rock(fileref).ok_or(InvalidReference)
    }

    pub fn glk_fileref_iterate(&self, fileref: Option<&GlkFileRef>) -> Option<IterationResult<FileRef>> {
        self.filerefs.iterate(fileref)
    }

    pub fn glk_gestalt(&self, sel: u32, val: u32) -> u32 {
        self.glk_gestalt_ext(sel, val, None)
    }

    #[allow(non_upper_case_globals)]
    pub fn glk_gestalt_ext(&self, sel: u32, val: u32, buf: Option<&mut [u32]>) -> u32 {
        match sel {
            gestalt_Version => 0x00000705,

            gestalt_CharInput => {
                if let keycode_Func12..=keycode_Unknown = val {
                    1
                }
                else {
                    char::from_u32(val).map(|ch| ch.is_control() as u32).unwrap_or(0)
                }
            },

            gestalt_LineInput => if let 32..=126 = val {1} else {0},

            gestalt_CharOutput => {
                // Output is always one character, even if mangled
                if let Some(buf) = buf {
                    buf[0] = 1;
                }
                // We can print anything except control characters
                char::from_u32(val).map(|ch| if ch.is_control() {gestalt_CharOutput_CannotPrint} else {gestalt_CharOutput_ExactPrint}).unwrap_or(gestalt_CharOutput_CannotPrint)
            },

            gestalt_MouseInput | gestalt_Timer => self.support.timers as u32,

            //gestalt_Graphics | gestalt_DrawImage => 1,

            //gestalt_Sound | gestalt_SoundVolume | gestalt_SoundNotify => 1,

            gestalt_Hyperlinks | gestalt_HyperlinkInput => self.support.hyperlinks as u32,

            //gestalt_SoundMusic => 1,

            //gestalt_GraphicsTransparency => 1,

            gestalt_Unicode => 1,

            //gestalt_UnicodeNorm => 1,

            //gestalt_LineInputEcho => 1,

            //gestalt_LineTerminators | gestalt_LineTerminatorKey => 1,

            //gestalt_DateTime => 1,

            //gestalt_Sound2 => 1,

            //gestalt_ResourceStream => 1,

            //gestalt_GraphicsCharInput => 1,

            //gestalt_GarglkText => 1,

            _ => 0,
        }
    }

    pub fn glk_get_buffer_stream<'a>(str: &GlkStream, buf: &mut [u8]) -> GlkResult<'a, u32> {
        lock!(str).get_buffer(&mut GlkBufferMut::U8(buf))
    }

    pub fn glk_get_buffer_stream_uni<'a>(str: &GlkStream, buf: &mut [u32]) -> GlkResult<'a, u32> {
        lock!(str).get_buffer(&mut GlkBufferMut::U32(buf))
    }

    pub fn glk_get_char_stream(str: &GlkStream) -> GlkResult<i32> {
        lock!(str).get_char(false)
    }

    pub fn glk_get_char_stream_uni(str: &GlkStream) -> GlkResult<i32> {
        lock!(str).get_char(true)
    }

    pub fn glk_get_line_stream<'a>(str: &GlkStream, buf: &mut [u8]) -> GlkResult<'a, u32> {
        lock!(str).get_line(&mut GlkBufferMut::U8(buf))
    }

    pub fn glk_get_line_stream_uni<'a>(str: &GlkStream, buf: &mut [u32]) -> GlkResult<'a, u32> {
        lock!(str).get_line(&mut GlkBufferMut::U32(buf))
    }

    pub fn glk_put_buffer(&mut self, buf: &[u8]) -> GlkResult<()> {
        self.glk_put_buffer_stream(current_stream!(self), buf)
    }

    pub fn glk_put_buffer_stream<'a>(&mut self, str: &GlkStream, buf: &[u8]) -> GlkResult<'a, ()> {
        let mut str = lock!(str);
        str.put_buffer(&GlkBuffer::U8(buf))?;
        write_stream!(self, str);
        Ok(())
    }

    pub fn glk_put_buffer_stream_uni<'a>(&mut self, str: &GlkStream, buf: &[u32]) -> GlkResult<'a, ()> {
        let mut str = lock!(str);
        str.put_buffer(&GlkBuffer::U32(buf))?;
        write_stream!(self, str);
        Ok(())
    }

    pub fn glk_put_buffer_uni(&mut self, buf: &[u32]) -> GlkResult<()> {
        self.glk_put_buffer_stream_uni(current_stream!(self), buf)
    }

    pub fn glk_put_char(&mut self, ch: u8) -> GlkResult<()> {
        self.glk_put_char_stream(current_stream!(self), ch)
    }

    pub fn glk_put_char_stream(&mut self, str: &GlkStream, ch: u8) -> GlkResult<()> {
        let mut str = lock!(str);
        str.put_char(ch as u32)?;
        write_stream!(self, str);
        Ok(())
    }

    pub fn glk_put_char_stream_uni(&mut self, str: &GlkStream, ch: u32) -> GlkResult<()> {
        let mut str = lock!(str);
        str.put_char(ch)?;
        write_stream!(self, str);
        Ok(())
    }

    pub fn glk_put_char_uni(&mut self, ch: u32) -> GlkResult<()> {
        self.glk_put_char_stream_uni(current_stream!(self), ch)
    }

    pub fn glk_request_char_event(&self, win: &GlkWindow) -> GlkResult<()> {
        self.request_char_event(win, false)
    }

    pub fn glk_request_char_event_uni(&self, win: &GlkWindow) -> GlkResult<()> {
        self.request_char_event(win, true)
    }

    pub fn glk_request_hyperlink_event(win: &GlkWindow) {
        let mut win = lock!(win);
        if let WindowType::Buffer | WindowType::Grid = win.wintype {
            win.input.hyperlink = Some(true);
        }
    }

    pub fn glk_request_line_event(&self, win: &GlkWindow, buf: Box<[u8]>, initlen: u32) -> GlkResult<()> {
        self.request_line_event(win, GlkOwnedBuffer::U8(buf), initlen)
    }

    pub fn glk_request_line_event_uni(&self, win: &GlkWindow, buf: Box<[u32]>, initlen: u32) -> GlkResult<()> {
        self.request_line_event(win, GlkOwnedBuffer::U32(buf), initlen)
    }

    pub fn glk_request_mouse_event(win: &GlkWindow) {
        let mut win = lock!(win);
        if let WindowType::Graphics | WindowType::Grid = win.wintype {
            win.input.mouse = Some(true);
        }
    }

    pub fn glk_request_timer_events(&mut self, msecs: u32) {
        self.timer.interval = msecs;
        self.timer.started = if msecs > 0 {Some(SystemTime::now())} else {None}
    }

    pub fn glk_select_poll(&mut self) -> GlkEvent {
        // Assume we're single threaded, so the only event we could have received is a timer event
        if self.timer.interval > 0 {
            let now = SystemTime::now();
            let diff = now.duration_since(self.timer.started.unwrap());
            if let Ok(dur) = diff {
                if dur.as_millis() as u32 > self.timer.interval {
                    self.timer.last_interval = 0;
                    self.timer.started = None;
                    return GlkEvent {
                        evtype: GlkEventType::Timer,
                        ..Default::default()
                    }
                }
            }
        }

        GlkEvent::default()
    }

    pub fn glk_set_hyperlink(&self, val: u32) -> GlkResult<()> {
        GlkApi::<S>::glk_set_hyperlink_stream(current_stream!(self), val);
        Ok(())
    }

    pub fn glk_set_hyperlink_stream(str: &GlkStream, val: u32) {
        lock!(str).set_hyperlink(val);
    }

    pub fn glk_set_style(&self, val: u32) -> GlkResult<()> {
        GlkApi::<S>::glk_set_style_stream(current_stream!(self), val);
        Ok(())
    }

    pub fn glk_set_style_stream(str: &GlkStream, val: u32) {
        lock!(str).set_style(val);
    }

    pub fn glk_set_window(&mut self, win: Option<&GlkWindow>) {
        self.current_stream = win.map(|win| lock!(win).str.clone())
    }

    pub fn glk_stream_close(&mut self, str_glkobj: GlkStream) -> GlkResult<StreamResultCounts> {
        let str = lock!(str_glkobj);
        let res = str.close();
        write_stream!(self, str);
        drop(str);
        self.streams.unregister(str_glkobj);
        res
    }

    pub fn glk_stream_get_current(&self) -> Option<GlkStream> {
        self.current_stream.as_ref().map(|str| Into::<GlkStream>::into(str))
    }

    pub fn glk_stream_get_position(str: &GlkStream) -> u32 {
        lock!(str).get_position()
    }

    pub fn glk_stream_get_rock(&self, str: &GlkStream) -> GlkResult<u32> {
        self.streams.get_rock(str).ok_or(InvalidReference)
    }

    pub fn glk_stream_iterate(&self, str: Option<&GlkStream>) -> Option<IterationResult<Stream>> {
        self.streams.iterate(str)
    }

    pub fn glk_stream_open_file(&mut self, fileref: &GlkFileRef, mode: u32, rock: u32) -> GlkResult<Option<GlkStream>> {
        self.create_file_stream(fileref, mode, rock, false)
    }

    pub fn glk_stream_open_file_uni(&mut self, fileref: &GlkFileRef, mode: u32, rock: u32) -> GlkResult<Option<GlkStream>> {
        self.create_file_stream(fileref, mode, rock, true)
    }

    pub fn glk_stream_open_memory(&mut self, buf: Box<[u8]>, fmode: FileMode, rock: u32) -> GlkResult<GlkStream> {
        self.create_memory_stream(buf, fmode, rock)
    }

    pub fn glk_stream_open_memory_uni(&mut self, buf: Box<[u32]>, fmode: FileMode, rock: u32) -> GlkResult<GlkStream> {
        self.create_memory_stream(buf, fmode, rock)
    }

    pub fn glk_stream_set_current(&mut self, str: Option<&GlkStream>) {
        self.current_stream = str.map(|str| str.downgrade());
    }

    pub fn glk_stream_set_position(str: &GlkStream, mode: SeekMode, pos: i32) {
        lock!(str).set_position(mode, pos);
    }

    pub fn glk_stylehint_clear(&mut self, wintype: WindowType, style: u32, hint: u32) {
        let selector = format!("{}.Style_{}", if hint == stylehint_Justification {"div"} else {"span"}, style_name(style));
        let remove_styles = |stylehints: &mut WindowStyles| {
            if stylehints.contains_key(&selector) {
                let props = stylehints.get_mut(&selector).unwrap();
                props.remove(stylehint_name(hint));
                if props.is_empty() {
                    stylehints.remove(&selector);
                }
            }
        };

        if wintype == WindowType::All || wintype == WindowType::Buffer {
            remove_styles(&mut self.stylehints_buffer);
        }
        if wintype == WindowType::All || wintype == WindowType::Grid {
            remove_styles(&mut self.stylehints_grid);
        }
    }

    pub fn glk_stylehint_set(&mut self, wintype: WindowType, style: u32, hint: u32, value: i32) {
        if style >= style_NUMSTYLES || hint >= stylehint_NUMHINTS {
            return;
        }

        match wintype {
            WindowType::All => {
                self.glk_stylehint_set(WindowType::Buffer, style, hint, value);
                self.glk_stylehint_set(WindowType::Grid, style, hint, value);
                return;
            },
            WindowType::Blank | WindowType::Graphics | WindowType::Pair => {
                return;
            },
            _ => {},
        };

        let stylehints = if wintype == WindowType::Buffer {&mut self.stylehints_buffer} else {&mut self.stylehints_grid};
        let selector = format!("{}.Style_{}", if hint == stylehint_Justification {"div"} else {"span"}, style_name(style));

        #[allow(non_upper_case_globals)]
        let css_value = match hint {
            stylehint_Indentation | stylehint_ParaIndentation => CSSValue::String(format!("{}em", value)),
            stylehint_Justification => CSSValue::String(justification(value).to_string()),
            stylehint_Size => CSSValue::String(format!("{}em", 1.0 + (value as f64) * 0.1)),
            stylehint_Weight => CSSValue::String(font_weight(value).to_string()),
            stylehint_Oblique => CSSValue::String((if value == 0 {"normal"} else {"italic"}).to_string()),
            stylehint_Proportional => CSSValue::Number(value as f64),
            stylehint_TextColor | stylehint_BackColor => CSSValue::String(colour_code_to_css(value as u32)),
            stylehint_ReverseColor => CSSValue::Number(value as f64),
            _ => unreachable!(),
        };

        if !stylehints.contains_key(&selector) {
            stylehints.insert(selector.clone(), CSSProperties::default());
        }

        let props = stylehints.get_mut(&selector).unwrap();
        props.insert(stylehint_name(hint).to_string(), css_value);
    }

    pub fn glk_window_clear(win: &GlkWindow) {
        lock!(win).data.clear();
    }

    pub fn glk_window_close(&mut self, win_glkobj: GlkWindow) -> GlkResult<StreamResultCounts> {
        let win_ptr = win_glkobj.as_ptr();
        let win = lock!(win_glkobj);

        let str = Into::<GlkStream>::into(&win.str);
        let res = lock!(str).close();

        let root_win = self.root_window.as_ref().unwrap();
        if root_win.as_ptr() == win_ptr {
            // Close the root window, which means all windows
            let root_win = Into::<GlkWindow>::into(root_win);
            self.root_window = None;
            self.remove_window(root_win, true);
        }
        else {
            let parent_win_glkobj = Into::<GlkWindow>::into(win.parent.as_ref().unwrap());
            let parent_win_ptr = parent_win_glkobj.as_ptr();
            let parent_win = lock!(parent_win_glkobj);
            let grandparent_win = parent_win.parent.as_ref().map(|win| Into::<GlkWindow>::into(win));
            if let WindowData::Pair(data) = &parent_win.data {
                let sibling_win = if data.child1.as_ptr() == win_ptr {&data.child2} else {&data.child1};
                let sibling_win = Into::<GlkWindow>::into(sibling_win);
                if let Some(grandparent_win) = grandparent_win {
                    let mut grandparent_win = lock!(grandparent_win);
                    if let WindowData::Pair(ref mut data) = grandparent_win.data {
                        if data.child1.as_ptr() == parent_win_ptr {
                            data.child1 = sibling_win.downgrade();
                        }
                        else {
                            data.child2 = sibling_win.downgrade();
                        }
                    }
                    else {
                        unreachable!();
                    }
                }
                else {
                    self.root_window = Some(sibling_win.downgrade());
                    lock!(sibling_win).parent = None;
                }
                self.rearrange_window(&sibling_win, parent_win.wbox)?;

            }
            else {
                unreachable!();
            }

            drop(parent_win);
            drop(win);
            self.remove_window(win_glkobj, true);
            self.remove_window(parent_win_glkobj, false);
        }

        res
    }

    pub fn glk_window_get_arrangement(win: &GlkWindow) -> GlkResult<(u32, u32, GlkWindow)> {
        let win = lock!(win);
        if let WindowData::Pair(data) = &win.data {
            let keywin = Into::<GlkWindow>::into(&data.key);
            let method = data.dir | (if data.fixed {winmethod_Fixed} else {winmethod_Proportional}) | (if data.border {winmethod_Border} else {winmethod_NoBorder});
            Ok((method, data.size, keywin))
        }
        else {
            Err(NotPairWindow)
        }
    }

    pub fn glk_window_get_echo_stream(win: &GlkWindow) -> Option<GlkStream> {
        lock!(win).echostr.as_ref().map(|str| Into::<GlkStream>::into(str))
    }

    pub fn glk_window_get_parent(win: &GlkWindow) -> Option<GlkWindow> {
        lock!(win).parent.as_ref().map(|win| Into::<GlkWindow>::into(win))
    }

    pub fn glk_window_get_rock(&self, win: &GlkWindow) -> GlkResult<u32> {
        self.windows.get_rock(win).ok_or(InvalidReference)
    }

    pub fn glk_window_get_root(&self) -> Option<GlkWindow> {
        self.root_window.as_ref().map(|win| Into::<GlkWindow>::into(win))
    }

    pub fn glk_window_get_sibling(win: &GlkWindow) -> GlkResult<Option<GlkWindow>> {
        let win_ptr = win.as_ptr();
        let win = lock!(win);
        if let Some(parent) = &win.parent {
            let parent = Into::<GlkWindow>::into(parent);
            let parent = lock!(parent);
            if let WindowData::Pair(data) = &parent.data {
                if data.child1.as_ptr() == win_ptr {
                    return Ok(Some(Into::<GlkWindow>::into(&data.child2)));
                }
                Ok(Some(Into::<GlkWindow>::into(&data.child1)))
            }
            else {
                Err(NotPairWindow)
            }
        }
        else {
            Ok(None)
        }
    }

    pub fn glk_window_get_size(&self, win: &GlkWindow) -> (usize, usize) {
        let win = lock!(win);
        match &win.data {
            WindowData::Buffer(_) => (
                normalise_window_dimension((win.wbox.bottom - win.wbox.top - self.metrics.buffermarginy) / self.metrics.buffercharheight),
                normalise_window_dimension((win.wbox.right - win.wbox.left - self.metrics.buffermarginx) / self.metrics.buffercharwidth),
            ),
            WindowData::Graphics(data) => (data.height, data.width),
            WindowData::Grid(data) => (data.data.height, data.data.width),
            _ => (0, 0),
        }
    }

    pub fn glk_window_get_stream(win: &GlkWindow) -> GlkStream {
        (&lock!(win).str).into()
    }

    pub fn glk_window_get_type(win: &GlkWindow) -> WindowType {
        lock!(win).wintype
    }

    pub fn glk_window_iterate(&self, win: Option<&GlkWindow>) -> Option<IterationResult<Window>> {
        self.windows.iterate(win)
    }

    pub fn glk_window_move_cursor(win: &GlkWindow, xpos: usize, ypos: usize) -> GlkResult<()> {
        let mut win = lock!(win);
        if let WindowData::Grid(data) = &mut win.data {
            data.data.x = xpos;
            data.data.y = ypos;
            Ok(())
        }
        else {
            Err(NotGridWindow)
        }
    }

    pub fn glk_window_open(&mut self, splitwin: Option<&GlkWindow>, method: u32, size: u32, wintype: WindowType, rock: u32) -> GlkResult<GlkWindow> {
        if self.root_window.is_none() {
            if splitwin.is_some() {
                return Err(SplitMustBeNull);
            }
        }
        else if let Some(splitwin) = splitwin {
            validate_winmethod(method, lock!(splitwin).wintype)?;
        }
        else {
            return Err(InvalidSplitwin);
        }

        // Create the window
        let windata = match wintype {
            WindowType::Blank => BlankWindow {}.into(),
            WindowType::Buffer => TextWindow::<BufferWindow>::new(&self.stylehints_buffer).into(),
            // Todo: graphics
            WindowType::Grid => TextWindow::<GridWindow>::new(&self.stylehints_grid).into(),
            _ => {return Err(InvalidWindowType);}
        };
        let (win, str) = Window::new(windata, wintype);
        self.windows.register(&win, rock);
        self.streams.register(&str, 0);

        // Rearrange the windows for the new window
        if let Some(splitwin) = splitwin {
            // Set up the pairwindata before turning it into a full window
            let mut pairwindata = PairWindow::new(&win, method, size);
            pairwindata.child1 = splitwin.downgrade();
            pairwindata.child2 = win.downgrade();

            // Now the pairwin object can be created and registered
            let (pairwin, pairwinstr) = Window::new(PairWindow::default().into(), WindowType::Pair);
            self.windows.register(&pairwin, 0);
            self.streams.register(&pairwinstr, 0);

            // Set up the rest of the relations
            let mut splitwin_inner = lock!(splitwin);
            let old_parent = splitwin_inner.parent.as_ref().map(|win| Into::<GlkWindow>::into(win));
            lock!(pairwin).parent = old_parent.as_ref().map(|win| win.downgrade());
            splitwin_inner.parent = Some(pairwin.downgrade());
            lock!(win).parent = Some(pairwin.downgrade());

            if let Some(old_parent) = old_parent {
                let mut old_parent_inner = lock!(old_parent);
                if let WindowData::Pair(old_parent_inner) = &mut old_parent_inner.data {
                    if old_parent_inner.child1.as_ptr() == splitwin.as_ptr() {
                        old_parent_inner.child1 = pairwin.downgrade();
                    }
                    else {
                        old_parent_inner.child2 = pairwin.downgrade();
                    }
                }
                else {
                    unreachable!();
                }
            }
            else {
                self.root_window = Some(pairwin.downgrade());
            }
            let wbox = splitwin_inner.wbox;
            self.rearrange_window(&pairwin, wbox)?;
        }
        else {
            self.root_window = Some(win.downgrade());
            self.rearrange_window(&win, WindowBox {
                bottom: self.metrics.height,
                right: self.metrics.width,
                ..Default::default()
            })?;
        }

        Ok(win)
    }

    pub fn glk_window_set_arrangement(&mut self, win_glkobj: &GlkWindow, method: u32, size: u32, keywin: Option<&GlkWindow>) -> GlkResult<()> {
        let win_ptr = win_glkobj.as_ptr();
        let mut win = lock!(win_glkobj);
        if let WindowData::Pair(data) = &mut win.data {
            // Check the keywin is valid
            if let Some(keywin_glkobj) = keywin {
                let keywin = lock!(keywin_glkobj);
                if keywin.wintype == WindowType::Pair {
                    return Err(KeywinCantBePair);
                }
                let mut win_parent = keywin_glkobj.downgrade();
                loop {
                    if win_parent.as_ptr() == win_ptr {
                        break;
                    }
                    let parent = Into::<GlkWindow>::into(&win_parent);
                    let parent = lock!(parent);
                    let parent = &parent.parent;
                    if let Some(parent) = parent {
                        win_parent = parent.clone();
                    }
                    else {
                        return Err(KeywinMustBeDescendant);
                    }
                }
            }

            let new_dir = method & winmethod_DirMask;
            let new_fixed = (method & winmethod_DivisionMask) == winmethod_Fixed;
            let new_vertical = new_dir == winmethod_Left || new_dir == winmethod_Right;
            let win_keywin = Into::<GlkWindow>::into(&data.key);
            let keywin = keywin.unwrap_or(&win_keywin);
            let keywin = lock!(keywin);
            if new_vertical && !data.vertical {
                return Err(CannotChangeWindowSplitDirection);
            }
            if new_fixed && keywin.wintype == WindowType::Blank {
                return Err(InvalidWindowDivisionBlank);
            }

            let new_backward = new_dir == winmethod_Left || new_dir == winmethod_Above;
            if new_backward != data.backward {
                // Switch the children
                mem::swap(&mut data.child1, &mut data.child2);
            }

            // Update the window
            data.backward = new_backward;
            data.border = (method & winmethod_BorderMask) == winmethod_BorderMask;
            data.dir = new_dir;
            data.fixed = new_fixed;
            data.key = win_keywin.downgrade();
            data.size = size;

            self.rearrange_window(win_glkobj, win.wbox)?;

            Ok(())
        }
        else {
            Err(NotPairWindow)
        }
    }

    pub fn glk_window_set_echo_stream(win: &GlkWindow, str: Option<&GlkStream>) {
        lock!(win).echostr = str.map(|str| str.downgrade());
    }

    // Internal functions

    fn create_fileref(&mut self, filename: String, rock: u32, usage: u32, system_fileref: Option<SystemFileRef>) -> GlkFileRef {
        let system_fileref = system_fileref.unwrap_or_else(|| {
            S::fileref_construct(filename, file_type(usage & fileusage_TypeMask), None)
        });

        let fref = FileRef::new(system_fileref, usage);
        let fref_glkobj = GlkObject::new(fref);
        self.filerefs.register(&fref_glkobj, rock);
        fref_glkobj
    }

    fn create_file_stream(&mut self, fileref: &GlkFileRef, mode: u32, rock: u32, uni: bool) -> GlkResult<Option<GlkStream>> {
        let fileref = lock!(fileref);
        let mode = file_mode(mode)?;
        if mode == FileMode::Read && !S::fileref_exists(&fileref.system_fileref) {
            return Ok(None);
        }

        let data: Vec<u8> = if mode == FileMode::Write {
            vec![]
        }
        else {
            S::fileref_read(&fileref.system_fileref)?.to_vec()
        };

        // Create an appopriate stream
        let str = create_stream_from_buffer(data, fileref.binary, mode, uni, &fileref.system_fileref)?;

        if mode == FileMode::WriteAppend {
            let mut str = lock!(str);
            str.set_position(SeekMode::End, 0);
        }

        self.streams.register(&str, rock);

        Ok(Some(str))
    }

    fn create_memory_stream<T>(&mut self, buf: Box<[T]>, fmode: FileMode, rock: u32) -> GlkResult<GlkStream>
    where Stream: From<ArrayBackedStream<T>> {
        if fmode == FileMode::WriteAppend {
            return Err(IllegalFilemode);
        }
        let str = GlkObject::new(if buf.len() == 0 {
            NullStream::default().into()
        }
        else {
            ArrayBackedStream::<T>::new(buf, fmode, None).into()
        });
        self.streams.register(&str, rock);
        Ok(str)
    }

    fn handle_line_input<T>(&mut self, win: &mut Window, win_data: &mut TextWindow<T>, input: &str, termkey: Option<TerminatorCode>) -> GlkResult<GlkEvent>
    where T: Default + WindowOperations {
        // The Glk spec is a bit ambiguous here
        // I'm going to echo first
        if win_data.request_echo_line_input {
            let mut input_linebreak = input.to_string();
            input_linebreak.push_str("\n");
            win.put_string(&input_linebreak, Some(style_Input));
            if let Some(str) = &win.echostr {
                let str: GlkStream = str.into();
                let mut str = lock!(str);
                str.put_string(&input_linebreak, Some(style_Input))?;
                write_stream!(self, str);
            }
        }

        // Convert the input to a buffer and copy into the window's buffer
        let src: GlkOwnedBuffer = input.into();
        let dest = win_data.line_input_buffer.as_mut().unwrap();
        let len = min(src.len(), dest.len());
        let src_unowned: GlkBuffer = (&src).into();
        let mut dest_unowned: GlkBufferMut = (dest).into();
        set_buffer(&src_unowned, 0, &mut dest_unowned, 0, len);

        // TODO: Unretain

        win.input.text_input_type = None;
        win_data.line_input_buffer = None;

        Ok(GlkEvent {
            evtype: GlkEventType::Line,
            val1: src.len() as u32,
            val2: termkey.map_or(0, |termkey| termkey as u32),
            ..Default::default()
        })
    }

    fn rearrange_window(&mut self, win: &GlkWindow, wbox: WindowBox) -> GlkResult<()> {
        self.windows_changed = true;
        let mut win = lock!(win);
        win.wbox = wbox;
        let boxheight = wbox.bottom - wbox.top;
        let boxwidth = wbox.right - wbox.left;

        // Adjust anything that needs adjusting
        match &mut win.data {
            WindowData::Graphics(win) => {
                win.height = normalise_window_dimension(boxheight - self.metrics.graphicsmarginy);
                win.width = normalise_window_dimension(boxwidth - self.metrics.graphicsmarginx);
            },
            WindowData::Grid(win) => {
                let height = normalise_window_dimension((boxheight - self.metrics.gridmarginy) / self.metrics.gridcharheight);
                let width = normalise_window_dimension((boxwidth - self.metrics.gridmarginx) / self.metrics.gridcharwidth);
                win.data.update_size(height, width);
            },
            WindowData::Pair(win) => {
                let (min, max, mut splitwidth) = if win.vertical {
                    (wbox.left, wbox.right, self.metrics.inspacingx)
                }
                else {
                    (wbox.top, wbox.bottom, self.metrics.inspacingy)
                };
                if !win.border {
                    splitwidth = 0.0;
                }
                let diff = max - min;

                // Calculate the split size
                let mut split = if win.fixed {
                    let keywin = Into::<GlkWindow>::into(&win.key);
                    let keywin = lock!(keywin);
                    match keywin.wintype {
                        WindowType::Buffer => if win.vertical {
                                win.size as f64 * self.metrics.buffercharwidth + self.metrics.buffermarginx
                            }
                            else {
                                win.size as f64 * self.metrics.buffercharheight + self.metrics.buffermarginy
                            }
                        WindowType::Graphics => win.size as f64 + (if win.vertical {self.metrics.graphicsmarginx} else {self.metrics.graphicsmarginy}),
                        WindowType::Grid => if win.vertical {
                                win.size as f64 * self.metrics.gridcharwidth + self.metrics.gridmarginx
                            }
                            else {
                                win.size as f64 * self.metrics.gridcharheight + self.metrics.gridmarginy
                            }
                        _ => unreachable!(),
                    }
                }
                else {
                    ((win.size as f64 * diff) / 100.0).floor()
                };

                // split is now a number between 0 and diff; now convert it to a number between min and max, and apply upside-down-ness
                split = if win.backward {min + split} else {max - split - splitwidth};

                // Make sure it really is between min and max
                split = if min >= max {min} else {split.max(min).min(max - splitwidth)};

                // Construct the two child boxes
                let (mut box1, mut box2) = if win.vertical {
                    (WindowBox {
                        bottom: wbox.bottom,
                        left: wbox.left,
                        right: split,
                        top: wbox.top,
                    }, WindowBox {
                        bottom: wbox.bottom,
                        left: split + splitwidth,
                        right: wbox.right,
                        top: wbox.top,
                    })
                }
                else {
                    (WindowBox {
                        bottom: split,
                        left: wbox.left,
                        right: wbox.right,
                        top: wbox.top,
                    }, WindowBox {
                        bottom: wbox.bottom,
                        left: wbox.left,
                        right: wbox.right,
                        top: split + splitwidth,
                    })
                };
                if win.backward {
                    mem::swap(&mut box1, &mut box2);
                }
                self.rearrange_window(&Into::<GlkWindow>::into(&win.child1), box1)?;
                self.rearrange_window(&Into::<GlkWindow>::into(&win.child2), box2)?;
            },
            _ => {},
        };

        Ok(())
    }

    fn remove_window(&mut self, win_glkobj: GlkWindow, recurse: bool) {
        self.windows_changed = true;

        // TODO: unretain array

        let win = lock!(win_glkobj);
        if let WindowData::Pair(data) = &win.data {
            if recurse {
                self.remove_window(Into::<GlkWindow>::into(&data.child1), true);
                self.remove_window(Into::<GlkWindow>::into(&data.child2), true);
            }
        }

        let str = Into::<GlkStream>::into(&win.str);
        self.streams.unregister(str);
        drop(win);
        self.windows.unregister(win_glkobj);
    }

    fn request_char_event(&self, win: &GlkWindow, uni: bool) -> GlkResult<()> {
        let mut win = lock!(win);
        if win.input.text_input_type.is_some() {
            return Err(PendingKeyboardRequest);
        }
        if let WindowType::Blank | WindowType::Pair = win.wintype {
            return Err(WindowDoesntSupportCharInput);
        }

        win.input.gen = Some(self.gen);
        win.input.text_input_type = Some(TextInputType::Char);
        win.uni_char_input = uni;

        Ok(())
    }

    fn request_line_event(&self, win: &GlkWindow, buf: GlkOwnedBuffer, initlen: u32) -> GlkResult<()> {
        let mut win = lock!(win);

        if win.input.text_input_type.is_some() {
            return Err(PendingKeyboardRequest);
        }
        if let WindowType::Buffer | WindowType::Grid = win.wintype {}
        else {
            return Err(WindowDoesntSupportCharInput);
        }

        win.input.gen = Some(self.gen);
        if initlen > 0 {
            //win.input.initial
        }
        win.input.text_input_type = Some(TextInputType::Line);
        match win.data {
            WindowData::Buffer(ref mut data) => {
                data.line_input_buffer = Some(buf);
                data.request_echo_line_input = data.data.echo_line_input;
            },
            WindowData::Grid(ref mut data) => {
                data.line_input_buffer = Some(buf);
            },
            _ => unreachable!(),
        };

        // TODO: retain array

        Ok(())
    }
}

/** A Glk event */
#[derive(Default)]
pub struct GlkEvent {
    pub evtype: GlkEventType,
    pub win: Option<GlkWindow>,
    pub val1: u32,
    pub val2: u32,
}

/** Final read/write character counts of a stream */
#[derive(Clone, Copy)]
#[repr(C)]
pub struct StreamResultCounts {
    pub read_count: u32,
    pub write_count: u32,
}

#[derive(Default)]
struct SupportedFeatures {
    hyperlinks: bool,
    timers: bool,
}

#[derive(Default)]
struct TimerData {
    interval: u32,
    last_interval: u32,
    started: Option<SystemTime>,
}

fn colour_code_to_css(colour: u32) -> String {
    // Uppercase colours are required by RegTest
    format!("#{:6X}", colour & 0xFFFFFF)
}

fn create_stream_from_buffer(buf: Vec<u8>, binary: bool, mode: FileMode, unicode: bool, fileref: &SystemFileRef) -> GlkResult<'static, GlkStream> {
    let data = match (unicode, binary) {
        (false, _) => GlkOwnedBuffer::U8(buf.into_boxed_slice()),
        (true, false) => str::from_utf8(&buf)?.into(),
        (true, true) => GlkOwnedBuffer::U32(u8slice_to_u32vec(&buf).into_boxed_slice()),
    };

    let str = GlkObject::new(if mode == FileMode::Read {
        match data {
            GlkOwnedBuffer::U8(buf) => ArrayBackedStream::<u8>::new(buf, mode, None).into(),
            GlkOwnedBuffer::U32(buf) => ArrayBackedStream::<u32>::new(buf, mode, None).into(),
        }
    }
    else {
        match data {
            GlkOwnedBuffer::U8(buf) => FileStream::<u8>::new(fileref, buf, mode).into(),
            GlkOwnedBuffer::U32(buf) => FileStream::<u32>::new(fileref, buf, mode).into(),
        }
    });
    Ok(str)
}

fn normalise_window_dimension(val: f64) -> usize {
    val.floor().max(0.0) as usize
}