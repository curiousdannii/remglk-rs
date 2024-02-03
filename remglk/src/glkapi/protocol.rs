/*

The GlkOte protocol
===================

Copyright (c) 2022 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::collections::HashMap;

use super::*;

/** The GlkOte protocol has two parts:
 * 1. GlkOte sends events to GlkApi/RemGlk
 * 2. GlkApi/RemGlk send content updates to GlkOte
*/

/** GlkOte->GlkApi/RemGlk input events */
pub enum Event {
    Arrange(ArrangeEvent),
    Char(CharEvent),
    Debug(DebugEvent),
    External(ExternalEvent),
    Hyperlink(HyperlinkEvent),
    Init(InitEvent),
    Line(LineEvent),
    Mouse(MouseEvent),
    Redraw(RedrawEvent),
    Refresh(RefreshEvent),
    Special(SpecialEvent),
    Timer(TimerEvent),
}

pub struct EventBase {
    /** Generation number */
    pub gen: u32,
    /** Partial line input values */
    pub partial: PartialInputs,
}

pub type PartialInputs = Option<HashMap<u32, String>>;

pub struct ArrangeEvent {
    pub base: EventBase,
    pub metrics: Metrics,
}

/** Character (single key) event */
pub struct CharEvent {
    pub base: EventBase,
    /** Character that was received */
    pub value: CharEventData,
    /** Window ID */
    pub window: u32,
}
pub enum CharEventData {
    NormalKey(char),
    SpecialKey(SpecialKeyCode),
}

pub struct DebugEvent {
    pub base: EventBase,
    pub value: String,
}

pub struct ExternalEvent {
    pub base: EventBase,
    // TODO?
    //value: any,
}

pub struct HyperlinkEvent {
    pub base: EventBase,
    pub value: u32,
    /** Window ID */
    pub window: u32,
}

/** Initilisation event */
pub struct InitEvent {
    pub base: EventBase,
    pub metrics: Metrics,
    /** Capabilities list */
    pub support: Vec<String>,
}

/** Line (text) event */
pub struct LineEvent {
    pub base: EventBase,
    /** Terminator key */
    pub terminator: Option<TerminatorCode>,
    /** Line input */
    pub value: String,
    /** Window ID */
    pub window: u32,
}

pub struct MouseEvent {
    pub base: EventBase,
    /** Window ID */
    pub window: u32,
    /** Mouse click X */
    pub x: u32,
    /** Mouse click Y */
    pub y: u32,
}

pub struct RedrawEvent {
    pub base: EventBase,
    /** Window ID */
    pub window: Option<u32>,
}

pub struct RefreshEvent {
    pub base: EventBase,
}

pub struct SpecialEvent {
    pub base: EventBase,
    /** Event value (file reference from Dialog) */
    pub value: Option<SystemFileRef>,
}

#[derive(Clone, Default)]
pub struct SystemFileRef {
    pub content: Option<String>,
    pub filename: String,
    pub gameid: Option<String>,
    // TODO: do we need null here?
    pub usage: Option<FileType>,
}

pub struct TimerEvent {
    pub base: EventBase,
}

/** Screen and font metrics - all potential options */
pub struct Metrics {
    /** Buffer character height */
    pub buffercharheight: Option<f64>,
    /** Buffer character width */
    pub buffercharwidth: Option<f64>,
    /** Buffer window margin */
    pub buffermargin: Option<f64>,
    /** Buffer window margin X */
    pub buffermarginx: Option<f64>,
    /** Buffer window margin Y */
    pub buffermarginy: Option<f64>,
    /** Character height (for both buffer and grid windows) */
    pub charheight: Option<f64>,
    /** Character width (for both buffer and grid windows) */
    pub charwidth: Option<f64>,
    /** Graphics window margin */
    pub graphicsmargin: Option<f64>,
    /** Graphics window margin X */
    pub graphicsmarginx: Option<f64>,
    /** Graphics window margin Y */
    pub graphicsmarginy: Option<f64>,
    /** Grid character height */
    pub gridcharheight: Option<f64>,
    /** Grid character width */
    pub gridcharwidth: Option<f64>,
    /** Grid window margin */
    pub gridmargin: Option<f64>,
    /** Grid window margin X */
    pub gridmarginx: Option<f64>,
    /** Grid window margin Y */
    pub gridmarginy: Option<f64>,
    pub height: f64,
    /** Inspacing */
    pub inspacing: Option<f64>,
    /** Inspacing X */
    pub inspacingx: Option<f64>,
    /** Inspacing Y */
    pub inspacingy: Option<f64>,
    /** Margin for all window types */
    pub margin: Option<f64>,
    /** Margin X for all window types */
    pub marginx: Option<f64>,
    /** Margin Y for all window types */
    pub marginy: Option<f64>,
    /** Outspacing */
    pub outspacing: Option<f64>,
    /** Outspacing X */
    pub outspacingx: Option<f64>,
    /** Outspacing Y */
    pub outspacingy: Option<f64>,
    /** Spacing */
    pub spacing: Option<f64>,
    /** Spacing X */
    pub spacingx: Option<f64>,
    /** Spacing Y */
    pub spacingy: Option<f64>,
    pub width: f64,
}

