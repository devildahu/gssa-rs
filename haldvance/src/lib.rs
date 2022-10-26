//! Hardware abstraction layer for the GBA.
//!
//! Requires nightly, currently only text mode video API is implemented,
//! see [`video`].
//!
//! Use the [`exec::full_game`] function to define a game.
#![no_std]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::redundant_pub_crate)]
#![feature(const_mut_refs, const_type_id)]

mod bitset;
mod block;
mod planckrand;
mod unique_id;

pub mod exec;
pub mod input;
pub mod log;
pub mod sane_assert;
pub mod video;

pub use gba::bios;
pub use input::Input;
pub use unique_id::UniqueId;
