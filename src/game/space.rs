use core::mem;

mod background;

use hal::{
    exec::ConsoleState,
    video::{
        self, colmod, mode, object, tile::cbb, tile::layer::affine, tile::map::AffineSize, Layer,
        Priority,
    },
};

use super::{state::Transition, Player, Ship, PLANET_SBB, STAR_SBB};
use crate::assets;

pub(crate) fn logic(_console: &mut ConsoleState) -> Transition {
    Transition::Stay
}
// allow: GBA's usize is 32, and rust reference says casting from uX to iX is
// a no-op. I just can't be harsed to explictly handle it.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub(crate) fn affine_draw(console: &mut ConsoleState, ctrl: &mut video::Control<mode::Affine>) {
    {
        let mut layer = ctrl.layer(affine::Slot::_2);
        layer.set_x_offset((console.frame as i32) * 2);
    }

    let mut layer = ctrl.layer(affine::Slot::_3);
    layer.set_x_offset((console.frame as i32) * 5);
}

pub(crate) struct Space {
    player: Player,
}

impl Space {
    pub(crate) fn start(selected_ship: Ship, slot: object::Slot) -> Self {
        Self { player: Player::new(slot, selected_ship) }
    }
}

pub(crate) fn setup_video(ctrl: &mut video::Control<mode::Affine>, console: &mut ConsoleState) {
    ctrl.enable_layer(Layer::<mode::Affine>::_2);
    ctrl.enable_layer(Layer::<mode::Affine>::_3);
    ctrl.enable_objects();
    ctrl.set_object_tile_mapping(object::TileMapping::OneDim);
    ctrl.load_palette(assets::space::background_pal.get());
    ctrl.load_object_palette(0, assets::space::objects_pal.get());
    ctrl.load_tileset(cbb::Slot::_0, &assets::space::background);
    ctrl.load_tileset(cbb::Slot::_1, &assets::space::ui);

    let background_size = AffineSize::Double;
    let mut layer = ctrl.layer(affine::Slot::_2);
    layer.set_overflow(true);
    layer.set_sbb(STAR_SBB);
    layer.set_priority(Priority::_2);
    layer.set_color_mode::<colmod::Bit8>();
    layer.set_size(background_size);
    mem::drop(layer);
    background::generate_stars(&mut console.rng, ctrl.sbb(STAR_SBB, background_size));

    let mut layer = ctrl.layer(affine::Slot::_3);
    layer.set_overflow(true);
    layer.set_sbb(PLANET_SBB);
    layer.set_priority(Priority::_0);
    layer.set_color_mode::<colmod::Bit8>();
    layer.set_size(AffineSize::Base);
    mem::drop(layer);
    background::generate_planets(&mut console.rng, ctrl.sbb(PLANET_SBB, AffineSize::Base));
}
