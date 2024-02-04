/*

Glk Windows
===========

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

use enum_dispatch::enum_dispatch;

use super::*;

/** A `Window` wrapped in an `Arc<Mutex>>` */
pub type GlkWindow = GlkObject<Window>;
pub type GlkWindowWeak = GlkObjectWeak<Window>;

#[derive(Default)]
pub struct Window {
    pub data: WindowData,
    pub echostr: Option<GlkStreamWeak>,
    pub input: InputUpdate,
    pub parent: Option<GlkWindowWeak>,
    pub str: GlkStreamWeak,
    pub wbox: WindowBox,
    pub wintype: WindowType,
    pub uni_char_input: bool,
}

#[enum_dispatch]
pub enum WindowData {
    Blank(BlankWindow),
    Buffer(TextWindow<BufferWindow>),
    Graphics(GraphicsWindow),
    Grid(TextWindow<GridWindow>),
    Pair(PairWindow),
}

#[derive(Default)]
pub struct WindowUpdate {
    pub content: Option<ContentUpdate>,
    pub input: InputUpdate,
    pub size: protocol::WindowUpdate,
}

impl Window {
    pub fn new(data: WindowData, wintype: WindowType) -> (GlkWindow, GlkStream) {
        let win = GlkObject::new(Window {
            data,
            wintype,
            ..Default::default()
        });
        let str = GlkObject::new(WindowStream::new(&win).into());
        win.lock().unwrap().str = str.downgrade();
        (win, str)
    }

    pub fn update(&mut self) -> WindowUpdate {
        if let WindowData::Blank(_) | WindowData::Pair(_) = self.data {
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
                //input_update.terminators = self.input.terminators.clone();
            }
        }

        // Now give it to the specific window types for them to fill in
        self.data.update(WindowUpdate {
            input: input_update,
            size: protocol::WindowUpdate {
                height: self.wbox.bottom - self.wbox.top,
                id: 0, // Must be replaced!
                left: self.wbox.left,
                rock: 0, // Must be replaced!
                top: self.wbox.top,
                wintype: self.wintype,
                width: self.wbox.right - self.wbox.left,
                ..Default::default()
            },
            ..Default::default()
        })
    }
}

