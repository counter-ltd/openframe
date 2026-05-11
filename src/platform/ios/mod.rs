//! UIKit + Metal backend for OpenFrame on iOS (`target_os = "ios"`).

pub(crate) type PlatformScreenCaptureFrame = ();

pub mod bridge;
pub mod accessibility;
mod clipboard;
mod dispatcher;
mod display;
mod keyboard;
mod metal_renderer;
mod platform;
mod window;

pub use accessibility::{snapshot_node_count, snapshot_query_node};
pub use bridge::{ios_inject_touch, ios_set_metal_layer_ptr};
pub(crate) use bridge::ios_store_active_window;
pub(crate) use dispatcher::*;
pub(crate) use display::*;
pub(crate) use keyboard::*;
pub(crate) use platform::*;
pub(crate) use window::*;
