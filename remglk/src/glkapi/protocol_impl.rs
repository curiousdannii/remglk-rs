/*

GlkOte protocol implementation helpers
======================================

Copyright (c) 2025 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

//use core::fmt;
use std::sync::Arc;

use serde::Serializer;
//use serde::de::{Deserialize, Deserializer, Visitor, MapAccess};

use super::*;

// Metrics

// Based on https://serde.rs/deserialize-struct.html
/*impl<'de> Deserialize<'de> for Metrics {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        struct MetricsVistor;

        impl<'de> Visitor<'de> for MetricsVistor {
            type Value = Metrics;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Metrics")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Metrics, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut new_metrics = Metrics::default();
                while let Some(key) = map.next_key::<&str>()? {
                    let value = map.next_value::<f64>()?;
                    match key {
                        "buffercharheight" => new_metrics.buffercharheight = Some(value),
                        "buffercharwidth" => new_metrics.buffercharwidth = Some(value),
                        "buffermargin" => new_metrics.buffermargin = Some(value),
                        "buffermarginx" => new_metrics.buffermarginx = Some(value),
                        "buffermarginy" => new_metrics.buffermarginy = Some(value),
                        "charheight" => new_metrics.charheight = Some(value),
                        "charwidth" => new_metrics.charheight = Some(value),
                        "graphicsmargin" => new_metrics.graphicsmargin = Some(value),
                        "graphicsmarginx" => new_metrics.graphicsmarginx = Some(value),
                        "graphicsmarginy" => new_metrics.graphicsmarginy = Some(value),
                        "gridcharheight" => new_metrics.gridcharheight = Some(value),
                        "gridcharwidth" => new_metrics.gridcharwidth = Some(value),
                        "gridmargin" => new_metrics.gridmargin = Some(value),
                        "gridmarginx" => new_metrics.gridmarginx = Some(value),
                        "gridmarginy" => new_metrics.gridmarginy = Some(value),
                        "height" => new_metrics.height = value,
                        "inspacing" => new_metrics.inspacing = Some(value),
                        "inspacingx" => new_metrics.inspacingx = Some(value),
                        "inspacingy" => new_metrics.inspacingy = Some(value),
                        "margin" => new_metrics.margin = Some(value),
                        "marginx" => new_metrics.marginx = Some(value),
                        "marginy" => new_metrics.marginy = Some(value),
                        "outspacing" => new_metrics.outspacing = Some(value),
                        "outspacingx" => new_metrics.outspacingx = Some(value),
                        "outspacingy" => new_metrics.outspacingy = Some(value),
                        "spacing" => new_metrics.spacing = Some(value),
                        "spacingx" => new_metrics.spacingx = Some(value),
                        "spacingy" => new_metrics.spacingy = Some(value),
                        "width" => new_metrics.width = value,
                        _ => {},
                    }
                }
                Ok(new_metrics)
            }
        }

        const FIELDS: &[&str] = &["buffercharheight", "buffercharwidth", "buffermargin", "buffermarginx", "buffermarginy", "charheight", "charwidth", "graphicsmargin", "graphicsmarginx", "graphicsmarginy", "gridcharheight", "gridcharwidth", "gridmargin", "gridmarginx", "gridmarginy", "height", "inspacing", "inspacingx", "inspacingy", "margin", "marginx", "marginy", "outspacing", "outspacingx", "outspacingy", "spacing", "spacingx", "spacingy", "width"];
        deserializer.deserialize_struct("Metrics", FIELDS, MetricsVistor)
    }
}*/

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

// Content updates

impl TextualWindowUpdate {
    pub fn new(id: u32) -> Self {
        TextualWindowUpdate {
            id,
            ..Default::default()
        }
    }
}

impl BufferWindowParagraphUpdate {
    pub fn new(textrun: TextRun) -> Self {
        BufferWindowParagraphUpdate {
            content: vec![LineData::TextRun(textrun)],
            ..Default::default()
        }
    }
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

impl InputUpdate {
    pub fn new(id: u32) -> Self {
        InputUpdate {
            id,
            ..Default::default()
        }
    }
}

pub fn emit_fileref_prompt<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str("fileref_prompt")
}