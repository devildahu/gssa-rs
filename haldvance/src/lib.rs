//! Hardware abstraction layer for the GBA.
//!
//! Requires nightly, currently only text mode video API is implemented,
//! see [`video`].
//!
//! Use the [`exec::full_game`] function to define a game.
#![no_std]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::redundant_pub_crate)]
#![feature(const_mut_refs)]

mod bitset;
mod planckrand;

pub mod exec;
pub mod input;
pub mod log;
pub mod sane_assert;
pub mod video;

pub use gba::bios;
pub use input::Input;
