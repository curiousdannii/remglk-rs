/*

Glk Windows
===========

Copyright (c) 2025 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

use enum_dispatch::enum_dispatch;

use super::*;

/** A `Window` wrapped in an `Arc<Mutex>>` */
pub type GlkWindowShared = GlkObject<GlkWindow>;
pub type GlkWindowMetadata = GlkObjectMetadata<GlkWindow>;
pub type GlkWindowWeak = GlkObjectWeak<GlkWindow>;

#[derive(Default)]
pub struct GlkWindow {
    pub data: WindowData,
    pub echostr: Option<GlkStreamWeak>,
    pub id: u32,
    pub input: InputUpdate,
    pub parent: Option<GlkWindowWeak>,
    pub rock: u32,
    pub str: GlkStreamWeak,
    pub wbox: WindowBox,
    pub wintype: WindowType,
    pub uni_char_input: bool,
}

#[enum_dispatch]
pub enum WindowData {
    Blank(BlankWindow),
    Buffer(BufferWindow),
    Graphics(GraphicsWindow),
    Grid(GridWindow),
    Pair(PairWindow),
}

#[derive(Default)]
pub struct WindowUpdate {
    pub content: Option<ContentUpdate>,
    id: u32,
    pub input: InputUpdate,
    pub size: protocol::WindowUpdate,
}

impl GlkWindow {
    pub fn new(data: WindowData, id: u32, rock: u32, wintype: WindowType) -> (GlkWindowShared, GlkStream) {
        let win = GlkObject::new(GlkWindow {
            data,
            id,
            input: InputUpdate::new(id),
            rock,
            wintype,
            ..Default::default()
        });
        let str = GlkObject::new(WindowStream::new(&win).into());
        lock!(win).str = str.downgrade();
        (win, str)
    }

    pub fn update(&mut self) -> WindowUpdate {
        if let WindowData::Blank(_) | WindowData::Pair(_) = self.data {
            Default::default()
        }

        // Fill in the input update here as WindowData doesn't have access to it
        let mut input_update = InputUpdate::new(self.id);
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
        let height = self.wbox.bottom - self.wbox.top;
        let width = self.wbox.right - self.wbox.left;
        self.data.update(WindowUpdate {
            id: self.id,
            input: input_update,
            size: protocol::WindowUpdate {
                height,
                hidden: height == 0.0 || width == 0.0,
                id: self.id,
                left: self.wbox.left,
                rock: self.rock,
                top: self.wbox.top,
                wintype: self.wintype,
                width,
                ..Default::default()
            },
            ..Default::default()
        })
    }
}

impl Deref for GlkWindow {
    type Target = WindowData;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for GlkWindow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl Default for WindowData {
    fn default() -> Self {
        WindowData::Blank(BlankWindow::default())
    }
}

impl GlkObjectClass for GlkWindow {
    fn get_object_class_id() -> u32 {
        0
    }
}

#[enum_dispatch(WindowData)]
pub trait WindowOperations {
    fn clear(&mut self) -> Option<u32> {None}
    fn put_string(&mut self, _str: &str, _style: Option<u32>) {}
    fn set_colours(&mut self, _fg: u32, _bg: u32) {}
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
pub struct BufferWindow {
    cleared: bool,
    cleared_bg: Option<u32>,
    cleared_fg: Option<u32>,
    content: Vec<BufferWindowParagraphUpdate>,
    pub echo_line_input: bool,
    last_bg: Option<u32>,
    last_fg: Option<u32>,
    pub line_input_buffer: Option<GlkOwnedBuffer>,
    sent_stylehints: bool,
    stylehints: WindowStyles,
}

impl BufferWindow {
    pub fn new(stylehints: &WindowStyles) -> Self {
        BufferWindow {
            cleared: true,
            content: vec![BufferWindowParagraphUpdate::new(TextRun::default())],
            echo_line_input: true,
            stylehints: stylehints.clone(),
            ..Default::default()
        }
    }

    fn clear_content(&mut self, new: Option<&TextRun>) {
        let new = new.unwrap_or_else(|| self.last_textrun()).clone("");
        self.content = vec![BufferWindowParagraphUpdate {
            append: true,
            content: vec![LineData::TextRun(new)],
            flowbreak: false,
        }];
    }

    /** If the last textrun isn't empty, then add a new one, ready for its styles to be modified */
    fn clone_last_textrun(&mut self, force: bool) {
        let last_textrun = self.last_textrun();
        if force || !last_textrun.text.is_empty() {
            let new = last_textrun.clone("");
            self.content.last_mut().unwrap().content.push(LineData::TextRun(new));
        }
    }

