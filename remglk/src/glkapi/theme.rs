/*

Theme table and glk_style_measure support
=========================================

Copyright (c) 2026 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use super::constants::*;
use super::protocol::{StyleEntry, StyleTable, Theme};

pub type HintMatrix = [[Option<i32>; stylehint_NUMHINTS as usize]; style_NUMSTYLES as usize];

/** Resolved measurable attributes for one style (after inheriting from normal) */
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ResolvedStyle {
    pub fg: Option<u32>,
    pub bg: Option<u32>,
    pub weight: Option<i32>,
    pub oblique: Option<u32>,
    pub proportional: Option<u32>,
    pub reverse: Option<u32>,
    pub size: Option<u32>,
}

impl ResolvedStyle {
    fn from_partial(partial: &StyleEntry) -> Self {
        Self {
            fg: partial.fg,
            bg: partial.bg,
            weight: partial.weight,
            oblique: partial.oblique,
            proportional: partial.proportional,
            reverse: partial.reverse,
            size: partial.size,
        }
    }

    fn overlay(&self, partial: &StyleEntry) -> Self {
        Self {
            fg: partial.fg.or(self.fg),
            bg: partial.bg.or(self.bg),
            weight: partial.weight.or(self.weight),
            oblique: partial.oblique.or(self.oblique),
            proportional: partial.proportional.or(self.proportional),
            reverse: partial.reverse.or(self.reverse),
            size: partial.size.or(self.size),
        }
    }
}

/** Runner-provided theme table used by glk_style_measure */
#[derive(Clone, Debug, PartialEq)]
pub struct ThemeState {
    pub honor_game_styles: bool,
    /// Present only after GlkOte has sent a theme; until then measure returns None.
    resolved: Option<ResolvedTheme>,
}

#[derive(Clone, Debug, PartialEq)]
struct ResolvedTheme {
    buffer: [ResolvedStyle; style_NUMSTYLES as usize],
    grid: [ResolvedStyle; style_NUMSTYLES as usize],
}

impl Default for ThemeState {
    fn default() -> Self {
        Self {
            honor_game_styles: true,
            resolved: None,
        }
    }
}

impl ThemeState {
    pub fn apply(&mut self, theme: Theme) {
        self.honor_game_styles = theme.honor_game_styles;
        self.resolved = Some(ResolvedTheme {
            buffer: resolve_table(&theme.buffer),
            grid: resolve_table(&theme.grid),
        });
    }

    pub fn entry(&self, wintype: WindowType, style: u32) -> Option<ResolvedStyle> {
        let resolved = self.resolved.as_ref()?;
        let table = match wintype {
            WindowType::Buffer => &resolved.buffer,
            WindowType::Grid => &resolved.grid,
            _ => return None,
        };
        if style < style_NUMSTYLES {
            Some(table[style as usize])
        } else {
            Some(table[style_Normal as usize])
        }
    }

    pub fn measure_theme(&self, wintype: WindowType, style: u32, hint: u32) -> Option<u32> {
        if style >= style_NUMSTYLES || hint >= stylehint_NUMHINTS {
            return None;
        }
        let entry = self.entry(wintype, style)?;

        #[allow(non_upper_case_globals)]
        match hint {
            stylehint_Indentation | stylehint_ParaIndentation => Some(0),
            stylehint_Justification => Some(stylehint_just_LeftFlush),
            stylehint_Size => entry.size,
            stylehint_Weight => entry.weight.map(|w| w as u32),
            stylehint_Oblique => entry.oblique,
            stylehint_Proportional => entry.proportional,
            stylehint_TextColor => entry.fg,
            stylehint_BackColor => entry.bg,
            stylehint_ReverseColor => entry.reverse,
            _ => None,
        }
    }
}