/** Normalised screen and font metrics */
#[derive(Default)]
pub struct NormalisedMetrics {
    /** Buffer character height */
    pub buffercharheight: f64,
    /** Buffer character width */
    pub buffercharwidth: f64,
    /** Buffer window margin X */
    pub buffermarginx: f64,
    /** Buffer window margin Y */
    pub buffermarginy: f64,
    /** Graphics window margin X */
    pub graphicsmarginx: f64,
    /** Graphics window margin Y */
    pub graphicsmarginy: f64,
    /** Grid character height */
    pub gridcharheight: f64,
    /** Grid character width */
    pub gridcharwidth: f64,
    /** Grid window margin X */
    pub gridmarginx: f64,
    /** Grid window margin Y */
    pub gridmarginy: f64,
    pub height: f64,
    /** Inspacing X */
    pub inspacingx: f64,
    /** Inspacing Y */
    pub inspacingy: f64,
    /** Outspacing X */
    pub outspacingx: f64,
    /** Outspacing Y */
    pub outspacingy: f64,
    pub width: f64,
}

/** GlkApi/RemGlk->GlkOte content updates */
pub enum Update {
    Error(ErrorUpdate),
    Exit(ExitUpdate),
    Pass(PassUpdate),
    Retry(RetryUpdate),
    State(StateUpdate),
}

pub struct ErrorUpdate {
    /** Error message */
    pub message: String,
}

pub struct ExitUpdate {}

pub struct PassUpdate {}

pub struct RetryUpdate {}

#[derive(Default)]
pub struct StateUpdate {
    /* Library specific autorestore data */
    //pub autorestore: Option,
    /** Content data */
    pub content: Vec<ContentUpdate>,
    /* Debug output */
    //pub debugoutput: Option<Vec<String>>,
    pub disable: bool,
    /** Generation number */
    pub gen: u32,
    /** Windows with active input */
    pub input: Vec<InputUpdate>,
    /** Background colour for the page margin (ie, outside of the gameport) */
    pub page_margin_bg: Option<String>,
    /** Special input */
    pub specialinput: Option<SpecialInput>,
    // TODO: do we need a null here?
    pub timer: Option<u32>,
    /** Updates to window (new windows, or changes to their arrangements) */
    pub windows: Vec<WindowUpdate>,
}

// Update structures

/** Content update */
pub enum ContentUpdate {
    Buffer(BufferWindowContentUpdate),
    Graphics(GraphicsWindowContentUpdate),
    Grid(GridWindowContentUpdate),
}

#[derive(Default)]
pub struct TextualWindowUpdate {
    /** Window ID */
    pub id: u32,
    /** Clear the window */
    pub clear: bool,
    /** Background colour after clearing */
    pub bg: Option<String>,
    /** Foreground colour after clearing */
    pub fg: Option<String>,
}

/** Buffer window content update */
pub struct BufferWindowContentUpdate {
    pub base: TextualWindowUpdate,
    /** text data */
    pub text: Vec<BufferWindowParagraphUpdate>,
}

/** A buffer window paragraph */
pub struct BufferWindowParagraphUpdate {
    /** Append to last input */
    pub append: bool,
    /** Line data */
    pub content: Vec<LineData>,
    /** Paragraph breaks after floating images */
    pub flowbreak: bool,
}

/** Graphics window content update */
pub struct GraphicsWindowContentUpdate {
    /** Window ID */
    pub id: u32,
    /** Operations */
    pub draw: Vec<GraphicsWindowOperation>,
}

/** Graphics window operation */
pub enum GraphicsWindowOperation {
    Fill(FillOperation),
    Image(ImageOperation),
    SetColor(SetColorOperation),
}

/** Fill operation */
pub struct FillOperation {
    /** CSS color */
    pub color: Option<String>,
    pub height: Option<u32>,
    pub width: Option<u32>,
    /** X coordinate */
    pub x: Option<u32>,
    /** Y coordinate */
    pub  y: Option<u32>,
}

/** Image operation */
pub struct ImageOperation {
    pub height: u32,
    /** Image number (from Blorb or similar) */
    pub image: Option<u32>,
    pub width: u32,
    /** Image URL */
    pub url: Option<String>,
    /** X position */
    pub x: u32,
    /** Y position */
    pub  y: u32,
}

