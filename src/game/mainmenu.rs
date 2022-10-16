//! Main menu handling.
mod cycle;

use core::mem;

use const_default::ConstDefault;

use gbassets::Image;
use hal::{
    exec::{ConsoleState, EnterMode},
    input::{Dir, Key},
    video::{
        self, colmod, mode, object,
        tile::{
            cbb,
            drawable::Windowed,
            layer,
            map::{AffineSize, Pos, Rect},
            sbb,
        },
        Layer, Priority,
    },
};

use crate::{
    assets,
    game::{background, cursor::Cursor},
    layout,
};

use super::blink::Blink;

const STAR_SBB: sbb::Slot = sbb::Slot::_20;
const MAIN_MENU_SBB: sbb::Slot = sbb::Slot::_16;
const SHIP_SELECT_SBB: sbb::Slot = sbb::Slot::_17;
const PLANET_SBB: sbb::Slot = sbb::Slot::_22;
pub(crate) const TITLE_SCREEN_SBB: sbb::Slot = sbb::Slot::_15;
const PRESS_START: &str = "Press A";
const DESCR_WIDTH: u16 = 21;

const PRESS_START_BLINK_RATE: usize = 1 << 6;

crate::cycling_enum! {
    #[derive(Clone, Copy)]
    pub(crate) enum Ship {
        Blank,
        Spear,
        Paladin,
    }
}
impl Ship {
    const fn image(self) -> Image {
        use assets::menu::player_ships;
        match self {
            Self::Blank => player_ships::blank,
            Self::Spear => player_ships::spear,
            Self::Paladin => player_ships::paladin,
        }
    }
    const fn name(self) -> &'static str {
        match self {
            Self::Blank => "Blank",
            Self::Spear => "Spear",
            Self::Paladin => "Paladin",
        }
    }
    const fn description(self) -> &'static str {
        match self {
            Self::Blank => "Good all around. Has\nthe power to banish\nbullets in a blink.",
            Self::Spear => "A very powerfull ship\nfavors offense at the\nexpense of defense.",
            Self::Paladin => "Was built to last.\nHas the ability to\nconvert bullets.",
        }
    }
    const fn go(self, dir: Dir) -> Self {
        match dir {
            Dir::Left => self.prev(),
            Dir::Right => self.next(),
            Dir::Down | Dir::Up => self,
        }
    }
}
crate::cycling_enum! {
    #[derive(Clone, Copy)]
    pub(crate) enum MainEntry {
        Start,
        ShipSelect,
    }
}
impl MainEntry {
    const fn go(self, dir: Dir) -> Self {
        match dir {
            Dir::Up => self.prev(),
            Dir::Down => self.next(),
            Dir::Left | Dir::Right => self,
        }
    }
}
pub(crate) enum Submenu {
    Title,
    Main(MainEntry),
    ShipSelect { highlight: Ship },
}
pub(crate) struct Mainmenu {
    pub(crate) selected_ship: Ship,
    pub(crate) menu: Submenu,
    pub(crate) data: MainMenuData,
    just_new_screen: bool,
    cursor: Cursor<PRESS_START_BLINK_RATE>,
}
impl ConstDefault for Mainmenu {
    const DEFAULT: Self = Self {
        selected_ship: Ship::Blank,
        menu: Submenu::Title,
        data: ConstDefault::DEFAULT,
        just_new_screen: true,
        cursor: Cursor::DEFAULT,
    };
}
impl Mainmenu {
    pub(crate) fn draw_new_screen(&self, ctrl: &mut video::Control<mode::Text>) {
        let menu_slot = match self.menu {
            Submenu::Title => TITLE_SCREEN_SBB,
            Submenu::Main { .. } => MAIN_MENU_SBB,
            Submenu::ShipSelect { .. } => SHIP_SELECT_SBB,
        };
        ctrl.layer(layer::Slot::_0).set_sbb(menu_slot);
    }

