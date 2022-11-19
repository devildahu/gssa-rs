//! Items that allow player to heal and change weapon.

use hal::info;
use hal::video::{self, mode, object, object::sprite, palette};

use crate::collide::{Collide, Shape};
use crate::game::{
    ship::{Player, Weapon},
    Posi,
};

#[derive(Copy, Clone, Debug)]
pub(crate) enum Kind {
    Weapon(Weapon),
    LifeUp,
}
impl Kind {
    const fn sprite_sheet_entry(self) -> u16 {
        match self {
            Self::LifeUp => 1,
            Self::Weapon(Weapon::Standard) => 2,
            Self::Weapon(Weapon::Momentum) => 3,
            Self::Weapon(Weapon::Charge) => 4,
            Self::Weapon(Weapon::Double) => 5,
        }
    }
}
#[derive(Debug)]
pub(crate) struct Item {
    pos: Posi,
    kind: Kind,
    // TODO: remove this
    should_die: bool,
    slot: object::Slot,
}
impl Collide for Item {
    fn shape(&self) -> Shape {
        Shape::Rectangle { size: Posi::new(8, 8) }
    }

    fn pos(&self) -> Posi {
        self.pos - Posi::new(4, 4)
    }
}
impl Item {
    pub(crate) const fn new(slot: object::Slot, pos: Posi, ty: Kind) -> Self {
        Self { pos, kind: ty, should_die: false, slot }
    }
    pub(crate) const fn into_slot(self) -> object::Slot {
        self.slot
    }
    pub(crate) fn draw(&self, ctrl: &mut video::Control<mode::Affine>) {
        let mut ctrl = ctrl.object(&self.slot);
        if let Ok(pos) = self.pos.try_into() {
            ctrl.set_pos(pos);
        }
    }
    pub(crate) fn setup_video(
        &self,
        sheet: &sprite::SheetSlot<7>,
        ctrl: &mut video::Control<mode::Affine>,
    ) {
        let mut ctrl = ctrl.object(&self.slot);
        if let Ok(pos) = self.pos.try_into() {
            ctrl.set_pos(pos);
        }
        ctrl.set_sprite(sheet.get(self.kind.sprite_sheet_entry()));
        ctrl.set_palette_mode(palette::Type::Full);
        ctrl.set_visible(true);
    }
    // NOTE: this `Item::update` may also update the player, I know this sucks
    // but hell, unless going full ECS, I don't see an alternative to this wonky
    // sharing of responsabilities
    pub(crate) fn update(&mut self, player: &mut Player) {
        self.pos.x -= 1;
        if player.overlaps(self) {
            info!("player picked up item: {self:?}");
            self.should_die = true;
            player.pick_up(self.kind);
        }
    }
    pub(crate) const fn should_die(&self) -> bool {
        self.pos.x < 0 || self.should_die
    }
}
