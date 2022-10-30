// TODO: rename this to "player.rs"
use hal::{
    exec::ConsoleState,
    video::{self, mode, object, palette, Pos},
    Input,
};

use crate::{assets::players, game::Posi};

const INITIAL_PLAYER_POS: Pos = Pos { x: 4, y: 52 };

crate::cycling_enum! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub(crate) enum Ship {
        Blank,
        Spear,
        Paladin,
    }
}
impl Ship {
    pub(crate) const fn asset(self) -> players::Ship {
        match self {
            Self::Blank => players::blank,
            Self::Spear => players::spear,
            Self::Paladin => players::paladin,
        }
    }
}

#[derive(Copy, Clone)]
enum HitPoints {
    _0,
    _1,
    _2,
    _3,
}

#[derive(Copy, Clone)]
pub(super) enum Weapon {
    Standard,
    Double,
    Momentum,
    Charge,
}
pub(super) struct Player {
    pub(super) pos: Pos,
    /// Last time player fired (for cooldown).
    last_fire_frame: usize,
    /// Last time the player was hit (for iframes).
    last_hit_frame: usize,
    pub(super) weapon: Weapon,
    life: HitPoints,
    slot: object::Slot,
}
impl Player {
    pub(super) const fn strength(&self) -> u8 {
        match self.life {
            HitPoints::_0 => 1,
            HitPoints::_1 => 2,
            HitPoints::_2 => 3,
            HitPoints::_3 => 4,
        }
    }
    pub(super) const fn new(slot: object::Slot, ship: Ship) -> Self {
        let (weapon, life) = match ship {
            Ship::Blank => (Weapon::Momentum, HitPoints::_1),
            Ship::Spear => (Weapon::Double, HitPoints::_3),
            Ship::Paladin => (Weapon::Standard, HitPoints::_2),
        };
        Self {
            pos: INITIAL_PLAYER_POS,
            last_fire_frame: 0,
            last_hit_frame: 0,
            weapon,
            life,
            slot,
        }
    }
    pub(crate) fn update(&mut self, input: Input) {
        let move_dir: Posi = input.current().into();
        self.pos = self.pos + move_dir;
    }

    pub(crate) fn draw(&self, ctrl: &mut video::Control<mode::Affine>) {
        let mut player = ctrl.object(&self.slot);
        player.set_pos(self.pos);
    }
    pub(super) fn init_video(
        &self,
        ctrl: &mut video::Control<mode::Affine>,
        console: &mut ConsoleState,
        ship: &players::Ship,
    ) {
        ctrl.load_object_palette(0, ship.pal.get());
        #[allow(clippy::option_if_let_else)]
        match ctrl.load_sprite(console, &ship.sprite) {
            None => {
                hal::error!("Couldn't load the player sprite");
            }
            Some(sprite) => {
                let mut player = ctrl.object(&self.slot);
                player.set_sprite(sprite);
                player.set_shape(*ship.sprite.shape());
                player.set_palette_mode(palette::Type::Full);
                player.set_visible(true);
                player.set_pos(self.pos);
            }
        }
    }
}
