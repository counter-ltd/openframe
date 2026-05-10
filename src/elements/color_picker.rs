//! Interactive color selection: saturation/value pad and a vertical hue ramp (opaque [`Rgba`]).
//!
//! Click either region to update the color. Wire [`crate::Context::listener`] or
//! [`crate::WeakEntity::update`] so your handler can mutate view state; this widget calls
//! [`crate::Context::notify`] after `on_change` so the UI refreshes.

use crate::{
    black, div, hsla, linear_color_stop, linear_gradient, px, transparent_white, white,
    App, Context, Div, Hitbox, InteractiveElement, MouseButton, MouseDownEvent, ParentElement,
    Rgba, Styled, WeakEntity, Window, hsv_to_rgb, rgb_to_hsv,
};

const SV_SIZE_PX: f32 = 160.;
const HUE_STRIP_W_PX: f32 = 22.;
const HUE_SEGMENTS: usize = 12;

fn hsl_from_degrees(h_deg: f32, s: f32, l: f32) -> crate::Hsla {
    hsla((h_deg / 360.).rem_euclid(1.), s.clamp(0., 1.), l.clamp(0., 1.), 1.)
}

/// Same as [`color_picker`] but takes a [`WeakEntity`] so callers can avoid borrowing [`Context`]
/// while building sibling subtrees (e.g. other `cx.listener` closures).
pub fn color_picker_with_weak<V: 'static>(
    weak: WeakEntity<V>,
    value: Rgba,
    on_change: impl Fn(&mut V, Rgba, &mut Context<V>) + Clone + 'static,
) -> Div {
    let pick_sv = {
        let on_change = on_change.clone();
        let weak = weak.clone();
        move |e: &MouseDownEvent, hitbox: &Hitbox, _window: &mut Window, app: &mut App| {
            let bx = &hitbox.bounds;
            let lx = f32::from(e.position.x) - f32::from(bx.origin.x);
            let ly = f32::from(e.position.y) - f32::from(bx.origin.y);
            let w = f32::from(bx.size.width).max(1.);
            let h = f32::from(bx.size.height).max(1.);
            let s = (lx / w).clamp(0., 1.);
            let v = (1. - (ly / h)).clamp(0., 1.);
            let (ch, _, _) = rgb_to_hsv(value);
            let new_c = hsv_to_rgb(ch, s, v);
            let _ = weak.update(app, |view, cx| {
                on_change(view, new_c, cx);
                cx.notify();
            });
        }
    };

    let pick_hue = {
        let on_change = on_change.clone();
        let weak = weak.clone();
        move |e: &MouseDownEvent, hitbox: &Hitbox, _window: &mut Window, app: &mut App| {
            let bx = &hitbox.bounds;
            let ly = f32::from(e.position.y) - f32::from(bx.origin.y);
            let h_px = f32::from(bx.size.height).max(1.);
            let hue_deg = (1. - (ly / h_px).clamp(0., 1.)) * 360.;
            let (_, cs, cv) = rgb_to_hsv(value);
            let new_c = hsv_to_rgb(hue_deg, cs, cv);
            let _ = weak.update(app, |view, cx| {
                on_change(view, new_c, cx);
                cx.notify();
            });
        }
    };

    let (ch, cs, cv) = rgb_to_hsv(value);
    let chroma_right = hsl_from_degrees(ch, 1., 0.5);
    let sv_gradient_h = linear_gradient(
        90.,
        linear_color_stop(white(), 0.),
        linear_color_stop(chroma_right, 1.),
    );
    let sv_gradient_v = linear_gradient(
        180.,
        linear_color_stop(transparent_white(), 0.),
        linear_color_stop(black(), 1.),
    );

    let dot_x = cs * SV_SIZE_PX - 5.;
    let dot_y = (1. - cv) * SV_SIZE_PX - 5.;

    div()
        .flex()
        .flex_row()
        .gap(px(8.))
        .child(
            div()
                .relative()
                .w(px(SV_SIZE_PX))
                .h(px(SV_SIZE_PX))
                .rounded(px(6.))
                .overflow_hidden()
                .border_1()
                .border_color(crate::rgba(0x00000080))
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .bg(sv_gradient_h),
                )
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .bg(sv_gradient_v),
                )
                .child(
                    div()
                        .absolute()
                        .top(px(dot_y))
                        .left(px(dot_x))
                        .w(px(10.))
                        .h(px(10.))
                        .rounded_full()
                        .border_2()
                        .border_color(white())
                        .shadow_md(),
                )
                .on_mouse_down_with_hitbox(MouseButton::Left, pick_sv),
        )
        .child({
            let hue_strip = div()
                .flex()
                .flex_col()
                .flex_none()
                .w(px(HUE_STRIP_W_PX))
                .h(px(SV_SIZE_PX))
                .rounded(px(6.))
                .overflow_hidden()
                .border_1()
                .border_color(crate::rgba(0x00000080))
                .children((0..HUE_SEGMENTS).map(|i| {
                    let hue = i as f32 * (360. / HUE_SEGMENTS as f32);
                    div()
                        .flex_1()
                        .w_full()
                        .bg(hsl_from_degrees(hue, 1., 0.5))
                }))
                .on_mouse_down_with_hitbox(MouseButton::Left, pick_hue);
            hue_strip
        })
}

/// Interactive HSV color picker (opaque RGB). Rebuild each frame with the current [`Rgba`]; clicks
/// invoke `on_change` with the new color (alpha is always `1`).
pub fn color_picker<V: 'static>(
    value: Rgba,
    cx: &mut Context<V>,
    on_change: impl Fn(&mut V, Rgba, &mut Context<V>) + Clone + 'static,
) -> Div {
    color_picker_with_weak(cx.weak_entity(), value, on_change)
}
