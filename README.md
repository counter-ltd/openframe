# OpenFrame

OpenFrame is Arcadia's in-tree adaptation of [Zed's GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui), forked from the published [`gpui` 0.2.2](https://crates.io/crates/gpui) crate.

## Adaptations

This fork extends GPUI with capabilities required for Arcadia's cross-platform UI needs:

### iOS Platform Support
Full UIKit backend under `src/platform/ios/` enabling OpenFrame to run on iOS devices:
- Grand Central Dispatch executor integration
- UIKit pasteboard bridging
- Metal presentation via host `CAMetalLayer` wiring
- Touch-to-mouse event translation for `PlatformInput`
- Non-blocking launch model (schedules on main dispatch queue vs macOS `NSApplication` run loop)

### Style Context System
Runtime style resolution with `StyleContext` for dynamic theming:
- Hierarchical style inheritance
- Reactive style updates across element trees

### Additional Elements
- **ColorPicker**: HSV-based color selection widget
- **GlyphBorder**: Decorative border element with icon/glyph integration

## Usage

```toml
openframe = { path = "../Libraries/OpenFrame" }
```

### iOS Cross-Compilation

```sh
rustup target add aarch64-apple-ios
rustup target add aarch64-apple-ios-sim
cargo check --target aarch64-apple-ios
```

See `ORIGINAL_README.md` for upstream GPUI documentation and `ORIGINAL_LICENSE-APACHE.md` for license terms.
