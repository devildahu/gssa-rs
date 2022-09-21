//! Embedded game asset definitions.

use core::ops::Range;

pub(crate) const HARDCODED_TILEMAP_WIDTH: u16 = 32;

// TODO: type-safe [`Tileset`] to make it impossible to missuse
// with regard to Color4bit and Color8bit.
// TODO: probably requires distinguishing "dynamic" images from
// fixed position images.
/// An image in a tileset.
///
/// It can be drawn and stuff, while [`Tileset`] is the raw data to load in VRAM.
pub(crate) struct Image {
    /// The **tileset**'s width.
    pub(crate) tileset_width: u16,
    pub(crate) offset: u16,
    pub(crate) width: usize,
    pub(crate) height: usize,
}
/// Define an [`Image`].
///
/// An [`Image`] is not the raw bytes of sprite, it is the offset
/// and position in the tile buffer of a specific image.
#[macro_export]
macro_rules! image {
    ($file:literal) => {
        Image {
            data: include_bytes!(concat!("../resources/", $file)),
        }
    };
}

/// A palette cycle.
///
/// This control palette cycling, for nice graphical effects.
pub(crate) struct Cycle {
    range: Range<usize>,
    frames_per_step: usize,
}
impl Cycle {
    pub(crate) const fn new(range: Range<usize>, frames_per_step: usize) -> Self {
        Self {
            range,
            frames_per_step,
        }
    }
}

/// A color palette, may be cycling.
pub(crate) struct Palette {
    data: &'static [u8],
    #[allow(unused)] // TODO
    cycles: &'static [Cycle],
}
impl Palette {
    // TODO: leaky abstraction, this should only be accessible in
    // video_control.rs
    pub(crate) const fn get(&self) -> &'static [u8] {
        self.data
    }
}

/// A set of tiles for text mode.
///
/// This is the raw data, not the tiles as represented by [`Image`].
pub(crate) struct Tileset {
    data: &'static [u8],
}
impl Tileset {
    // TODO: leaky abstraction, this should only be accessible in
    // video_control.rs
    pub(crate) const fn get(&self) -> &'static [u8] {
        self.data
    }
}

/// Define a [`Palette`].
///
/// Directly pass the file name, prefixes the path to the resources
/// directory.
///
/// You may define palette cycles by adding arguments in the
/// following form:
/// ```text
/// cycle($range: Range<usize>, $frames_per_step: usize)
/// ```
/// Accepts multiple `cycle` arguments to define multiple cycling zones.
#[macro_export]
macro_rules! palette {
    ($file:literal) => {
        Palette {
            data: include_bytes!(concat!("../resources/", $file)),
            cycles: &[],
        }
    };
    ($file:literal, $( cycle ($range:expr, $rate:expr) ),+ $(,)?) => {
        Palette {
            data: include_bytes!(concat!("../resources/", $file)),
            cycles: &[ $( Cycle::new($range, $rate), )+ ],
        }
    };
}

/// Define a [`Tileset`].
///
/// Directly pass the file name, prefixes the path to the resources
/// directory.
#[macro_export]
macro_rules! tileset {
    ($file:literal) => {
        Tileset {
            data: include_bytes!(concat!("../resources/", $file)),
        }
    };
}

/// Asset definitions of the main game graphical elements.
#[allow(non_upper_case_globals)]
pub(crate) mod space {
    use super::*;

