//! Define a blinking cursor to navigate a menu, see [`Cursor`].

use core::mem;

use const_default::ConstDefault;
use hal::{
    exec::ConsoleState,
    video::{
        tile::{map::Pos, sbb},
        Tile,
    },
};

/// A drawable cursor that supports clearing when moved and
/// blinking at given `RATE`.
///
/// Please use a power of 2 for `RATE` for improved performance.
#[derive(ConstDefault)]
pub(super) struct Cursor<const RATE: usize> {
    current: Pos,
    blink_offset: isize,
    previous: Option<Pos>,
}
impl<const RATE: usize> Cursor<RATE> {
    /// This must be called at the start of each frame, otherwise the previous
    /// cursor position will constantly be overwritten.
    pub(super) const fn clear_previous(&mut self) {
        self.previous = None;
    }
    pub(super) const fn update(&mut self, new_pos: Pos, console: &ConsoleState) {
        self.previous = Some(mem::replace(&mut self.current, new_pos));
        // Keeps track of last update frame, frequency will be offset by this value,
        // so that when the cursor is updated, the new position is always immediately visible
        self.blink_offset = -1 - console.frame as isize;
    }
    // TODO: if update doesn't change the cursor position, it shouldn't be cleared.
    // TODO: the `previous` should be associated with the sbb or more simply the exact
    // memory location, instead of just the position.
    // Otherwise, the cursor never gets cleared when swapping screens.
    pub(super) fn draw(&self, console: &ConsoleState, video: &mut sbb::TextHandle) {
        let half_blink = RATE / 2;
        let offset = self.blink_offset;
        console.every(offset, RATE, |_| {
            video.set_tiles(self.current, &">");
        });
        console.every(offset.wrapping_add_unsigned(half_blink), RATE, |_| {
            video.set_tile(Tile::EMPTY, self.current);
        });
        if let Some(previous) = self.previous {
            video.set_tile(Tile::EMPTY, previous);
        }
    }
}
