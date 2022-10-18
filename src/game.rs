//! Game core logic.

use hal::video::tile::sbb;

pub(crate) mod blink;
pub(crate) mod cursor;
pub(crate) mod mainmenu;
mod ship;
pub(crate) mod space;
pub(crate) mod state;

use ship::Player;
pub(crate) use ship::Ship;
pub(crate) use space::Space;

const STAR_SBB: sbb::Slot = sbb::Slot::_20;
const PLANET_SBB: sbb::Slot = sbb::Slot::_22;