    // TODO: consider using enum-map instead.
    /// The list of all possible enemy ships.
    pub(crate) struct Ships<T> {
        pub(crate) small_blue: T,
        pub(crate) small_green1: T,
        pub(crate) small_green2: T,
        pub(crate) small_green3: T,
        pub(crate) small_green4: T,
        pub(crate) medium_blue1: T,
        pub(crate) medium_blue2: T,
        pub(crate) medium_green: T,
        pub(crate) medium_violet1: T,
        pub(crate) medium_violet2: T,
        pub(crate) long_green1: T,
        pub(crate) long_green2: T,
        pub(crate) big_blue: T,
        pub(crate) big_green: T,
        pub(crate) big_violet: T,
    }
    /// Enemy ship tile size specification.
    pub(crate) struct ShipTile {
        width: usize,
        height: usize,
    }
    impl ShipTile {
        const fn new(width: usize, height: usize) -> Self {
            Self { width, height }
        }
        const BIG: Self = Self::new(4, 4);
        const MED: Self = Self::new(2, 2);
        const LONG: Self = Self::new(2, 1);
        const SMALL: Self = Self::new(1, 1);
    }
    /// The bullet tiles, includes player and enemy bullets.
    pub(crate) const bullets: Tileset = tileset!("bulls_til.bin");
    /// The space background tiles.
    pub(crate) const background: Tileset = tileset!("gamesetbg_til.bin");
    /// The palette for the space background.
    pub(crate) const background_pal: Palette = palette!(
        "gamesetbg_pal.bin",
        // Cycle star colors for shimmering effect
        cycle(16..16 + 4, 32),
    );
    /// Palette for space foreground items such a ships and bullets.
    pub(crate) const objects_pal: Palette = palette!(
        "gamesetfg_pal.bin",
        // Enemy bullets
        cycle(32..32 + 8, 8),
        // Player bullets
        cycle(26..26 + 6, 8),
    );
    /// The tiles of all possible enemy ships.
    pub(crate) const ships: Ships<Tileset> = Ships {
        small_blue: tileset!("bigB1_til.bin"),
        small_green1: tileset!("smallG1_til.bin"),
        small_green2: tileset!("smallG2_til.bin"),
        small_green3: tileset!("smallG3_til.bin"),
        small_green4: tileset!("smallG4_til.bin"),
        medium_blue1: tileset!("medB1_til.bin"),
        medium_blue2: tileset!("medB2_til.bin"),
        medium_green: tileset!("medG1_til.bin"),
        medium_violet1: tileset!("medV1_til.bin"),
        medium_violet2: tileset!("medV2_til.bin"),
        long_green1: tileset!("smlongG1_til.bin"),
        long_green2: tileset!("smlongG2_til.bin"),
        big_blue: tileset!("bigB1_til.bin"),
        big_green: tileset!("bigG1_til.bin"),
        big_violet: tileset!("bigV1_til.bin"),
    };
    /// The sizes of all possible enemy ships.
    pub(crate) const ship_images: Ships<ShipTile> = Ships {
        small_blue: ShipTile::SMALL,
        small_green1: ShipTile::SMALL,
        small_green2: ShipTile::SMALL,
        small_green3: ShipTile::SMALL,
        small_green4: ShipTile::SMALL,
        medium_blue1: ShipTile::MED,
        medium_blue2: ShipTile::MED,
        medium_green: ShipTile::MED,
        medium_violet1: ShipTile::MED,
        medium_violet2: ShipTile::MED,
        long_green1: ShipTile::LONG,
        long_green2: ShipTile::LONG,
        big_blue: ShipTile::BIG,
        big_green: ShipTile::BIG,
        big_violet: ShipTile::BIG,
    };
    // TODO: all the space tileset individual images
    // This is probably worth writting a custom editor for.
    // (I could define them individually like the ships and
    // use the allocator to lay them into video memory,
    // actually that's a great idea, since I then just need
    // to use a 1D allocator, which code I already written)
}

/// Asset definitions of playable ships.
#[allow(non_upper_case_globals)]
pub(crate) mod players {
    use super::*;

    /// Tile and palette of playable ship.
    pub(crate) struct Ship {
        pub(crate) set: Tileset,
        pub(crate) pal: Palette,
    }
    /// The various palettes of player bullets, changing colors
    /// according to player level.
    pub(crate) struct BulletsPalette(
        pub(crate) Palette,
        pub(crate) Palette,
        pub(crate) Palette,
        pub(crate) Palette,
    );

    pub(crate) const paladin: Ship = Ship {
        set: tileset!("paladin_til.bin"),
        pal: palette!("paladin_pal.bin"),
    };
    pub(crate) const spear: Ship = Ship {
        set: tileset!("spear_til.bin"),
        pal: palette!("spear_pal.bin"),
    };
    pub(crate) const blank: Ship = Ship {
        set: tileset!("blank_til.bin"),
        pal: palette!("blank_pal.bin"),
    };
    pub(crate) const bullet_pal: BulletsPalette = BulletsPalette(
        palette!("pbull_lv0_pal.bin"),
        palette!("pbull_lv1_pal.bin"),
        palette!("pbull_lv2_pal.bin"),
        palette!("pbull_lv3_pal.bin"),
    );
}

/// Asset definitions of the main menu.
#[allow(non_upper_case_globals)]
pub(crate) mod menu {
    use super::*;

    pub(crate) const set: Tileset = tileset!("menuset_til.bin");
    pub(crate) const palette: Palette = palette!("menuset_pal.bin");
    // TODO: all the main menu tileset individual images
    pub(crate) const title_card: Image = Image {
        tileset_width: HARDCODED_TILEMAP_WIDTH,
        offset: 96,
        width: 17,
        height: 9,
    };
}
