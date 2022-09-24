//! Main menu handling.
use const_default::ConstDefault;

use hal::{
    exec::ConsoleState,
    input::Key,
    video::{
        mode,
        tile::{layer, map::Pos, sbb},
        Tile, VideoControl,
    },
};

use crate::{assets, layout, text::EmptyLine};

pub(crate) const TITLE_SCREEN_SBB: sbb::Slot = sbb::Slot::_15;
const MAIN_MENU_SBB: sbb::Slot = sbb::Slot::_16;
const SHIP_SELECT_SBB: sbb::Slot = sbb::Slot::_17;
const PRESS_START: &str = "Press A";
const PRESS_START_LEN: usize = PRESS_START.as_bytes().len();

const PRESS_START_BLINK_RATE: usize = 1 << 6;

#[derive(Clone, Copy)]
pub(crate) enum Ship {
    Blank,
    Spear,
    Paladin,
}
impl Ship {
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
    fn draw(
        &self,
        console: &ConsoleState,
        pos: &MainButtons,
        video: &mut VideoControl<mode::Text>,
    ) {
        let highlighted = match self {
            MainEntry::Start => pos.start_game - Pos::x(2),
            MainEntry::ShipSelect => pos.ships - Pos::x(2),
        };
        let half_blink = PRESS_START_BLINK_RATE / 2;
        let mut sbb = video.sbb(MAIN_MENU_SBB);
        console.every(0, PRESS_START_BLINK_RATE, |_| {
            sbb.set_tiles(highlighted, &">");
        });
        console.every(half_blink, PRESS_START_BLINK_RATE, |_| {
            sbb.set_tile(highlighted, Tile::EMPTY);
        });
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
    just_new_screen: bool,
}
impl ConstDefault for Mainmenu {
    const DEFAULT: Self = Self {
        ship: Ship::Blank,
        menu: Submenu::Title,
        data: ConstDefault::DEFAULT,
        just_new_screen: true,
    };
}
impl Mainmenu {
    pub(crate) fn draw_new_screen(&self, ctrl: &mut VideoControl<mode::Text>) {
        let menu_slot = match self.menu {
            Submenu::Title => TITLE_SCREEN_SBB,
            Submenu::Main { .. } => MAIN_MENU_SBB,
            Submenu::ShipSelect { .. } => SHIP_SELECT_SBB,
        };
        ctrl.layer(layer::Slot::_0).set_sbb(menu_slot);
    }

    pub(crate) fn text_draw(&self, console: &ConsoleState, ctrl: &mut VideoControl<mode::Text>) {
        match &self.menu {
            Submenu::Title => self.data.draw_title_screen(console, ctrl),
            Submenu::Main(entry) => entry.draw(console, &self.data.mainmenu_buttons, ctrl),
            Submenu::ShipSelect { .. } => {}
        }
        if self.just_new_screen {
            self.draw_new_screen(ctrl);
        }
    }

    #[inline(never)]
    pub(crate) fn logic(&mut self, console: &ConsoleState) {
        self.just_new_screen = false;
        if console.input.just_pressed(Key::A) {
            match self.menu {
                Submenu::Title => {
                    self.just_new_screen = true;
                    self.menu = Submenu::Main(MainEntry::Start);
                }
                Submenu::Main(MainEntry::ShipSelect) => {
                    self.just_new_screen = true;
                    self.menu = Submenu::ShipSelect {
                        highlight: self.ship,
                    };
                }
                Submenu::Main(MainEntry::Start) => {
                    todo!("Implement the rest of the actual game");
                }
                Submenu::ShipSelect { highlight } => {
                    self.ship = highlight;
                }
            }
        } else if console.input.just_pressed(Key::B) {
            if let Submenu::ShipSelect { .. } = self.menu {
                self.just_new_screen = true;
                self.menu = Submenu::Main(MainEntry::ShipSelect);
            }
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
        horizontal(
            space(2),
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
            ),
        )
    };
    layout! {
        #[sbb(ctrl.sbb(MAIN_MENU_SBB))]
        horizontal(
            space(5),
            vertical(
                space(5),
                select(start_game, "Start Game!!"),
                space(2),
                select(ships, "Ship select"),
            ),
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
            ),
        )
    };
}
