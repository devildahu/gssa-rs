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
    pub fn u64(&mut self) -> u64 {
        self.seed = self.seed.wrapping_add(P0);
        random(self.seed, self.seed ^ P1)
    }
    /// An infinite iterator, each item an `usize` of which `bit_count` bits
    /// are randomly set.
    ///
    /// # Panics
    /// (const time only) If `COUNT > 32`
    pub fn random_bits<const COUNT: u32>(&mut self) -> RandBitsIter<COUNT> {
        assert!(COUNT <= usize::BITS);
        RandBitsIter {
            random_value: self.u64(),
            inner: self,
            items_for_value: 0,
        }
    }
    pub fn reseed(&mut self, seed: u64) {
        self.seed = seed;
    }
}
/// Iterator for the [`Rng::random_bits`] return value.
pub struct RandBitsIter<'a, const COUNT: u32> {
    inner: &'a mut Rng,
    random_value: u64,
    items_for_value: u32,
}
impl<'a, const COUNT: u32> RandBitsIter<'a, COUNT> {
    const ITEM_PER_VALUE: u32 = 64 / COUNT;
}
impl<'a, const COUNT: u32> Iterator for RandBitsIter<'a, COUNT> {
    type Item = usize;
    /// Note that this iterator is unbound, therefore is never `None`.
    fn next(&mut self) -> Option<Self::Item> {
        if self.items_for_value == Self::ITEM_PER_VALUE {
            self.random_value = self.inner.u64();
            self.items_for_value = 0;
        }
        // Bitmask within range provided by user.
        let ret = self.random_value & (2_u64.pow(COUNT) - 1);
        self.items_for_value += 1;
        // "delete" the random bits we just used and replace them with the
        // newly generated ones.
        self.random_value >>= COUNT;
        // unwrap: by construction, COUNT will always be lower than usize::BITS,
        // therefore
        Some(ret.try_into().unwrap())
    }
}
