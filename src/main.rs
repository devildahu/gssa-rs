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
use hal::video::{
    colmod, mode,
    tile::{cbb, layer, sbb},
    Layer, VideoControl,
};
use hal::{
    exec::{full_game, panic_handler, GameState},
    Input,
};

use game::mainmenu::Mainmenu;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_handler(info)
}

// TODO: devildahu logo + rust logo
enum Screen {
    Initial,
    TitleCard { blink: u8 },
    Mainmenu(Mainmenu),
}
struct State {
    screen: Screen,
    new_screen: bool,
}
impl GameState for State {
    const INITIAL: Self = Self {
        new_screen: false,
        screen: Screen::Initial,
    };

    fn logic(&mut self, pad: Input) {
        self.new_screen = false;
        match self.screen {
            Screen::TitleCard { ref mut blink } => {
                *blink = blink.wrapping_add(1);
                if pad.start() {
                    self.screen = Screen::Mainmenu(Mainmenu::DEFAULT);
                    self.new_screen = true;
                };
            }
            Screen::Initial => {
                self.screen = Screen::TitleCard { blink: 0 };
                self.new_screen = true;
            }
            Screen::Mainmenu(_) => {}
        }
    }

    fn text_draw(&self, ctrl: &mut VideoControl<mode::Text>) {
        if self.new_screen {
            match &self.screen {
                Screen::TitleCard { .. } => {
                    ctrl.layer(layer::Slot::_0).set_sbb(sbb::Slot::_15);
                }
                Screen::Initial => {}
                Screen::Mainmenu(mainmenu) => mainmenu.draw_new_screen(ctrl),
            }
        }
        match &self.screen {
            Screen::TitleCard { blink: 0 } => {}
            Screen::TitleCard { blink: 128 } => {}
            _ => {}
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
        layer.set_sbb(sbb::Slot::_15);
    }
    game::mainmenu::init_menu(&mut video_control);
    mem::drop(video_control);
    // TODO move logic from top to here
    unsafe { full_game::<State>() };
}
