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
mod macros;
mod objects;
mod protocol;
mod streams;
mod windows;

use std::mem;
use std::num::NonZeroU32;

use arrays::*;
use common::*;
use GlkApiError::*;
use constants::*;
use macros::*;
use objects::*;
use streams::*;
use windows::*;

#[derive(Default)]
pub struct GlkApi {
    streams: GlkObjectStore<Stream>,
    current_stream: Option<NonZeroU32>,
    metrics: protocol::NormalisedMetrics,
    root_window: Option<NonZeroU32>,
    stylehints_buffer: protocol::WindowStyles,
    stylehints_grid: protocol::WindowStyles,
    windows: GlkObjectStore<Window>,
    windows_changed: bool,
}

impl GlkApi {
    pub fn glk_get_buffer_stream(&mut self, str_id: Option<NonZeroU32>, buf: &mut [u8]) -> GlkResult<u32> {
        stream_op!(self, str_id, |str: &mut Stream| str.get_buffer(&mut GlkBufferMut::U8(buf)))
    }

    pub fn glk_get_buffer_stream_uni(&mut self, str_id: Option<NonZeroU32>, buf: &mut [u32]) -> GlkResult<u32> {
        stream_op!(self, str_id, |str: &mut Stream| str.get_buffer(&mut GlkBufferMut::U32(buf)))
    }

    pub fn glk_get_char_stream(&mut self, str_id: Option<NonZeroU32>) -> GlkResult<i32> {
        stream_op!(self, str_id, |str: &mut Stream| str.get_char(false))
    }

    pub fn glk_get_char_stream_uni(&mut self, str_id: Option<NonZeroU32>) -> GlkResult<i32> {
        stream_op!(self, str_id, |str: &mut Stream| str.get_char(true))
    }

    pub fn glk_get_line_stream(&mut self, str_id: Option<NonZeroU32>, buf: &mut [u8]) -> GlkResult<u32> {
        stream_op!(self, str_id, |str: &mut Stream| str.get_line(&mut GlkBufferMut::U8(buf)))
    }

    pub fn glk_get_line_stream_uni(&mut self, str_id: Option<NonZeroU32>, buf: &mut [u32]) -> GlkResult<u32> {
        stream_op!(self, str_id, |str: &mut Stream| str.get_line(&mut GlkBufferMut::U32(buf)))
    }

    pub fn glk_put_buffer(&mut self, buf: &[u8]) -> GlkResult<()> {
        stream_op!(self, self.current_stream, |str: &mut Stream| str.put_buffer(&GlkBuffer::U8(buf)))
    }

    pub fn glk_put_buffer_stream(&mut self, str_id: Option<NonZeroU32>, buf: &[u8]) -> GlkResult<()> {
        stream_op!(self, str_id, |str: &mut Stream| str.put_buffer(&GlkBuffer::U8(buf)))
    }

    pub fn glk_put_buffer_stream_uni(&mut self, str_id: Option<NonZeroU32>, buf: &[u32]) -> GlkResult<()> {
        stream_op!(self, str_id, |str: &mut Stream| str.put_buffer(&GlkBuffer::U32(buf)))
    }

    pub fn glk_put_buffer_uni(&mut self, buf: &[u32]) -> GlkResult<()> {
        stream_op!(self, self.current_stream, |str: &mut Stream| str.put_buffer(&GlkBuffer::U32(buf)))
    }

    pub fn glk_put_char(&mut self, ch: u8) -> GlkResult<()> {
        stream_op!(self, self.current_stream, |str: &mut Stream| str.put_char(ch as u32))
    }

    pub fn glk_put_char_stream(&mut self, str_id: Option<NonZeroU32>, ch: u8) -> GlkResult<()> {
        stream_op!(self, str_id, |str: &mut Stream| str.put_char(ch as u32))
    }

    pub fn glk_put_char_stream_uni(&mut self, str_id: Option<NonZeroU32>, ch: u32) -> GlkResult<()> {
        stream_op!(self, str_id, |str: &mut Stream| str.put_char(ch))
    }

    pub fn glk_put_char_uni(&mut self, ch: u32) -> GlkResult<()> {
        stream_op!(self, self.current_stream, |str: &mut Stream| str.put_char(ch))
    }

    pub fn glk_window_clear(&mut self, win_id: Option<NonZeroU32>) -> GlkResult<()> {
        win_mut!(self, win_id).data.clear();
        Ok(())
    }

    pub fn glk_stream_close(&mut self, str_id: Option<NonZeroU32>) -> GlkResult<StreamResultCounts> {
        let res = stream_op!(self, str_id, |str: &mut Stream| str.close());
        self.streams.unregister(str_id.unwrap());
        res
    }

    pub fn glk_stream_get_current(&self) -> Option<NonZeroU32> {
        self.current_stream
    }

    pub fn glk_stream_get_position(&self, str_id: Option<NonZeroU32>) -> GlkResult<u32> {
        Ok(str!(self, str_id).get_position())
    }

    pub fn glk_stream_get_rock(&self, str_id: Option<NonZeroU32>) -> GlkResult<u32> {
        self.streams.get_rock(str_id).ok_or(InvalidReference)
    }

    pub fn glk_stream_iterate(&self, str_id: Option<NonZeroU32>) -> Option<IterationResult> {
        self.streams.iterate(str_id)
    }

    pub fn glk_stream_open_memory(&mut self, buf: Box<[u8]>, fmode: FileMode, rock: u32) -> GlkResult<NonZeroU32> {
        self.create_memory_stream(buf, fmode, rock)
    }

