use hal::Input;

use crate::game::{ship::Player, ship::Weapon, Posi, SCREEN_AREA};

pub(super) struct Bullet {
    kind: Kind,
    pos: Posi,
    damage: Damage,
}
impl Bullet {
    pub(super) fn new_from_player(player: &Player, frame: usize, input: Input) -> Self {
        Self {
            kind: match player.weapon {
                Weapon::Standard => Kind::Standard,
                Weapon::Double => Kind::Helix { spawn_time: frame },
                Weapon::Momentum => Kind::Momentum { velocity: input.current().into() },
                Weapon::Charge => Kind::Charge { die_time: frame + 60 },
            },
            pos: player.pos.into(),
            damage: Damage(player.strength()),
        }
    }
    pub(super) fn update(&mut self, frame: usize) {
        match self.kind {
            Kind::Standard => self.pos.x += 1,
            Kind::Helix { spawn_time } => {
                let offset = Posi {
                    y: ((frame - spawn_time) % 64) as i32 - 32,
                    x: 1,
                };
                self.pos += offset;
            }
            Kind::Momentum { velocity } => self.pos += velocity,
            Kind::Charge { .. } => {}
        }
    }
    pub(super) fn should_die(&self, frame: usize) -> bool {
        if let Kind::Charge { die_time } = self.kind {
            if die_time < frame {
                return true;
            }
        }
        !self.pos.within(&SCREEN_AREA)
    }
}
struct Damage(u8);
enum Kind {
    Standard,
    Helix { spawn_time: usize },
    Momentum { velocity: Posi },
    Charge { die_time: usize },
}
