use hal::exec::Rng;

use crate::assets::space::Ships;

fn random_enemy(rng: &mut Rng) -> Ships {
    let random = (rng.u64() % 16) as u8;
    Ships::try_from_u8(random).unwrap_or(Ships::SmallGreen1)
}
