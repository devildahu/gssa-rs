use const_default::ConstDefault;
use hal::video::{self, mode, object, object::sprite, palette};

use crate::assets::space::bullets::Bullets;
use crate::collide::{Collide, Shape};
use crate::game::{ship::Player, ship::Weapon, Posi, SCREEN_AREA};

pub(crate) struct Bullet {
    kind: Kind,
    pos: Posi,
    damage: Damage,
    slot: object::Slot,
}
impl Collide for Bullet {
    fn shape(&self) -> Shape {
        Shape::Point
    }

    fn pos(&self) -> Posi {
        self.pos
    }
}
impl Bullet {
    pub(super) const fn into_slot(self) -> object::Slot {
        self.slot
    }
    // allow: because of `player.pos.into()` this can't be const
    #[allow(clippy::missing_const_for_fn)]
    pub(in crate::game) fn new_from_player(
        player: &mut Player,
        frame: usize,
        mut velocity: Posi,
        slot: object::Slot,
    ) -> Self {
        hal::info!("Player: {player:?} is spawning a bullet");
        velocity += Posi::x(1);
        let fix = velocity == Posi::DEFAULT;
        let velocity = if fix { Posi::x(1) } else { velocity };
        Self {
            slot,
            kind: match player.weapon {
                Weapon::Standard => Kind::Standard,
                Weapon::Double => Kind::Helix { spawn_time: frame },
                Weapon::Momentum => Kind::Momentum { velocity },
                Weapon::Charge => Kind::Charge { die_time: frame + 60 },
            },
            pos: player.pos + Posi::y(player.last_fire.flip()),
            damage: Damage(player.strength()),
        }
    }
    pub(super) fn update(&mut self, frame: usize) {
        match self.kind {
            Kind::Standard => self.pos.x += 1,
            Kind::Momentum { velocity } => self.pos += velocity,
            Kind::Charge { .. } => {}
            Kind::Helix { spawn_time } => {
                let offset = Posi::new(((frame - spawn_time) % 64) as i32 - 32, 1);
                self.pos += offset;
            }
        }
    }
    pub(super) fn should_die(&self, frame: usize) -> bool {
        if let Kind::Charge { die_time } = self.kind {
            if die_time < frame {
                return true;
            }
        }
        !SCREEN_AREA.overlaps(self)
    }

    pub(crate) fn draw(&self, ctrl: &mut video::Control<mode::Affine>) {
        let mut ctrl = ctrl.object(&self.slot);
        if let Ok(pos) = self.pos.try_into() {
            ctrl.set_pos(pos);
        }
    }
    pub(crate) fn setup_video(
        &self,
        sheet: &sprite::SheetSlot<14>,
        ctrl: &mut video::Control<mode::Affine>,
    ) {
        let mut ctrl = ctrl.object(&self.slot);
        if let Ok(pos) = self.pos.try_into() {
            ctrl.set_pos(pos);
        }
        ctrl.set_sprite(sheet.get(Bullets::from(self.kind) as u16));
        ctrl.set_palette_mode(palette::Type::Full);
        ctrl.set_visible(true);
    }
}
struct Damage(u8);
#[derive(Copy, Clone)]
enum Kind {
    Standard,
    Helix { spawn_time: usize },
    Momentum { velocity: Posi },
    Charge { die_time: usize },
}
impl From<Kind> for Bullets {
    fn from(value: Kind) -> Self {
        match value {
            Kind::Standard => Self::PlayerDash,
            Kind::Helix { .. } => Self::PlayerDot,
            Kind::Momentum { .. } | Kind::Charge { .. } => Self::PlayerLine,
        }
    }
}