    pub fn put_image(&mut self, mut image: BufferWindowImage) {
        image.hyperlink = self.last_textrun().hyperlink;
        self.clone_last_textrun(true);
        let content = &mut self.content.last_mut().unwrap();
        let last_par = &mut content.content;
        last_par.insert(last_par.len() - 1, LineData::Image(image));
    }

    /** Return the last textrun, which must exist, and actually be a textrun not an image */
    fn last_textrun(&mut self) -> &mut TextRun {
        match self.content.last_mut().unwrap().content.last_mut().unwrap() {
            LineData::TextRun(textrun) => textrun,
            _ => unreachable!()
        }
    }

    pub fn set_flow_break(&mut self) {
        self.content.last_mut().unwrap().flowbreak = true;
    }
}

impl WindowOperations for BufferWindow {
    fn clear(&mut self) -> Option<u32> {
        self.cleared = true;
        self.cleared_bg = self.last_bg;
        self.cleared_fg = self.last_fg;
        self.clear_content(None);
        self.cleared_bg
    }

    fn put_string(&mut self, str: &str, style: Option<u32>) {
        let old_style = self.last_textrun().style;
        if let Some(val) = style {
            self.set_style(val);
        }
        for (i, line) in str.split('\n').enumerate() {
            if i > 0 {
                let textrun = self.last_textrun().clone("");
                self.content.push(BufferWindowParagraphUpdate::new(textrun));
            }
            self.last_textrun().text.push_str(line);
        }
        if style.is_some() {
            self.set_style(old_style);
        }
    }

    fn set_colours(&mut self, fg: u32, bg: u32) {
        set_window_colours!(self, fg, bg);
    }

    fn set_css(&mut self, name: &str, val: Option<&CSSValue>) {
        if let Some(css_styles) = &self.last_textrun().css_styles {
            if lock!(css_styles).get(name) == val {
                return;
            }
        }
        self.clone_last_textrun(false);
        set_css(&mut self.last_textrun().css_styles, name, val);
    }

    fn set_hyperlink(&mut self, val: u32) {
        let val = if val > 0 {Some(val)} else {None};
        if self.last_textrun().hyperlink != val {
            self.clone_last_textrun(false);
            self.last_textrun().hyperlink = val;
        }
    }

    fn set_style(&mut self, val: u32) {
        if self.last_textrun().style != val {
            self.clone_last_textrun(false);
            self.last_textrun().style = val;
        }
    }

    fn update(&mut self, mut update: WindowUpdate) -> WindowUpdate {
        // Send stylehints once
        if !self.sent_stylehints && !self.stylehints.is_empty() {
            self.sent_stylehints = true;
            update.size.styles = Some(self.stylehints.clone());
        }

        // Fill in the maxlen as we didn't have access to it in Window.update
        if let Some(buf) = &self.line_input_buffer {
            if let Some(TextInputType::Line) = update.input.text_input_type {
                update.input.maxlen = Some(buf.len() as u32);
            }
        }

        // Clone the textrun now because the css_style could get deleted in cleanup_paragraph_styles
        let last_textrun = self.last_textrun().clone("");

        // Exclude empty text runs
        for par in self.content.iter_mut() {
            par.content = cleanup_paragraph_styles(par.content.drain(..).filter(|line| match line {
                LineData::Image(_) => true,
                LineData::TextRun(textrun) => !textrun.text.is_empty(),
            }).collect());
        }
        // Only send an update if there is new content or the window has been cleared
        if self.cleared || self.content.len() > 1 || !self.content[0].content.is_empty() {
            let mut content_update = BufferWindowContentUpdate {
                base: TextualWindowUpdate::new(update.id),
                text: mem::take(&mut self.content),
            };
            if self.cleared {
                content_update.base.clear = true;
                content_update.base.bg = self.cleared_bg.map(colour_code_to_css);
                content_update.base.fg = self.cleared_fg.map(colour_code_to_css);
                self.cleared = false;
            }
            update.content = Some(ContentUpdate::Buffer(content_update));
        }

        self.clear_content(Some(&last_textrun));
        update
    }
}

#[derive(Default)]
pub struct GraphicsWindow {
    pub draw: Vec<GraphicsWindowOperation>,
    pub height: usize,
    pub uni_input: bool,
    pub width: usize,
}

impl WindowOperations for GraphicsWindow {
    fn clear(&mut self) -> Option<u32> {
        self.draw = self.draw.drain(..).filter(|op| {
            matches!(op, GraphicsWindowOperation::SetColor(_))
        }).collect();
        self.draw.reverse();
        self.draw.shrink_to(1);
        self.draw.push(GraphicsWindowOperation::Fill(FillOperation::default()));
        None
    }

