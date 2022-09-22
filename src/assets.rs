//! Embedded game asset definitions.

use assets::{palette, Cycle, Image, Palette};
use hal::tileset;
use hal::video::{colmod, tile::map::HARDCODED_TILEMAP_WIDTH, Tileset};

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
    pub(crate) const bullets: Tileset<colmod::Bit8> = tileset!("bulls_til.bin");
    /// The space background tiles.
    pub(crate) const background: Tileset<colmod::Bit8> = tileset!("gamesetbg_til.bin");
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
    pub(crate) const ships: Ships<Tileset<colmod::Bit8>> = Ships {
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
        pub(crate) set: Tileset<colmod::Bit8>,
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

    pub(crate) const set: Tileset<colmod::Bit8> = tileset!("menuset_til.bin");
    pub(crate) const palette: Palette = palette!("menuset_pal.bin");
    // TODO: all the main menu tileset individual images
    pub(crate) const title_card: Image = Image {
        tileset_width: HARDCODED_TILEMAP_WIDTH,
        offset: 96,
        width: 17,
        height: 9,
    };
}
