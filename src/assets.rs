//! Embedded game asset definitions.

use gbassets::{image, palette, Cycle, Image, Palette};
use hal::video::{colmod, object, object::sprite, Tileset};
use hal::{sprite, sprite_sheet, tileset};

/// Asset definitions of the main game graphical elements.
#[allow(non_upper_case_globals, clippy::wildcard_imports)]
pub(crate) mod space {
    use super::*;

    pub(crate) const star_count: u32 = 16;

    /// The in-game menu tiles.
    pub(crate) const ui: Tileset<colmod::Bit8> = tileset!("gamesetui_til.bin");
    pub(crate) mod bullets {
        use super::*;

        /// Available Bullet sprites.
        #[repr(u16)]
        pub(crate) enum Bullets {
            Circle,
            Cross,
            Dash,
            Dot,
            Plus,
            FatDot,
            Diamond,
            Squiggle,
            I,
            PlayerDash,
            PlayerDot,
            PlayerLine,
            PlayerParticles,
            Egg,
        }
        /// The bullet tiles, includes player and enemy bullets.
        pub(crate) const tiles: sprite::Sheet<14> = sprite_sheet!("bulls_til.bin");

        /// The various palettes of player bullets, changing colors
        /// according to player level.
        pub(crate) struct BulletsPalette(
            pub(crate) Palette,
            pub(crate) Palette,
            pub(crate) Palette,
            pub(crate) Palette,
        );
        pub(crate) const bullet_pal: BulletsPalette = BulletsPalette(
            palette!("pbull_lv0_pal.bin"),
            palette!("pbull_lv1_pal.bin"),
            palette!("pbull_lv2_pal.bin"),
            palette!("pbull_lv3_pal.bin"),
        );
    }
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

    #[repr(u8)]
    pub(crate) enum Ships {
        SmallBlue,
        SmallGreen1,
        SmallGreen2,
        SmallGreen3,
        SmallGreen4,
        MediumBlue1,
        MediumBlue2,
        MediumGreen,
        MediumViolet1,
        MediumViolet2,
        LongGreen1,
        LongGreen2,
        BigBlue,
        BigGreen,
        BigViolet,
    }