    pub(crate) fn text_draw(&self, console: &ConsoleState, ctrl: &mut video::Control<mode::Text>) {
        match &self.menu {
            Submenu::Title => self.data.draw_title_screen(console, ctrl),
            Submenu::ShipSelect { .. } => self
                .cursor
                .draw(console, &mut ctrl.basic_sbb(SHIP_SELECT_SBB)),
            Submenu::Main { .. } => self
                .cursor
                .draw(console, &mut ctrl.basic_sbb(MAIN_MENU_SBB)),
        }
        if self.just_new_screen {
            self.draw_new_screen(ctrl);
            self.data.ship_menu.draw_selected(self.selected_ship, ctrl);
        }
    }

    pub(crate) fn logic(&mut self, console: &mut ConsoleState) {
        self.just_new_screen = false;
        self.cursor.clear_previous();
        if console.input.just_pressed(Key::A) {
            match self.menu {
                Submenu::Title => {
                    self.just_new_screen = true;
                    self.menu = Submenu::Main(MainEntry::Start);
                    let cursor_pos = self.data.menu_select.of(MainEntry::Start) - Pos::x(2);
                    self.cursor.update(cursor_pos, console);
                }
                Submenu::Main(MainEntry::ShipSelect) => {
                    self.just_new_screen = true;
                    self.menu = Submenu::ShipSelect { highlight: self.selected_ship };
                    let cursor_pos = self.data.ship_menu.of(self.selected_ship) - Pos::x(1);
                    self.cursor.update(cursor_pos, console);
                }
                Submenu::Main(MainEntry::Start) => {
                    console.enter_video_mode = Some(EnterMode::Affine(|ctrl, console| {
                        ctrl.enable_layer(Layer::<mode::Affine>::_2);
                        ctrl.enable_layer(Layer::<mode::Affine>::_3);
                        ctrl.enable_objects();
                        ctrl.set_object_tile_mapping(object::TileMapping::OneDim);
                        ctrl.load_palette(assets::space::background_pal.get());
                        ctrl.load_object_palette(0, assets::space::objects_pal.get());
                        ctrl.load_tileset(cbb::Slot::_0, &assets::space::background);
                        ctrl.load_tileset(cbb::Slot::_1, &assets::space::ui);

                        let background_size = AffineSize::Double;
                        let mut layer = ctrl.layer(layer::AffineSlot::_2);
                        layer.set_overflow(true);
                        layer.set_sbb(STAR_SBB);
                        layer.set_priority(Priority::_2);
                        layer.set_color_mode::<colmod::Bit8>();
                        layer.set_size(background_size);
                        mem::drop(layer);
                        background::generate_stars(
                            &mut console.rng,
                            ctrl.sbb(STAR_SBB, background_size),
                        );

                        let mut layer = ctrl.layer(layer::AffineSlot::_3);
                        layer.set_overflow(true);
                        layer.set_sbb(PLANET_SBB);
                        layer.set_priority(Priority::_0);
                        layer.set_color_mode::<colmod::Bit8>();
                        layer.set_size(AffineSize::Base);
                        mem::drop(layer);
                        background::generate_planets(
                            &mut console.rng,
                            ctrl.sbb(PLANET_SBB, AffineSize::Base),
                        );
                    }));
                }
                Submenu::ShipSelect { highlight } => {
                    self.selected_ship = highlight;
                    self.just_new_screen = true;
                }
            }
        } else if console.input.just_pressed(Key::B) {
            if let Submenu::ShipSelect { .. } = self.menu {
                self.just_new_screen = true;
                self.menu = Submenu::Main(MainEntry::ShipSelect);
                let ship_pos = self.data.menu_select.of(MainEntry::ShipSelect);
                self.cursor.update(ship_pos - Pos::x(2), console);
            }
        } else {
            match &mut self.menu {
                Submenu::Title => {}
                Submenu::Main(entry) => {
                    if let Some(dir) = console.input.just_direction() {
                        *entry = entry.go(dir);
                        let cursor_pos = self.data.menu_select.of(*entry) - Pos::x(2);
                        self.cursor.update(cursor_pos, console);
                    }
                }
                Submenu::ShipSelect { highlight } => {
                    if let Some(dir) = console.input.just_direction() {
                        *highlight = highlight.go(dir);
                        let cursor_pos = self.data.ship_menu.of(*highlight) - Pos::x(1);
                        self.cursor.update(cursor_pos, console);
                    }
                }
            }
        }
    }
}

