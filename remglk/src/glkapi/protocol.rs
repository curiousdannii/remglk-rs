/*

The GlkOte protocol
===================

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use std::collections::HashMap;
use std::ops::Not;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize, Serializer};

use super::*;

/** The GlkOte protocol has two parts:
 * 1. GlkOte sends events to GlkApi/RemGlk
 * 2. GlkApi/RemGlk send content updates to GlkOte
*/

/** GlkOte->GlkApi/RemGlk input events */
#[derive(Deserialize)]
pub struct Event {
    /** Generation number */
    pub gen: u32,
    /** Partial line input values */
    pub partial: PartialInputs,
    /** The specific event data */
    #[serde(flatten)]
    pub data: EventData,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum EventData {
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
    Sound(SoundEvent),
    #[serde(rename = "specialresponse")]
    Special(SpecialEvent),
    Timer(TimerEvent),
    Volume(VolumeEvent),
}

pub type PartialInputs = Option<HashMap<u32, String>>;

#[derive(Deserialize)]
pub struct ArrangeEvent {
    pub metrics: Metrics,
}

/** Character (single key) event */
#[derive(Deserialize)]
pub struct CharEvent {
    /** Character that was received */
    pub value: String,
    /** Window ID */
    pub window: u32,
}

#[derive(Deserialize)]
pub struct DebugEvent {
    pub value: String,
}

#[derive(Deserialize)]
pub struct ExternalEvent {
    // TODO?
    //value: any,
}

#[derive(Deserialize)]
pub struct HyperlinkEvent {
    pub value: u32,
    /** Window ID */
    pub window: u32,
}

/** Initilisation event */
#[derive(Deserialize)]
pub struct InitEvent {
    pub metrics: Metrics,
    /** Capabilities list */
    pub support: Vec<String>,
}

/** Line (text) event */
#[derive(Deserialize)]
pub struct LineEvent {
    /* Terminator key */
    pub terminator: Option<TerminatorCode>,
    /** Line input */
    pub value: String,
    /** Window ID */
    pub window: u32,
}

#[derive(Deserialize)]
pub struct MouseEvent {
    /** Window ID */
    pub window: u32,
    /** Mouse click X */
    pub x: u32,
    /** Mouse click Y */
    pub y: u32,
}

#[derive(Deserialize)]
pub struct RedrawEvent {
    /** Window ID */
    pub window: Option<u32>,
}

#[derive(Deserialize)]
pub struct RefreshEvent {}

#[derive(Deserialize)]
pub struct SoundEvent {
    pub notify: u32,
    pub snd: u32,
}

#[derive(Deserialize)]
pub struct SpecialEvent {
    /** Response type */
    pub response: String,
    /** Event value (file reference from Dialog) */
    pub value: Option<FileRefResponse>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum FileRefResponse {
    Path(String),
    Fref(SystemFileRef),
}

/** SystemFileRefs aren't used internally, but may be returned from `glk_fileref_create_by_prompt` */
#[derive(Clone, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct SystemFileRef {
    //pub content: Option<String>,
    pub filename: String,
    //pub gameid: Option<String>,
    //pub usage: Option<FileType>,
}

#[derive(Deserialize)]
pub struct TimerEvent {}

#[derive(Deserialize)]
pub struct VolumeEvent {
    pub notify: u32,
}

/** Screen and font metrics - all potential options */
#[derive(Default, Deserialize)]
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
    pub width: f64,
}

impl Default for NormalisedMetrics {
    fn default() -> Self {
        NormalisedMetrics {
            buffercharheight: 1.0,
            buffercharwidth: 1.0,
            buffermarginx: 0.0,
            buffermarginy: 0.0,
            graphicsmarginx: 0.0,
            graphicsmarginy: 0.0,
            gridcharheight: 1.0,
            gridcharwidth: 1.0,
            gridmarginx: 0.0,
            gridmarginy: 0.0,
            height: 50.0,
            inspacingx: 0.0,
            inspacingy: 0.0,
            width: 80.0,
        }
    }
}

