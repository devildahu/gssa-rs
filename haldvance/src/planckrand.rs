//! Tinny tinny tinny random crate using the wyhash algorithm.
//!
//! Taken from <https://github.com/eldruin/wyhash-rs>

use const_default::ConstDefault;

const P0: u64 = 0xa076_1d64_78bd_642f;
const P1: u64 = 0xe703_7ed1_a0b4_28db;

const fn random(a: u64, b: u64) -> u64 {
    let hh = (a >> 32) * (b >> 32);
    let hl = (a >> 32) * (b & 0xFFFF_FFFF);
    let lh = (a & 0xFFFF_FFFF) * (b >> 32);
    let ll = (a & 0xFFFF_FFFF) * (b & 0xFFFF_FFFF);
    let a = hl.rotate_left(32) ^ hh;
    let b = lh.rotate_left(32) ^ ll;
    a ^ b
}

/// A random seed generator.
/// Use the [`Rng::get`] method to get a random number.
pub struct Rng {
    seed: u64,
}
impl ConstDefault for Rng {
    const DEFAULT: Self = Self::new(P0);
}
// TODO: implement a "almost divisionless" mean to translate to a smaller
// random value, or "really divisionless" one as in
// https://dotat.at/@/2022-04-20-really-divisionless.html
impl Rng {
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self { seed }
    }
    /// A random `u64`, advances the rng.
    #[must_use]
    pub fn get(&mut self) -> u64 {
        self.seed = self.seed.wrapping_add(P0);
        random(self.seed, self.seed ^ P1)
    }
}
