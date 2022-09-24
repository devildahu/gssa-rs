use hal::{
    exec::ConsoleState,
    video::{
        mode,
        tile::{map::Pos, sbb, Drawable},
    },
};

pub(super) struct Blink<T: Drawable, const RATE: usize>(pub(super) T);
impl<T: Drawable, const RATE: usize> Blink<T, RATE> {
    pub(super) fn draw(
        self,
        pos: Pos,
        console: &ConsoleState,
        video: &mut sbb::Handle<mode::Text>,
    ) {
        let half_blink = RATE / 2;
        console.every(0, RATE, |_| {
            video.set_tiles(pos, &self.0);
        });
        console.every(half_blink as isize, RATE, |_| {
            video.clear_tiles(pos, &self.0);
        });
    }
}
