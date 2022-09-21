pub(crate) mod draw;
pub(crate) mod layout;
mod palette;

pub(crate) use draw::Draw;
pub(crate) use palette::Palette;

use gba::mmio_types::TextTile;

/// A tile for [`crate::video_control::SbbHandle::set_tile`].
#[derive(Clone, Copy)]
pub(crate) struct Tile(TextTile);
impl Tile {
    pub(crate) const fn new(tile_id: u16) -> Self {
        Self(TextTile::from_tile_id(tile_id))
    }
    pub(crate) const fn flip_hori(self) -> Self {
        Self(self.0.with_hflip(!self.0.hflip()))
    }
    pub(crate) const fn flip_vert(self) -> Self {
        Self(self.0.with_vflip(!self.0.vflip()))
    }
    /// In [`crate::video_control::Color4bit`] mode, each [`crate::assets::Tileset`]
    /// has only 16 colors, but the palette for each tile can be
    /// specified in the `Tile` data.
    ///
    /// This has no effect if the color mode of the background is `Color8bit`.
    pub(crate) const fn with_palette(self, palette: Palette) -> Self {
        Self(self.0.with_palbank(palette.0))
    }
    // TODO: leaky abstraction, somehow SbbHandle should have access
    // to .0 while other modules do not.
    pub(crate) const fn get(self) -> TextTile {
        self.0
    }
}
