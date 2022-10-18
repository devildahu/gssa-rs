use hal::{
    exec::ConsoleState,
    video::{self, tile::layer::affine},
};

use super::state::Transition;

pub(crate) fn logic(_console: &mut ConsoleState) -> Transition {
    Transition::Stay
}
// allow: GBA's usize is 32, and rust reference says casting from uX to iX is
// a no-op. I just can't be harsed to explictly handle it.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub(crate) fn affine_draw(
    console: &mut ConsoleState,
    ctrl: &mut video::Control<video::mode::Affine>,
) {
    {
        let mut layer = ctrl.layer(affine::Slot::_2);
        layer.set_x_offset((console.frame as i32) * 2);
    }

    let mut layer = ctrl.layer(affine::Slot::_3);
    layer.set_x_offset((console.frame as i32) * 5);
}
