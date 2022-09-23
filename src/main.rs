// TODO: Design a generic hierarchical state machine to build the game around them
#![no_std]
#![no_main]
#![warn(clippy::pedantic, clippy::nursery)]
#![feature(const_mut_refs)]

mod assets;
mod game;
mod text;

use core::mem;

use const_default::ConstDefault;
use hal::{
    exec::ConsoleState,
    video::{
        colmod, mode,
        tile::{cbb, layer, sbb},
        Layer, VideoControl,
    },
};
use hal::{
    exec::{full_game, panic_handler, GameState},
    Input,
};

use game::mainmenu::{Mainmenu, TITLE_SCREEN_SBB};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_handler(info)
}

// TODO: devildahu logo + rust logo
enum Screen {
    Mainmenu(Mainmenu),
}
struct State {
    screen: Screen,
}
impl GameState for State {
    fn logic(&mut self, _: &ConsoleState, pad: Input) {}

    fn text_draw(&self, console: &ConsoleState, ctrl: &mut VideoControl<mode::Text>) {
        match &self.screen {
            Screen::Mainmenu(mainmenu) => mainmenu.text_draw(console, ctrl),
        }
    }
}

#[no_mangle]
pub fn main() -> ! {
    // SAFETY: I, Nicola Papale, solemnly promise that I will not
    // call VideoControl::init or full_game until this video_control
    // instance is dropped.
    let mut video_control = unsafe { VideoControl::<mode::Text>::init() };
    video_control.reset_display_control();
    video_control.load_tileset(cbb::Slot::_0, &assets::menu::set);
    video_control.load_palette(&assets::menu::palette.get());
    video_control.enable_layer(Layer::<mode::Text>::_0);
    hal::warn!("hello world");
    {
        let mut layer = video_control.layer(layer::Slot::_0);
        layer.set_color_mode::<colmod::Bit8>();
        layer.set_sbb(TITLE_SCREEN_SBB);
    }
    let mut mainmenu = Mainmenu::DEFAULT;
    game::mainmenu::init_menu(&mut mainmenu.data, &mut video_control);
    let state = State {
        screen: Screen::Mainmenu(mainmenu),
    };
    mem::drop(video_control);
    // TODO move logic from top to here
    unsafe { full_game(state) };
}
