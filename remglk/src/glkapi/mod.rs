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

use std::mem;

use arrays::*;
use common::*;
use GlkApiError::*;
use constants::*;
use constants::FileMode;
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
    streams: GlkObjectStore<Stream>,
    current_stream: Option<GlkStream>,
    metrics: protocol::NormalisedMetrics,
    root_window: Option<GlkWindow>,
    stylehints_buffer: protocol::WindowStyles,
    stylehints_grid: protocol::WindowStyles,
    windows: GlkObjectStore<Window>,
    windows_changed: bool,
}

impl GlkApi {
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
        current_stream!(self).put_buffer(&GlkBuffer::U8(buf), None)
    }

    pub fn glk_put_buffer_stream<'a>(str: &GlkStream, buf: &[u8]) -> GlkResult<'a, ()> {
        lock!(str).put_buffer(&GlkBuffer::U8(buf), None)
    }

    pub fn glk_put_buffer_stream_uni<'a>(str: &GlkStream, buf: &[u32]) -> GlkResult<'a, ()> {
        lock!(str).put_buffer(&GlkBuffer::U32(buf), None)
    }

    pub fn glk_put_buffer_uni(&mut self, buf: &[u32]) -> GlkResult<()> {
        current_stream!(self).put_buffer(&GlkBuffer::U32(buf), None)
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

    pub fn glk_stream_close(&mut self, str: GlkStream) -> GlkResult<StreamResultCounts> {
        let res = lock!(str).close();
        self.streams.unregister(str);
        res
    }

    pub fn glk_stream_get_current(&self) -> Option<&GlkStream> {
        self.current_stream.as_ref()
    }

    pub fn glk_stream_get_position(str: &GlkStream) -> GlkResult<u32> {
        Ok(lock!(str).get_position())
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
        self.current_stream = str.cloned();
    }

    pub fn glk_stream_set_position(str: &GlkStream, mode: SeekMode, pos: i32) -> GlkResult<()> {
        lock!(str).set_position(mode, pos);
        Ok(())
    }

    pub fn glk_window_clear(win: &GlkWindow) -> GlkResult<()> {
        lock!(win).data.clear();
        Ok(())
    }

    /*pub fn glk_window_get_parent<'a>(&'a mut self, win: &'a GlkWindow) -> GlkResult<Option<&'a GlkWindow>> {
        Ok(lock!(win).parent.as_ref())
    }*/

    pub fn glk_window_get_rock(&self, win: &GlkWindow) -> GlkResult<u32> {
        self.windows.get_rock(win).ok_or(InvalidReference)
    }

    pub fn glk_window_get_root(&self) -> Option<&GlkWindow> {
        self.root_window.as_ref()
    }

    pub fn glk_window_get_type(win: &GlkWindow) -> GlkResult<WindowType> {
        Ok(lock!(win).wintype)
    }

    pub fn glk_window_iterate(&self, win: Option<&GlkWindow>) -> Option<IterationResult<Window>> {
        self.windows.iterate(win)
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
            pairwindata.child1 = splitwin.clone();
            pairwindata.child2 = win.clone();

            // Now the pairwin object can be created and registered
            let (pairwin, pairwinstr) = Window::new(PairWindow::default().into(), WindowType::Pair);
            self.windows.register(&pairwin, 0);
            self.streams.register(&pairwinstr, 0);

            // Set up the rest of the relations
            let mut splitwin_inner = lock!(splitwin);
            let old_parent = splitwin_inner.parent.as_ref().cloned();
            lock!(pairwin).parent = old_parent.clone();
            splitwin_inner.parent = Some(pairwin.clone());
            lock!(win).parent = Some(pairwin.clone());

            if let Some(old_parent) = old_parent {
                let mut old_parent_inner = lock!(old_parent);
                match &mut old_parent_inner.data {
                    WindowData::Pair(old_parent_inner) => {
                        if &old_parent_inner.child1 == splitwin {
                            old_parent_inner.child1 = pairwin.clone();
                        }
                        else {
                            old_parent_inner.child2 = pairwin.clone();
                        }
                    },
                    _ => unreachable!(),
                };
            }
            else {
                self.root_window = Some(pairwin.clone());
            }
            let wbox = splitwin_inner.wbox;
            self.rearrange_window(&pairwin, wbox)?;
        }
        else {
            self.root_window = Some(win.clone());
            self.rearrange_window(&win, WindowBox {
                bottom: self.metrics.height,
                right: self.metrics.width,
                ..Default::default()
            })?;
        }

        Ok(win)
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

    fn rearrange_window(&mut self, win: &GlkWindow, wbox: WindowBox) -> GlkResult<()> {
        self.windows_changed = true;
        win.lock().unwrap().wbox = wbox;
        let boxheight = wbox.bottom - wbox.top;
        let boxwidth = wbox.right - wbox.left;

        // Adjust anything that needs adjusting
        match &mut win.lock().unwrap().data {
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
                    match win.key.lock().unwrap().wintype {
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
                self.rearrange_window(&win.child1, box1)?;
                self.rearrange_window(&win.child2, box2)?;
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