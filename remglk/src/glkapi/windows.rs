/*

Glk Windows
===========

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use enum_dispatch::enum_dispatch;

use super::*;
use constants::*;
use protocol::*;

#[derive(Default)]
pub struct Window {
    pub data: WindowData,
    echostr: Option<Stream>,
    input: InputUpdate,
    next: Option<NonZeroU32>,
    parent: Option<NonZeroU32>,
    prev: Option<NonZeroU32>,
    str: WindowStream,
    wbox: WindowBox,
    pub wintype: WindowType,
}

#[enum_dispatch]
pub enum WindowData {
    BlankWindow,
    BufferWindowU8(TextWindow<BufferWindow, u8>),
    BufferWindowU32(TextWindow<BufferWindow, u32>),
    GraphicsWindow,
    GridWindowU8(TextWindow<GridWindow, u8>),
    GridWindowU32(TextWindow<GridWindow, u32>),
    PairWindow,
}

#[derive(Default)]
pub struct WindowUpdate {
    content: Option<ContentUpdate>,
    input: Option<InputUpdate>,
    size: Option<protocol::WindowUpdate>,
}

impl Window {
    pub fn update(&mut self) -> WindowUpdate {
        if let WindowData::BlankWindow(_) | WindowData::PairWindow(_) = self.data {
            Default::default()
        }

        // Fill in the input update here as WindowData doesn't have access to it
        let mut input_update = InputUpdate::new(self.input.id);
        input_update.hyperlink = self.input.hyperlink;
        input_update.mouse = self.input.mouse;
        if let Some(text_input_type) = self.input.text_input_type {
            input_update.gen = self.input.gen;
            input_update.text_input_type = self.input.text_input_type;
            if text_input_type == TextInputType::Line {
                input_update.initial = self.input.initial.take();
                input_update.terminators = self.input.terminators.clone();
            }
        }

        // Now give it to the specific window types for them to fill in
        self.data.update(WindowUpdate {
            input: Some(input_update),
            size: Some(protocol::WindowUpdate {
                height: self.wbox.bottom - self.wbox.top,
                id: 0, // Must be replaced!
                left: self.wbox.left,
                rock: 0, // Must be replaced!
                top: self.wbox.top,
                wintype: self.wintype,
                width: self.wbox.right - self.wbox.left,
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}

impl Default for WindowData {
    fn default() -> Self {
        WindowData::BlankWindow(BlankWindow::default())
    }
}

#[enum_dispatch(WindowData)]
pub trait WindowOperations {
    fn clear(&mut self) {}
    fn put_string(&mut self, _str: &str, _style: Option<u32>) {}
    fn set_css(&mut self, _name: &str, _val: Option<CSSValue>) {}
    fn set_hyperlink(&mut self, _val: u32) {}
    fn set_style(&mut self, _val: u32) {}
    fn update(&mut self, update: WindowUpdate) -> WindowUpdate {update}
}

#[derive(Default)]
pub struct WindowBox {
    bottom: f64,
    left: f64,
    right: f64,
    top: f64,
}

#[derive(Default)]
pub struct BlankWindow {}

impl WindowOperations for BlankWindow {}

#[derive(Default)]
pub struct TextWindow<T, B>
where T: Default + WindowOperations {
    data: T,
    line_input_buffer: Option<Box<[B]>>,
    request_echo_line_input: bool,
    stylehints: WindowStyles,
    uni_input: bool,
}

impl<T, B> TextWindow<T, B>
where T: Default + WindowOperations, B: Default {
    fn new(stylehints: WindowStyles) -> Self {
        TextWindow::<T, B> {
            request_echo_line_input: true,
            stylehints,
            ..Default::default()
        }
    }
}

impl<T, B> WindowOperations for TextWindow<T, B>
where T: Default + WindowOperations {
    fn clear(&mut self) {
        self.data.clear();
    }

    fn put_string(&mut self, str: &str, style: Option<u32>) {
        self.data.put_string(str, style);
    }

    fn set_css(&mut self, name: &str, val: Option<CSSValue>) {
        self.data.set_css(name, val);
    }

    fn set_hyperlink(&mut self, val: u32) {
        self.data.set_hyperlink(val);
    }

    fn set_style(&mut self, val: u32) {
        self.data.set_style(val);
    }

    fn update(&mut self, mut update: WindowUpdate) -> WindowUpdate {
        // TODO: don't resend stylehints when only metrics have changed?
        if self.stylehints.len() > 0 {
            update.size.as_mut().unwrap().styles = Some(self.stylehints.clone());
        }

        // Fill in the maxlen as we didn't have access to it in Window.update
        let input_update = update.input.as_mut().unwrap();
        if let Some(buf) = &self.line_input_buffer {
            if let Some(TextInputType::Line) = input_update.text_input_type {
                input_update.maxlen = Some(buf.len() as u32);
            }
        }

        self.data.update(update)
    }
}

#[derive(Default)]
pub struct BufferWindow {
    cleared: bool,
}

impl WindowOperations for BufferWindow {}

pub struct GraphicsWindow {
    draw: Vec<GraphicsWindowOperation>,
    height: u32,
    uni_input: bool,
    width: u32,
}

impl WindowOperations for GraphicsWindow {}

#[derive(Default)]
pub struct GridWindow {
    cleared: bool,
    current_styles: TextRun,
    height: usize,
    lines: Vec<GridLine>,
    width: usize,
    x: usize,
    y: usize,
}

#[derive(Clone, Default)]
struct GridLine {
    changed: bool,
    content: Vec<TextRun>,
}

impl GridWindow {
    fn fit_cursor(&mut self) -> bool {
        if self.x >= self.width {
            self.x = 0;
            self.y += 1;
        }
        if self.y >= self.height {
            return true
        }
        return false
    }

    fn update_size(&mut self, height: usize, width: usize) {
        self.height = height;
        self.width = width;

        // Set the right number of lines, as well as each line to have the right width
        self.lines.resize(height, GridLine {
            changed: true,
            content: Vec::new(),
        });
        for line in &mut self.lines {
            line.content.resize(width, TextRun::new(" "));
        }
    }
}

impl WindowOperations for GridWindow {
    fn clear(&mut self) {
        let height = self.height;
        self.cleared = true;
        self.update_size(0, self.width);
        self.update_size(height, self.width);
        self.x = 0;
        self.y = 0;
    }

    fn put_string(&mut self, str: &str, style: Option<u32>) {
        let old_style = self.current_styles.style;
        if let Some(val) = style {
            self.set_style(val);
        }
        for char in str.chars() {
            if self.fit_cursor() {
                break;
            }
            if char == '\n' {
                self.x = 0;
                self.y += 1;
            }
            else {
                let line = &mut self.lines[self.x];
                line.changed = true;
                line.content[self.y] = self.current_styles.clone(char.to_string());
            }
        }
        if style.is_some() {
            self.set_style(old_style);
        }
    }

    fn set_css(&mut self, name: &str, val: Option<CSSValue>) {
        set_css(&mut self.current_styles.css_styles, name, val);
    }

    fn set_hyperlink(&mut self, val: u32) {
        self.current_styles.hyperlink = match val {
            0 => None,
            val => Some(val),
        };
    }

    fn set_style(&mut self, val: u32) {
        self.current_styles.style = val;
    }

    fn update(&mut self, mut update: WindowUpdate) -> WindowUpdate {
        if self.lines.iter().any(|line| line.changed) {
            let mut grid_content = GridWindowContentUpdate {
                base: TextualWindowUpdate::default(),
                lines: self.lines.iter_mut().enumerate().filter_map(|(i, line)| {
                    if !line.changed {
                        return None;
                    }
                    line.changed = false;
                    // Merge grid characters with the same styles together
                    let content = line.content.iter().fold(vec![], |mut acc, cur| {
                        if acc.len() == 0 {
                            return vec![cur.clone(cur.text.to_string())];
                        }
                        else {
                            let last = acc.last_mut().unwrap();
                            if cur == last {
                                last.text.push_str(&cur.text);
                            }
                            else {
                                let new = last.clone(cur.text.clone());
                                acc.push(new);
                            }
                        }
                        acc
                    }).into_iter().map(|tr| LineData::TextRun(tr)).collect();
                    Some(GridWindowLine {
                        content: cleanup_paragraph_styles(content),
                        line: i as u32,
                    })
                }).collect(),
            };
            if self.cleared {
                grid_content.base.clear = Some(true);
                self.cleared = false;
            }
            update.content = Some(ContentUpdate::Grid(grid_content));
        }

        let input_update = update.input.as_mut().unwrap();
        if input_update.text_input_type.is_some() {
            let (x, y) = if self.fit_cursor() {
                (self.width - 1, self.height - 1)
            }
            else {
                (self.x, self.y)
            };
            input_update.xpos = Some(x as u32);
            input_update.ypos = Some(y as u32);
        }

        let size = update.size.as_mut().unwrap();
        size.gridheight = Some(self.height as u32);
        size.gridwidth = Some(self.width as u32);

        update
    }
}

#[derive(Default)]
pub struct PairWindow {
    pub backward: bool,
    pub border: bool,
    pub child1: Option<NonZeroU32>,
    pub child2: Option<NonZeroU32>,
    pub dir: u32,
    pub fixed: bool,
    pub key: Option<NonZeroU32>,
    pub size: u32,
    pub vertical: bool,
}

impl PairWindow {
    fn new(keywin: Option<NonZeroU32>, method: u32, size: u32) -> Self {
        let dir = method & winmethod_DirMask;
        PairWindow {
            backward: dir == winmethod_Left || dir == winmethod_Above,
            border: (method & winmethod_BorderMask) == winmethod_BorderMask,
            dir,
            fixed: (method & winmethod_DivisionMask) == winmethod_Fixed,
            key: keywin,
            size,
            vertical: dir == winmethod_Left || dir == winmethod_Right,
            ..Default::default()
        }
    }
}

impl WindowOperations for PairWindow {}

/** Remove css_styles from a paragraph when empty */
fn cleanup_paragraph_styles(par: Vec<LineData>) -> Vec<LineData> {
    par.into_iter().map(|content: LineData| {
        match content {
            LineData::TextRun(mut tr) => {
                if let Some(ref styles) = tr.css_styles {
                    if styles.lock().unwrap().len() == 0 {
                        tr.css_styles = None;
                    }
                }
                LineData::TextRun(tr)
            },
            x => x,
        }
    }).collect()
}

fn set_css(css_styles: &mut Option<Arc<Mutex<CSSProperties>>>, name: &str, val: Option<CSSValue>) {
    // We need to either clone the existing styles, or insert an empty one
    let mut styles = css_styles.take().map_or(HashMap::new(), |old| old.lock().unwrap().clone());
    match val {
        None => {
            styles.remove(name);
        },
        Some(style) => {
            styles.insert(name.to_string(), style);
        }
    };
    *css_styles = Some(Arc::new(Mutex::new(styles)));
}