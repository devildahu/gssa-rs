use core::mem;

use arrayvec::ArrayVec;
use const_default::ConstDefault;
use utils::Bitset128;

mod background;
mod bullet;
mod enemy;

use hal::{
    exec::ConsoleState,
    video::{
        self, colmod, mode, object, object::sprite, tile::cbb, tile::layer::affine,
        tile::map::AffineSize, Layer, Priority,
    },
};

use super::{state::Transition, Player, Ship, PLANET_SBB, STAR_SBB};
use crate::assets;
pub(crate) use bullet::Bullet;

const MAX_BULLETS: usize = 88;

pub(crate) struct Space {
    player: Player,
    bullets: ArrayVec<Bullet, MAX_BULLETS>,
    new_bullets: Bitset128,
    bullet_sprites: sprite::SheetSlot<14>,
    ship: Ship,
}

impl Space {
    pub(crate) fn update(&mut self, console: &mut ConsoleState) -> Transition {
        self.bullets.iter_mut().for_each(|bullet| {
            bullet.update(console.frame);
        });
        if let Some(new_bullet) = self.player.update(console) {
            match self.bullets.try_push(new_bullet) {
                Ok(()) => {
                    self.new_bullets.reserve((self.bullets.len() - 1) as u32);
                }
                Err(_) => {
                    hal::error!("Couldn't spawn a bullet, too many already on screen!");
                }
            }
        }
        Transition::Stay
    }

    // allow: GBA's usize is 32, and rust reference says casting from uX to iX is
    // a no-op. I just can't be harsed to explictly handle it.
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub(crate) fn affine_draw(
        &mut self,
        console: &mut ConsoleState,
        ctrl: &mut video::Control<mode::Affine>,
    ) {
        self.bullets = self
            .bullets
            .drain(..)
            .enumerate()
            .filter_map(|(i, bullet)| match () {
                () if bullet.should_die(console.frame) => {
                    let slot = bullet.into_slot();
                    ctrl.object(&slot).set_visible(false);
                    console.free_object(slot);
                    None
                }
                () if !self.new_bullets.free(i as u32) => {
                    bullet.setup_video(&self.bullet_sprites, ctrl);
                    Some(bullet)
                }
                () => Some(bullet),
            })
            .collect();
        let mut layer = ctrl.layer(affine::Slot::_2);
        layer.set_x_offset((console.frame as i32) * 2);
        mem::drop(layer);

        let mut layer = ctrl.layer(affine::Slot::_3);
        layer.set_x_offset((console.frame as i32) * 5);
        mem::drop(layer);

        self.player.draw(ctrl);
        self.bullets.iter().for_each(|bullet| bullet.draw(ctrl));
    }
    pub(crate) const fn start(
        selected_ship: Ship,
        player_slot: object::Slot,
        bullet_sprites: sprite::SheetSlot<14>,
    ) -> Self {
        Self {
            player: Player::new(player_slot, selected_ship),
            bullets: ArrayVec::new_const(),
            new_bullets: Bitset128::DEFAULT,
            bullet_sprites,
            ship: selected_ship,
        }
    }
    pub(crate) fn setup_video(
        &self,
        ctrl: &mut video::Control<mode::Affine>,
        console: &mut ConsoleState,
    ) {
        ctrl.enable_layer(Layer::<mode::Affine>::_2);
        ctrl.enable_layer(Layer::<mode::Affine>::_3);
        ctrl.enable_objects();
        ctrl.reset_objects();
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

        let ship = self.ship.asset();
        self.player.init_video(ctrl, console, &ship);

        let rng = &mut console.rng;
        background::generate_stars(rng, ctrl.sbb(STAR_SBB, background_size));

        let mut layer = ctrl.layer(affine::Slot::_3);
        layer.set_overflow(true);
        layer.set_sbb(PLANET_SBB);
        layer.set_priority(Priority::_0);
        layer.set_color_mode::<colmod::Bit8>();
        layer.set_size(AffineSize::Base);
        mem::drop(layer);
        background::generate_planets(rng, ctrl.sbb(PLANET_SBB, AffineSize::Base));
    }
}