impl NormalisedMetrics {
    fn apply_unnormalised(&mut self, metrics: &Metrics) {
        if let Some(val) = metrics.buffercharheight {
            self.buffercharheight = val;
        }
        if let Some(val) = metrics.buffercharwidth {
            self.buffercharwidth = val;
        }
        if let Some(val) = metrics.buffermarginx {
            self.buffermarginx = val;
        }
        if let Some(val) = metrics.buffermarginy {
            self.buffermarginy = val;
        }
        if let Some(val) = metrics.graphicsmarginx {
            self.graphicsmarginx = val;
        }
        if let Some(val) = metrics.graphicsmarginy {
            self.graphicsmarginy = val;
        }
        if let Some(val) = metrics.gridcharheight {
            self.gridcharheight = val;
        }
        if let Some(val) = metrics.gridcharwidth {
            self.gridcharwidth = val;
        }
        if let Some(val) = metrics.gridmarginx {
            self.gridmarginx = val;
        }
        if let Some(val) = metrics.gridmarginy {
            self.gridmarginy = val;
        }
        self.height = metrics.height;
        if let Some(val) = metrics.inspacingx {
            self.inspacingx = val;
        }
        if let Some(val) = metrics.inspacingy {
            self.inspacingy = val;
        }
        self.width = metrics.width;
    }
}

impl From<Metrics> for GlkResult<'static, NormalisedMetrics> {
    fn from(metrics: Metrics) -> Self {
        if let Some(val) = metrics.outspacing {
            if val > 0.0 {
                return Err(OutspacingMustBeZero);
            }
        }
        if let Some(val) = metrics.outspacingx {
            if val > 0.0 {
                return Err(OutspacingMustBeZero);
            }
        }
        if let Some(val) = metrics.outspacingy {
            if val > 0.0 {
                return Err(OutspacingMustBeZero);
            }
        }

        let mut normalised_metrics = NormalisedMetrics::default();

        if let Some(val) = metrics.charheight {
            normalised_metrics.buffercharheight = val;
            normalised_metrics.gridcharheight = val;
        }
        if let Some(val) = metrics.charwidth {
            normalised_metrics.buffercharwidth = val;
            normalised_metrics.gridcharwidth = val;
        }

        if let Some(val) = metrics.margin {
            normalised_metrics.buffermarginx = val;
            normalised_metrics.buffermarginy = val;
            normalised_metrics.graphicsmarginx = val;
            normalised_metrics.graphicsmarginy = val;
            normalised_metrics.gridmarginx = val;
            normalised_metrics.gridmarginy = val;
        }
        if let Some(val) = metrics.buffermargin {
            normalised_metrics.buffermarginx = val;
            normalised_metrics.buffermarginy = val;
        }
        if let Some(val) = metrics.graphicsmargin {
            normalised_metrics.graphicsmarginx = val;
            normalised_metrics.graphicsmarginy = val;
        }
        if let Some(val) = metrics.gridmargin {
            normalised_metrics.gridmarginx = val;
            normalised_metrics.gridmarginy = val;
        }
        if let Some(val) = metrics.marginx {
            normalised_metrics.buffermarginx = val;
            normalised_metrics.graphicsmarginx = val;
            normalised_metrics.gridmarginx = val;
        }
        if let Some(val) = metrics.marginy {
            normalised_metrics.buffermarginy = val;
            normalised_metrics.graphicsmarginy = val;
            normalised_metrics.gridmarginy = val;
        }

        if let Some(val) = metrics.spacing {
            normalised_metrics.inspacingx = val;
            normalised_metrics.inspacingy = val;
        }
        if let Some(val) = metrics.inspacing {
            normalised_metrics.inspacingx = val;
            normalised_metrics.inspacingy = val;
        }
        if let Some(val) = metrics.spacingx {
            normalised_metrics.inspacingx = val;
        }
        if let Some(val) = metrics.spacingy {
            normalised_metrics.inspacingy = val;
        }

        normalised_metrics.apply_unnormalised(&metrics);
        Ok(normalised_metrics)
    }
}

/** GlkApi/RemGlk->GlkOte content updates */
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum Update {
    Error(ErrorUpdate),
    Pass(PassUpdate),
    Retry(RetryUpdate),
    #[serde(rename = "update")]
    State(StateUpdate),
}

#[derive(Serialize)]
pub struct ErrorUpdate {
    /** Error message */
    pub message: String,
}

#[derive(Serialize)]
pub struct PassUpdate {}

