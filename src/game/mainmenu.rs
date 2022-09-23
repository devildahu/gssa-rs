//! Main menu handling.
use const_default::ConstDefault;

use hal::{
    exec::ConsoleState,
    video::{
        mode,
        tile::{layer, map::Pos, sbb},
        VideoControl,
    },
};

use crate::{assets, layout, text::EmptyLine};

pub(crate) const TITLE_SCREEN_SBB: sbb::Slot = sbb::Slot::_15;
const MAIN_MENU_SBB: sbb::Slot = sbb::Slot::_16;
const SHIP_SELECT_SBB: sbb::Slot = sbb::Slot::_17;
const PRESS_START: &str = "Press Start";
const PRESS_START_LEN: usize = PRESS_START.as_bytes().len();

const PRESS_START_BLINK_RATE: usize = 1 << 6;

pub(crate) enum Ship {
    Blank,
    Spear,
    Paladin,
}
impl Ship {
    const ALL: [Self; 3] = [Self::Blank, Self::Spear, Self::Paladin];
    const fn name(&self) -> &'static str {
        match self {
            Self::Blank => "Blank",
            Self::Spear => "Spear",
            Self::Paladin => "Paladin",
        }
    }
    const fn description(&self) -> &'static str {
        match self {
            Self::Blank => {
                "Good all around. Has \
                the power to banish \
                bullets in a blink"
            }
            Self::Spear => {
                "A very powerfull ship \
                favors offense at the \
                expense of defense"
            }
            Self::Paladin => {
                "This ship was built to \
                last. Has the ability \
                to convert bullets"
            }
        }
    }
}
pub(crate) enum MainEntry {
    Start,
    ShipSelect,
}
impl MainEntry {
    const ALL: [Self; 2] = [Self::Start, Self::ShipSelect];
    const fn text(&self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::ShipSelect => "select ship",
        }
    }
}
pub(crate) enum Submenu {
    Title,
    Main(MainEntry),
    ShipSelect { highlight: Ship },
}
pub(crate) struct Mainmenu {
    pub(crate) ship: Ship,
    pub(crate) menu: Submenu,
    pub(crate) data: MainmenuData,
}
impl ConstDefault for Mainmenu {
    const DEFAULT: Self = Self {
        ship: Ship::Blank,
        menu: Submenu::Title,
        data: ConstDefault::DEFAULT,
    };
}
impl Mainmenu {
    pub(crate) fn draw_new_screen(&self, ctrl: &mut VideoControl<mode::Text>) {
        match self.menu {
            Submenu::Main { .. } => {
                ctrl.layer(layer::Slot::_0).set_sbb(MAIN_MENU_SBB);
            }
            Submenu::ShipSelect { .. } => {
                ctrl.layer(layer::Slot::_0).set_sbb(SHIP_SELECT_SBB);
            }
            Submenu::Title => {
                ctrl.layer(layer::Slot::_0).set_sbb(TITLE_SCREEN_SBB);
            }
        }
    }
    pub(crate) fn text_draw(&self, console: &ConsoleState, ctrl: &mut VideoControl<mode::Text>) {
        match self.menu {
            Submenu::ShipSelect { .. } | Submenu::Main { .. } => {}
            Submenu::Title => self.data.draw_title_screen(console, ctrl),
        }
    }
}
#[derive(ConstDefault)]
struct ShipButtons {
    paladin: Pos,
    spear: Pos,
    blank: Pos,
}
#[derive(ConstDefault)]
struct MainButtons {
    start_game: Pos,
    ships: Pos,
}
#[derive(ConstDefault)]
pub(crate) struct MainmenuData {
    mainmenu_buttons: MainButtons,
    ship_buttons: ShipButtons,
    ship_image: Pos,
    ship_descr: Pos,
    press_start: Pos,
}
impl MainmenuData {
    fn draw_title_screen(&self, console: &ConsoleState, video: &mut VideoControl<mode::Text>) {
        let half_blink = PRESS_START_BLINK_RATE / 2;
        let mut sbb = video.sbb(TITLE_SCREEN_SBB);
        console.every(0, PRESS_START_BLINK_RATE, |_| {
            sbb.set_tiles(self.press_start, &PRESS_START);
        });
        console.every(half_blink, PRESS_START_BLINK_RATE, |_| {
            sbb.set_tiles(self.press_start, &EmptyLine::<PRESS_START_LEN>);
        });
    }
}
pub(crate) fn init_menu(data: &mut MainmenuData, ctrl: &mut VideoControl<mode::Text>) {
    let MainmenuData {
        mainmenu_buttons: MainButtons { start_game, ships },
        ship_buttons:
            ShipButtons {
                paladin,
                spear,
                blank,
            },
        ship_image,
        ship_descr,
        press_start,
    } = data;

    layout! {
        #[sbb(ctrl.sbb(SHIP_SELECT_SBB))]
        vertical(
            space(2),
            text("Select your ship:"),
            space(1),
            horizontal(
                select(blank, "Blank"),
                space(5),
                select(spear, "Spear"),
                space(5),
                select(paladin, "Paladin"),
            ),
            space(2),
            text("Current ship:"),
            space(1),
            horizontal(
                rect(ship_image, 3 x 3),
                space(1),
                rect(ship_descr, 20 x 3),
            ),
        )
    };
    layout! {
        #[sbb(ctrl.sbb(MAIN_MENU_SBB))]
        vertical(
            space(5),
            select(start_game, "Start Game!!"),
            space(2),
            select(ships, "Ship select"),
        )
    };
    layout! {
        #[sbb(ctrl.sbb(TITLE_SCREEN_SBB))]
        horizontal(
            space(5),
            vertical(
                space(5),
                image(assets::menu::title_card),
                space(1),
                horizontal(space(3), select(press_start, PRESS_START)),
            )
        )
    };
}
