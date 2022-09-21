#![no_std]
#![no_main]
#![feature(const_mut_refs, generic_const_exprs)]

// TODO: split the game into two crates:
// 1. A HAL (hardware abstraction layer) that defines assets and hardware access
// 2. Game logic. My thinking right now is a generic hierarchical state machine
//    definition and build the game around state machines.
mod assets;
mod game;
mod runtime_asserts;
mod text;
mod video_control;
mod volmatrix;

pub(crate) use video_control::VideoControl;
mod colmod {
    #[allow(unused)]
    pub(crate) use crate::video_control::{Color4bit, Color8bit, ColorMode};
}
mod vidmod {
    #[allow(unused)]
    pub(crate) use crate::video_control::{Affine, Mixed, Mode, Text, TileMode};
}

use const_default::ConstDefault;
use gba::mmio_addresses::KEYINPUT;
use gba::mmio_types::Keys;

use game::mainmenu::Mainmenu;
use video_control::bg;
use voladdress::{Safe, VolAddress};

pub const VCOUNT: VolAddress<u16, Safe, ()> = unsafe { voladdress::VolAddress::new(0x0400_0006) };

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    gba::fatal!("{}", info)
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
impl State {
    const DEFAULT: Self = Self {
        new_screen: false,
        screen: Screen::Initial,
    };
}

fn logic(state: &mut State, pad: Keys) {
    state.new_screen = false;
    match state.screen {
        Screen::TitleCard { ref mut blink } => {
            *blink = blink.wrapping_add(1);
            if pad.start() {
                state.screen = Screen::Mainmenu(Mainmenu::DEFAULT);
                state.new_screen = true;
            };
        }
        Screen::Initial => {
            state.screen = Screen::TitleCard { blink: 0 };
            state.new_screen = true;
        }
        Screen::Mainmenu(_) => {}
    }
}

fn draw(ctrl: &mut VideoControl<vidmod::Text>, state: &State) {
    if state.new_screen {
        match &state.screen {
            Screen::TitleCard { .. } => {
                ctrl.layer(bg::TextLayerSlot::_0).set_sbb(bg::SbbSlot::_15);
            }
            Screen::Initial => {}
            Screen::Mainmenu(mainmenu) => mainmenu.draw_new_screen(ctrl),
        }
    }
    match &state.screen {
        Screen::TitleCard { blink: 0 } => {}
        Screen::TitleCard { blink: 128 } => {}
        _ => {}
    }
}

fn read_keypad() -> Keys {
    KEYINPUT.read().into()
}

#[no_mangle]
pub fn main() -> ! {
    // SAFETY: I, Nicola Papale, solemnly promise that this instantiation
    // of VideoControl as the first line of the "main" function will forever
    // be the only place in this video game where VideoControl::init is called.
    let mut video_control = unsafe { VideoControl::init() };
    let mut state = State::DEFAULT;
    video_control.reset_display_control();
    video_control.load_tileset(bg::CbbSlot::_0, &assets::menu::set);
    video_control.load_palette(&assets::menu::palette);
    video_control.enable_layer(bg::TextLayerSlot::_0);
    {
        let mut layer = video_control.layer(bg::TextLayerSlot::_0);
        layer.set_color_mode::<colmod::Color8bit>();
        layer.set_sbb(bg::SbbSlot::_15);
    }
    game::mainmenu::init_menu(&mut video_control);
    loop {
        // logic(&mut state, read_keypad());
        wait_vblank();
        // draw(&mut video_control, &state);
        wait_vdraw();
    }
}

const VBLANK_START: u16 = 160;
fn wait_vblank() {
    while VCOUNT.read() < VBLANK_START {}
}
fn wait_vdraw() {
    while VCOUNT.read() >= VBLANK_START {}
}
