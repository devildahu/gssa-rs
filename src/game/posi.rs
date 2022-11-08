// TODO: allow cast_sign_loss and cast_possible_truncation
// allow: use_self: in ops trait implementations, it makes it clearer what
// returns what.
#![allow(
    clippy::use_self,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
use core::ops;

use const_default::ConstDefault;
use hal::{
    input::{Dir, Key, Keys},
    video::Pos,
};

#[derive(Copy, Clone, PartialEq, Eq, ConstDefault)]
pub(crate) struct Posi {
    pub x: i32,
    pub y: i32,
}
impl Posi {
    pub const fn within(&self, area: &Area) -> bool {
        self.x > area.pos.x
            && self.x < area.pos.x + area.rect.width as i32
            && self.y > area.pos.y
            && self.y < area.pos.y + area.rect.height as i32
    }

    pub(crate) const fn y(value: i32) -> Self {
        Self { x: 0, y: value }
    }

    pub(crate) const fn x(value: i32) -> Self {
        Self { x: value, y: 0 }
    }
}
impl From<Keys> for Posi {
    fn from(keys: Keys) -> Self {
        use Dir::{Down, Left, Right, Up};
        let pressed_dir = |dir, value| if keys.pressed(Key::Dpad(dir)) { value } else { 0 };
        let mut ret = Self::DEFAULT;
        ret.y += pressed_dir(Down, 1) - pressed_dir(Up, 1);
        ret.x += pressed_dir(Right, 1) - pressed_dir(Left, 1);
        ret
    }
}
impl From<Pos> for Posi {
    fn from(Pos { x, y }: Pos) -> Posi {
        Posi { y: i32::from(y), x: i32::from(x) }
    }
}
impl From<Posi> for Pos {
    fn from(Posi { x, y }: Posi) -> Pos {
        Pos { y: y as u16, x: x as u16 }
    }
}
impl ops::AddAssign for Posi {
    fn add_assign(&mut self, rhs: Posi) {
        *self = *self + rhs;
    }
}
impl ops::SubAssign for Posi {
    fn sub_assign(&mut self, rhs: Posi) {
        *self = *self - rhs;
    }
}
impl ops::Add<Posi> for Posi {
    type Output = Posi;
    fn add(self, Posi { x, y }: Posi) -> Posi {
        Posi { x: self.x + x, y: self.y + y }
    }
}
impl ops::Add<Posi> for Pos {
    type Output = Pos;
    fn add(self, Posi { x, y }: Posi) -> Pos {
        Pos {
            x: self.x.saturating_add_signed(x as i16),
            y: self.y.saturating_add_signed(y as i16),
        }
    }
}
impl ops::Sub<Posi> for Posi {
    type Output = Posi;
    fn sub(self, rhs: Posi) -> Posi {
        self + -rhs
    }
}
impl ops::Neg for Posi {
    type Output = Posi;
    fn neg(self) -> Posi {
        Posi { x: -self.x, y: -self.y }
    }
}

pub(crate) struct Rect {
    pub width: u32,
    pub height: u32,
}
pub(crate) struct Area {
    pub rect: Rect,
    pub pos: Posi,
}
