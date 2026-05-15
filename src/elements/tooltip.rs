//! Reusable tooltip view and builder helpers.
//!
//! Use [`tooltip`] for the common single-text case:
//! ```rust,ignore
//! button.tooltip(tooltip("Click to submit"))
//! ```
//!
//! For richer content build a [`Tooltip`] manually:
//! ```rust,ignore
//! .tooltip(|_w, cx| Tooltip::build(cx, |t| t.title("Warning").item("missing: foo").item("missing: bar")))
//! ```

use std::sync::Arc;

use crate::{
    AnyElement, AnyView, App, AppContext as _, Context, FontWeight, IntoElement, ParentElement,
    Render, SharedString, Styled, Window, div, px, rgb, rgba,
};

// ---------------------------------------------------------------------------
// Default visual constants — neutral dark box works on both light/dark hosts.
// ---------------------------------------------------------------------------

const BG: u32 = 0x1a1f2a;
const BG_ALPHA: u32 = 0xee;
const BORDER: u32 = 0x38404e;
const BORDER_ALPHA: u32 = 0xcc;
const TITLE_TEXT: u32 = 0xf1f5f9;
const BODY_TEXT: u32 = 0xb0b8c8;

// ---------------------------------------------------------------------------
// Tooltip view
// ---------------------------------------------------------------------------

/// A styled tooltip container. Create with [`Tooltip::build`] or
/// convert into a closure via [`Tooltip::into_fn`] for use with
/// [`StatefulInteractiveElement::tooltip`].
pub struct Tooltip {
    title: Option<SharedString>,
    items: Vec<SharedString>,
    extra: Vec<AnyElement>,
}

impl Default for Tooltip {
    fn default() -> Self {
        Self::new()
    }
}

impl Tooltip {
    /// Create an empty tooltip. Add content with [`title`](Self::title), [`item`](Self::item), or [`child`](Self::child).
    pub fn new() -> Self {
        Self {
            title: None,
            items: Vec::new(),
            extra: Vec::new(),
        }
    }

    /// Bold header line.
    pub fn title(mut self, text: impl Into<SharedString>) -> Self {
        self.title = Some(text.into());
        self
    }

    /// Bullet-prefixed body line.
    pub fn item(mut self, text: impl Into<SharedString>) -> Self {
        self.items.push(text.into());
        self
    }

    /// Arbitrary element appended after items.
    pub fn child(mut self, el: impl IntoElement) -> Self {
        self.extra.push(el.into_any_element());
        self
    }

    // -----------------------------------------------------------------------
    // Convenience constructors that return the closure expected by `.tooltip()`
    // -----------------------------------------------------------------------

    /// Build a tooltip and return the `AnyView` used in a tooltip closure.
    /// Convenience for calling `.tooltip(|_w, cx| Tooltip::build(cx, |t| t.title(...)))`.
    pub fn build(
        cx: &mut App,
        f: impl FnOnce(Tooltip) -> Tooltip,
    ) -> AnyView {
        cx.new(|_| f(Tooltip::new())).into()
    }

    /// Consume self and return the closure needed by [`StatefulInteractiveElement::tooltip`].
    /// Prefer [`tooltip`] for the single-string case.
    pub fn into_fn(self) -> impl Fn(&mut Window, &mut App) -> AnyView {
        let shared = Arc::new(std::sync::Mutex::new(Some(self)));
        move |_window, cx| {
            let tooltip = shared
                .lock()
                .unwrap()
                .take()
                .unwrap_or_default();
            cx.new(|_| tooltip).into()
        }
    }
}

impl Render for Tooltip {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let bg = rgba((BG << 8) | BG_ALPHA);
        let border = rgba((BORDER << 8) | BORDER_ALPHA);
        let title_col = rgb(TITLE_TEXT);
        let body_col = rgb(BODY_TEXT);

        let mut d = div()
            .px_2()
            .py_1()
            .rounded(px(5.))
            .bg(bg)
            .border_1()
            .border_color(border)
            .flex()
            .flex_col()
            .gap_y_1()
            .text_sm();

        if let Some(title) = self.title.take() {
            d = d.child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(title_col)
                    .child(title),
            );
        }

        for item in std::mem::take(&mut self.items) {
            d = d.child(
                div()
                    .text_color(body_col)
                    .child(format!("• {item}")),
            );
        }

        for el in std::mem::take(&mut self.extra) {
            d = d.child(el);
        }

        d
    }
}

// ---------------------------------------------------------------------------
// Free-function helpers
// ---------------------------------------------------------------------------

/// Returns a tooltip closure for a single text string — the common case.
///
/// ```rust,ignore
/// button.tooltip(tooltip("Save file"))
/// ```
pub fn tooltip(
    text: impl Into<SharedString>,
) -> impl Fn(&mut Window, &mut App) -> AnyView {
    let text: SharedString = text.into();
    move |_window, cx| {
        let t = text.clone();
        cx.new(|_| Tooltip::new().title(t)).into()
    }
}
