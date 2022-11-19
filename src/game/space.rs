use core::mem;

use arrayvec::ArrayVec;
use const_default::ConstDefault;
use enumflags2::{bitflags, BitFlags};
use utils::{Bitset128, Bitset8};

mod background;
mod bullet;
mod enemy;
pub(super) mod items;

use hal::{
    exec::ConsoleState,
    video::{
        self, colmod, mode, object, object::sprite, tile::cbb, tile::layer::affine,
        tile::map::AffineSize, Layer, Priority,
    },
};

use super::{ship::Weapon, state::Transition, Player, Posi, Ship, PLANET_SBB, STAR_SBB};
use crate::assets;
pub(crate) use bullet::Bullet;
pub(super) use items::Item;

const MAX_BULLETS: usize = 88;
const MAX_ITEMS: usize = 5;

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone)]
enum Cheats {
    PowerupSpawn,
}

#[cfg(feature = "cheat-powerups")]
const DEFAULT_CHEATS: BitFlags<Cheats> = enumflags2::make_bitflags!(Cheats::{PowerupSpawn});

#[cfg(not(feature = "cheat-powerups"))]
const DEFAULT_CHEATS: BitFlags<Cheats> = BitFlags::EMPTY;

pub(crate) struct Space {
    player: Player,
    // TODO: probably split between player and enemy bullets
    bullets: ArrayVec<Bullet, MAX_BULLETS>,
    items: ArrayVec<Item, MAX_ITEMS>,
    bullet_sprites: sprite::SheetSlot<14>,
    item_sprites: sprite::SheetSlot<7>,
    ship: Ship,
    new_bullets: Bitset128,
    new_items: Bitset8,
    cheats: BitFlags<Cheats>,
}

impl Space {
    pub(crate) fn update(&mut self, console: &mut ConsoleState) -> Transition {
        self.bullets.iter_mut().for_each(|bullet| {
            bullet.update(console.frame);
        });
        self.items.iter_mut().for_each(|item| {
            item.update(&mut self.player);
        });
        if self.cheats.contains(Cheats::PowerupSpawn) {
            let mut random = console.rng.u64();
            let should_spawn = (random & 127) == 0;
            if should_spawn {
                random >>= 7;
                let x = (random & 255) as i32;
                random >>= 8;
                let y = (random & 127) as i32;
                random >>= 7;
                let position = Posi::new(x + 5, y + 7);
                let kind = match random & 3 {
                    0 => items::Kind::LifeUp,
                    1 => items::Kind::Weapon(Weapon::Double),
                    2 => items::Kind::Weapon(Weapon::Momentum),
                    3 => items::Kind::Weapon(Weapon::Standard),
                    _ => unreachable!("Literally impossible"),
                };

                if let Some(item_slot) = console.reserve_object() {
                    let new_item = Item::new(item_slot, position, kind);
                    hal::info!("Spawning a new item: {new_item:?}");
                    match self.items.try_push(new_item) {
                        Ok(()) => {
                            self.new_items.reserve((self.items.len() - 1) as u32);
                        }
                        Err(_) => {
                            hal::error!("Couldn't spawn an item, too many already on screen!");
                        }
                    }
                }
            }
        }
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
        self.items = self
            .items
            .drain(..)
            .enumerate()
            .filter_map(|(i, item)| match () {
                () if item.should_die() => {
                    let slot = item.into_slot();
                    ctrl.object(&slot).set_visible(false);
                    console.free_object(slot);
                    None
                }
                () if !self.new_items.free(i as u32) => {
                    item.setup_video(&self.item_sprites, ctrl);
                    Some(item)
                }
                () => Some(item),
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
        self.items.iter().for_each(|item| item.draw(ctrl));
    }
    pub(crate) const fn start(
        selected_ship: Ship,
        player_slot: object::Slot,
        bullet_sprites: sprite::SheetSlot<14>,
        item_sprites: sprite::SheetSlot<7>,
    ) -> Self {
        Self {
            player: Player::new(player_slot, selected_ship),
            bullets: ArrayVec::new_const(),
            new_bullets: Bitset128::DEFAULT,
            items: ArrayVec::new_const(),
            new_items: Bitset8::DEFAULT,
            bullet_sprites,
            item_sprites,
            ship: selected_ship,
            cheats: DEFAULT_CHEATS,
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