fn resolve_table(table: &StyleTable) -> [ResolvedStyle; style_NUMSTYLES as usize] {
    let normal_partial = table.get("normal").cloned().unwrap_or_default();
    let normal = ResolvedStyle::from_partial(&normal_partial);
    let mut out = [normal; style_NUMSTYLES as usize];
    for (name, partial) in table.iter() {
        if let Some(style) = style_from_name(name) {
            out[style as usize] = normal.overlay(partial);
        }
    }
    out[style_Normal as usize] = normal;
    out
}

pub fn measure_style(
    theme: &ThemeState,
    honor_game_styles: bool,
    hint_matrix: &HintMatrix,
    wintype: WindowType,
    style: u32,
    hint: u32,
) -> Option<u32> {
    if style >= style_NUMSTYLES || hint >= stylehint_NUMHINTS {
        return None;
    }

    if honor_game_styles {
        if let Some(value) = hint_matrix[style as usize][hint as usize] {
            return Some(value as u32);
        }
    }

    theme.measure_theme(wintype, style, hint)
}

pub fn set_hint(hint_matrix: &mut HintMatrix, style: u32, hint: u32, value: i32) {
    if style < style_NUMSTYLES && hint < stylehint_NUMHINTS {
        hint_matrix[style as usize][hint as usize] = Some(value);
    }
}

