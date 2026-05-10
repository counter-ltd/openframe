use crate::{
    AnyElement, App, IntoElement, ParentElement, Rgba, Styled, Window, div, px, rems, rgb,
};
use smallvec::SmallVec;

const DEFAULT_CHARS: [char; 7] = ['┌', '─', '┐', '│', '└', '─', '┘'];

#[derive(Clone, Copy)]
enum CornerSlot {
    Tl,
    Tr,
    Bl,
    Br,
}

/// A container that draws a border from seven glyph slots (see [`GlyphBorder::border_chars`]).
///
/// Callers (e.g. app themes) should set [`GlyphBorder::border_font_family`], size, and rail width
/// when they are not using the default built-in look.
///
/// ```text
/// ┌──────────────────────────┐
/// │  <children>              │
/// └──────────────────────────┘
/// ```
///
/// Optional [`GlyphBorder::border_horizontal_pattern`] / [`GlyphBorder::border_vertical_pattern`]
/// repeat over **edges** (corners stay [`GlyphBorder::border_chars`] indices 0, 2, 4, 6). Patterns may
/// include spaces for gaps (`"- "`, `"nunu"`, `"│ "`).
pub struct GlyphBorder {
    children: SmallVec<[AnyElement; 2]>,
    border_color: Rgba,
    bg: Rgba,
    chars: [char; 7],
    /// When `None`, [`Self::render`] uses `"monospace"`.
    border_font_family: Option<String>,
    /// When `None`, uses `0.75` rem (same scale as `text_xs` in this framework).
    border_font_size_rems: Option<f32>,
    /// When `None`, uses `11` px — wide enough for typical single-column box-drawing glyphs.
    border_side_rail_px: Option<f32>,
    /// When `None`, uses [`DEFAULT_VERTICAL_RAIL_LINES`]. Lower this for short boxes (e.g. list rows)
    /// so scroll/layout does not process thousands of clipped newline-separated glyphs.
    vertical_rail_lines: Option<usize>,
    /// When set, top and bottom rules repeat this sequence instead of `chars[1]` / `chars[5]`.
    horizontal_rule_pattern: Option<String>,
    /// When set, side rails cycle one character per line through this sequence instead of `chars[3]`.
    vertical_rail_pattern: Option<String>,
}

/// Construct a [`GlyphBorder`] with default demo colors (apps normally override).
pub fn glyph_border() -> GlyphBorder {
    GlyphBorder {
        children: SmallVec::new(),
        border_color: rgb(0x00cc88),
        bg: rgb(0x111111),
        chars: DEFAULT_CHARS,
        border_font_family: None,
        border_font_size_rems: None,
        border_side_rail_px: None,
        vertical_rail_lines: None,
        horizontal_rule_pattern: None,
        vertical_rail_pattern: None,
    }
}

/// Repeat `pat` Unicode scalar sequence `unit_count` times (spaces and multi-char cycles allowed).
fn repeat_pattern_units(pat: &str, unit_count: usize) -> String {
    let chars: Vec<char> = pat.chars().collect();
    if chars.is_empty() {
        return String::new();
    }
    (0..unit_count)
        .map(|i| chars[i % chars.len()])
        .collect()
}

/// Horizontal repeat count for top/bottom rules; clipped by `overflow_hidden` — keep modest for perf.
const HORIZONTAL_RULE_REPEAT: usize = 512;
/// Default newline-separated side glyphs for the middle band (tall scrollable panels).
const DEFAULT_VERTICAL_RAIL_LINES: usize = 512;

/// Horizontal rules use `━ ` — dash + space share ~one monospace advance each. Vertical must use
/// **similar px height** for dash and gap rows; full text line-height made ┃ segments huge vs ━.
fn vertical_dash_gap_px(font_rems: f32) -> (f32, f32) {
    let cell = (font_rems * 16.0 * 0.46).clamp(5.0, 10.5);
    (cell, cell)
}

/// How many structured vertical rows to generate — enough to paint tall panels without thousands of
/// DOM nodes per rail (scroll perf).
fn structured_vertical_row_budget(
    dash_px: f32,
    gap_px: f32,
    vert_cycle: &[char],
) -> usize {
    if vert_cycle.is_empty() {
        return DEFAULT_VERTICAL_RAIL_LINES.min(480);
    }
    let mut cycle_px = 0.0_f32;
    for &c in vert_cycle {
        cycle_px += if c.is_whitespace() { gap_px } else { dash_px };
    }
    cycle_px = cycle_px.max(dash_px + gap_px);
    const TARGET_COVER_PX: f32 = 3200.0;
    let cycles = (TARGET_COVER_PX / cycle_px).ceil() as usize;
    let rows = cycles.saturating_mul(vert_cycle.len());
    // `┃ `–style borders pair glyph+gap into one element — micro-rows can stay modest.
    let lower = vert_cycle.len().saturating_mul(4).min(384);
    rows.max(lower).min(384)
}

impl GlyphBorder {
    /// Override border / corner-glyph color.
    pub fn border_color(mut self, color: Rgba) -> Self {
        self.border_color = color;
        self
    }

