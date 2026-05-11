//! Accessibility metadata for assistive technologies (VoiceOver on iOS).

use crate::SharedString;

/// Properties attached to a [`crate::Hitbox`] for platform accessibility snapshots.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessibilityProperties {
    /// Primary accessibility label.
    pub label: SharedString,
    /// Optional hint describing interaction outcome.
    pub hint: Option<SharedString>,
    /// Raw traits bitmask; matches UIKit `UIAccessibilityTraits` when bridged to iOS.
    pub traits: u64,
}

/// Bit compatible with `UIAccessibilityTraitButton`.
pub const ACCESSIBILITY_TRAIT_BUTTON: u64 = 1;