    impl Ships {
        pub(crate) const fn try_from_u8(value: u8) -> Option<Self> {
            match value {
                0 => Some(Self::SmallBlue),
                1 => Some(Self::SmallGreen1),
                2 => Some(Self::SmallGreen2),
                3 => Some(Self::SmallGreen3),
                4 => Some(Self::SmallGreen4),
                5 => Some(Self::MediumBlue1),
                6 => Some(Self::MediumBlue2),
                7 => Some(Self::MediumGreen),
                8 => Some(Self::MediumViolet1),
                9 => Some(Self::MediumViolet2),
                10 => Some(Self::LongGreen1),
                11 => Some(Self::LongGreen2),
                12 => Some(Self::BigBlue),
                13 => Some(Self::BigGreen),
                14 => Some(Self::BigViolet),
                _ => None,
            }
        }
        pub(crate) const fn sprite(self) -> object::Sprite {
            match self {
                Self::SmallBlue => ships::small_blue,
                Self::SmallGreen1 => ships::small_green1,
                Self::SmallGreen2 => ships::small_green2,
                Self::SmallGreen3 => ships::small_green3,
                Self::SmallGreen4 => ships::small_green4,
                Self::MediumBlue1 => ships::medium_blue1,
                Self::MediumBlue2 => ships::medium_blue2,
                Self::MediumGreen => ships::medium_green,
                Self::MediumViolet1 => ships::medium_violet1,
                Self::MediumViolet2 => ships::medium_violet2,
                Self::LongGreen1 => ships::long_green1,
                Self::LongGreen2 => ships::long_green2,
                Self::BigBlue => ships::big_blue,
                Self::BigGreen => ships::big_green,
                Self::BigViolet => ships::big_violet,
            }
        }
    }
    /// The sprites of all possible enemy ships.
    pub(crate) mod ships {
        use super::*;
        use object::Shape::{_1x1, _2x1, _2x2, _4x4};
        use object::Sprite;
        pub(crate) const small_blue: Sprite = sprite!("bigB1_til.bin", _1x1);
        pub(crate) const small_green1: Sprite = sprite!("smallG1_til.bin", _1x1);
        pub(crate) const small_green2: Sprite = sprite!("smallG2_til.bin", _1x1);
        pub(crate) const small_green3: Sprite = sprite!("smallG3_til.bin", _1x1);
        pub(crate) const small_green4: Sprite = sprite!("smallG4_til.bin", _1x1);
        pub(crate) const medium_blue1: Sprite = sprite!("medB1_til.bin", _2x2);
        pub(crate) const medium_blue2: Sprite = sprite!("medB2_til.bin", _2x2);
        pub(crate) const medium_green: Sprite = sprite!("medG1_til.bin", _2x2);
        pub(crate) const medium_violet1: Sprite = sprite!("medV1_til.bin", _2x2);
        pub(crate) const medium_violet2: Sprite = sprite!("medV2_til.bin", _2x2);
        pub(crate) const long_green1: Sprite = sprite!("smlongG1_til.bin", _2x1);
        pub(crate) const long_green2: Sprite = sprite!("smlongG2_til.bin", _2x1);
        pub(crate) const big_blue: Sprite = sprite!("bigB1_til.bin", _4x4);
        pub(crate) const big_green: Sprite = sprite!("bigG1_til.bin", _4x4);
        pub(crate) const big_violet: Sprite = sprite!("bigV1_til.bin", _4x4);
    }
    // TODO: all the space tileset individual images
    // This is probably worth writting a custom editor for.
    // (I could define them individually like the ships and
    // use the allocator to lay them into video memory,
    // actually that's a great idea, since I then just need
    // to use a 1D allocator, which code I already written)
    pub(crate) const big_planet_offset: u16 = background_width * 3;
    pub(crate) const background_width: u16 = 32;
    pub(crate) const big_planet_size: u16 = 4;
}

/// Asset definitions of playable ships.
#[allow(non_upper_case_globals, clippy::wildcard_imports)]
pub(crate) mod players {
    use super::*;

    /// Tile and palette of playable ship.
    pub(crate) struct Ship {
        pub(crate) sprite: object::Sprite,
        pub(crate) pal: Palette,
    }
    pub(crate) const paladin: Ship = Ship {
        sprite: sprite!("paladin_til.bin", object::Shape::_2x2),
        pal: palette!("paladin_pal.bin"),
    };
    pub(crate) const spear: Ship = Ship {
        sprite: sprite!("spear_til.bin", object::Shape::_2x2),
        pal: palette!("spear_pal.bin"),
    };
    pub(crate) const blank: Ship = Ship {
        sprite: sprite!("blank_til.bin", object::Shape::_2x2),
        pal: palette!("blank_pal.bin"),
    };
}

/// Asset definitions of the main menu.
#[allow(non_upper_case_globals, clippy::wildcard_imports)]
pub(crate) mod menu {
    use super::*;

    pub(crate) mod player_ships {
        use super::*;

        const fn ship_offset(index: u16) -> u16 {
            title_card_offset + title_card_width + index * 4
        }
        pub(crate) const blank: Image = image!(ship_offset(0), 3, 3, 32);
        pub(crate) const spear: Image = image!(ship_offset(1), 3, 3, 32);
        pub(crate) const paladin: Image = image!(ship_offset(2), 3, 3, 32);
    }

    const title_card_offset: u16 = 96;
    const title_card_width: u16 = 17;

    pub(crate) const set: Tileset<colmod::Bit8> = tileset!("menuset_til.bin");
    pub(crate) const palette: Palette = palette!("menuset_pal.bin");
    // TODO: all the main menu tileset individual images
    pub(crate) const title_card: Image = image!(title_card_offset, title_card_width, 9, 32);
}
