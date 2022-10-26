use hal::{
    exec::ConsoleState,
    input::{Dir, Key},
    video::{self, mode, object, Pos},
    Input,
};

use crate::assets::players;

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
enum Weapon {
    Standard,
    Double,
    Momentum,
    Charge,
}
pub(super) struct Player {
    pos: Pos,
    /// Last time player fired (for cooldown).
    last_fire_frame: usize,
    /// Last time the player was hit (for iframes).
    last_hit_frame: usize,
    weapon: Weapon,
    life: HitPoints,
    pub(super) slot: object::Slot,
}
impl Player {
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
        use Dir::{Down, Left, Right, Up};
        let pressed_dir = |dir, value| if input.pressed(Key::Dpad(dir)) { value } else { 0 };
        self.pos.y += pressed_dir(Down, 1) - pressed_dir(Up, 1);
        self.pos.x += pressed_dir(Right, 1) - pressed_dir(Left, 1);
    }

    pub(crate) fn draw(&self, ctrl: &mut video::Control<mode::Affine>) {
        let mut player = ctrl.object(&self.slot);
        player.set_pos(self.pos);
    }
}
