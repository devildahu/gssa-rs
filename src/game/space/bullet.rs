use hal::video::{self, mode, object, object::sprite, palette};

use crate::game::{ship::Player, ship::Weapon, Posi, SCREEN_AREA};

pub(crate) struct Bullet {
    kind: Kind,
    pos: Posi,
    damage: Damage,
    slot: object::Slot,
}
impl Bullet {
    pub(in crate::game) fn new_from_player(
        player: &Player,
        frame: usize,
        velocity: Posi,
        slot: object::Slot,
    ) -> Self {
        Self {
            slot,
            kind: match player.weapon {
                Weapon::Standard => Kind::Standard,
                Weapon::Double => Kind::Helix { spawn_time: frame },
                Weapon::Momentum => Kind::Momentum { velocity },
                Weapon::Charge => Kind::Charge { die_time: frame + 60 },
            },
            pos: player.pos.into(),
            damage: Damage(player.strength()),
        }
    }
    pub(super) fn update(&mut self, frame: usize) {
        match self.kind {
            Kind::Standard => self.pos.x += 1,
            Kind::Momentum { velocity } => self.pos += velocity,
            Kind::Charge { .. } => {}
            Kind::Helix { spawn_time } => {
                let offset = Posi {
                    y: ((frame - spawn_time) % 64) as i32 - 32,
                    x: 1,
                };
                self.pos += offset;
            }
        }
    }
    pub(super) const fn should_die(&self, frame: usize) -> bool {
        if let Kind::Charge { die_time } = self.kind {
            if die_time < frame {
                return true;
            }
        }
        !self.pos.within(&SCREEN_AREA)
    }

    pub(crate) fn draw(&self, ctrl: &mut video::Control<mode::Affine>) {
        let mut ctrl = ctrl.object(&self.slot);
        ctrl.set_pos(self.pos.into());
    }
    pub(crate) fn setup_video(
        &self,
        sprite: sprite::Slot,
        ctrl: &mut video::Control<mode::Affine>,
    ) {
        let mut ctrl = ctrl.object(&self.slot);
        ctrl.set_pos(self.pos.into());
        ctrl.set_sprite(sprite);
        ctrl.set_palette_mode(palette::Type::Full);
        ctrl.set_visible(true);
    }
}
struct Damage(u8);
enum Kind {
    Standard,
    Helix { spawn_time: usize },
    Momentum { velocity: Posi },
    Charge { die_time: usize },
}
