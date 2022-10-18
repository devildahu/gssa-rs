use hal::video::{object, Pos};

const INITIAL_PLAYER_POS: Pos = Pos { x: 4, y: 52 };

crate::cycling_enum! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub(crate) enum Ship {
        Blank,
        Spear,
        Paladin,
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
    slot: object::Slot,
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
}
