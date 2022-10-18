use gbassets::DynamicImage;
use hal::{
    exec::Rng,
    video::tile::{map, sbb},
};

use crate::assets::space;

const PLANET_COUNT: usize = 3;

// algorithm: knowing we have a region.surface_size() tiles to fill, we
// place N tiles on it, tiles are taken from three different sets:
// 1. Set of 1×1 tiles that can be placed anywherrre
// 2. 2×2 tiles
// 3. 4×4 tiles
/// Generate the space background by randomly laying out stars.
pub(crate) fn generate_stars(rng: &mut Rng, mut sbb: sbb::AffineHandle) {
    let region = sbb.size();

    for y in 0..region.height() {
        // TODO: this can be improved by only using more bits for random tile
        // if we satisfy the 25% chance hit
        let iter = rng
            .random_bits::<8>()
            .take(region.width() as usize)
            .map(|rand| {
                // unwrap: never fails because % 16 will always be within range of u8
                let tile: u8 = (rand % space::star_count).try_into().unwrap();
                // True 1 time out of 16
                let should_show = rand & 0b1111_0000 == 0b1111_0000;
                should_show.then_some(tile).unwrap_or_default()
            });
        sbb.set_line(map::Pos::y(y), iter);
    }
}
pub(crate) fn generate_planets(rng: &mut Rng, mut sbb: sbb::AffineHandle) {
    let region = sbb.size();

    let bits_used = |value: u16| u16::BITS - value.leading_zeros() - 1;

    // We chose three random spot on our map to place our 4×4 planet
    for position in rng.random_bits::<10>().take(PLANET_COUNT) {
        // SAFETY: return values of rng.random_bits are guarenteed to be <u16::MAX
        // (because of ::<10> const argument)
        let mut position: u16 = unsafe { position.try_into().unwrap_unchecked() };
        let x = (position % region.width()).saturating_sub(3);

        position >>= bits_used(region.width());

        let y = (position % region.height()).saturating_sub(3);

        position >>= bits_used(region.height());

        let planet_size = space::big_planet_size;
        let planet = space::big_planet_offset + planet_size * (position % 4);
        let image = DynamicImage::<16>::new(planet, space::background_width, planet_size);

        sbb.set_tiles(map::Pos { x, y }, &&image);
    }
}
