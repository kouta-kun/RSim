#![feature(variant_count)]
#![no_std]
#![no_main]


extern crate alloc;

use alloc::boxed::Box;
use alloc::format;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter, Write};
use core::mem::size_of;
use core::ops::{DerefMut, Neg};
use agb::display::affine::{AffineMatrix, AffineMatrixBackground};
use agb::display::object::{AffineMatrixInstance, AffineMode, Graphics, OamManaged, OamUnmanaged, Object, ObjectUnmanaged, SpriteLoader, SpriteVram, Tag};
use agb::display::{Font, Priority};
use agb::display::tiled::{AffineBackgroundSize, MapLoan, RegularBackgroundSize, RegularMap, Tiled0, Tiled1, TiledMap, TileFormat, TileSetting, VRamManager};
use agb::display::tiled::RegularBackgroundSize::{Background32x32, Background64x64};
use agb::display::video::Video;
use agb::fixnum::{FixedNum, Num, num, Number, Vector2D};
use agb::input::{Button, ButtonController, Tri};
use agb::interrupt::Interrupt;
use agb::{Gba, println};
use agb::display::font::TextRenderer;
use agbrs_flash::FlashMemory;
use once_cell::sync::Lazy;
use rand::{Rng, SeedableRng};
use rand_xoshiro::SplitMix64;
use serde::{Deserialize, Serialize};
use gamemode::GameMode;
mod gamemode;
mod state;
mod traits;

static FONT: Font = agb::include_font!("DeltaBlock-Regular.ttf", 20);

#[derive(Debug)]
struct Mode7Params {
    pa: FixedNum<8>,
    pc: FixedNum<8>,
    x: FixedNum<8>,
    y: FixedNum<8>,
}

#[derive(Debug, Copy, Clone)]
struct Vector3D<T: Number> {
    pub x: T,
    pub y: T,
    pub z: T,
}

const C_128: Lazy<FixedNum<8>> = Lazy::new(|| FixedNum::new(128));
const C_120: Lazy<FixedNum<8>> = Lazy::new(|| FixedNum::from_raw(120 << 8));

fn m7_hbl_a(cam: Vector3D<FixedNum<8>>, vcount: u16) -> Mode7Params {
    let phi = num!(0.4921875);
    let lam = cam.y / FixedNum::new((vcount | 1) as i32);
    let xs = (*C_120) * lam;
    let ys = (*C_128) * lam;

    let cos_phi = phi.cos();
    let sin_phi = phi.sin();
    let pa = (cos_phi) * lam;
    let pc = (sin_phi) * lam;
    let x = cam.x - (xs * (cos_phi) - ys * (sin_phi));
    let y = cam.z - (xs * (sin_phi) + ys * (cos_phi));

    Mode7Params { pa, pc, x, y }
}

fn draw_menu_items(fg_color: u8, bg_color: u8, font_fg: &mut RegularMap, vram: &mut VRamManager, newgame: bool, can_reload: bool, renderer: &mut TextRenderer) -> () {
    renderer.clear(vram);
    font_fg.clear(vram);
    let mut writer = renderer.writer(fg_color, bg_color, font_fg, vram);

    writeln!(&mut writer, "{}New Game", if newgame {">"} else {" "}).unwrap();
    if can_reload {
        writeln!(&mut writer, "{}Continue", if newgame { " " } else { ">" }).unwrap();
    }

    writer.commit();
    font_fg.commit(vram);
}