#[derive(Serialize)]
pub struct RetryUpdate {}

#[derive(Default, Serialize)]
pub struct StateUpdate {
    /* Library specific autorestore data */
    //pub autorestore: Option,
    /** Content data */
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<ContentUpdate>,
    /* Debug output */
    //pub debugoutput: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Not::not")]
    pub disable: bool,
    /** Generation number */
    pub gen: u32,
    /** Windows with active input */
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub input: Vec<InputUpdate>,
    /** Background colour for the page margin (ie, outside of the gameport); blank means remove the current background colour */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_margin_bg: Option<String>,
    /** Sound channels */
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub schannels: Vec<SoundChannelUpdate>,
    /* Special input */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub specialinput: Option<SpecialInput>,
    // TODO: do we need a null here?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timer: Option<u32>,
    /** Updates to window (new windows, or changes to their arrangements) */
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub windows: Vec<WindowUpdate>,
}

// Update structures

/** Content update */
#[derive(Serialize)]
#[serde(untagged)]
pub enum ContentUpdate {
    Buffer(BufferWindowContentUpdate),
    Graphics(GraphicsWindowContentUpdate),
    Grid(GridWindowContentUpdate),
}

#[derive(Default, Serialize)]
pub struct TextualWindowUpdate {
    /** Window ID */
    pub id: u32,
    /** Clear the window */
    #[serde(skip_serializing_if = "Not::not")]
    pub clear: bool,
    /** Background colour after clearing */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<String>,
    /** Foreground colour after clearing */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fg: Option<String>,
}

impl TextualWindowUpdate {
    pub fn new(id: u32) -> Self {
        TextualWindowUpdate {
            id,
            ..Default::default()
        }
    }
}

/** Buffer window content update */
#[derive(Serialize)]
pub struct BufferWindowContentUpdate {
    #[serde(flatten)]
    pub base: TextualWindowUpdate,
    /** text data */
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub text: Vec<BufferWindowParagraphUpdate>,
}

/** A buffer window paragraph */
#[derive(Default, Serialize)]
pub struct BufferWindowParagraphUpdate {
    /** Append to last input */
    #[serde(skip_serializing_if = "Not::not")]
    pub append: bool,
    /** Line data */
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<LineData>,
    /** Paragraph breaks after floating images */
    #[serde(skip_serializing_if = "Not::not")]
    pub flowbreak: bool,
}

impl BufferWindowParagraphUpdate {
    pub fn new(textrun: TextRun) -> Self {
        BufferWindowParagraphUpdate {
            content: vec![LineData::TextRun(textrun)],
            ..Default::default()
        }
    }
}

/** Graphics window content update */
#[derive(Serialize)]
pub struct GraphicsWindowContentUpdate {
    /** Window ID */
    pub id: u32,
    /** Operations */
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub draw: Vec<GraphicsWindowOperation>,
}

/** Graphics window operation */
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "special")]
pub enum GraphicsWindowOperation {
    Fill(FillOperation),
    Image(ImageOperation),
    SetColor(SetColorOperation),
}

/** Fill operation */
#[derive(Default, Serialize)]
pub struct FillOperation {
    /** CSS color */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /** X coordinate */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<i32>,
    /** Y coordinate */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<i32>,
}

/** Image operation */
#[derive(Serialize)]
pub struct ImageOperation {
    pub height: u32,
    /** Image number (from Blorb or similar) */
    pub image: u32,
    pub width: u32,
    /** X position */
    pub x: i32,
    /** Y position */
    pub y: i32,
}

/** Setcolor operation */
#[derive(Serialize)]
pub struct SetColorOperation {
    /** CSS color */
    pub color: String,
}

/** Grid window content update */
#[derive(Serialize)]
pub struct GridWindowContentUpdate {
    #[serde(flatten)]
    pub base: TextualWindowUpdate,
    /** Lines data */
    pub lines: Vec<GridWindowLine>,
}
#[derive(Serialize)]
pub struct GridWindowLine {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<LineData>,
    pub line: u32,
}

/** Line data */
#[derive(Clone, Serialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "special")]
pub enum LineData {
    //StylePair(String, String),
    Image(BufferWindowImage),
    #[serde(untagged)]
    TextRun(TextRun),
}

