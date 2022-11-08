#![cfg_attr(not(feature = "test"), no_std)]
#![warn(clippy::pedantic, clippy::nursery)]
#![feature(const_mut_refs)]

#[cfg(all(test, target = "thumbv4t-none-eabi"))]
compile_error!("Tests cannot be ran in thumbv4t mode, you should use the host's architecture");

mod bitset;
mod block;

pub use bitset::Bitset128;
pub use block::Blocks;
