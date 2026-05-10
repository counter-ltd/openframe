use crate::Global;

/// The active render style for the application window.
///
/// Set via `cx.set_global(RenderStyle::Custom)` when a Python-registered glyph style is
/// active, or `RenderStyle::Default` to return to the standard appearance.
/// Views read it via `cx.try_global::<RenderStyle>()`.
#[derive(Clone, PartialEq, Default, Debug)]
pub enum RenderStyle {
    /// Standard GPU-rendered appearance (default).
    #[default]
    Default,
    /// A Python-registered glyph style is active. Read the Desktop-level
    /// `ActiveGlyphStyle` global for full rendering parameters.
    Custom,
}

impl Global for RenderStyle {}