/** Setcolor operation */
pub struct SetColorOperation {
    /** CSS color */
    pub color: String,
}

/** Grid window content update */
pub struct GridWindowContentUpdate {
    pub base: TextualWindowUpdate,
    /** Lines data */
    pub lines: Vec<GridWindowLine>,
}
pub struct GridWindowLine {
    pub content: Vec<LineData>,
    pub line: u32,
}

/** Line data */
#[derive(Clone)]
pub enum LineData {
    //StylePair(String, String),
    Image(BufferWindowImage),
    TextRun(TextRun),
}

/** Buffer window image */
#[derive(Clone)]
pub struct BufferWindowImage {
    /** Image alignment */
    pub alignment: Option<BufferWindowImageAlignment>,
    /** Image alt text */
    pub alttext: Option<String>,
    pub height: u32,
    /** Hyperlink value */
    pub hyperlink: Option<u32>,
    /** Image number */
    pub image: Option<u32>,
    pub width: u32,
    /** Image URL */
    pub url: Option<String>,
}

/** Text run */
#[derive(Clone, Default)]
pub struct TextRun {
    /** Additional CSS styles */
    pub css_styles: Option<CSSProperties>,
    /** Hyperlink value */
    pub hyperlink: Option<u32>,
    /** Run style */
    pub style: u32,
    /** Run content */
    pub text: String,
}

/** Windows with active input */
#[derive(Default)]
pub struct InputUpdate {
    /** Generation number, for when the textual input was first requested */
    pub gen: Option<u32>,
    /** Hyperlink requested */
    pub hyperlink: bool,
    /** Window ID */
    pub id: u32,
    /** Preloaded line input */
    pub initial: Option<String>,
    /** Maximum line input length */
    pub maxlen: Option<u32>,
    /** Mouse input requested */
    pub mouse: bool,
    /** Line input terminators */
    pub terminators: Option<Vec<TerminatorCode>>,
    /** Textual input type */
    pub text_input_type: Option<TextInputType>,
    /** Grid window coordinate X */
    pub xpos: Option<u32>,
    /** Grid window coordinate Y */
    pub ypos: Option<u32>,
}

impl InputUpdate {
    pub fn new(id: u32) -> Self {
        InputUpdate {
            id,
            ..Default::default()
        }
    }
}

pub struct SpecialInput {
    /** File mode */
    pub filemode: FileMode,
    /** File type */
    pub filetype: FileType,
    /** Game ID */
    pub gameid: Option<String>,
}

/** Updates to window (new windows, or changes to their arrangements) */
#[derive(Default)]
pub struct WindowUpdate {
    /** Graphics height (pixels) */
    pub graphheight: Option<u32>,
    /** Graphics width (pixels) */
    pub graphwidth: Option<u32>,
    /** Grid height (chars) */
    pub gridheight: Option<u32>,
    /** Grid width (chars) */
    pub gridwidth: Option<u32>,
    pub height: f64,
    /** Window ID */
    pub id: u32,
    /** Left position */
    pub left: f64,
    /** Rock value */
    pub rock: u32,
    /** Window styles */
    pub styles: Option<WindowStyles>,
    /** Top position */
    pub top: f64,
    /** Type */
    pub wintype: WindowType,
    pub width: f64,
}

/** CSS Properties
 *
 * CSS property names and values, with a few special property:
 * - `monospace`: sets a run to be monospaced, but adding class `monospace` to the `span`. Should only be used for `span`s, may misbehave if set on a `div`.
 * - `reverse`: enables reverse mode. If you provide colours then do not pre-reverse them.
 *   Ex: `background-color: #FFF, color: #000, reverse: 1` will be displayed as white text on a black background
 */
pub type CSSProperties = HashMap<String, CSSValue>;
#[derive(Clone, PartialEq)]
pub enum CSSValue {
    String(String),
    Number(f64),
}

/** CSS styles
 * Keys will usually be for Glk styles, ex: `div.Style_header` or `span.Style_user1`
 * But they can be anything else. Use a blank string to target the window itself.
*/
pub type WindowStyles = HashMap<String, CSSProperties>;

#[derive(Copy, Clone, PartialEq)]
pub enum BufferWindowImageAlignment {InlineCenter,InlineDown, InlineUp, MarginLeft, MarginRight}
#[derive(Copy, Clone, PartialEq)]
pub enum SpecialKeyCode {Delete, Down, End, Escape, Func1, Func2, Func3, Func4, Func5, Func6, Func7, Func8, Func9, Func10, Func11, Func12, Home, Left, Pagedown, Pageup, Return, Right, Tab, Up}
#[derive(Copy, Clone, PartialEq)]
pub enum TextInputType {Char, Line}