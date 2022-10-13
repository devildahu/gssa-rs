use hal::{exec::Rng, video::tile::sbb};

pub(crate) fn generate(rng: &mut Rng, mut sbb: sbb::AffineHandle) {
    let random = rng.get();
}