    fn update(&mut self, mut update: WindowUpdate) -> WindowUpdate {
        if !self.draw.is_empty() {
            update.content = Some(ContentUpdate::Graphics(GraphicsWindowContentUpdate {
                id: update.id,
                draw: mem::take(&mut self.draw),
            }));
        }
        update.size.graphheight = Some(self.height as u32);
        update.size.graphwidth = Some(self.width as u32);
        update
    }
}

#[derive(Default)]
pub struct GridWindow {
    cleared: bool,
    cleared_bg: Option<u32>,
    cleared_fg: Option<u32>,
    current_styles: TextRun,
    pub height: usize,
    last_bg: Option<u32>,
    last_fg: Option<u32>,
    pub line_input_buffer: Option<GlkOwnedBuffer>,
    lines: Vec<GridLine>,
    sent_stylehints: bool,
    stylehints: WindowStyles,
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
    pub fn new(stylehints: &WindowStyles) -> Self {
        GridWindow {
            stylehints: stylehints.clone(),
            ..Default::default()
        }
    }

    /** Fit the cursor within the window; returns true if the cursor is actually outside the window */
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
        // Garglk extension quirk: expanding a 0 line window has to update the background colour just like clearing
        if self.lines.is_empty() {
            self.cleared = true;
            self.cleared_bg = self.last_bg;
            self.cleared_fg = self.last_fg;
        }

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
    fn clear(&mut self) -> Option<u32> {
        let height = self.height;
        // We set the cleared status in `update_size`, so don't need to do it here
        self.update_size(0, self.width);
        self.update_size(height, self.width);
        self.x = 0;
        self.y = 0;
        self.cleared_bg
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
                let line = &mut self.lines[self.y];
                line.changed = true;
                line.content[self.x] = self.current_styles.clone(&char.to_string());
                self.x += 1;
            }
        }
        if style.is_some() {
            self.set_style(old_style);
        }
    }

    fn set_colours(&mut self, fg: u32, bg: u32) {
        set_window_colours!(self, fg, bg);
    }

    fn set_css(&mut self, name: &str, val: Option<&CSSValue>) {
        if let Some(css_styles) = &self.current_styles.css_styles {
            if lock!(css_styles).get(name) == val {
                return;
            }
        }
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
        // Send stylehints once
        if !self.sent_stylehints && !self.stylehints.is_empty() {
            self.sent_stylehints = true;
            update.size.styles = Some(self.stylehints.clone());
        }

        // Fill in the maxlen as we didn't have access to it in Window.update
        if let Some(buf) = &self.line_input_buffer {
            if let Some(TextInputType::Line) = update.input.text_input_type {
                update.input.maxlen = Some(buf.len() as u32);
            }
        }

        if self.lines.iter().any(|line| line.changed) {
            let mut grid_content = GridWindowContentUpdate {
                base: TextualWindowUpdate::new(update.id),
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
                                let new = cur.clone(&cur.text);
                                acc.push(new);
                            }
                        }
                        acc
                    }).into_iter().map(LineData::TextRun).collect();
                    Some(GridWindowLine {
                        content: cleanup_paragraph_styles(content),
                        line: i as u32,
                    })
                }).collect(),
            };
            if self.cleared {
                grid_content.base.clear = true;
                grid_content.base.bg = self.cleared_bg.map(colour_code_to_css);
                grid_content.base.fg = self.cleared_fg.map(colour_code_to_css);
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
    pub fn new(keywin: &GlkWindowShared, method: u32, size: u32) -> Self {
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
                    if lock!(styles).is_empty() {
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
    // We assume that the calling function has already checked if this style is already set
    // We need to either clone the existing styles, or insert an empty one
    let mut styles = css_styles.take().map_or(HashMap::new(), |old| lock!(old).clone());
    if let Some(style) = val {
        styles.insert(name.to_string(), style.clone());
    }
    else {
        styles.remove(name);
    }
    let _ = css_styles.insert(Arc::new(Mutex::new(styles)));
}

/** Set colours on a window */
macro_rules! set_window_colours {
    ($self: ident, $fg: expr, $bg: expr) => {
        #[allow(non_upper_case_globals)]
        match $fg {
            0 ..= 0xFFFFFF => {
                $self.last_fg = Some($fg);
                $self.set_css("color", Some(&CSSValue::String(colour_code_to_css($fg))));
            },
            zcolor_Default => {
                $self.last_fg = None;
                $self.set_css("color", None);
            },
            _ => {},
        };
        #[allow(non_upper_case_globals)]
        match $bg {
            0 ..= 0xFFFFFF => {
                $self.last_bg = Some($bg);
                $self.set_css("background-color", Some(&CSSValue::String(colour_code_to_css($bg))));
            },
            zcolor_Default => {
                $self.last_bg = None;
                $self.set_css("background-color", None);
            },
            _ => {},
        };
    };
}
use set_window_colours;