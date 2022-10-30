//! Game core logic.

use hal::video::tile::sbb;

pub(crate) mod blink;
pub(crate) mod cursor;
pub(crate) mod mainmenu;
mod posi;
mod ship;
pub(crate) mod space;
pub(crate) mod state;

pub(crate) use posi::{Area, Posi, Rect};
use ship::Player;
pub(crate) use ship::Ship;
pub(crate) use space::Space;

const STAR_SBB: sbb::Slot = sbb::Slot::_20;
const PLANET_SBB: sbb::Slot = sbb::Slot::_22;
const SCREEN_AREA: Area = Area {
    rect: Rect { width: 240, height: 160 },
    pos: Posi { x: 0, y: 0 },
};
