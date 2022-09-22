use const_default::ConstDefault;

use hal::video::{
    mode,
    tile::{layer, map::Pos, sbb},
    VideoControl,
};

use crate::layout;

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
    Main(MainEntry),
    ShipSelect { highlight: Ship },
}
pub(crate) struct Mainmenu {
    pub(crate) ship: Ship,
    pub(crate) menu: Submenu,
}
impl ConstDefault for Mainmenu {
    const DEFAULT: Self = Self {
        ship: Ship::Blank,
        menu: Submenu::Main(MainEntry::Start),
    };
}
impl Mainmenu {
    pub(crate) fn draw_new_screen(&self, ctrl: &mut VideoControl<mode::Text>) {
        match self.menu {
            Submenu::Main { .. } => {
                ctrl.layer(layer::Slot::_0).set_sbb(sbb::Slot::_16);
            }
            Submenu::ShipSelect { .. } => {
                ctrl.layer(layer::Slot::_0).set_sbb(sbb::Slot::_17);
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
pub(crate) struct MainmenuData {
    menu_buttons: MainButtons,
    ship_buttons: ShipButtons,
    ship_image: Pos,
    ship_descr: Pos,
}
pub(crate) fn init_menu(ctrl: &mut VideoControl<mode::Text>) -> MainmenuData {
    let mut image_ref = Pos::DEFAULT;
    let mut text_ref = Pos::DEFAULT;
    let mut ship_buttons = ShipButtons::DEFAULT;
    let mut main_buttons = MainButtons::DEFAULT;

    layout! {
        #[sbb(ctrl.sbb(sbb::Slot::_17))]
        vertical(
            space(2),
            text("Select your ship:"),
            space(1),
            horizontal(
                select(&mut ship_buttons.blank, "Blank"),
                space(5),
                select(&mut ship_buttons.spear, "Spear"),
                space(5),
                select(&mut ship_buttons.paladin, "Paladin"),
            ),
            space(2),
            text("Current ship:"),
            space(1),
            horizontal(
                rect(&mut image_ref, 3 x 3),
                space(1),
                rect(&mut text_ref, 20 x 3),
            ),
        )
    };
    layout! {
        #[sbb(ctrl.sbb(sbb::Slot::_16))]
        vertical(
            space(5),
            select(&mut main_buttons.start_game, "Start Game!!"),
            space(2),
            select(&mut main_buttons.ships, "Ship select"),
        )
    };
    layout! {
        #[sbb(ctrl.sbb(sbb::Slot::_15))]
        vertical(
            space(5),
            image(crate::assets::menu::title_card),
            space(1),
            text("Press Start"),
        )
    };
    MainmenuData {
        ship_buttons,
        menu_buttons: main_buttons,
        ship_descr: text_ref,
        ship_image: image_ref,
    }
}
