//! Manage color palettes

/// A palette added to VRAM.
///
/// Note that there is no dynamic allocations, and all usages of
/// `Palette` will be limited to `TextControl`.
pub(crate) struct Palette(pub(super) u16);
