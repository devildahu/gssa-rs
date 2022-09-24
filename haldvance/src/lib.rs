//! Hardware abstraction layer for the GBA.
//!
//! Requires nightly, currently only text mode video API is implemented,
//! see [`video`].
//!
//! Use the [`exec::full_game`] function to define a game.
#![no_std]
#![warn(clippy::pedantic, clippy::nursery)]
// mixed_integer_ops: going to stabilize something like next week
#![feature(const_mut_refs, mixed_integer_ops)]

pub mod exec;
pub mod input;
pub mod log;
pub mod video;

pub use input::Input;