    /// Override background color of the inner content area.
    pub fn bg(mut self, color: Rgba) -> Self {
        self.bg = color;
        self
    }

    /// Override the border glyphs (any characters — themes pick symbols from fonts users install).
    ///
    /// Order: `[top_left, top, top_right, side, bottom_left, bottom, bottom_right]`
    pub fn border_chars(mut self, chars: [char; 7]) -> Self {
        self.chars = chars;
        self
    }

    /// Font family for all border/corner glyphs (horizontal runs and vertical rails).
    pub fn border_font_family(mut self, name: impl Into<String>) -> Self {
        self.border_font_family = Some(name.into());
        self
    }

    /// Font size for border glyphs, in **rem** (must match [`Self::border_font_family`] metrics).
    pub fn border_font_size_rems(mut self, rems: f32) -> Self {
        self.border_font_size_rems = Some(rems);
        self
    }

    /// Width in px of the left/right vertical-rail column (increase for wide glyphs).
    pub fn border_side_rail_px(mut self, px: f32) -> Self {
        self.border_side_rail_px = Some(px);
        self
    }

    /// How many `side` glyphs (plus newlines) build each vertical rail. Short boxes (e.g. table rows)
    /// should pass ~`48..96` so nested lists stay cheap to scroll.
    pub fn vertical_rail_lines(mut self, lines: usize) -> Self {
        self.vertical_rail_lines = Some(lines.max(8));
        self
    }

    /// Repeating sequence for top and bottom horizontal rules (same pattern on both rows).
    pub fn border_horizontal_pattern(mut self, pattern: impl Into<String>) -> Self {
        let s = pattern.into();
        self.horizontal_rule_pattern = if s.is_empty() { None } else { Some(s) };
        self
    }

    /// One repeating sequence for vertical rails: line *i* uses pattern character `i % len`.
    pub fn border_vertical_pattern(mut self, pattern: impl Into<String>) -> Self {
        let s = pattern.into();
        self.vertical_rail_pattern = if s.is_empty() { None } else { Some(s) };
        self
    }
}

impl ParentElement for GlyphBorder {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl crate::IntoElement for GlyphBorder {
    type Element = crate::Component<Self>;

    fn into_element(self) -> Self::Element {
        crate::Component::new(self)
    }
}

impl crate::RenderOnce for GlyphBorder {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let bc = self.border_color;
        let bg = self.bg;
        let ch = self.chars;

        // Owned so border text closures can borrow `'static`-friendly `&str` for `font_family`.
        let font_name = self
            .border_font_family
            .clone()
            .unwrap_or_else(|| "monospace".to_string());
        let font_rems = self.border_font_size_rems.unwrap_or(0.75);
        let rail_px = self.border_side_rail_px.unwrap_or(11.);
        let side_lines = self
            .vertical_rail_lines
            .unwrap_or(DEFAULT_VERTICAL_RAIL_LINES)
            .max(8);
        let h_pattern = self.horizontal_rule_pattern.clone();
        let v_pattern = self.vertical_rail_pattern.clone();

        // Pin each corner inside a column exactly `rail_px` wide (same as vertical rails). Intrinsic
        // glyph width is smaller; without fixed width + inner-edge alignment, horizontal rules and
        // vertical rails meet at different X positions (stepped corners).

        let corner_cell = |c: char, slot: CornerSlot| {
            let mut cell = div()
                .flex()
                .flex_col()
                .flex_none()
                .w(px(rail_px))
                .flex_shrink_0()
                .text_color(bc)
                .font_family(font_name.clone())
                .text_size(rems(font_rems))
                .line_height(rems(font_rems));
            cell = match slot {
                CornerSlot::Tl => cell.justify_start().items_end(),
                CornerSlot::Tr => cell.justify_start().items_start(),
                CornerSlot::Bl => cell.justify_end().items_end(),
                CornerSlot::Br => cell.justify_end().items_start(),
            };
            cell.child(c.to_string())
        };

        let horiz_fill = |mid: char| {
            let text = if let Some(ref pat) = h_pattern {
                repeat_pattern_units(pat, HORIZONTAL_RULE_REPEAT)
            } else {
                std::iter::repeat(mid)
                    .take(HORIZONTAL_RULE_REPEAT)
                    .collect::<String>()
            };
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .text_color(bc)
                .font_family(font_name.clone())
                .text_size(rems(font_rems))
                .line_height(rems(font_rems))
                .whitespace_nowrap()
                .child(text)
        };

        let vert_cycle: Vec<char> = v_pattern
            .as_deref()
            .map(|s| s.chars().collect::<Vec<char>>())
            .filter(|v| !v.is_empty())
            .unwrap_or_default();

        let pattern_has_whitespace_gap = v_pattern
            .as_deref()
            .is_some_and(|s| s.chars().any(|c| c.is_whitespace()));

        let use_structured_vertical =
            !vert_cycle.is_empty() && pattern_has_whitespace_gap;

        let (dash_px, gap_px) = vertical_dash_gap_px(font_rems);

        let rail_fill_rows = if use_structured_vertical {
            structured_vertical_row_budget(dash_px, gap_px, &vert_cycle)
        } else {
            side_lines
        };