/** Buffer window image */
#[derive(Clone, Serialize)]
pub struct BufferWindowImage {
    /* Image alignment */
    pub alignment: BufferWindowImageAlignment,
    /** Image alt text */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alttext: Option<String>,
    pub height: u32,
    /** Hyperlink value */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hyperlink: Option<u32>,
    /** Image number */
    pub image: u32,
    pub width: u32,
}

/** Text run */
#[derive(Clone, Default, Serialize)]
pub struct TextRun {
    /** Additional CSS styles */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css_styles: Option<Arc<Mutex<CSSProperties>>>,
    /** Hyperlink value */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hyperlink: Option<u32>,
    /** Run style */
    #[serde(serialize_with = "serialize_style_name")]
    pub style: u32,
    /** Run content */
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

/** Windows with active input */
#[derive(Default, Serialize)]
pub struct InputUpdate {
    /** Generation number, for when the textual input was first requested */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gen: Option<u32>,
    /** Hyperlink requested */
    #[serde(skip_serializing_if = "Not::not")]
    pub hyperlink: bool,
    /** Window ID */
    pub id: u32,
    /** Preloaded line input */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial: Option<String>,
    /** Maximum line input length */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxlen: Option<u32>,
    /** Mouse input requested */
    #[serde(skip_serializing_if = "Not::not")]
    pub mouse: bool,
    /* Line input terminators */
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub terminators: Vec<TerminatorCode>,
    /** Textual input type */
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_input_type: Option<TextInputType>,
    /** Grid window coordinate X */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xpos: Option<u32>,
    /** Grid window coordinate Y */
    #[serde(skip_serializing_if = "Option::is_none")]
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

#[derive(Default, Serialize)]
pub struct SoundChannelUpdate {
    /** Sound channel ID */
    pub id: u32,
    /** Operations */
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ops: Vec<SoundChannelOperation>,
}

/** Sound channel operation */
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "op")]
pub enum SoundChannelOperation {
    Pause,
    Play(PlayOperation),
    Stop,
    Unpause,
    Volume(SetVolumeOperation),
}

#[derive(Default, Serialize)]
pub struct PlayOperation {
    /** Notification value */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify: Option<u32>,
    /** Number of repeats (default: 1) */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeats: Option<u32>,
    /** Sound resource ID (from a Blorb) */
    pub snd: u32,
}

#[derive(Default, Serialize)]
pub struct SetVolumeOperation {
    /** Duration in milliseconds */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dur: Option<u32>,
    /** Notification value */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify: Option<u32>,
    /** The volume as a number between 0 and 1 */
    pub vol: f64,
}

#[derive(Default, Serialize)]
pub struct SpecialInput {
    /** File mode */
    pub filemode: FileMode,
    /** File type */
    pub filetype: FileType,
    /** Game ID */
    pub gameid: Option<String>,
    #[serde(rename = "type")]
    #[serde(serialize_with = "emit_fileref_prompt")]
    pub specialtype: (),
}

fn emit_fileref_prompt<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str("fileref_prompt")
}

/** Updates to window (new windows, or changes to their arrangements) */
#[derive(Default, Serialize)]
pub struct WindowUpdate {
    /** Graphics height (pixels) */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graphheight: Option<u32>,
    /** Graphics width (pixels) */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graphwidth: Option<u32>,
    /** Grid height (chars) */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gridheight: Option<u32>,
    /** Grid width (chars) */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gridwidth: Option<u32>,
    pub height: f64,
    /** Whether the window should be completely hidden, though could potentially still respond to character events */
    #[serde(skip_serializing_if = "Not::not")]
    pub hidden: bool,
    /** Window ID */
    pub id: u32,
    /** Left position */
    pub left: f64,
    /** Rock value */
    pub rock: u32,
    /** Window styles */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub styles: Option<WindowStyles>,
    /** Top position */
    pub top: f64,
    /** Type */
    #[serde(rename = "type")]
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
#[derive(Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum CSSValue {
    String(String),
    Number(f64),
}

/** CSS styles
 * Keys will usually be for Glk styles, ex: `.Style_header` or `.Style_user1_par`
 * But they can be anything else. Use a blank string to target the window itself.
*/
pub type WindowStyles = HashMap<String, CSSProperties>;

#[derive(Copy, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TextInputType {Char, Line}