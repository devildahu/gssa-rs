//! Game core logic.

use hal::video::tile::sbb;

mod background;
pub(crate) mod blink;
pub(crate) mod cursor;
pub(crate) mod mainmenu;
pub(crate) mod space;
pub(crate) mod state;

const STAR_SBB: sbb::Slot = sbb::Slot::_20;
const PLANET_SBB: sbb::Slot = sbb::Slot::_22;