        let mut side_column = String::with_capacity(side_lines.saturating_mul(8));
        if !use_structured_vertical {
            for line_idx in 0..side_lines {
                let c = if !vert_cycle.is_empty() {
                    vert_cycle[line_idx % vert_cycle.len()]
                } else {
                    ch[3]
                };
                side_column.push(c);
                side_column.push('\n');
            }
        }

        // Rails are wider than a single glyph so box-drawing fonts have room; glyphs must hug the
        // **inner** edge (toward `children`), not the outer edge — otherwise a visible gap opens
        // between content and the vertical border (see align: left rail → end, right rail → start).
        let side_fill = |hug_inner: bool| {
            let inner: AnyElement = if use_structured_vertical {
                // Common TUI pattern `┃ ` — one stacked cell per dash+gap cycle (~half the DOM nodes).
                let pair_glyph_gap = vert_cycle.len() == 2
                    && !vert_cycle[0].is_whitespace()
                    && vert_cycle[1].is_whitespace();
                let reserve = if pair_glyph_gap {
                    rail_fill_rows / 2 + 1
                } else {
                    rail_fill_rows.min(384)
                };
                let mut rows: Vec<AnyElement> = Vec::with_capacity(reserve);

                if pair_glyph_gap {
                    let glyph_ch = vert_cycle[0];
                    let band_h = dash_px + gap_px;
                    let pair_count = rail_fill_rows / 2;
                    for _ in 0..pair_count {
                        let mut stripe = div()
                            .flex_none()
                            .w_full()
                            .h(px(band_h))
                            .relative()
                            .overflow_hidden();
                        let mut glyph_plane = div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .right_0()
                            .h(px(dash_px))
                            .overflow_hidden()
                            .flex()
                            .flex_row()
                            .items_start()
                            .child(glyph_ch.to_string());
                        glyph_plane = if hug_inner {
                            glyph_plane.justify_end().relative().left(px(1.))
                        } else {
                            glyph_plane.justify_start()
                        };
                        stripe = stripe.child(glyph_plane);
                        rows.push(stripe.into_any_element());
                    }
                    if rail_fill_rows % 2 == 1 {
                        let mut tail = div()
                            .flex_none()
                            .w_full()
                            .h(px(dash_px))
                            .overflow_hidden()
                            .flex()
                            .flex_row()
                            .items_start()
                            .child(glyph_ch.to_string());
                        tail = if hug_inner {
                            tail.justify_end().relative().left(px(1.))
                        } else {
                            tail.justify_start()
                        };
                        rows.push(tail.into_any_element());
                    }
                } else {
                    for line_idx in 0..rail_fill_rows {
                        let c = vert_cycle[line_idx % vert_cycle.len()];
                        if c.is_whitespace() {
                            rows.push(
                                div()
                                    .flex_none()
                                    .w_full()
                                    .h(px(gap_px))
                                    .into_any_element(),
                            );
                        } else {
                            let mut glyph_row = div()
                                .flex_none()
                                .w_full()
                                .h(px(dash_px))
                                .overflow_hidden()
                                .flex()
                                .flex_row()
                                .items_start()
                                .child(c.to_string());
                            glyph_row = if hug_inner {
                                glyph_row.justify_end().relative().left(px(1.))
                            } else {
                                glyph_row.justify_start()
                            };
                            rows.push(glyph_row.into_any_element());
                        }
                    }
                }

                div()
                    .flex()
                    .flex_col()
                    .justify_start()
                    .text_color(bc)
                    .font_family(font_name.clone())
                    .text_size(rems(font_rems))
                    .line_height(rems(font_rems))
                    .children(rows)
                    .into_any_element()
            } else {
                div().child(side_column.clone()).into_any_element()
            };

            let mut layer = div()
                .absolute()
                .top_0()
                .bottom_0()
                .left_0()
                .right_0()
                .overflow_hidden()
                .flex()
                .flex_col()
                .justify_start();
            if !use_structured_vertical {
                layer = layer
                    .text_color(bc)
                    .font_family(font_name.clone())
                    .text_size(rems(font_rems))
                    .line_height(rems(font_rems));
            }
            let layer = if hug_inner {
                layer.items_end()
            } else {
                layer.items_start()
            };
            div()
                .flex_none()
                .w(px(rail_px))
                .relative()
                .child(layer.child(inner))
        };

        div()
            .w_full()
            .flex_none()
            .flex()
            .flex_col()
            .bg(bg)
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .gap_0()
                    .items_baseline()
                    .child(corner_cell(ch[0], CornerSlot::Tl))
                    .child(horiz_fill(ch[1]))
                    .child(corner_cell(ch[2], CornerSlot::Tr)),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .child(side_fill(true))
                    .child(div().flex_1().min_w_0().p_2().children(self.children))
                    .child(side_fill(false)),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .gap_0()
                    .items_baseline()
                    .child(corner_cell(ch[4], CornerSlot::Bl))
                    .child(horiz_fill(ch[5]))
                    .child(corner_cell(ch[6], CornerSlot::Br)),
            )
    }
}
