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
mod protocol;
mod streams;
mod windows;

use std::cmp::min;
use std::mem;
use std::time::SystemTime;

use arrays::*;
use common::*;
use GlkApiError::*;
use constants::*;
use objects::*;
use protocol::*;
use streams::*;
use windows::*;

// Expose for so they can be turned into pointers
pub use objects::GlkObject;
pub use streams::Stream;
pub use windows::Window;

#[derive(Default)]
pub struct GlkApi {
    current_stream: Option<GlkStreamWeak>,
    gen: u32,
    metrics: NormalisedMetrics,
    partial_inputs: PartialInputs,
    root_window: Option<GlkWindowWeak>,
    streams: GlkObjectStore<Stream>,
    stylehints_buffer: WindowStyles,
    stylehints_grid: WindowStyles,
    timer: TimerData,
    windows: GlkObjectStore<Window>,
    windows_changed: bool,
}

impl GlkApi {
    // The Glk API

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
        current_stream!(self).put_buffer(&GlkBuffer::U8(buf))
    }

    pub fn glk_put_buffer_stream<'a>(str: &GlkStream, buf: &[u8]) -> GlkResult<'a, ()> {
        lock!(str).put_buffer(&GlkBuffer::U8(buf))
    }

    pub fn glk_put_buffer_stream_uni<'a>(str: &GlkStream, buf: &[u32]) -> GlkResult<'a, ()> {
        lock!(str).put_buffer(&GlkBuffer::U32(buf))
    }

    pub fn glk_put_buffer_uni(&mut self, buf: &[u32]) -> GlkResult<()> {
        current_stream!(self).put_buffer(&GlkBuffer::U32(buf))
    }

    pub fn glk_put_char(&mut self, ch: u8) -> GlkResult<()> {
        current_stream!(self).put_char(ch as u32)
    }

    pub fn glk_put_char_stream(str: &GlkStream, ch: u8) -> GlkResult<()> {
        lock!(str).put_char(ch as u32)
    }

    pub fn glk_put_char_stream_uni(str: &GlkStream, ch: u32) -> GlkResult<()> {
        lock!(str).put_char(ch)
    }

    pub fn glk_put_char_uni(&mut self, ch: u32) -> GlkResult<()> {
        current_stream!(self).put_char(ch)
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

    pub fn glk_set_hyperlink(&self, val: u32) -> GlkResult<()> {
        Ok(current_stream!(self).set_hyperlink(val))
    }

    pub fn glk_set_hyperlink_stream(str: &GlkStream, val: u32) {
        lock!(str).set_hyperlink(val);
    }

    pub fn glk_set_style(&self, val: u32) -> GlkResult<()> {
        Ok(current_stream!(self).set_style(val))
    }

    pub fn glk_set_style_stream(str: &GlkStream, val: u32) {
        lock!(str).set_style(val);
    }

    pub fn glk_set_window(&mut self, win: Option<&GlkWindow>) {
        self.current_stream = win.map(|win| lock!(win).str.clone())
    }

    pub fn glk_stream_close(&mut self, str: GlkStream) -> GlkResult<StreamResultCounts> {
        let res = lock!(str).close();
        self.streams.unregister(str);
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

    fn handle_line_input<T>(&self, win: &mut Window, win_data: &mut TextWindow<T>, input: &str, termkey: Option<TerminatorCode>) -> GlkResult<GlkEvent>
    where T: Default + WindowOperations {
        // The Glk spec is a bit ambiguous here
        // I'm going to echo first
        if win_data.request_echo_line_input {
            let mut input_linebreak = input.to_string();
            input_linebreak.push_str("\n");
            win.put_string(&input_linebreak, Some(style_Input));
            if let Some(str) = &win.echostr {
                let str: GlkStream = str.into();
                lock!(str).put_string(&input_linebreak, Some(style_Input))?;
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
struct TimerData {
    interval: u32,
    last_interval: u32,
    started: Option<SystemTime>,
}

fn colour_code_to_css(colour: u32) -> String {
    // Uppercase colours are required by RegTest
    format!("#{:6X}", colour & 0xFFFFFF)
}

fn normalise_window_dimension(val: f64) -> usize {
    val.floor().min(0.0) as usize
}