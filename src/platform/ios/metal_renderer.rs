//! Bytecode for the path/scene pipeline, produced by `build.rs` as `OUT_DIR/shaders.metallib`.

#[cfg(not(feature = "runtime_shaders"))]
pub(crate) const SHADERS_METALLIB: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/shaders.metallib"));

#[cfg(feature = "runtime_shaders")]
pub(crate) const SHADERS_METALLIB: &[u8] = &[];