    pub fn glk_stream_open_memory_uni(&mut self, buf: Box<[u32]>, fmode: FileMode, rock: u32) -> GlkResult<NonZeroU32> {
        self.create_memory_stream(buf, fmode, rock)
    }

    pub fn glk_stream_set_current(&mut self, str_id: Option<NonZeroU32>) {
        self.current_stream = str_id;
    }

    pub fn glk_stream_set_position(&mut self, str_id: Option<NonZeroU32>, mode: SeekMode, pos: i32) -> GlkResult<()> {
        str_mut!(self, str_id).set_position(mode, pos);
        Ok(())
    }

    pub fn glk_window_get_parent(&mut self, win_id: Option<NonZeroU32>) -> GlkResult<Option<NonZeroU32>> {
        Ok(window_op!(self, win_id, |win: &Window| win.parent))
    }

    pub fn glk_window_get_rock(&self, win_id: Option<NonZeroU32>) -> GlkResult<u32> {
        self.windows.get_rock(win_id).ok_or(InvalidReference)
    }

    pub fn glk_window_get_root(&self) -> Option<NonZeroU32> {
        self.root_window
    }

    pub fn glk_window_get_type(&mut self, win_id: Option<NonZeroU32>) -> GlkResult<WindowType> {
        Ok(win!(self, win_id).wintype)
    }

    pub fn glk_window_iterate(&self, win_id: Option<NonZeroU32>) -> Option<IterationResult> {
        self.windows.iterate(win_id)
    }

    pub fn glk_window_open(&mut self, splitwin_id: Option<NonZeroU32>, method: u32, size: u32, wintype: WindowType, rock: u32) -> GlkResult<NonZeroU32> {
        if self.root_window.is_none() {
            if splitwin_id.is_some() {
                return Err(SplitMustBeNull);
            }
        }
        else {
            if splitwin_id.is_none() {
                return Err(InvalidSplitwin);
            }
            let splitwin = win_mut!(self, splitwin_id);
            validate_winmethod(method, splitwin)?;
        }

        // Create the window
        let windata = match wintype {
            WindowType::Blank => BlankWindow {}.into(),
            WindowType::Buffer => TextWindow::<BufferWindow>::new(&self.stylehints_buffer).into(),
            // Todo: graphics
            WindowType::Grid => TextWindow::<GridWindow>::new(&self.stylehints_grid).into(),
            _ => {return Err(InvalidWindowType);}
        };
        let win = Window::new(windata, wintype);
        let win_id = Some(self.windows.register(win, rock));
        // TODO: Window stream!

        // Rearrange the windows for the new window
        if splitwin_id.is_some() {
            // This section is convoluted because of borrowing rules
            let oldparent_id = win_mut!(self, splitwin_id).parent;

            // Set up the pairwindata before turning it into a full window
            let mut pairwindata = PairWindow::new(win_id, wintype, method, size);
            pairwindata.child1 = splitwin_id;
            pairwindata.child2 = win_id;

            // Now the pairwin object can be created and registered
            let mut pairwin = Window::new(PairWindow::default().into(), WindowType::Pair);
            pairwin.parent = oldparent_id;
            let pairwin_id = Some(self.windows.register(pairwin, 0));

            win_mut!(self, splitwin_id).parent = pairwin_id;
            win_mut!(self, win_id).parent = pairwin_id;

            if oldparent_id.is_some() {
                let oldparent = win_mut!(self, oldparent_id);
                match &mut oldparent.data {
                    WindowData::Pair(oldparent_data) => {
                        if oldparent_data.child1 == splitwin_id {
                            oldparent_data.child1 = pairwin_id;
                        }
                        else {
                            oldparent_data.child2 = pairwin_id;
                        }
                    },
                    _ => unreachable!(),
                };
            }
            else {
                self.root_window = pairwin_id;
            }
            let wbox = win!(self, splitwin_id).wbox;
            self.rearrange_window(pairwin_id, wbox)?;
        }
        else {
            self.root_window = win_id;
            self.rearrange_window(win_id, WindowBox {
                bottom: self.metrics.height,
                right: self.metrics.width,
                ..Default::default()
            })?;
        }

        Ok(win_id.unwrap())
    }

    fn create_memory_stream<T>(&mut self, buf: Box<[T]>, fmode: FileMode, rock: u32) -> GlkResult<NonZeroU32>
    where Stream: From<ArrayBackedStream<T>> {
        if fmode == FileMode::WriteAppend {
            return Err(IllegalFilemode);
        }
        let str: Stream = if buf.len() == 0 {
            NullStream::default().into()
        }
        else {
            ArrayBackedStream::<T>::new(buf, fmode, None).into()
        };
        Ok(self.streams.register(str, rock))
    }

    fn rearrange_window(&mut self, win_id: Option<NonZeroU32>, wbox: WindowBox) -> GlkResult<()> {
        self.windows_changed = true;
        let win = win_mut!(self, win_id);
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
                    match win.key_wintype {
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
                let child1 = win.child1;
                let child2 = win.child2;
                self.rearrange_window(child1, box1)?;
                self.rearrange_window(child2, box2)?;
            },
            _ => {},
        };

        Ok(())
    }
}

/** Final read/write character counts of a stream */
#[repr(C)]
pub struct StreamResultCounts {
    pub read_count: u32,
    pub write_count: u32,
}

fn normalise_window_dimension(val: f64) -> usize {
    val.floor().min(0.0) as usize
}