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
pub mod objects;
pub mod protocol;
mod schannels;
mod streams;
mod windows;

use std::cmp::min;
use std::ffi::c_char;
use std::iter::zip;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::str;
use std::time::SystemTime;

use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime, TimeZone, Timelike, Utc};
use unicode_normalization::UnicodeNormalization;

use super::*;
pub use arrays::*;
use blorb::*;
pub use common::*;
pub use GlkApiError::*;
use constants::*;
use filerefs::*;
use objects::*;
use protocol::*;
use schannels::*;
use streams::*;
pub use streams::StreamOperations;
use windows::*;

// Expose for so they can be turned into pointers
pub use filerefs::FileRef;
pub use objects::{GlkObject, GlkObjectMetadata};
pub use schannels::SoundChannel;
pub use streams::Stream;
pub use windows::Window;

#[derive(Default)]
pub struct GlkApi<S>
where S: Default + GlkSystem {
    buffer_window_count: u32,
    current_stream: Option<GlkStreamWeak>,
    exited: bool,
    pub filerefs: GlkObjectStore<FileRef>,
    pub dirs: Directories,
    gen: u32,
    metrics: NormalisedMetrics,
    partial_inputs: PartialInputs,
    pub retain_array_callbacks_u8: Option<RetainArrayCallbacks<u8>>,
    pub retain_array_callbacks_u32: Option<RetainArrayCallbacks<u32>>,
    root_window: Option<GlkWindowWeak>,
    page_margin: PageMargin,
    pub schannels: GlkObjectStore<SoundChannel>,
    schannels_changed: bool,
    special: Option<SpecialInput>,
    pub streams: GlkObjectStore<Stream>,
    stylehints_buffer: WindowStyles,
    stylehints_grid: WindowStyles,
    support: SupportedFeatures,
    pub system: S,
    tempfile_counter: u32,
    timer: TimerData,
    pub windows: GlkObjectStore<Window>,
    windows_changed: bool,
}

