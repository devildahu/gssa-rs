//! Game core logic.

use hal::video::tile::sbb;

pub(crate) mod blink;
pub(crate) mod cursor;
pub(crate) mod mainmenu;
mod posi;
mod ship;
pub(crate) mod space;
pub(crate) mod state;

pub(crate) use posi::{Area, Posi};
use ship::Player;
pub(crate) use ship::Ship;
pub(crate) use space::Space;

const STAR_SBB: sbb::Slot = sbb::Slot::_20;
const PLANET_SBB: sbb::Slot = sbb::Slot::_22;
const SCREEN_AREA: Area = Area {
    size: Posi::new(240, 160),
    pos: Posi::new(0, 0),
};