impl Deref for Window {
    type Target = WindowData;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Window {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl Default for WindowData {
    fn default() -> Self {
        WindowData::Blank(BlankWindow::default())
    }
}

impl GlkObjectClass for Window {
    fn get_object_class_id() -> u32 {
        0
    }
}

#[enum_dispatch(WindowData)]
pub trait WindowOperations {
    fn clear(&mut self) {}
    fn put_string(&mut self, _str: &str, _style: Option<u32>) {}
    fn set_css(&mut self, _name: &str, _val: Option<&CSSValue>) {}
    fn set_hyperlink(&mut self, _val: u32) {}
    fn set_style(&mut self, _val: u32) {}
    fn update(&mut self, update: WindowUpdate) -> WindowUpdate {update}
}

#[derive(Clone, Copy, Default)]
pub struct WindowBox {
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
    pub top: f64,
}

#[derive(Default)]
pub struct BlankWindow {}

impl WindowOperations for BlankWindow {}

#[derive(Default)]
pub struct TextWindow<T>
where T: Default + WindowOperations {
    pub data: T,
    pub line_input_buffer: Option<GlkOwnedBuffer>,
    pub request_echo_line_input: bool,
    stylehints: WindowStyles,
}

impl<T> TextWindow<T>
where T: Default + WindowOperations {
    pub fn new(stylehints: &WindowStyles) -> Self {
        TextWindow::<T> {
            request_echo_line_input: true,
            stylehints: stylehints.clone(),
            ..Default::default()
        }
    }
}

impl<T> WindowOperations for TextWindow<T>
where T: Default + WindowOperations {
    fn clear(&mut self) {
        self.data.clear();
    }

    fn put_string(&mut self, str: &str, style: Option<u32>) {
        self.data.put_string(str, style);
    }

    fn set_css(&mut self, name: &str, val: Option<&CSSValue>) {
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
        if !self.stylehints.is_empty() {
            update.size.styles = Some(self.stylehints.clone());
        }

        // Fill in the maxlen as we didn't have access to it in Window.update
        if let Some(buf) = &self.line_input_buffer {
            if let Some(TextInputType::Line) = update.input.text_input_type {
                update.input.maxlen = Some(buf.len() as u32);
            }
        }

        self.data.update(update)
    }
}

#[derive(Default)]
pub struct BufferWindow {
    cleared: bool,
    content: Vec<Paragraph>,
    pub echo_line_input: bool,
}

impl BufferWindow {
    pub fn new() -> Self {
        BufferWindow {
            cleared: true,
            content: vec![Paragraph::new(TextRun::default())],
            echo_line_input: true,
        }
    }

    fn clear_content(&mut self, new: Option<&TextRun>) {
        let new = new.unwrap_or(self.last_textrun()).clone("");
        self.content = vec![Paragraph {
            append: true,
            content: vec![LineData::TextRun(new)],
            flowbreak: false,
        }];
    }

    /** If the last textrun isn't empty, then add a new one, ready for its styles to be modified */
    fn clone_last_textrun(&mut self) {
        let last_textrun = self.last_textrun();
        if !last_textrun.text.is_empty() {
            let new = last_textrun.clone("");
            self.content.last_mut().unwrap().content.push(LineData::TextRun(new));
        }
    }

    /** Return the last textrun, which must exist, and actually be a textrun not an image */
    fn last_textrun(&mut self) -> &mut TextRun {
        match self.content.last_mut().unwrap().content.last_mut().unwrap() {
            LineData::TextRun(textrun) => textrun,
            _ => unreachable!()
        }
    }

    fn _set_flow_break(&mut self) {
        self.content.last_mut().unwrap().flowbreak = true;
    }
}

impl WindowOperations for BufferWindow {
    fn clear(&mut self) {
        self.cleared = true;
        self.clear_content(None);
    }

    fn put_string(&mut self, str: &str, style: Option<u32>) {
        let old_style = self.last_textrun().style;
        if let Some(val) = style {
            self.set_style(val);
        }
        for (i, line) in str.lines().enumerate() {
            if i > 0 {
                let textrun = self.last_textrun().clone("");
                self.content.push(Paragraph::new(textrun));
            }
            self.last_textrun().text.push_str(line);
        }
        if style.is_some() {
            self.set_style(old_style);
        }
    }

    fn set_css(&mut self, name: &str, val: Option<&CSSValue>) {
        if let Some(css_styles) = &self.last_textrun().css_styles {
            if css_styles.lock().unwrap().get(name) != val {
                self.clone_last_textrun();
                set_css(&mut self.last_textrun().css_styles, name, val);
            }
        }
    }

    fn set_hyperlink(&mut self, val: u32) {
        let val = if val > 0 {Some(val)} else {None};
        if self.last_textrun().hyperlink != val {
            self.clone_last_textrun();
            self.last_textrun().hyperlink = val;
        }
    }

    fn set_style(&mut self, val: u32) {
        if self.last_textrun().style != val {
            self.clone_last_textrun();
            self.last_textrun().style = val;
        }
    }

    fn update(&mut self, mut update: WindowUpdate) -> WindowUpdate {
        // Clone the textrun now because the css_style could get deleted in cleanup_paragraph_styles
        let last_textrun = self.last_textrun().clone("");

        // Exclude empty text runs
        for par in self.content.iter_mut() {
            par.content = cleanup_paragraph_styles(par.content.drain(..).filter(|line| match line {
                LineData::_Image(_) => true,
                LineData::TextRun(textrun) => !textrun.text.is_empty(),
            }).collect());
        }
        // Only send an update if there is new content or the window has been cleared
        if self.cleared || self.content.len() > 1 || !self.content[0].content.is_empty() {
            let mut content_update = BufferWindowContentUpdate {
                base: TextualWindowUpdate::default(),
                text: self.content.drain(..).map(|par| par.into()).collect(),
            };
            if self.cleared {
                content_update.base.clear = true;
                self.cleared = false;
            }
            update.content = Some(ContentUpdate::Buffer(content_update));
        }

        self.clear_content(Some(&last_textrun));
        update
    }
}

/** A modified version of BufferWindowParagraphUpdate that always has content */
#[derive(Default)]
struct Paragraph {
    append: bool,
    content: Vec<LineData>,
    flowbreak: bool,
}

impl Paragraph {
    fn new(textrun: TextRun) -> Self {
        Paragraph {
            content: vec![LineData::TextRun(textrun)],
            ..Default::default()
        }
    }
}

impl From<Paragraph> for BufferWindowParagraphUpdate {
    fn from(par: Paragraph) -> Self {
        BufferWindowParagraphUpdate {
            append: par.append,
            content: port_line_data(par.content),
            flowbreak: par.flowbreak,
        }
    }
}

#[derive(Clone)]
enum LineData {
    //StylePair(String, String),
    _Image(BufferWindowImage),
    TextRun(TextRun),
}

impl From<LineData> for protocol::LineData {
    fn from(ld: LineData) -> Self {
        match ld {
            LineData::_Image(image) => protocol::LineData::Image(image),
            LineData::TextRun(tr) => protocol::LineData::TextRun(tr.into()),
        }
    }
}

fn port_line_data(lines: Vec<LineData>) -> Vec<protocol::LineData> {
    lines.into_iter().map(|ld| ld.into()).collect()
}

#[derive(Clone, Default)]
struct TextRun {
    pub css_styles: Option<Arc<Mutex<CSSProperties>>>,
    pub hyperlink: Option<u32>,
    pub style: u32,
    pub text: String,
}

impl TextRun {
    pub fn new(text: &str) -> Self {
        TextRun {
            text: text.to_string(),
            ..Default::default()
        }
    }

    /** Clone a text run, sharing CSS */
    pub fn clone(&self, text: &str) -> Self {
        TextRun {
            css_styles: self.css_styles.as_ref().cloned(),
            hyperlink: self.hyperlink,
            style: self.style,
            text: text.to_string(),
        }
    }
}

// Two TextRuns are considered equal if everything except their text matches...
impl PartialEq for TextRun {
    fn eq(&self, other: &Self) -> bool {
        self.hyperlink == other.hyperlink && self.style == other.style && match (&self.css_styles, &other.css_styles) {
            (Some(self_styles), Some(other_styles)) => Arc::ptr_eq(self_styles, other_styles),
            (None, None) => true,
            _ => false,
        }
    }
}

impl From<TextRun> for protocol::TextRun {
    fn from(textrun: TextRun) -> Self {
        protocol::TextRun {
            css_styles: textrun.css_styles.map(|textrun| textrun.lock().unwrap().clone()),
            hyperlink: textrun.hyperlink,
            style: style_name(textrun.style).to_string(),
            text: textrun.text,
        }
    }
}

#[derive(Default)]
pub struct GraphicsWindow {
    pub draw: Vec<GraphicsWindowOperation>,
    pub height: usize,
    pub uni_input: bool,
    pub width: usize,
}

impl WindowOperations for GraphicsWindow {}

#[derive(Default)]
pub struct GridWindow {
    cleared: bool,
    current_styles: TextRun,
    pub height: usize,
    lines: Vec<GridLine>,
    pub width: usize,
    pub x: usize,
    pub y: usize,
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
            return true;
        }
        false
    }

