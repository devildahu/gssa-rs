// TODO: Design a generic hierarchical state machine to build the game around them
#![no_std]
#![no_main]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::redundant_pub_crate)]
// mixed_integer_ops: going to stabilize something like next week
#![feature(const_mut_refs, const_replace)]

mod assets;
mod game;
mod text;

use const_default::ConstDefault;
use hal::exec::{full_game, panic_handler, GameState};
use hal::{
    exec::ConsoleState,
    video::{
        self, colmod, mode,
        tile::{cbb, layer},
        Layer,
    },
};

use game::{
    mainmenu::{Mainmenu, TITLE_SCREEN_SBB},
    space,
    state::Transition,
};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_handler(info)
}

// TODO: devildahu logo + rust logo
enum Screen {
    Mainmenu(Mainmenu),
    Space(game::Space),
}
struct State {
    screen: Screen,
}
impl GameState for State {
    fn logic(&mut self, console: &mut ConsoleState) {
        match &mut self.screen {
            Screen::Mainmenu(mainmenu) => {
                let result = mainmenu.logic(console);
                if let Transition::EnterGame(ship) = result {
                    // TODO: unwrap
                    let slot = console.objects.reserve().unwrap();
                    self.screen = Screen::Space(game::Space::start(ship, slot));
                };
            }
            Screen::Space(ship) => {
                space::logic(console);
            }
        }
    }

    fn text_draw(&self, console: &mut ConsoleState, ctrl: &mut video::Control<mode::Text>) {
        if let Screen::Mainmenu(mainmenu) = &self.screen {
            mainmenu.text_draw(console, ctrl);
        }
    }
    fn affine_draw(&self, console: &mut ConsoleState, ctrl: &mut video::Control<mode::Affine>) {
        if let Screen::Space(_) = &self.screen {
            space::affine_draw(console, ctrl);
        }
    }
}

#[no_mangle]
pub fn main() -> ! {
    // SAFETY: I, Nicola Papale, solemnly promise that I will not
    // call video::Control::init or full_game until this video_control
    // instance is dropped.
    let mut video_control = unsafe { video::Control::<mode::Text>::init() };
    video_control.reset_display_control();
    video_control.load_tileset(cbb::Slot::_0, &assets::menu::set);
    video_control.load_palette(assets::menu::palette.get());
    video_control.enable_layer(Layer::<mode::Text>::_0);
    hal::warn!("babbooon metal world");
    {
        let mut layer = video_control.layer(layer::text::Slot::_0);
        layer.set_color_mode::<colmod::Bit8>();
        layer.set_sbb(TITLE_SCREEN_SBB);
    }
    let mut mainmenu = Mainmenu::DEFAULT;
    game::mainmenu::init_menu(&mut mainmenu.data, &mut video_control);
    let state = State { screen: Screen::Mainmenu(mainmenu) };
    // TODO move logic from top to here
    unsafe { full_game(state) };
}
