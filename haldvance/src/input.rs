//! GBA input.
//!
//! Check the [`Input`] struct.
use gba::mmio_types::Keys;

/// The GBA input state.
///
/// In [`crate::exec::full_game`], the `Input` struct passed as argument
/// to the `logic` method is updated every frame.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Input {
    pub(crate) keypad: Keys,
}
impl Input {
    pub const fn a(&self) -> bool {
        self.keypad.a()
    }
    pub const fn b(&self) -> bool {
        self.keypad.b()
    }
    pub const fn l(&self) -> bool {
        self.keypad.l()
    }
    pub const fn r(&self) -> bool {
        self.keypad.r()
    }
    pub const fn select(&self) -> bool {
        self.keypad.select()
    }
    pub const fn start(&self) -> bool {
        self.keypad.start()
    }
    pub const fn right(&self) -> bool {
        self.keypad.right()
    }
    pub const fn left(&self) -> bool {
        self.keypad.left()
    }
    pub const fn up(&self) -> bool {
        self.keypad.up()
    }
    pub const fn down(&self) -> bool {
        self.keypad.down()
    }
}
