//! GBA input.
//!
//! Check the [`Input`] struct.
use core::ops;

use const_default::ConstDefault;

use volmatrix::{Safe, VolAddress};

// SAFETY: non-zero, proper access pattern
pub(crate) const KEYINPUT: VolAddress<Keys, Safe, ()> = unsafe { VolAddress::new(0x0400_0130) };

/// The GBA buttons state.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Keys(u16);
impl ConstDefault for Keys {
    const DEFAULT: Self = Self(0xFFFF);
}
impl Keys {
    /// Is **any** of the buttons of this [`KeyGroup`] pressed?
    #[must_use]
    pub const fn any_pressed(self, keys: KeyGroup) -> bool {
        // NOTE: bit=1 means the button is released, while
        // bit=0 means it's pressed
        keys.0 & self.0 != keys.0
    }
    /// Are **all** of the buttons of this [`KeyGroup`] pressed?
    #[must_use]
    pub const fn all_pressed(self, keys: KeyGroup) -> bool {
        // NOTE: bit=1 means the button is released, while
        // bit=0 means it's pressed
        keys.0 & self.0 == 0
    }
}

/// A direction, usually DPAD
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Dir {
    Right = 1 << 0,
    Left = 1 << 1,
    Up = 1 << 2,
    Down = 1 << 3,
}
/// A GBA button.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Key(u16);

// To emulate enum members with consts
#[allow(non_snake_case, non_upper_case_globals)]
impl Key {
    pub const A: Self = Self(1 << 0);
    pub const B: Self = Self(1 << 1);
    pub const Select: Self = Self(1 << 2);
    pub const Start: Self = Self(1 << 3);
    #[must_use]
    pub const fn Dpad(dir: Dir) -> Self {
        Self((dir as u16) << 4)
    }
    pub const L: Self = Self(1 << 8);
    pub const R: Self = Self(1 << 9);
}
/// Multiple GBA buttons.
///
/// Use the `From<Key>` and `From<Dir>` impl to create a
/// `KeyGroup`, and use `|` to combine multiple.
///
/// # Example
///
/// ```
/// use haldvance::input::{Key, KeyGroup, Dir};
///
/// let group1 = Key::A | Key::L;
/// let group2 = Key::B | Key::R;
/// let group3 = Key::A | Key::L | Key::B | Key::R;
/// assert_eq!(group1 | group2, group3);
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct KeyGroup(u16);
impl ops::BitOr<Self> for KeyGroup {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}
impl ops::BitOr<Self> for Key {
    type Output = KeyGroup;

    fn bitor(self, rhs: Self) -> Self::Output {
        KeyGroup(self.0 | rhs.0)
    }
}
impl ops::BitOr<Key> for KeyGroup {
    type Output = Self;

    fn bitor(self, rhs: Key) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}
impl From<Dir> for KeyGroup {
    fn from(dir: Dir) -> Self {
        Key::Dpad(dir).into()
    }
}
impl From<Key> for KeyGroup {
    fn from(key: Key) -> Self {
        Self(key.0)
    }
}

/// The GBA input state.
///
/// In [`crate::exec::full_game`], the `Input` struct passed as argument
/// to the `logic` method is updated every frame.
#[derive(Clone, Copy, ConstDefault)]
pub struct Input {
    pub(crate) current: Keys,
    pub(crate) previous: Keys,
}
impl Input {
    // TODO: optimization
    #[must_use]
    pub fn direction(self) -> Option<Dir> {
        match () {
            () if self.pressed(Dir::Down) => Some(Dir::Down),
            () if self.pressed(Dir::Up) => Some(Dir::Up),
            () if self.pressed(Dir::Left) => Some(Dir::Left),
            () if self.pressed(Dir::Right) => Some(Dir::Right),
            () => None,
        }
    }
    // TODO: optimization
    #[must_use]
    pub const fn just_direction(self) -> Option<Dir> {
        match () {
            () if self.just_pressed(Key::Dpad(Dir::Down)) => Some(Dir::Down),
            () if self.just_pressed(Key::Dpad(Dir::Up)) => Some(Dir::Up),
            () if self.just_pressed(Key::Dpad(Dir::Left)) => Some(Dir::Left),
            () if self.just_pressed(Key::Dpad(Dir::Right)) => Some(Dir::Right),
            () => None,
        }
    }
    // TODO: make those functions const once <https://github.com/rust-lang/rust/issues/67792>
    // lands
    #[allow(clippy::missing_const_for_fn)]
    pub fn pressed(self, key: impl Into<KeyGroup>) -> bool {
        let key = key.into();
        self.current.any_pressed(key)
    }
    pub fn released(self, key: impl Into<KeyGroup>) -> bool {
        !self.pressed(key)
    }
    // TODO: this doesn't optimize like I'd expect:
    // I was expecting that this would fold into a single comparison instruction,
    // but it seems not.
    // TODO: Key => impl Into<KeyGroup>
    #[must_use]
    pub const fn just_pressed(self, key: Key) -> bool {
        let key = KeyGroup(key.0);
        let current = self.current.any_pressed(key);
        let previous = self.previous.any_pressed(key);
        current && !previous
    }
    #[must_use]
    pub const fn just_released(self, key: Key) -> bool {
        let key = KeyGroup(key.0);
        let current = self.current.any_pressed(key);
        let previous = self.previous.any_pressed(key);
        !current && previous
    }
}
