use hal::{
    exec::ConsoleState,
    video::tile::{map::Pos, sbb, Drawable},
};

pub(super) struct Blink<T: Drawable, const RATE: usize>(pub(super) T);
impl<T: Drawable, const RATE: usize> Blink<T, RATE> {
    // Allow: since RATE is divided by 2, result will always be in isize domain
    #[allow(clippy::cast_possible_wrap)]
    pub(super) fn draw(self, pos: Pos, console: &ConsoleState, video: &mut sbb::TextHandle) {
        let half_blink = (RATE / 2) as isize;
        console.every(0, RATE, |_| {
            video.set_tiles(pos, &self.0);
        });
        console.every(half_blink, RATE, |_| {
            video.clear_tiles(pos, &self.0);
        });
    }
}