impl<S> GlkApi<S>
where S: Default + GlkSystem {
    pub fn new(system: S) -> Self {
        GlkApi {
            dirs: S::get_directories(),
            system,
            ..Default::default()
        }
    }

    // The Glk API

    pub fn glk_buffer_canon_decompose_uni(buf: &mut [u32], initlen: usize) -> usize {
        let res = u32slice_to_string(&buf[..initlen]).nfd().map(|ch| ch as u32).collect::<Vec<u32>>();
        write_common_buffer(&res, buf)
    }

    pub fn glk_buffer_canon_normalize_uni(buf: &mut [u32], initlen: usize) -> usize {
        let res = u32slice_to_string(&buf[..initlen]).nfc().map(|ch| ch as u32).collect::<Vec<u32>>();
        write_common_buffer(&res, buf)
    }

    pub fn glk_buffer_to_lower_case_uni(buf: &mut [u32], initlen: usize) -> usize {
        let res = str_to_u32vec(&u32slice_to_string(&buf[..initlen]).to_lowercase());
        write_common_buffer(&res, buf)
    }

    pub fn glk_buffer_to_title_case_uni(buf: &mut [u32], initlen: usize, lowerrest: bool) -> usize {
        let titlecase = unicode_case_mapping::to_titlecase(char::from_u32(buf[0]).unwrap());
        let mut res = vec![if titlecase[0] > 0 {titlecase[0]} else {buf[0]}];
        if titlecase[1] > 0 {
            res.push(titlecase[1]);
        }
        if titlecase[2] > 0 {
            res.push(titlecase[2]);
        }
        if lowerrest {
            res.append(&mut str_to_u32vec(&u32slice_to_string(&buf[1..initlen]).to_lowercase()));
        }
        else {
            res.extend_from_slice(&buf[1..initlen]);
        }
        write_common_buffer(&res, buf)
    }

    pub fn glk_buffer_to_upper_case_uni(buf: &mut [u32], initlen: usize) -> usize {
        let res = str_to_u32vec(&u32slice_to_string(&buf[..initlen]).to_uppercase());
        write_common_buffer(&res, buf)
    }

    pub fn glk_cancel_char_event(win: &GlkWindow) {
        lock!(win).input.text_input_type = None;
    }

    pub fn glk_cancel_hyperlink_event(win: &GlkWindow) {
        lock!(win).input.hyperlink = false;
    }

    pub fn glk_cancel_line_event(&mut self, win_glkobj: &GlkWindow) -> GlkResult<GlkEvent> {
        let (id, text_input_type) = {
            let win = lock!(win_glkobj);
            (win.id, win.input.text_input_type)
        };
        if let Some(TextInputType::Line) = text_input_type {
            let partial = self.partial_inputs.as_mut().and_then(|partials| partials.remove(&id)).unwrap_or("".to_string());
            // Do we need to drop win here?
            let res = self.handle_line_input(win_glkobj, &partial, None)?;
            Ok(res)
        }
        else {
            Ok(GlkEvent::default())
        }
    }

    pub fn glk_cancel_mouse_event(win: &GlkWindow) {
        lock!(win).input.mouse = false;
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

    pub fn glk_current_simple_time(factor: u32) -> i32 {
        datetime_to_simpletime(&Local::now(), factor)
    }

    pub fn glk_current_time() -> GlkTime {
        datetime_to_glktime(&Local::now())
    }

    pub fn glk_date_to_simple_time_local(date: &GlkDate, factor: u32) -> i32 {
        datetime_to_simpletime(&glkdate_to_datetime(Local, date), factor)
    }

    pub fn glk_date_to_simple_time_utc(date: &GlkDate, factor: u32) -> i32 {
        datetime_to_simpletime(&glkdate_to_datetime(Utc, date), factor)
    }

    pub fn glk_date_to_time_local(date: &GlkDate) -> GlkTime {
        datetime_to_glktime(&glkdate_to_datetime(Local, date))
    }

    pub fn glk_date_to_time_utc(date: &GlkDate) -> GlkTime {
        datetime_to_glktime(&glkdate_to_datetime(Utc, date))
    }

    pub fn glk_exit(&mut self) {
        self.exited = true;
        self.delete_temp_files();
        let update = self.update();
        self.system.send_glkote_update(update);
    }

    pub fn glk_fileref_create_by_name(&mut self, usage: u32, filename: String, rock: u32) -> GlkFileRef {
        let filetype = file_type(usage & fileusage_TypeMask);
        let path = self.dirs.working.join(clean_filename(filename, filetype)).to_str().unwrap().to_owned();
        self.create_fileref(path, rock, usage)
    }

    pub fn glk_fileref_create_by_prompt(&mut self, usage: u32, fmode: FileMode, rock: u32) -> GlkResult<Option<GlkFileRef>> {
        let filetype = file_type(usage & fileusage_TypeMask);
        self.special = Some(SpecialInput {
            filemode: fmode,
            filetype,
            // TODO: gameid
            gameid: None,
            ..Default::default()
        });
        let update = self.update();
        self.system.send_glkote_update(update);
        let event = self.system.get_glkote_event();
        if let Some(event) = event {
            let res = self.handle_event(event)?;
            if let Some(fref) = res.fref {
                let filename = match fref {
                    FileRefResponse::Fref(fref) => fref.filename,
                    FileRefResponse::Path(path) => path,
                };
                // If we're given a full file path, great! If not, add an extension and set relative to the working dir
                let mut path = self.dirs.working.join(filename);
                if path.extension().is_none() {
                    path.set_extension(&filetype_suffix(filetype)[1..]);
                }
                return Ok(Some(self.create_fileref(path.to_str().unwrap().to_owned(), rock, usage)));
            }
        }
        else {
            self.glk_exit();
        }
        Ok(None)
    }

    pub fn glk_fileref_create_from_fileref(&mut self, usage: u32, fileref: &GlkFileRef, rock: u32) -> GlkFileRef {
        let fileref = lock!(fileref);
        self.create_fileref(fileref.path.clone(), rock, usage)
    }

    pub fn glk_fileref_create_temp(&mut self, usage: u32, rock: u32) -> GlkFileRef {
        let path = self.temp_file_path(self.tempfile_counter);
        self.tempfile_counter += 1;
        self.create_fileref(path, rock, usage)
    }

    pub fn glk_fileref_delete_file(&mut self, fileref: &GlkFileRef) {
        let fileref = lock!(fileref);
        self.system.file_delete(&fileref.path);
    }

    pub fn glk_fileref_destroy(&mut self, fileref: GlkFileRef) {
        self.filerefs.unregister(fileref);
    }

    pub fn glk_fileref_does_file_exist(&mut self, fileref: &GlkFileRef) -> bool {
        let fileref = lock!(fileref);
        self.system.file_exists(&fileref.path)
    }

    pub fn glk_fileref_get_rock(fileref: &GlkFileRef) -> GlkResult<u32> {
        Ok(lock!(fileref).rock)
    }

    pub fn glk_fileref_iterate(&self, fileref: Option<&GlkFileRef>) -> Option<GlkFileRef> {
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

            gestalt_MouseInput => (val == wintype_TextGrid || (self.support.graphics && val == wintype_Graphics)) as u32,

            gestalt_Timer => self.support.timers as u32,

            gestalt_Graphics | gestalt_GraphicsTransparency | gestalt_GraphicsCharInput => self.support.graphics as u32,

            gestalt_DrawImage => (self.support.graphics && (val == wintype_Graphics || val == wintype_TextBuffer)) as u32,

            gestalt_Sound | gestalt_SoundVolume | gestalt_SoundNotify | gestalt_SoundMusic | gestalt_Sound2 => self.support.sounds as u32,

            gestalt_Hyperlinks => self.support.hyperlinks as u32,

            gestalt_HyperlinkInput => (self.support.hyperlinks && (val == wintype_TextBuffer || val == wintype_TextGrid))  as u32,

            gestalt_Unicode => 1,

            gestalt_UnicodeNorm => 1,

            gestalt_LineInputEcho => 1,

            gestalt_LineTerminators => 1,

            gestalt_LineTerminatorKey => (val == keycode_Escape || (keycode_Func12..=keycode_Func1).contains(&val)) as u32,

            gestalt_DateTime => 1,

            gestalt_ResourceStream => 1,

            gestalt_GarglkText | gestalt_Stylehints => self.support.garglktext as u32,

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

    pub fn glk_image_draw(win: &GlkWindow, image: u32, val1: i32, val2: i32) -> u32 {
        let info = get_image_info(image);
        if let Some(info) = info {
            let height = info.height;
            let width = info.width;
            GlkApi::<S>::draw_image(win, info, height, val1, val2, width)
        }
        else {
            0
        }
    }

    pub fn glk_image_draw_scaled(win: &GlkWindow, image: u32, val1: i32, val2: i32, width: u32, height: u32) -> u32 {
        let info = get_image_info(image);
        if let Some(info) = info {
            GlkApi::<S>::draw_image(win, info, height, val1, val2, width)
        }
        else {
            0
        }
    }

    pub fn glk_image_get_info(image: u32) -> Option<ImageInfo> {
        get_image_info(image)
    }

    pub fn glk_put_buffer(&mut self, buf: &[u8]) -> GlkResult<()> {
        self.glk_put_buffer_stream(current_stream!(self), buf)
    }

    pub fn glk_put_buffer_stream<'a>(&mut self, str: &GlkStream, buf: &[u8]) -> GlkResult<'a, ()> {
        let mut str = lock!(str);
        str.put_buffer(&GlkBuffer::U8(buf))?;
        Ok(())
    }

    pub fn glk_put_buffer_stream_uni<'a>(&mut self, str: &GlkStream, buf: &[u32]) -> GlkResult<'a, ()> {
        let mut str = lock!(str);
        str.put_buffer(&GlkBuffer::U32(buf))?;
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
        Ok(())
    }

    pub fn glk_put_char_stream_uni(&mut self, str: &GlkStream, ch: u32) -> GlkResult<()> {
        let mut str = lock!(str);
        str.put_char(ch)?;
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
            win.input.hyperlink = true;
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
            win.input.mouse = true;
        }
    }

    pub fn glk_request_timer_events(&mut self, msecs: u32) {
        self.timer.interval = msecs;
        self.timer.started = if msecs > 0 {Some(SystemTime::now())} else {None}
    }

    pub fn glk_schannel_create(&mut self, rock: u32) -> SoundChannelRef {
        self.schannels_changed = true;
        let schannel = SoundChannel::default();
        let schannel_glkobj = GlkObject::new(schannel);
        self.schannels.register(&schannel_glkobj, rock);
        schannel_glkobj
    }

    pub fn glk_schannel_create_ext(&mut self, rock: u32, vol: u32) -> SoundChannelRef {
        let schannel_glkobj = self.glk_schannel_create(rock);
        {
            let mut schannel = lock!(schannel_glkobj);
            schannel.ops.push(SoundChannelOperation::Volume(SetVolumeOperation {
                vol: (vol as f64 / SCHANNEL_MAX_VOL),
                ..Default::default()
            }));
        }
        schannel_glkobj
    }

    pub fn glk_schannel_destroy(&mut self, schannel: SoundChannelRef) {
        self.schannels_changed = true;
        self.schannels.unregister(schannel);
    }

    pub fn glk_schannel_get_rock(schannel: &SoundChannelRef) -> GlkResult<u32> {
        Ok(lock!(schannel).rock)
    }

    pub fn glk_schannel_iterate(&self, schannel: Option<&SoundChannelRef>) -> Option<SoundChannelRef> {
        self.schannels.iterate(schannel)
    }

    pub fn glk_schannel_pause(&mut self, schannel: &SoundChannelRef) {
        self.schannels_changed = true;
        let mut schannel = lock!(schannel);
        schannel.ops.push(SoundChannelOperation::Pause);
    }

    pub fn glk_schannel_play(&mut self, schannel: &SoundChannelRef, snd: u32) -> u32 {
        self.glk_schannel_play_ext(schannel, snd, 1, 0)
    }

    pub fn glk_schannel_play_ext(&mut self, schannel: &SoundChannelRef, snd: u32, repeats: u32, notify: u32) -> u32 {
        let mut schannel = lock!(schannel);
        if repeats == 0 {
            schannel.ops.push(SoundChannelOperation::Stop);
        }
        else {
            if let Some(data) = get_blorb_resource(giblorb_ID_Snd, snd) {
                let id = &data[0..4];
                // For now only support Ogg/Vorbis and AIFF
                if id == b"OggS" || (id == b"FORM" && &data[8..12] == b"AIFF") {
                    schannel.ops.push(SoundChannelOperation::Play(PlayOperation {
                        notify: if notify != 0 {Some(notify)} else {None},
                        repeats: if repeats != 1 {Some(repeats)} else {None},
                        snd,
                    }));
                }
                else {
                    return 0;
                }
            }
            else {
                return 0;
            }
        }
        self.schannels_changed = true;
        // TODO: check for previous play operations?
        // TODO: return 0 for MOD resources?
        1
    }

    pub fn glk_schannel_play_multi(&mut self, schannels: Vec<SoundChannelRef>, sounds: &[u32], notify: u32) -> u32 {
        zip(schannels, sounds).fold(0, |acc, sound| {
            let (schannel, &snd) = sound;
            acc + self.glk_schannel_play_ext(&schannel, snd, 1, notify)
        })
    }

    pub fn glk_schannel_set_volume(&mut self, schannel: &SoundChannelRef, vol: u32) {
        self.glk_schannel_set_volume_ext(schannel, vol, 0, 0);
    }

    pub fn glk_schannel_set_volume_ext(&mut self, schannel: &SoundChannelRef, vol: u32, duration: u32, notify: u32) {
        self.schannels_changed = true;
        let mut schannel = lock!(schannel);
        schannel.ops.push(SoundChannelOperation::Volume(SetVolumeOperation {
            dur: if duration > 0 {Some(duration)} else {None},
            notify: if notify > 0 {Some(notify)} else {None},
            vol: (vol as f64 / SCHANNEL_MAX_VOL),
        }));
    }

    pub fn glk_schannel_stop(&mut self, schannel: &SoundChannelRef) {
        self.schannels_changed = true;
        let mut schannel = lock!(schannel);
        schannel.ops.push(SoundChannelOperation::Stop);
    }

    pub fn glk_schannel_unpause(&mut self, schannel: &SoundChannelRef) {
        self.schannels_changed = true;
        let mut schannel = lock!(schannel);
        schannel.ops.push(SoundChannelOperation::Unpause);
    }

    pub fn glk_select(&mut self) -> GlkResult<GlkEvent> {
        let update = self.update();
        self.system.send_glkote_update(update);
        let event = self.system.get_glkote_event();
        if let Some(event) = event {
            self.handle_event(event)
        }
        else {
            self.glk_exit();
            Ok(GlkEvent::default())
        }
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

    pub fn glk_set_echo_line_event(win: &GlkWindow, val: u32) {
        let mut win = lock!(win);
        if let WindowData::Buffer(data) = &mut win.data {
            data.data.echo_line_input = val > 0;
        }
    }

    pub fn glk_set_hyperlink(&self, val: u32) -> GlkResult<()> {
        GlkApi::<S>::glk_set_hyperlink_stream(current_stream!(self), val);
        Ok(())
    }

    pub fn glk_set_hyperlink_stream(str: &GlkStream, val: u32) {
        let str = lock!(str);
        window_stream_operation!(str, set_hyperlink, val);
    }

    pub fn glk_set_style(&self, val: u32) -> GlkResult<()> {
        GlkApi::<S>::glk_set_style_stream(current_stream!(self), val);
        Ok(())
    }

    pub fn glk_set_style_stream(str: &GlkStream, val: u32) {
        let str = lock!(str);
        window_stream_operation!(str, set_style, val);
    }

    pub fn glk_set_terminators_line_event(win: &GlkWindow, keycodes: Vec<TerminatorCode>) {
        let mut win = lock!(win);
        win.input.terminators = keycodes;
    }

    pub fn glk_set_window(&mut self, win: Option<&GlkWindow>) {
        self.current_stream = win.map(|win| lock!(win).str.clone())
    }

    pub fn glk_simple_time_to_date_local(time: i32, factor: u32) -> GlkDate {
        let time: DateTime<Local> = Local.timestamp_millis_opt(time as i64 * 1000 * factor as i64).unwrap();
        datetime_to_glkdate(&time)
    }

    pub fn glk_simple_time_to_date_utc(time: i32, factor: u32) -> GlkDate {
        let time: DateTime<Utc> = Utc.timestamp_millis_opt(time as i64 * 1000 * factor as i64).unwrap();
        datetime_to_glkdate(&time)
    }

    pub fn glk_stream_close(&mut self, str_glkobj: GlkStream) -> GlkResult<StreamResultCounts> {
        let str_ptr = str_glkobj.as_ptr();
        let mut str = lock!(str_glkobj);
        if matches!(str.deref().deref(), Stream::Window(_)) {
            return Err(GlkApiError::CannotCloseWindowStream);
        }

        if let Some(current_stream) = &self.current_stream {
            if current_stream.as_ptr() == str_ptr {
                self.current_stream = None;
            }
        }

        let res = str.close();
        if let Some((fileref, buf)) = stream_to_file_buffer(&mut str) {
            self.system.file_write_buffer(fileref, buf);
        }

        let disprock = str.array_disprock;
        match str.deref_mut().deref_mut() {
            Stream::ArrayBackedU8(str) => self.unretain_array(GlkOwnedBuffer::U8(str.take_buffer()), disprock),
            Stream::ArrayBackedU32(str) => self.unretain_array(GlkOwnedBuffer::U32(str.take_buffer()), disprock),
            _ => {},
        };

        drop(str);
        self.streams.unregister(str_glkobj);
        Ok(res)
    }

    pub fn glk_stream_get_current(&self) -> Option<GlkStream> {
        self.current_stream.as_ref().map(Into::<GlkStream>::into)
    }

    pub fn glk_stream_get_position(str: &GlkStream) -> u32 {
        lock!(str).get_position()
    }

    pub fn glk_stream_get_rock(str: &GlkStream) -> GlkResult<u32> {
        Ok(lock!(str).rock)
    }

    pub fn glk_stream_iterate(&self, str: Option<&GlkStream>) -> Option<GlkStream> {
        self.streams.iterate(str)
    }

    pub fn glk_stream_open_file(&mut self, fileref: &GlkFileRef, mode: FileMode, rock: u32) -> GlkResult<Option<GlkStream>> {
        self.create_file_stream(fileref, mode, rock, false)
    }

    pub fn glk_stream_open_file_uni(&mut self, fileref: &GlkFileRef, mode: FileMode, rock: u32) -> GlkResult<Option<GlkStream>> {
        self.create_file_stream(fileref, mode, rock, true)
    }

    pub fn glk_stream_open_memory(&mut self, buf: Box<[u8]>, fmode: FileMode, rock: u32) -> GlkResult<GlkStream> {
        let disprock = if buf.is_empty() {None} else {self.retain_array_callbacks_u8.as_ref().map(|_| {
            self.retain_array(&GlkBuffer::U8(&buf))
        })};
        self.create_memory_stream(buf, fmode, rock, disprock)
    }

    pub fn glk_stream_open_memory_uni(&mut self, buf: Box<[u32]>, fmode: FileMode, rock: u32) -> GlkResult<GlkStream> {
        let disprock = if buf.is_empty() {None} else {self.retain_array_callbacks_u8.as_ref().map(|_| {
            self.retain_array(&GlkBuffer::U32(&buf))
        })};
        self.create_memory_stream(buf, fmode, rock, disprock)
    }

    pub fn glk_stream_open_resource(&mut self, filenum: u32, rock: u32) -> GlkResult<Option<GlkStream>> {
        self.create_resource_stream(filenum, rock, false)
    }

    pub fn glk_stream_open_resource_uni(&mut self, filenum: u32, rock: u32) -> GlkResult<Option<GlkStream>> {
        self.create_resource_stream(filenum, rock, true)
    }

    pub fn glk_stream_set_current(&mut self, str: Option<&GlkStream>) {
        self.current_stream = str.map(|str| str.downgrade());
    }

    pub fn glk_stream_set_position(str: &GlkStream, pos: i32, mode: SeekMode) {
        lock!(str).set_position(mode, pos);
    }

    pub fn glk_stylehint_clear(&mut self, wintype: WindowType, style: u32, hint: u32) {
        let selector = format!(".Style_{}{}", style_name(style), if hint <= stylehint_Justification {"_par"} else {""});
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
            if style == style_Normal && hint == stylehint_BackColor {
                self.page_margin.set_stylehint(None);
            }
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
        let selector = format!(".Style_{}{}", style_name(style), if hint <= stylehint_Justification {"_par"} else {""});

        #[allow(non_upper_case_globals)]
        let css_value = match hint {
            stylehint_Indentation | stylehint_ParaIndentation => CSSValue::String(format!("{}em", value)),
            stylehint_Justification => CSSValue::String(justification(value).to_string()),
            stylehint_Size => CSSValue::String(format!("{}em", 1.0 + (value as f64) * 0.1)),
            stylehint_Weight => CSSValue::String(font_weight(value).to_string()),
            stylehint_Oblique => CSSValue::String((if value == 0 {"normal"} else {"italic"}).to_string()),
            stylehint_Proportional => CSSValue::Number(if value == 0 {1} else {0} as f64),
            stylehint_TextColor | stylehint_BackColor => CSSValue::String(colour_code_to_css(value as u32)),
            stylehint_ReverseColor => CSSValue::Number(value as f64),
            _ => unreachable!(),
        };

        if !stylehints.contains_key(&selector) {
            stylehints.insert(selector.clone(), CSSProperties::default());
        }

        let props = stylehints.get_mut(&selector).unwrap();
        props.insert(stylehint_name(hint).to_string(), css_value);

        if wintype == WindowType::Buffer && style == style_Normal && hint == stylehint_BackColor {
            self.page_margin.set_stylehint(Some(value as u32));
        }
    }

    pub fn glk_time_to_date_local(time: &GlkTime) -> GlkDate {
        let time = glktime_to_datetime(Local, time);
        datetime_to_glkdate(&time)
    }

    pub fn glk_time_to_date_utc(time: &GlkTime) -> GlkDate {
        let time = glktime_to_datetime(Utc, time);
        datetime_to_glkdate(&time)
    }

    pub fn glk_window_clear(&mut self, win: &GlkWindow) {
        let mut win = lock!(win);
        let colour = win.data.clear();
        if match win.data {
            WindowData::Buffer(_) => true,
            WindowData::Grid(_) => self.buffer_window_count == 0,
            _ => false,
        } {
            self.page_margin.set_garglk(colour);
        }
    }

    pub fn glk_window_close(&mut self, win_glkobj: GlkWindow) -> GlkResult<StreamResultCounts> {
        let win_ptr = win_glkobj.as_ptr();
        let win = lock!(win_glkobj);

        let str = Into::<GlkStream>::into(&win.str);
        let res = lock!(str).close();
        drop(str);

        if win.wintype == WindowType::Buffer {
            self.buffer_window_count -= 1;
        }

        let root_win = self.root_window.as_ref().unwrap();
        if root_win.as_ptr() == win_ptr {
            // Close the root window, which means all windows
            self.root_window = None;
            drop(win);
            self.remove_window(win_glkobj, true);
        }
        else {
            let parent_win_glkobj = Into::<GlkWindow>::into(win.parent.as_ref().unwrap());
            let parent_win_ptr = parent_win_glkobj.as_ptr();
            let parent_win = lock!(parent_win_glkobj);
            let grandparent_win = parent_win.parent.as_ref().map(Into::<GlkWindow>::into);
            if let WindowData::Pair(data) = &parent_win.data {
                let sibling_win = if data.child1.as_ptr() == win_ptr {&data.child2} else {&data.child1};
                let sibling_win = Into::<GlkWindow>::into(sibling_win);
                if let Some(grandparent_win_glkobj) = grandparent_win {
                    let mut grandparent_win = lock!(grandparent_win_glkobj);
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
                    lock!(sibling_win).parent = Some(grandparent_win_glkobj.downgrade());
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

        Ok(res)
    }

    pub fn glk_window_erase_rect(win: &GlkWindow, left: i32, top: i32, width: u32, height: u32) -> GlkResult<()> {
        let mut win = lock!(win);
        if let WindowData::Graphics(data) = &mut win.data {
            data.draw.push(GraphicsWindowOperation::Fill(FillOperation {
                color: None,
                height: Some(height),
                x: Some(left),
                y: Some(top),
                width: Some(width),
            }));
            Ok(())
        }
        else {
            Err(NotGraphicsWindow)
        }
    }

    pub fn glk_window_fill_rect(win: &GlkWindow, colour: u32, left: i32, top: i32, width: u32, height: u32) -> GlkResult<()> {
        let mut win = lock!(win);
        if let WindowData::Graphics(data) = &mut win.data {
            data.draw.push(GraphicsWindowOperation::Fill(FillOperation {
                color: Some(colour_code_to_css(colour)),
                height: Some(height),
                x: Some(left),
                y: Some(top),
                width: Some(width),
            }));
            Ok(())
        }
        else {
            Err(NotGraphicsWindow)
        }
    }

    pub fn glk_window_flow_break(win: &GlkWindow) {
        let mut win = lock!(win);
        if let WindowData::Buffer(data) = &mut win.data {
            data.data.set_flow_break();
        }
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
        lock!(win).echostr.as_ref().map(Into::<GlkStream>::into)
    }

    pub fn glk_window_get_parent(win: &GlkWindow) -> Option<GlkWindow> {
        lock!(win).parent.as_ref().map(Into::<GlkWindow>::into)
    }

    pub fn glk_window_get_rock(win: &GlkWindow) -> GlkResult<u32> {
        Ok(lock!(win).rock)
    }

    pub fn glk_window_get_root(&self) -> Option<GlkWindow> {
        self.root_window.as_ref().map(Into::<GlkWindow>::into)
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

    pub fn glk_window_iterate(&self, win: Option<&GlkWindow>) -> Option<GlkWindow> {
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
        else if splitwin.is_some() {
            // Check method
            let division = method & winmethod_DivisionMask;
            let direction = method & winmethod_DirMask;
            if division != winmethod_Fixed && division != winmethod_Proportional {
                return Err(InvalidWindowDivision)
            }
            if division == winmethod_Fixed && wintype == WindowType::Blank {
                return Err(InvalidWindowDivisionBlank)
            }
            #[allow(non_upper_case_globals)]
            if let winmethod_Above | winmethod_Below | winmethod_Left | winmethod_Right = direction {}
            else {
                return Err(InvalidWindowDirection)
            }
        }
        else {
            return Err(InvalidSplitwin);
        }

        // Create the window
        let windata = match wintype {
            WindowType::Blank => BlankWindow {}.into(),
            WindowType::Buffer => {
                self.buffer_window_count += 1;
                TextWindow::<BufferWindow>::new(&self.stylehints_buffer).into()
            },
            WindowType::Graphics => GraphicsWindow::default().into(),
            WindowType::Grid => TextWindow::<GridWindow>::new(&self.stylehints_grid).into(),
            _ => {return Err(InvalidWindowType);}
        };
        // TODO: try new_cyclic
        let (win_glkobj, str) = Window::new(windata, self.windows.next_id(), rock, wintype);
        self.windows.register(&win_glkobj, rock);
        self.streams.register(&str, 0);

        // Rearrange the windows for the new window
        if let Some(splitwin_glkobj) = splitwin {
            // Set up the pairwindata before turning it into a full window
            let mut pairwindata = PairWindow::new(&win_glkobj, method, size);
            pairwindata.child1 = splitwin_glkobj.downgrade();
            pairwindata.child2 = win_glkobj.downgrade();

            // Now the pairwin object can be created and registered
            let (pairwin_glkobj, pairwinstr) = Window::new(pairwindata.into(), self.windows.next_id(), 0, WindowType::Pair);
            self.windows.register(&pairwin_glkobj, 0);
            self.streams.register(&pairwinstr, 0);

            // Set up the rest of the relations
            let mut splitwin = lock!(splitwin_glkobj);
            let wbox = splitwin.wbox;
            let grandparent_opt = splitwin.parent.as_ref().map(Into::<GlkWindow>::into);
            lock!(pairwin_glkobj).parent = grandparent_opt.as_ref().map(|win| win.downgrade());
            splitwin.parent = Some(pairwin_glkobj.downgrade());
            lock!(win_glkobj).parent = Some(pairwin_glkobj.downgrade());
            drop(splitwin);

            if let Some(grandparent_glkobj) = grandparent_opt {
                let mut grandparent = lock!(grandparent_glkobj);
                if let WindowData::Pair(grandparent_data) = &mut grandparent.data {
                    if grandparent_data.child1.as_ptr() == splitwin_glkobj.as_ptr() {
                        grandparent_data.child1 = pairwin_glkobj.downgrade();
                    }
                    else {
                        grandparent_data.child2 = pairwin_glkobj.downgrade();
                    }
                }
                else {
                    return Err(SplitParentIsntPair);
                }
            }
            else {
                self.root_window = Some(pairwin_glkobj.downgrade());
            }
            self.rearrange_window(&pairwin_glkobj, wbox)?;
        }
        else {
            self.root_window = Some(win_glkobj.downgrade());
            self.rearrange_window(&win_glkobj, WindowBox {
                bottom: self.metrics.height,
                right: self.metrics.width,
                ..Default::default()
            })?;
        }

        Ok(win_glkobj)
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

            let wbox = win.wbox;
            drop(keywin);
            drop(win);
            self.rearrange_window(win_glkobj, wbox)?;

            Ok(())
        }
        else {
            Err(NotPairWindow)
        }
    }

    pub fn glk_window_set_background_color(win: &GlkWindow, colour: u32) -> GlkResult<()> {
        let mut win = lock!(win);
        if let WindowData::Graphics(data) = &mut win.data {
            data.draw.push(GraphicsWindowOperation::SetColor(SetColorOperation {
                color: colour_code_to_css(colour),
            }));
            Ok(())
        }
        else {
            Err(NotGraphicsWindow)
        }
    }

    pub fn glk_window_set_echo_stream(win: &GlkWindow, str: Option<&GlkStream>) {
        lock!(win).echostr = str.map(|str| str.downgrade());
    }

    // Extensions

    pub fn garglk_set_reversevideo(&self, val: u32) -> GlkResult<()> {
        GlkApi::<S>::garglk_set_reversevideo_stream(current_stream!(self), val);
        Ok(())
    }

    pub fn garglk_set_reversevideo_stream(str: &GlkStream, val: u32) {
        let str = lock!(str);
        window_stream_operation!(str, set_css, "reverse", if val != 0 {Some(&CSSValue::Number(1.0))} else {None});
    }

    pub fn garglk_set_zcolors(&self, fg: u32, bg: u32) -> GlkResult<()> {
        GlkApi::<S>::garglk_set_zcolors_stream(current_stream!(self), fg, bg);
        Ok(())
    }

    pub fn garglk_set_zcolors_stream(str: &GlkStream, fg: u32, bg: u32) {
        let str = lock!(str);
        window_stream_operation!(str, set_colours, fg, bg);
    }

    pub fn glkunix_fileref_create_by_name_uncleaned(&mut self, usage: u32, filename: String, rock: u32) -> GlkFileRef {
        let path = self.dirs.system_cwd.join(filename).to_str().unwrap().to_owned();
        self.create_fileref(path, rock, usage)
    }

    pub fn glkunix_set_base_file(&mut self, path: String) {
        S::set_base_file(&mut self.dirs, path);
    }

    // The GlkOte protocol functions

    pub fn get_glkote_init(&mut self) {
        let event = self.system.get_glkote_event();
        if let Some(event) = event {
            self.handle_event(event).unwrap();
        }
        else {
            self.glk_exit();
        }
    }

    pub fn handle_event(&mut self, mut event: Event) -> GlkResult<GlkEvent> {
        if event.gen != self.gen {
            if let EventData::Init(_) = event.data {}
            else {
                return Err(WrongGeneration(self.gen, event.gen));
            }
        }
        self.gen += 1;

        self.partial_inputs = event.partial.take();

        let mut glkevent = GlkEvent::default();
        match event.data {
            EventData::Init(data) => {
                self.metrics = normalise_metrics(data.metrics)?;
                for support in data.support {
                    match support.as_ref() {
                        "garglktext" => self.support.garglktext = true,
                        "graphics" => self.support.graphics = true,
                        "hyperlinks" => self.support.hyperlinks = true,
                        "sounds" => self.support.sounds = true,
                        "timer" => self.support.timers = true,
                        _ => {},
                    };
                }
            },

            EventData::Arrange(data) => {
                self.metrics = normalise_metrics(data.metrics)?;
                if let Some(win) = self.root_window.as_ref() {
                    let win = Into::<GlkWindow>::into(win);
                    self.rearrange_window(&win, WindowBox {
                        bottom: self.metrics.height,
                        right: self.metrics.width,
                        ..Default::default()
                    })?;
                }
                glkevent = GlkEvent {
                    evtype: GlkEventType::Arrange,
                    ..Default::default()
                };
            },

            EventData::Char(data) => {
                if let Some(win_glkobj) = self.windows.get_by_id(data.window) {
                    let mut win = lock!(win_glkobj);
                    if let Some(TextInputType::Char) = win.input.text_input_type {
                        win.input.text_input_type = None;
                        // We can't simply use the length of value, because it's a UTF-8 byte array
                        let mut chars = data.value.chars();
                        let first_char = chars.next().unwrap() as u32;
                        let val = if chars.next().is_none() {
                            if !win.uni_char_input && first_char < keycode_Func12 && first_char > MAX_LATIN1 {
                                QUESTION_MARK
                            }
                            else {
                                first_char
                            }
                        }
                        else {
                            key_name_to_code(&data.value)
                        };
                        glkevent = GlkEvent {
                            evtype: GlkEventType::Char,
                            win: Some(win_glkobj.clone()),
                            val1: val,
                            ..Default::default()
                        };
                    }
                }
            },

            EventData::Hyperlink(data) => {
                if let Some(win_glkobj) = self.windows.get_by_id(data.window) {
                    let mut win = lock!(win_glkobj);
                    if win.input.hyperlink {
                        win.input.hyperlink = false;
                        glkevent = GlkEvent {
                            evtype: GlkEventType::Hyperlink,
                            win: Some(win_glkobj.clone()),
                            val1: data.value,
                            ..Default::default()
                        };
                    }
                }
            },

            EventData::Line(data) => {
                if let Some(win_glkobj) = self.windows.get_by_id(data.window) {
                    let text_input_type = lock!(win_glkobj).input.text_input_type;
                    if let Some(TextInputType::Line) = text_input_type {
                        glkevent = self.handle_line_input(&win_glkobj, &data.value, data.terminator)?;
                    }
                }
            },

            EventData::Mouse(data) => {
                if let Some(win_glkobj) = self.windows.get_by_id(data.window) {
                    let mut win = lock!(win_glkobj);
                    if win.input.mouse {
                        win.input.mouse = false;
                        glkevent = GlkEvent {
                            evtype: GlkEventType::Mouse,
                            win: Some(win_glkobj.clone()),
                            val1: data.x,
                            val2: data.y,
                            ..Default::default()
                        };
                    }
                }
            },

            EventData::Redraw(_) => {
                glkevent = GlkEvent {
                    evtype: GlkEventType::Redraw,
                    ..Default::default()
                }
            },

            EventData::Sound(data) => {
                glkevent = GlkEvent {
                    evtype: GlkEventType::SoundNotify,
                    val1: data.snd,
                    val2: data.notify,
                    ..Default::default()
                }
            },

            EventData::Special(data) => {
                glkevent = GlkEvent {
                    fref: data.value,
                    ..Default::default()
                }
            },

            EventData::Timer(_) => {
                self.timer.started = Some(SystemTime::now());
                glkevent = GlkEvent {
                    evtype: GlkEventType::Timer,
                    ..Default::default()
                }
            },

            EventData::Volume(data) => {
                glkevent = GlkEvent {
                    evtype: GlkEventType::VolumeNotify,
                    val1: 0,
                    val2: data.notify,
                    ..Default::default()
                }
            },

            _ => {
                return Err(EventNotSupported);
            },
        };

        Ok(glkevent)
    }

    fn update(&mut self) -> Update {
        let mut state = StateUpdate {
            gen: self.gen,
            ..Default::default()
        };

        if self.exited {
            state.disable = true;
        }

        // Get the window updates
        for win in self.windows.iter() {
            let mut win = lock!(win);
            if let WindowData::Blank(_) | WindowData::Pair(_) = win.data {
                continue;
            }
            let mut update = win.update();
            if let Some(content) = update.content.take() {
                state.content.push(content);
            }
            if update.input.hyperlink || update.input.mouse || update.input.text_input_type.is_some() {
                state.input.push(update.input);
            }
            if self.windows_changed {
                state.windows.push(update.size);
            }
        }
        self.windows_changed = false;

        let page_margin_bg = self.page_margin.get_page_margin_bg();
        if page_margin_bg != self.page_margin.transmitted {
            state.page_margin_bg = page_margin_bg.map(colour_code_to_css);
            self.page_margin.transmitted = page_margin_bg;
        }

        if self.schannels_changed {
            self.schannels_changed = false;
            state.schannels = self.schannels.iter().map(|schannel| {
                let mut schannel = lock!(schannel);
                let ops = mem::take(&mut schannel.ops);
                protocol::SoundChannelUpdate {
                    id: schannel.id,
                    ops,
                }
            }).collect();
        }

        state.specialinput = mem::take(&mut self.special);

        // Timer
        if self.timer.last_interval != self.timer.interval {
            state.timer = Some(self.timer.interval);
            self.timer.last_interval = self.timer.interval;
        }

        // TODO: Autorestore state

        self.flush_file_streams();

        Update::State(state)
    }

    // Internal functions

    fn create_fileref(&mut self, path: String, rock: u32, usage: u32) -> GlkFileRef {
        let fref = FileRef::new(path, usage);
        let fref_glkobj = GlkObject::new(fref);
        self.filerefs.register(&fref_glkobj, rock);
        fref_glkobj
    }

    fn create_file_stream(&mut self, fileref: &GlkFileRef, mode: FileMode, rock: u32, uni: bool) -> GlkResult<Option<GlkStream>> {
        let (binary, path) = {
            let fileref = lock!(fileref);
            (fileref.binary, fileref.path.clone())
        };
        if mode == FileMode::Read && !self.system.file_exists(&path) {
            return Ok(None);
        }

        // Read in the data, or create a blank file
        let data = if mode == FileMode::Write {
            None
        }
        else {
            self.system.file_read(&path)
        };
        let data: GlkResult<Box<[u8]>> = data.map_or_else(|| {
            self.system.file_write_buffer(&path, vec![].into_boxed_slice());
            Ok(vec![].into_boxed_slice())
        }, Ok);
        let data = data?;

        // Create an appopriate stream
        let str = create_stream_from_buffer(data, binary, mode, uni, Some(fileref))?;

        if mode == FileMode::WriteAppend {
            let mut str = lock!(str);
            str.set_position(SeekMode::End, 0);
        }

        self.streams.register(&str, rock);

        Ok(Some(str))
    }

    fn create_memory_stream<T>(&mut self, buf: Box<[T]>, fmode: FileMode, rock: u32, disprock: Option<DispatchRock>) -> GlkResult<GlkStream>
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
        if disprock.is_some() {
            let mut str = lock!(str);
            str.array_disprock = disprock;
        }
        self.streams.register(&str, rock);
        Ok(str)
    }

    fn create_resource_stream(&mut self, filenum: u32, rock: u32, uni: bool) -> GlkResult<Option<GlkStream>> {
        let resource = get_blorb_data_resource(filenum);
        if let Some(resource) = resource {
            // Create an appopriate stream
            let str = create_stream_from_buffer(resource.data.into(), resource.binary, FileMode::Read, uni, None)?;
            self.streams.register(&str, rock);
            Ok(Some(str))
        }
        else {
            Ok(None)
        }
    }

    fn delete_temp_files(&mut self) {
        for file_num in 0..self.tempfile_counter {
            let path = self.temp_file_path(file_num);
            self.system.file_delete(&path);
        }
    }

    fn draw_image(win: &GlkWindow, info: ImageInfo, height: u32, val1: i32, val2: i32, width: u32) -> u32 {
        let mut win = lock!(win);
        match &mut win.data {
            WindowData::Buffer(data) => {
                data.data.put_image(BufferWindowImage {
                    alignment: image_alignment(val1),
                    alttext: None,
                    height,
                    image: info.image,
                    hyperlink: None,
                    width,
                });
                1
            },
            WindowData::Graphics(data) => {
                data.draw.push(GraphicsWindowOperation::Image(ImageOperation {
                    height,
                    image: info.image,
                    width,
                    x: val1,
                    y: val2,
                }));
                1
            },
            _ => 0,
        }
    }

    fn flush_file_streams(&mut self) {
        for str in self.streams.iter() {
            let mut str = lock!(str);
            if let Some((fileref, buf)) = stream_to_file_buffer(&mut str) {
                self.system.file_write_buffer(fileref, buf);
            }
        }
        self.system.flush_writeable_files();
    }

    fn handle_line_input(&mut self, win_glkobj: &GlkWindow, input: &str, termkey: Option<TerminatorCode>) -> GlkResult<GlkEvent> {
        let mut win = lock!(win_glkobj);
        let (request_echo_line_input, mut line_input_buffer) = match &mut win.data {
            WindowData::Buffer(data) => (data.request_echo_line_input, data.line_input_buffer.take().unwrap()),
            WindowData::Grid(data) => (data.request_echo_line_input, data.line_input_buffer.take().unwrap()),
            _ => unreachable!(),
        };

        // The Glk spec is a bit ambiguous here
        // I'm going to echo first
        if request_echo_line_input {
            let mut input_linebreak = input.to_string();
            input_linebreak.push('\n');
            win.put_string(&input_linebreak, Some(style_Input));
            if let Some(str) = &win.echostr {
                let str: GlkStream = str.into();
                let mut str = lock!(str);
                str.put_string(&input_linebreak, Some(style_Input))?;
            }
        }

        // Convert the input to a buffer and copy into the window's buffer
        let src: GlkOwnedBuffer = input.into();
        let len = min(src.len(), line_input_buffer.len());
        let src_unowned: GlkBuffer = (&src).into();
        let mut dest_unowned: GlkBufferMut = (&mut line_input_buffer).into();
        set_buffer(&src_unowned, 0, &mut dest_unowned, 0, len);

        self.unretain_array(line_input_buffer, win.array_disprock);

        win.input.text_input_type = None;

        Ok(GlkEvent {
            evtype: GlkEventType::Line,
            win: Some(win_glkobj.clone()),
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
                        WindowType::Buffer => if win.size > 0 {
                            if win.vertical {
                                win.size as f64 * self.metrics.buffercharwidth + self.metrics.buffermarginx
                            }
                            else {
                                win.size as f64 * self.metrics.buffercharheight + self.metrics.buffermarginy
                            }
                        } else {0.0},
                        WindowType::Graphics => if win.size > 0 {win.size as f64 + (if win.vertical {self.metrics.graphicsmarginx} else {self.metrics.graphicsmarginy})} else {0.0},
                        WindowType::Grid => if win.size > 0 {
                            if win.vertical {
                                win.size as f64 * self.metrics.gridcharwidth + self.metrics.gridmarginx
                            }
                            else {
                                win.size as f64 * self.metrics.gridcharheight + self.metrics.gridmarginy
                            }
                        } else {0.0},
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
        let mut win = lock!(win_glkobj);

        match &mut win.data {
            WindowData::Buffer(data) => {
                if let Some(buf) = data.line_input_buffer.take() {
                    self.unretain_array(buf, win.array_disprock);
                }
            },
            WindowData::Grid(data) => {
                if let Some(buf) = data.line_input_buffer.take() {
                    self.unretain_array(buf, win.array_disprock);
                }
            },
            _ => {},
        }

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
        if let WindowType::Blank | WindowType::Pair = win.wintype {
            return Err(WindowDoesntSupportCharInput);
        }
        // Some games expect lax implementations which allow multiple calls to `glk_request_char_event`. We'll only raise an error if the existing input type is not the same as what is currently be requested.
        // See https://github.com/iftechfoundation/ifarchive-if-specs/issues/25
        if let Some(text_input_type) = win.input.text_input_type {
            if text_input_type == TextInputType::Char && uni == win.uni_char_input {
                return Ok(());
            }
            return Err(PendingKeyboardRequest);
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
            return Err(WindowDoesntSupportLineInput);
        }

        if self.retain_array_callbacks_u8.is_some() {
            win.array_disprock = Some(self.retain_array(&(&buf).into()));
        }

        win.input.gen = Some(self.gen);
        if initlen > 0 {
            win.input.initial = Some(buf.to_string(initlen as usize));
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

        Ok(())
    }

    pub fn retain_array(&self, buf: &GlkBuffer) -> DispatchRock {
        match buf {
            GlkBuffer::U8(buf) => (self.retain_array_callbacks_u8.as_ref().unwrap().retain)(buf.as_ptr(), buf.len() as u32, c"&+#!Cn".as_ptr()),
            GlkBuffer::U32(buf) => (self.retain_array_callbacks_u32.as_ref().unwrap().retain)(buf.as_ptr(), buf.len() as u32, c"&+#!Iu".as_ptr()),
        }
    }

    fn temp_file_path(&self, file_num: u32) -> String {
        let filename = format!("remglktempfile-{}", file_num);
        self.dirs.temp.join(filename).to_str().unwrap().to_owned()
    }

    /** Unretain an array, or leak if no callbacks setup */
    pub fn unretain_array(&self, buf: GlkOwnedBuffer, disprock: Option<DispatchRock>) {
        let len = buf.len() as u32;
        match buf {
            GlkOwnedBuffer::U8(buf) => {
                let leaked_buf = Box::leak(buf);
                if let Some(disprock) = disprock {
                    (self.retain_array_callbacks_u8.as_ref().unwrap().unretain)(leaked_buf.as_ptr(), len, c"&+#!Cn".as_ptr(), disprock);
                }
            },
            GlkOwnedBuffer::U32(buf) => {
                let leaked_buf = Box::leak(buf);
                if let Some(disprock) = disprock {
                    (self.retain_array_callbacks_u32.as_ref().unwrap().unretain)(leaked_buf.as_ptr(), len, c"&+#!Iu".as_ptr(), disprock);
                }
            },
        };
    }
}

#[derive(Default)]
pub struct Directories {
    /** The storyfile directory, used by `garglk_add_resource_from_file` */
    pub storyfile: PathBuf,
    /** The system current working directory, used by `glkunix_stream_open_pathname` */
    pub system_cwd: PathBuf,
    /** Temp folder */
    pub temp: PathBuf,
    /** The Glk "current directory", used by `glk_fileref_create_by_name`/`glk_fileref_create_by_prompt` */
    pub working: PathBuf,
}

/** A Glk event */
#[derive(Default)]
pub struct GlkEvent {
    pub evtype: GlkEventType,
    pub fref: Option<FileRefResponse>,
    pub win: Option<GlkWindow>,
    pub val1: u32,
    pub val2: u32,
}

/** A Glk Time struct */
#[repr(C)]
pub struct GlkTime {
    high_sec: i32,
    low_sec: u32,
    microsec: i32,
}

/** A Glk Date struct */
#[repr(C)]
pub struct GlkDate {
    year: i32,     /* full (four-digit) year */
    month: i32,    /* 1-12, 1 is January */
    day: i32,      /* 1-31 */
    weekday: i32,  /* 0-6, 0 is Sunday */
    hour: i32,     /* 0-23 */
    minute: i32,   /* 0-59 */
    second: i32,   /* 0-59, maybe 60 during a leap second */
    microsec: i32, /* 0-999999 */
}

#[derive(Default)]
struct PageMargin {
    garglk: Option<u32>,
    last_was_garglk: bool,
    stylehint: Option<u32>,
    transmitted: Option<u32>,
}

impl PageMargin {
    /** We have two methods to set the margin colour: stylehints, and garglk_set_zcolors
        If both are used, then use whichever was most recently set */
    fn get_page_margin_bg(&self) -> Option<u32> {
        if self.stylehint.is_some() {
            if self.last_was_garglk && self.garglk.is_some() {
                self.garglk
            }
            else {
                self.stylehint
            }
        }
        else {
            self.garglk
        }
    }

    fn set_garglk(&mut self, val: Option<u32>) {
        self.last_was_garglk = true;
        self.garglk = val;
    }

    fn set_stylehint(&mut self, val: Option<u32>) {
        self.last_was_garglk = false;
        self.stylehint = val;
    }
}

// Retained array callbacks
pub type RetainArrayCallback<T> = extern fn(*const T, u32, *const c_char) -> DispatchRock;
pub type UnretainArrayCallback<T> = extern fn(*const T, u32, *const c_char, DispatchRock);

pub struct RetainArrayCallbacks<T> {
    pub retain: RetainArrayCallback<T>,
    pub unretain: UnretainArrayCallback<T>,
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
    garglktext: bool,
    graphics: bool,
    hyperlinks: bool,
    sounds: bool,
    timers: bool,
}

#[derive(Default)]
struct TimerData {
    interval: u32,
    last_interval: u32,
    started: Option<SystemTime>,
}

fn clean_filename(filename: String, filetype: FileType) -> String {
    let mut fixed_filename = String::default();
        for char in filename.chars() {
            match char {
                '"' | '\\' | '/' | '>' | '<' | ':' | '|' | '?' | '*' | char::REPLACEMENT_CHARACTER => {},
                '.' => break,
                _ => fixed_filename.push(char),
            };
        }
        if fixed_filename.is_empty() {
            fixed_filename = "null".to_string();
        }
        fixed_filename.push_str(filetype_suffix(filetype));
        fixed_filename
}

pub fn colour_code_to_css(colour: u32) -> String {
    // Uppercase colours are required by RegTest
    format!("#{:06X}", colour & 0xFFFFFF)
}

fn create_stream_from_buffer(buf: Box<[u8]>, binary: bool, mode: FileMode, unicode: bool, fileref: Option<&GlkFileRef>) -> GlkResult<'static, GlkStream> {
    let data = match (unicode, binary) {
        (false, _) => GlkOwnedBuffer::U8(buf),
        (true, false) => str::from_utf8(&buf)?.into(),
        (true, true) => GlkOwnedBuffer::U32(u8slice_to_u32vec(&buf).into_boxed_slice()),
    };

    let str = GlkObject::new(if mode == FileMode::Read {
        match data {
            GlkOwnedBuffer::U8(buf) => ArrayBackedStream::<u8>::new(buf, mode, fileref).into(),
            GlkOwnedBuffer::U32(buf) => ArrayBackedStream::<u32>::new(buf, mode, fileref).into(),
        }
    }
    else {
        match data {
            GlkOwnedBuffer::U8(buf) => FileStream::<u8>::new(fileref.unwrap(), buf, mode).into(),
            GlkOwnedBuffer::U32(buf) => FileStream::<u32>::new(fileref.unwrap(), buf, mode).into(),
        }
    });
    Ok(str)
}

fn datetime_to_glkdate<T: TimeZone>(datetime: &DateTime<T>) -> GlkDate {
    GlkDate {
        year: datetime.year(),
        month: datetime.month() as i32,
        day: datetime.day() as i32,
        weekday: datetime.weekday().num_days_from_sunday() as i32,
        hour: datetime.hour() as i32,
        minute: datetime.minute() as i32,
        second: datetime.second() as i32,
        microsec: (datetime.nanosecond() / 1000) as i32,
    }
}

fn datetime_to_glktime<T: TimeZone>(time: &DateTime<T>) -> GlkTime {
    let timestamp = time.timestamp();
    GlkTime {
        high_sec: (timestamp >> 32) as i32,
        low_sec: timestamp as u32,
        microsec: time.timestamp_subsec_micros() as i32,
    }
}

fn datetime_to_simpletime<T: TimeZone>(time: &DateTime<T>, factor: u32) -> i32 {
    let timestamp = time.timestamp();
    // Unfortunately we can't simply divide, as we must round to negative infinity
    if timestamp >= 0 {
        (timestamp / (factor as i64)) as i32
    }
    else {
        -1 - ((-1 - timestamp) / (factor as i64)) as i32
    }
}

fn glkdate_to_datetime<T: TimeZone>(timezone: T, date: &GlkDate) -> DateTime<T> {
    // We must normalise the date, which is not fun
    let mut normalised_date = NaiveDate::from_ymd_opt(date.year, 1, 1).unwrap();
    let months = date.month - 1;
    if months > 0 {
        normalised_date = normalised_date.checked_add_months(chrono::Months::new(months as u32)).unwrap();
    }
    if months < 0 {
        normalised_date = normalised_date.checked_sub_months(chrono::Months::new((-months) as u32)).unwrap();
    }
    let mut normalised_date = NaiveDateTime::from(normalised_date).and_utc();
    let duration = Duration::try_days(date.day as i64 - 1).unwrap()
        + Duration::try_hours(date.hour as i64).unwrap()
        + Duration::try_minutes(date.minute as i64).unwrap()
        + Duration::try_seconds(date.second as i64).unwrap()
        + Duration::nanoseconds(date.microsec as i64 * 1000);
    normalised_date = normalised_date.checked_add_signed(duration).unwrap();
    normalised_date.with_timezone(&timezone)
}

fn glktime_to_datetime<T: TimeZone>(timezone: T, time: &GlkTime) -> DateTime<T> {
    let timestamp = ((time.high_sec as i64) << 32 | (time.low_sec as i64)) * 1000000 + (time.microsec as i64);
    let naive = DateTime::from_timestamp_micros(timestamp).unwrap().naive_utc();
    naive.and_local_timezone(timezone).unwrap()
}

fn normalise_metrics(metrics: Metrics) -> GlkResult<'static, NormalisedMetrics> {
    let res: GlkResult<NormalisedMetrics> = metrics.into();
    res
}

fn normalise_window_dimension(val: f64) -> usize {
    val.floor().max(0.0) as usize
}

fn stream_to_file_buffer(str: &mut Stream) -> Option<(&str, Box<[u8]>)> {
    fn inner_processor<T>(str: &mut FileStream<T>) -> Option<(&str, Box<[u8]>)>
    where T: Clone + Default, Box<[T]>: GlkArray {
        if str.changed {
            str.changed = false;
            Some((&str.path, str.to_file_buffer()))
        }
        else {
            None
        }
    }
    match str {
        Stream::FileStreamU8(str) => inner_processor(str),
        Stream::FileStreamU32(str) => inner_processor(str),
        _ => None,
    }
}

/** Run a window function on a stream, and the window's echo stream; must be given a locked GlkStream */
macro_rules! window_stream_operation {
    ($str: expr, $func: ident, $val: expr) => {
        if let Stream::Window(str) = $str.deref().deref() {
            let win = str.win.upgrade().unwrap();
            let mut win = lock!(win);
            win.$func($val);
            if let Some(str) = &win.echostr {
                let str = str.upgrade().unwrap();
                let str = lock!(str);
                if let Stream::Window(str) = str.deref().deref() {
                    let win = str.win.upgrade().unwrap();
                    let mut win = lock!(win);
                    win.$func($val);
                }
            }
        }
    };
    ($str: expr, $func: ident, $val1: expr, $val2: expr) => {
        if let Stream::Window(str) = $str.deref().deref() {
            let win = str.win.upgrade().unwrap();
            let mut win = lock!(win);
            win.$func($val1, $val2);
            if let Some(str) = &win.echostr {
                let str = str.upgrade().unwrap();
                let str = lock!(str);
                if let Stream::Window(str) = str.deref().deref() {
                    let win = str.win.upgrade().unwrap();
                    let mut win = lock!(win);
                    win.$func($val1, $val2);
                }
            }
        }
    };
}
pub(crate) use window_stream_operation;