/// Positions of various elements in the ship selection screen.
#[derive(ConstDefault, Copy, Clone)]
struct ShipMenuPos {
    paladin: Pos,
    spear: Pos,
    blank: Pos,
    image: Pos,
    descr: Pos,
    name: Pos,
}
impl ShipMenuPos {
    fn draw_selected(self, selected: Ship, ctrl: &mut video::Control<mode::Text>) {
        let mut sbb = ctrl.basic_sbb(SHIP_SELECT_SBB);
        let win = |inner, width, height| Windowed { inner, window: Rect { width, height } };
        sbb.clear_tiles(self.image, &selected.image());
        sbb.clear_tiles(self.name, &win(selected.name(), 7, 1));
        sbb.clear_tiles(self.descr, &win(selected.description(), DESCR_WIDTH, 3));
        sbb.set_tiles(self.image, &selected.image());
        sbb.set_tiles(self.name, &win(selected.name(), 7, 1));
        sbb.set_tiles(self.descr, &win(selected.description(), DESCR_WIDTH, 3));
    }
    const fn of(self, ship: Ship) -> Pos {
        match ship {
            Ship::Blank => self.blank,
            Ship::Spear => self.spear,
            Ship::Paladin => self.paladin,
        }
    }
}
#[derive(ConstDefault, Clone, Copy)]
struct MenuSelectPos {
    start_game: Pos,
    ships: Pos,
}
impl MenuSelectPos {
    const fn of(self, entry: MainEntry) -> Pos {
        match entry {
            MainEntry::Start => self.start_game,
            MainEntry::ShipSelect => self.ships,
        }
    }
}
#[derive(ConstDefault)]
pub(crate) struct MainMenuData {
    menu_select: MenuSelectPos,
    ship_menu: ShipMenuPos,
    press_start: Pos,
}
impl MainMenuData {
    fn draw_title_screen(&self, console: &ConsoleState, video: &mut video::Control<mode::Text>) {
        let mut sbb = video.basic_sbb(TITLE_SCREEN_SBB);
        let blink = Blink::<_, PRESS_START_BLINK_RATE>(PRESS_START);
        blink.draw(self.press_start, console, &mut sbb);
    }
}
pub(crate) fn init_menu(data: &mut MainMenuData, ctrl: &mut video::Control<mode::Text>) {
    let MainMenuData {
        menu_select: MenuSelectPos { start_game, ships },
        ship_menu: ShipMenuPos { paladin, spear, blank, image, descr, name },
        press_start,
    } = data;

    layout! {
        #[sbb(ctrl.basic_sbb(SHIP_SELECT_SBB))]
        horizontal(
            space(2),
            vertical(
                space(2),
                text("Select your ship:"),
                space(1),
                horizontal(
                    select(blank, "Blank"),
                    space(4),
                    select(spear, "Spear"),
                    space(4),
                    select(paladin, "Paladin"),
                ),
                space(4),
                horizontal(
                    text("Current ship:"),
                    space(2),
                    rect(name, 7 x 1),
                ),
                space(2),
                horizontal(
                    rect(image, 3 x 3),
                    space(1),
                    rect(descr, DESCR_WIDTH x 3),
                ),
            ),
        )
    };
    layout! {
        #[sbb(ctrl.basic_sbb(MAIN_MENU_SBB))]
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
        #[sbb(ctrl.basic_sbb(TITLE_SCREEN_SBB))]
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