    pub fn update_size(&mut self, height: usize, width: usize) {
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
                line.content[self.y] = self.current_styles.clone(&char.to_string());
            }
        }
        if style.is_some() {
            self.set_style(old_style);
        }
    }

    fn set_css(&mut self, name: &str, val: Option<&CSSValue>) {
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
                        if acc.is_empty() {
                            return vec![cur.clone(&cur.text)];
                        }
                        else {
                            let last = acc.last_mut().unwrap();
                            if cur == last {
                                last.text.push_str(&cur.text);
                            }
                            else {
                                let new = last.clone(&cur.text);
                                acc.push(new);
                            }
                        }
                        acc
                    }).into_iter().map(LineData::TextRun).collect();
                    Some(GridWindowLine {
                        content: port_line_data(cleanup_paragraph_styles(content)),
                        line: i as u32,
                    })
                }).collect(),
            };
            if self.cleared {
                grid_content.base.clear = true;
                self.cleared = false;
            }
            update.content = Some(ContentUpdate::Grid(grid_content));
        }

        if update.input.text_input_type.is_some() {
            let (x, y) = if self.fit_cursor() {
                (self.width - 1, self.height - 1)
            }
            else {
                (self.x, self.y)
            };
            update.input.xpos = Some(x as u32);
            update.input.ypos = Some(y as u32);
        }

        update.size.gridheight = Some(self.height as u32);
        update.size.gridwidth = Some(self.width as u32);

        update
    }
}

#[derive(Default)]
pub struct PairWindow {
    pub backward: bool,
    pub border: bool,
    pub child1: GlkWindowWeak,
    pub child2: GlkWindowWeak,
    pub dir: u32,
    pub fixed: bool,
    pub key: GlkWindowWeak,
    pub size: u32,
    pub vertical: bool,
}

impl PairWindow {
    pub fn new(keywin: &GlkWindow, method: u32, size: u32) -> Self {
        let dir = method & winmethod_DirMask;
        PairWindow {
            backward: dir == winmethod_Left || dir == winmethod_Above,
            border: (method & winmethod_BorderMask) == winmethod_BorderMask,
            dir,
            fixed: (method & winmethod_DivisionMask) == winmethod_Fixed,
            key: keywin.downgrade(),
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
                    if styles.lock().unwrap().is_empty() {
                        tr.css_styles = None;
                    }
                }
                LineData::TextRun(tr)
            },
            x => x,
        }
    }).collect()
}

fn set_css(css_styles: &mut Option<Arc<Mutex<CSSProperties>>>, name: &str, val: Option<&CSSValue>) {
    // Don't do anything if this style is already set
    if let Some(css_styles) = css_styles {
        if css_styles.lock().unwrap().get(name) == val {
            return;
        }
    }
    // We need to either clone the existing styles, or insert an empty one
    let mut styles = css_styles.take().map_or(HashMap::new(), |old| old.lock().unwrap().clone());
    if let Some(style) = val {
        styles.insert(name.to_string(), style.clone());
    }
    else {
        styles.remove(name);
    }
    *css_styles = Some(Arc::new(Mutex::new(styles)));
}