pub fn clear_hint(hint_matrix: &mut HintMatrix, style: u32, hint: u32) {
    if style < style_NUMSTYLES && hint < stylehint_NUMHINTS {
        hint_matrix[style as usize][hint as usize] = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn empty_hints() -> HintMatrix {
        [[None; stylehint_NUMHINTS as usize]; style_NUMSTYLES as usize]
    }

    fn full_entry(fg: u32, bg: u32, size: u32) -> StyleEntry {
        StyleEntry {
            fg: Some(fg),
            bg: Some(bg),
            weight: Some(0),
            oblique: Some(0),
            proportional: Some(1),
            reverse: Some(0),
            size: Some(size),
        }
    }

    fn test_theme() -> ThemeState {
        let mut buffer = HashMap::new();
        buffer.insert("normal".to_string(), full_entry(0x222222, 0xFFFFFF, 16));
        buffer.insert("input".to_string(), StyleEntry {
            fg: Some(0x0B4C8E),
            ..Default::default()
        });
        let mut grid = HashMap::new();
        grid.insert("normal".to_string(), StyleEntry {
            fg: Some(0x000000),
            bg: Some(0xFFFFFF),
            weight: Some(0),
            oblique: Some(0),
            proportional: Some(0),
            reverse: Some(0),
            size: Some(14),
        });
        let mut theme = ThemeState::default();
        theme.apply(Theme {
            honor_game_styles: true,
            buffer,
            grid,
        });
        theme
    }

    #[test]
    fn no_theme_returns_none() {
        let theme = ThemeState::default();
        let hints = empty_hints();
        assert_eq!(
            measure_style(&theme, true, &hints, WindowType::Buffer, style_Normal, stylehint_TextColor),
            None
        );
    }

    #[test]
    fn hint_round_trip_when_enabled() {
        let theme = test_theme();
        let mut hints = empty_hints();
        set_hint(&mut hints, style_Normal, stylehint_TextColor, 0x123456);
        set_hint(&mut hints, style_Normal, stylehint_BackColor, 0x654321);

        assert_eq!(
            measure_style(&theme, true, &hints, WindowType::Buffer, style_Normal, stylehint_TextColor),
            Some(0x123456)
        );
        assert_eq!(
            measure_style(&theme, true, &hints, WindowType::Buffer, style_Normal, stylehint_BackColor),
            Some(0x654321)
        );
    }

    #[test]
    fn theme_wins_when_game_styles_not_honored() {
        let theme = test_theme();
        let mut hints = empty_hints();
        set_hint(&mut hints, style_Normal, stylehint_TextColor, 0x123456);

        assert_eq!(
            measure_style(&theme, false, &hints, WindowType::Buffer, style_Normal, stylehint_TextColor),
            Some(0x222222)
        );
    }

    #[test]
    fn grid_vs_buffer_window_type() {
        let mut buffer = HashMap::new();
        buffer.insert("normal".to_string(), full_entry(0x222222, 0xFFFFFF, 16));
        let mut grid = HashMap::new();
        grid.insert("normal".to_string(), StyleEntry {
            fg: Some(0xDDDDDD),
            bg: Some(0x111111),
            weight: Some(0),
            oblique: Some(0),
            proportional: Some(0),
            reverse: Some(0),
            size: Some(14),
        });
        let mut theme = ThemeState::default();
        theme.apply(Theme {
            honor_game_styles: true,
            buffer,
            grid,
        });
        let hints = empty_hints();

        assert_eq!(
            measure_style(&theme, true, &hints, WindowType::Grid, style_Normal, stylehint_TextColor),
            Some(0xDDDDDD)
        );
        assert_eq!(
            measure_style(&theme, true, &hints, WindowType::Grid, style_Normal, stylehint_BackColor),
            Some(0x111111)
        );
    }

    #[test]
    fn sparse_style_inherits_from_normal() {
        let theme = test_theme();
        let hints = empty_hints();

        assert_eq!(
            measure_style(&theme, true, &hints, WindowType::Buffer, style_Input, stylehint_TextColor),
            Some(0x0B4C8E)
        );
        assert_eq!(
            measure_style(&theme, true, &hints, WindowType::Buffer, style_Input, stylehint_BackColor),
            Some(0xFFFFFF)
        );
        assert_eq!(
            measure_style(&theme, true, &hints, WindowType::Buffer, style_Emphasized, stylehint_TextColor),
            Some(0x222222)
        );
    }

    #[test]
    fn measured_font_size() {
        let theme = test_theme();
        let hints = empty_hints();

        assert_eq!(
            measure_style(&theme, true, &hints, WindowType::Buffer, style_Normal, stylehint_Size),
            Some(16)
        );
        assert_eq!(
            measure_style(&theme, true, &hints, WindowType::Grid, style_Normal, stylehint_Size),
            Some(14)
        );
    }

    #[test]
    fn non_colour_hardcoded_hints() {
        let theme = test_theme();
        let hints = empty_hints();

        assert_eq!(
            measure_style(&theme, true, &hints, WindowType::Buffer, style_Normal, stylehint_Indentation),
            Some(0)
        );
        assert_eq!(
            measure_style(&theme, true, &hints, WindowType::Buffer, style_Normal, stylehint_Justification),
            Some(stylehint_just_LeftFlush)
        );
    }

    #[test]
    fn clear_hint_restores_theme() {
        let theme = test_theme();
        let mut hints = empty_hints();
        set_hint(&mut hints, style_Normal, stylehint_TextColor, 0x123456);
        clear_hint(&mut hints, style_Normal, stylehint_TextColor);

        assert_eq!(
            measure_style(&theme, true, &hints, WindowType::Buffer, style_Normal, stylehint_TextColor),
            Some(0x222222)
        );
    }

    #[test]
    fn apply_theme_from_protocol() {
        let mut theme = ThemeState::default();
        let mut buffer = HashMap::new();
        buffer.insert("normal".to_string(), full_entry(0x111111, 0xEEEEEE, 18));
        let mut grid = HashMap::new();
        grid.insert("normal".to_string(), StyleEntry {
            fg: Some(0x222222),
            bg: Some(0x333333),
            weight: Some(0),
            oblique: Some(0),
            proportional: Some(0),
            reverse: Some(0),
            size: Some(12),
        });
        theme.apply(Theme {
            honor_game_styles: false,
            buffer,
            grid,
        });
        assert!(!theme.honor_game_styles);
        assert_eq!(theme.entry(WindowType::Buffer, style_Normal).unwrap().fg, Some(0x111111));
        assert_eq!(theme.entry(WindowType::Grid, style_Normal).unwrap().bg, Some(0x333333));
    }
}
