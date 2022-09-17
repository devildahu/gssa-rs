#![no_std]
#![feature(start, const_if_match, const_fn, const_mut_refs)]

mod assets;
mod mainmenu;
mod textdraw;
mod textlayout;

use core::mem::transmute;

use gba::{
    io::{display::VBLANK_SCANLINE, keypad},
    vram::{self, text::TextScreenblockEntry, Tile8bpp},
};
use rubidium::{Color as RColor, VCOUNT};
use voladdress::VolBlock;

use mainmenu::Mainmenu;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    gba::fatal!("{}", info);
    loop {}
}

const BG_PAL: &[u8; 512] = include_bytes!("../resources/menuset_pal.bin");
const BG_TIL: &[u8; 24576] = include_bytes!("../resources/menuset_til.bin");
// const BG_PAL: &[RColor; 256] = unsafe {
//     transmute(include_bytes!("../resources/menuset_pal.bin"))
// };
// const BG_TIL: &[Tile8bpp; 384] = unsafe {
//     transmute(include_bytes!("../resources/menuset_til.bin"))
// };

fn set_tiles() {
    unsafe {
        let tile_map: VolBlock<Tile8bpp, { 24576 / 4 / 16 }> = VolBlock::new(vram::VRAM_BASE_USIZE);
        let t_bg_til: &[Tile8bpp; { 24576 / 4 / 16 }] = transmute(BG_TIL);
        for (addr, &tile) in tile_map.iter().zip(t_bg_til.iter()) {
            addr.write(tile)
        }
    }
}

fn set_palette() {
    let palram = rubidium::PALRAM;
    unsafe {
        let t_bg_pal: &[RColor; 256] = transmute(BG_PAL);
        for (i, &color) in t_bg_pal.iter().enumerate() {
            palram.bg_8bpp(i as u8).write(color);
        }
    }
}

/// Returns Screen Base Block of position idx. Idx must be a valid screen base
/// block. Any non-valid will result in undefined behavior.
unsafe fn sbb(idx: usize) -> VolBlock<TextScreenblockEntry, 2048> {
    VolBlock::new(vram::VRAM_BASE_USIZE + (idx * 2048))
}

struct Position {
    pub width: u16,
    pub height: u16,
    pub pos: Point,
}
struct Point {
    pub x: u16,
    pub y: u16,
}

fn draw_pic(map_loc: usize, pos: Position, sprite: usize) {
    let Position { width, height, pos } = pos;
    let tile_map = unsafe { sbb(map_loc) };
    for y in 0..height {
        for x in 0..width {
            let tile_map_pos = ((y + pos.y) * 32) + x + pos.x;
            let sprite_pos = sprite as u16 + y * 32 + x;
            let addr = tile_map.index(tile_map_pos as usize);
            addr.write(addr.read().with_tile_id(sprite_pos));
        }
    }
}

fn draw_text(text: &str, map_loc: usize, pos: Point) {
    let tile_map = unsafe { sbb(map_loc).iter().skip((pos.x + (32 * pos.y)) as usize) };
    text.bytes().zip(tile_map).for_each(|(byte, addr)| {
        addr.write(TextScreenblockEntry::from_tile_id((byte - 0x20) as u16));
    });
}

enum Screen {
    TitleCard { blink: u8 },
    Mainmenu(Mainmenu),
}
struct State {
    screen: Screen,
    /// If set, ran once by the draw function, then set to None
    /// Useful for initializing graphics
    oneshot_draw: Option<fn()>,
}

fn logic(state: &mut State, pad: keypad::KeyInput) {
    match state.screen {
        Screen::TitleCard { ref mut blink } => {
            *blink = blink.wrapping_add(1);
            if pad.start() {
                state.screen = Screen::Mainmenu(Default::default())
            };
        }
        Screen::Mainmenu(_) => todo!(),
    }
}

fn draw(
    State {
        screen,
        oneshot_draw,
    }: &State,
) {
    if let Some(init_fn) = oneshot_draw {
        init_fn();
    }
    match screen {
        Screen::Mainmenu(_) => todo!(),
        Screen::TitleCard { blink } => {
            let text = if blink % 50 < 25 {
                "Press start"
            } else {
                "           "
            };
            draw_text(
                text,
                15,
                Point {
                    x: 7 + 3,
                    y: 2 + 9 + 2,
                },
            );
        }
    }
}

#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    use rubidium::{
        BackgroundControlValue, DisplayControlValue, VideoMode, BG0CNT, BG1CNT, DISPCNT,
    };
    let display_settings = DisplayControlValue::new()
        .with_video_mode(VideoMode::VideoMode0)
        .with_display_bg0(true)
        .with_display_bg1(true);
    let bg0_settings = BackgroundControlValue::new()
        .with_screen_base_block(15)
        .with_priority(1)
        .with_bpp8(true);
    let bg1_settings = BackgroundControlValue::new()
        .with_screen_base_block(14)
        .with_priority(2)
        .with_bpp8(true);
    DISPCNT.write(display_settings);
    BG0CNT.write(bg0_settings);
    BG1CNT.write(bg1_settings);

    set_palette();
    set_tiles();
    let mut state = State {
        screen: Screen::TitleCard { blink: 0 },
        oneshot_draw: Some(|| {
            draw_pic(
                15,
                Position {
                    width: 17,
                    height: 9,
                    pos: Point { x: 7, y: 2 },
                },
                96,
            );
        }),
    };
    loop {
        logic(&mut state, keypad::read_key_input());
        wait_vblank();
        draw(&state);
        wait_vdraw();
        state.oneshot_draw = None;
    }
}

fn wait_vdraw() {
    while VCOUNT.read() >= VBLANK_SCANLINE {}
}
fn wait_vblank() {
    while VCOUNT.read() < VBLANK_SCANLINE {}
}
// #[no_mangle]
// static __IRQ_HANDLER: extern "C" fn() = irq_handler;

// extern "C" fn irq_handler() {}
