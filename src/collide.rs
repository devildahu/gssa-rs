// allow: Clippy mistakenly thinks I can make const functions calling
// ops::Sub impl on Posi.
#![allow(clippy::missing_const_for_fn)]
use crate::game::Posi;

struct Rectangle {
    pos: Posi,
    size: Posi,
}
impl Rectangle {
    const fn left(&self) -> i32 {
        self.pos.x
    }
    const fn right(&self) -> i32 {
        self.pos.x + self.size.x
    }
    const fn top(&self) -> i32 {
        self.pos.y
    }
    const fn bottom(&self) -> i32 {
        self.pos.y + self.size.y
    }
    // From https://stackoverflow.com/questions/306316/determine-if-two-rectangles-overlap-each-other
    fn overlaps(&self, other: &Self) -> bool {
        self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }

    fn within(&self, point: Posi) -> bool {
        let Posi { x, y } = point - self.pos;
        x >= 0 && y >= 0 && x < self.size.x && y < self.size.y
    }
}

pub(crate) enum Shape {
    Point,
    Rectangle { size: Posi },
}
pub(crate) trait Collide {
    fn shape(&self) -> Shape;
    fn pos(&self) -> Posi;
    fn overlaps(&self, other: &impl Collide) -> bool {
        match (self.shape(), other.shape()) {
            (Shape::Point, Shape::Point) => self.pos() == other.pos(),
            (Shape::Point, Shape::Rectangle { size }) => {
                Rectangle { pos: other.pos(), size }.within(self.pos())
            }
            (Shape::Rectangle { size }, Shape::Point) => {
                Rectangle { pos: self.pos(), size }.within(other.pos())
            }
            (Shape::Rectangle { size: self_size }, Shape::Rectangle { size: other_size }) => {
                let self_rect = Rectangle { size: self_size, pos: self.pos() };
                let other_rect = Rectangle { size: other_size, pos: other.pos() };
                self_rect.overlaps(&other_rect)
            }
        }
    }
}
