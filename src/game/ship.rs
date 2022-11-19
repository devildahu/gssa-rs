// TODO: rename this to "player.rs"
use hal::{
    exec::ConsoleState,
    input::Key,
    video::{self, mode, object, palette},
};

use crate::collide::{Collide, Shape};
use crate::game::space::Bullet;
use crate::{assets::players, game::Posi};

use super::space::items;

const INITIAL_PLAYER_POS: Posi = Posi::new(4, 52);

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

#[derive(Copy, Clone, Debug)]
enum HitPoints {
    _0,
    _1,
    _2,
    _3,
}
impl HitPoints {
    fn increment(&mut self) {
        *self = match self {
            Self::_0 => Self::_1,
            Self::_1 => Self::_2,
            Self::_2 | Self::_3 => Self::_3,
        };
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum Weapon {
    Standard,
    Double,
    Momentum,
    Charge,
}
impl Weapon {
    const fn cooldown(self) -> usize {
        match self {
            Self::Standard => 32,
            Self::Double => 45,
            Self::Momentum => 20,
            Self::Charge => 50,
        }
    }
}
/// From where is the player shooting.
#[derive(Clone, Copy, Debug)]
pub(super) enum FirePosition {
    Top,
    Bottom,
}
impl FirePosition {
    pub(super) fn flip(&mut self) -> i32 {
        let out = match self {
            Self::Top => 2,
            Self::Bottom => 8,
        };
        let new_value = match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
        };
        *self = new_value;
        out
    }
}
#[derive(Debug)]
pub(crate) struct Player {
    pub(super) pos: Posi,
    /// Last time player fired (for cooldown).
    next_fire_frame: usize,
    /// Last time the player was hit (for iframes).
    next_hit_frame: usize,
    pub(super) weapon: Weapon,
    life: HitPoints,
    pub(super) last_fire: FirePosition,
    slot: object::Slot,
}
impl Collide for Player {
    fn shape(&self) -> Shape {
        Shape::Rectangle { size: Posi::new(16, 16) }
    }
    fn pos(&self) -> Posi {
        self.pos - Posi::new(8, 8)
    }
}
impl Player {
    pub(super) fn pick_up(&mut self, item: items::Kind) {
        match item {
            items::Kind::Weapon(weapon) => self.weapon = weapon,
            items::Kind::LifeUp => self.life.increment(),
        }
    }

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
            next_fire_frame: 0,
            next_hit_frame: 0,
            last_fire: FirePosition::Bottom,
            weapon,
            life,
            slot,
        }
    }
    pub(crate) fn update(&mut self, console: &mut ConsoleState) -> Option<Bullet> {
        let input = console.input;
        let frame = console.frame;
        let momentum: Posi = input.current().into();
        self.pos += momentum;
        if input.pressed(Key::A) && self.next_fire_frame < frame {
            let slot = console.reserve_object()?;
            self.next_fire_frame = frame + self.weapon.cooldown();

            Some(Bullet::new_from_player(self, frame, momentum, slot))
        } else {
            None
        }
    }

    pub(crate) fn draw(&self, ctrl: &mut video::Control<mode::Affine>) {
        let mut player = ctrl.object(&self.slot);
        if let Ok(pos) = self.pos.try_into() {
            player.set_pos(pos);
        }
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
                if let Ok(pos) = self.pos.try_into() {
                    player.set_pos(pos);
                }
            }
        }
    }
}
