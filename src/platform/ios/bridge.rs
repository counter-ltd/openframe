//! Public bridge API for embedding OpenFrame in a UIKit host.
//!
//! The Swift/Obj-C host calls the Rust `extern "C"` functions defined in the
//! embedding crate (e.g. `arcadia_ios_start`). Those functions in turn call
//! into this module to wire up the `CAMetalLayer` and forward `UITouch` events.
//!
//! Usage flow:
//! 1. `IosPlatform::open_window` stores a clone of the `IosWindow` here via
//!    `ios_store_active_window`.
//! 2. The embedding crate calls `ios_set_metal_layer_ptr` after its `run`
//!    callback fires to attach the host `CAMetalLayer`.
//! 3. On every `UITouch*` callback the embedding crate calls `ios_inject_touch`.

use super::IosWindow;
use crate::interactive::{MouseDownEvent, MouseMoveEvent, MouseUpEvent, PlatformInput};
use crate::geometry::{Pixels, Point};
use crate::MouseButton;
use std::cell::RefCell;

thread_local! {
    static ACTIVE_WINDOW: RefCell<Option<IosWindow>> = RefCell::new(None);
}

/// Called by `IosPlatform::open_window` to register the freshly-created window.
pub(crate) fn ios_store_active_window(win: IosWindow) {
    ACTIVE_WINDOW.with(|w| *w.borrow_mut() = Some(win));
}

/// Set the `CAMetalLayer` pointer on the active OpenFrame iOS window.
/// Call this immediately after the `Application::run` callback fires.
pub fn ios_set_metal_layer_ptr(ptr: usize) {
    ACTIVE_WINDOW.with(|w| {
        if let Some(win) = w.borrow().as_ref() {
            win.set_host_metal_layer_ptr(ptr);
        }
    });
}

/// Inject a UIKit touch event into the OpenFrame event loop.
/// `phase`: 0 = began, 1 = moved, 2 = ended, 3 = cancelled.
pub fn ios_inject_touch(x: f32, y: f32, phase: u8) {
    let position = Point::new(Pixels(x), Pixels(y));
    let event = match phase {
        0 => PlatformInput::MouseDown(MouseDownEvent {
            button: MouseButton::Left,
            position,
            modifiers: Default::default(),
            click_count: 1,
            first_mouse: false,
        }),
        1 => PlatformInput::MouseMove(MouseMoveEvent {
            position,
            pressed_button: Some(MouseButton::Left),
            modifiers: Default::default(),
        }),
        _ => PlatformInput::MouseUp(MouseUpEvent {
            button: MouseButton::Left,
            position,
            modifiers: Default::default(),
            click_count: 1,
        }),
    };
    ACTIVE_WINDOW.with(|w| {
        if let Some(win) = w.borrow().as_ref() {
            win.inject_platform_input(event);
        }
    });
}
