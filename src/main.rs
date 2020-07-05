#![no_std]
#![feature(start)]

use core::mem::transmute;
use gba::{
    io::display::VBLANK_SCANLINE,
    vram::{self, text::TextScreenblockEntry, Tile8bpp},
};
use rubidium::{Color as RColor, VCOUNT};
use voladdress::VolBlock;

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
    let tile_map = vram::get_8bpp_character_block(0);
    unsafe {
        let t_bg_til: &[Tile8bpp; 384] = transmute(BG_TIL);
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

unsafe fn sbb(idx: usize) -> VolBlock<TextScreenblockEntry, 2048> {
    VolBlock::new(vram::VRAM_BASE_USIZE + (idx * 2048))
}

fn draw_pic(sprite_loc: usize, w: u16, h: u16, xpos: u16, ypos: u16, map_loc: usize) {
    let tile_map = unsafe { sbb(map_loc) };
    for y in 0..h {
        for x in 0..w {
            let tile_map_pos = ((y + ypos) * 32) + x + xpos;
            let sprite_pos = sprite_loc as u16 + y * 32 + x;
            let addr = tile_map.index(tile_map_pos as usize);
            addr.write(addr.read().with_tile_id(sprite_pos));
        }
    }
}

fn draw_text(text: &str, xpos: usize, ypos: usize, map_loc: usize) {
    let tile_map = unsafe { sbb(map_loc).iter().skip(xpos + (32 * ypos)) };
    text.bytes().zip(tile_map).for_each(|(byte, addr)| {
        addr.write(TextScreenblockEntry::from_tile_id((byte - 0x20) as u16));
    });
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
    draw_text("Hello world", 0, 0, 15);
    draw_pic(96, 17, 9, 7, 2, 15);
    loop {
        //logic
        wait_vblank();
        //draw
        wait_vdraw();
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