fn menu_mode(gba: &mut Gba, memory: &mut FlashMemory) -> (bool, u64) {
    let (tiled, mut vram) = gba.display.video.tiled1();
    let vblank = agb::interrupt::VBlank::get();
    let tileset = &gamemode::background::tiles256.tiles;

    vram.set_background_palettes(gamemode::background::PALETTES);

    let mut floor = tiled.affine(Priority::P1, AffineBackgroundSize::Background16x16);
    let mut bg = tiled.regular(Priority::P2, Background64x64, TileFormat::EightBpp);

    // configure floor tiles
    for y in 0..16u16 {
        for x in 0..16u16 {
            let i = y * 16 + x;
            let tile_id = 0;
            floor.set_tile(&mut vram, (x, y), tileset, tile_id as u8)
        }
    }
    floor.set_visible(false);
    floor.commit(&mut vram);

    // configure background (water) tiles
    for y in 0..64u16 {
        for x in 0..64u16 {
            bg.set_tile(&mut vram, (x, y), tileset, gamemode::background::tiles256.tile_settings[1]);
        }
    }
    bg.set_visible(false);
    bg.commit(&mut vram);


    let pos: Vector3D<FixedNum<8>> = (Vector3D {
        x: num!(65.0),
        y: num!(32.0),
        z: num!(65.0),
    });



    // we cast to usize to avoid complaints of object being passed through (ABSURDLY UNSAFE)
    let bg_affine_matrix = floor.bg_affine_matrix().as_ptr() as usize;
    let ih = unsafe {
        agb::interrupt::add_interrupt_handler(Interrupt::HBlank, move |cs| {
            const VCOUNT: *const u16 = 0x0400_0006 as *const u16; // REG_VCOUNT contains Y line after HBlank
            let line_count = *VCOUNT;
            let bg_affine_matrix = bg_affine_matrix as *mut AffineMatrixBackground;

            let params = m7_hbl_a(pos, line_count);

            *bg_affine_matrix = AffineMatrix::from_raw(params.pa, 0.into(), params.pc, 0.into(), params.x, params.y).to_background_wrapping();
        })
    };


    let mut font_fg = tiled.regular(Priority::P0, Background32x32, TileFormat::FourBpp);

    let font_bg_tile = vram.new_dynamic_tile().fill_with(0);

    for y in 0..20u16 {
        for x in 0..30u16 {
            font_fg.set_tile(&mut vram, (x,y), &font_bg_tile.tile_set(), font_bg_tile.tile_setting());
        }
    }

    vram.remove_dynamic_tile(font_bg_tile);

    // find bgcolor and fgcolor indices that coincide with expected colors
    let bg_color = (0..16).find(|&a| gamemode::background::PALETTES[0].colour(a) == 0x5dfa).unwrap() as u8;
    let fg_color = (0..16).find(|&a| gamemode::background::PALETTES[0].colour(a) == 0x7fff).unwrap() as u8;


    let mut input = ButtonController::new();
    let mut newgame = true;
    let can_reload = memory.have_structure();
    let mut renderer = FONT.render_text((10u16, 3u16));
    draw_menu_items(fg_color, bg_color, &mut font_fg, &mut vram, newgame, can_reload, &mut renderer);
    let mut frame = 0u64;
    loop {
        input.update();
        if (input.is_just_pressed(Button::DOWN) || input.is_just_pressed(Button::UP)) && can_reload {
            newgame = !newgame;
            draw_menu_items(fg_color, bg_color, &mut font_fg, &mut vram, newgame, can_reload, &mut renderer);
        }
        if input.is_just_pressed(Button::A) {
            break;
        }
        for i in 0..1 {
            vblank.wait_for_vblank();
        }
        font_fg.set_visible(true);
        floor.set_visible(true);
        bg.set_visible(true);
        frame += 1;
    }
    font_fg.set_visible(false);
    floor.set_visible(false);
    bg.set_visible(false);
    drop(ih);
    renderer.clear(&mut vram);
    return (newgame, frame);
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut memory = FlashMemory::new_flash_128k(&mut gba);

    let (do_newgame, frame) = menu_mode(&mut gba, &mut memory);

    let seed = frame << 8 | (frame >> (64-8) & 0xff);

    let (mut tiled, mut vram) = gba.display.video.tiled0();
    let (mut oam, mut spriteloader) = gba.display.object.get_unmanaged();


    let mut game = GameMode::new(&mut tiled, &mut vram, &mut oam, &mut spriteloader, memory, !do_newgame, seed);

    loop {
        game.step();
        game.update();
        agb::display::busy_wait_for_vblank();
    }
}