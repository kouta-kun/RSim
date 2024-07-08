use alloc::vec::Vec;
use agb::display::affine::AffineMatrix;
use agb::display::tiled::{MapLoan, RegularMap, Tiled0, TiledMap, TileFormat, TileSetting, VRamManager};
use agb::display::object::{AffineMatrixInstance, AffineMode, Graphics, OamUnmanaged, ObjectUnmanaged, SpriteLoader, SpriteVram};
use agbrs_flash::FlashMemory;
use agb::input::{Button, ButtonController};
use agb::display::Priority;
use agb::display::tiled::RegularBackgroundSize::Background32x32;
use agb::fixnum::{Num, Vector2D};
use crate::state::gamestate::GameState;
use crate::state::player::Direction;
use crate::state::serialized::SerializedState;
use crate::state::inventory::ItemType;
use crate::traits::{Digits, NextTo};

agb::include_background_gfx!(pub background, "d77bba", tiles256 => 256 "map.aseprite", tiles16 => 16 "map.aseprite", font => "font.png");

pub struct GameMode<'a, 'b> {
    vram: &'b mut VRamManager,
    tiled: &'b Tiled0<'a>,
    state: GameState,
    oam: &'b mut OamUnmanaged<'a>,
    button_controller: ButtonController,
    background: MapLoan<'a, RegularMap>,
    menumap: MapLoan<'a, RegularMap>,
    tree_obj: [ObjectUnmanaged; TREECOUNT],
    memory: FlashMemory,
    player_obj: ObjectUnmanaged,
    player_sprites: Vec<SpriteVram>,
}

impl<'a, 'b> GameMode<'a, 'b>
where
    'b: 'a,
{
    pub fn new(tiled: &'b Tiled0<'a>, vram: &'b mut VRamManager, oam: &'b mut OamUnmanaged<'a>, spriteloader: &'b mut SpriteLoader, mut memory: FlashMemory, reload_state: bool, seed: u64) -> Self {
        let button_controller = ButtonController::new();
        let mut state = if reload_state {
            if let Some(state) = memory.read_structure::<SerializedState>() {
                GameState::from_save_state(state)
            } else {
                GameState::new(seed)
            }
        } else {
            GameState::new(seed)
        };
        let mut background = tiled.background(Priority::P3, Background32x32, TileFormat::FourBpp);
        state.upload(vram, &mut background);
        let tree_sprite = spriteloader.get_vram_sprite(&TREE_SPRITE.sprites()[0]);
        let tree_obj = state.map_data().get_tree_positions().map(|tree| {
            ObjectUnmanaged::new(tree_sprite.clone())
        });
        let player_sprites = MAN_SPRITE.sprites().iter().map(|sprite| spriteloader.get_vram_sprite(sprite)).collect::<Vec<_>>();
        let mut player_obj = ObjectUnmanaged::new(player_sprites[0].clone());
        let mut menumap = tiled.background(Priority::P0, Background32x32, TileFormat::FourBpp);
        let mut game = Self { tiled, vram, state, oam, button_controller, background, tree_obj, menumap, memory, player_obj, player_sprites };
        game
    }

    pub fn step(&mut self) {
        self.button_controller.update();
        let mut movement_pressed = None;
        for (button, direction) in [Button::UP, Button::DOWN, Button::LEFT, Button::RIGHT].into_iter().zip([Direction::UP, Direction::DOWN, Direction::LEFT, Direction::RIGHT].into_iter()) {
            if self.button_controller.is_pressed(button) {
                self.state.player_obj_mut().set_direction(direction);
                if self.button_controller.is_just_pressed(button) { movement_pressed = Some(direction); }
                break;
            }
        }
        if let Some(dir) = movement_pressed {
            let (nx, ny) = self.state.player_obj().move_direction(dir);
            let is_walkable = !(self.state.map_data().get_terrain_point((nx as usize, ny as usize)) && !self.state.map_data().get_bridge_point((nx as usize, ny as usize)));
            let collides_tree = self.state.map_data().has_tree((nx as u16, ny as u16));
            if is_walkable && !collides_tree {
                self.state.player_obj_mut().set_position((nx, ny));
            }
        }

        for (_index, (tree, obj)) in self.state.map_data().get_tree_positions().iter().zip(self.tree_obj.iter_mut()).enumerate() {
            let (x, y, state) = *tree;
            let px = x as i32 * 8;
            let y = (y as i16) - (if self.state.player_obj().get_position().1 as u16 > Y_SCROLL_THRESHOLD as u16 {
                self.state.player_obj().get_position().1 as i16 - Y_SCROLL_THRESHOLD as i16
            } else { 0 }).min((MAPHEIGHT as i16) - 20);
            let py = (y as i32 * 8) - 8;
            obj.set_position(Vector2D::new(px, py));
            obj.set_priority(Priority::P1);
            if state > 0 && py < 128 {
                obj.show();
            } else {
                obj.hide();
            }
        }
        if self.button_controller.is_just_pressed(Button::A) {
            let (px, py) = self.state.player_obj().get_position();
            let (px, py) = (px as u16, py as u16);
            let mut found_wood = 0;
            for tree in self.state.map_data_mut().get_tree_positions_mut().iter_mut() {
                let (tx, ty, active) = *tree;
                if (tx, ty).is_next_to(&(px, py)) && active > 0 {
                    tree.2 = 0;
                    found_wood += 3;
                }
            }
            *self.state.inventory_mut().item_mut(ItemType::WoodPlank) += found_wood;
        }

        if self.button_controller.is_just_pressed(Button::B) {
            let (px, py) = self.state.player_obj().get_position();
            let (px, py) = (px as u16, py as u16);
            let direction = self.state.player_obj().get_direction();
            let (tx, ty) = match direction {
                Direction::UP => (px, py.wrapping_sub(1)),
                Direction::DOWN => (px, py + 1),
                Direction::LEFT => (px.wrapping_sub(1), py),
                Direction::RIGHT => (px + 1, py),
            };
            if !(tx >= 32 || ty >= 32) {
                let point = (tx as usize, ty as usize);
                if self.state.map_data().get_terrain_point(point) && !self.state.map_data().get_bridge_point(point) && *self.state.inventory().item(ItemType::WoodPlank) > 0 {
                    *self.state.inventory_mut().item_mut(ItemType::WoodPlank) -= 1;
                    self.state.map_data_mut().set_bridge_point(point, true);
                    self.state.upload(self.vram, &mut self.background);
                }
            }
        }
        let current_frame = self.state.step_frame();
        self.player_obj.set_sprite(self.player_sprites[(current_frame / FRAME_SCALE) % self.player_sprites.len()].clone());

        let angle: Num<i32, 8> = agb::fixnum::num!(0.25) * match self.state.player_obj().get_direction() {
            Direction::UP => 0,
            Direction::LEFT => 1,
            Direction::DOWN => 2,
            Direction::RIGHT => 3,
        };
        let matrix = AffineMatrix::from_rotation(angle);
        self.player_obj.set_affine_matrix(AffineMatrixInstance::new(matrix.to_object_wrapping()));
        let (ox, oy) = self.state.player_obj().get_position();
        let (ox, oy) = (ox as i32, oy as i32);
        let oy = oy - (if oy > Y_SCROLL_THRESHOLD {
            oy - Y_SCROLL_THRESHOLD
        } else { 0 }).min((MAPHEIGHT as i32) - 20);
        let pos: Vector2D<i32> = Vector2D::new(ox * 8 - 4, oy * 8 - 4);
        self.player_obj.set_position(pos);
        self.player_obj.set_priority(Priority::P2);
        self.player_obj.show_affine(AffineMode::Affine);

        if current_frame % 60 * 10 == 0 {
            let (px, py) = self.state.player_obj().get_position();
            let serialized = SerializedState {
                player: (px, py, self.state.player_obj().get_direction()),
                inventory: *(self.state.inventory()),
                map_data: *(self.state.map_data()),
                frame: current_frame,
            };
            self.memory.write_structure(&serialized);
        }
    }

    pub fn update(&mut self) {
        let y_scroll = (if (self.state.player_obj().get_position().1 as i16) > Y_SCROLL_THRESHOLD as i16 {
            self.state.player_obj().get_position().1 as i16 - Y_SCROLL_THRESHOLD as i16
        } else { 0 }).min((MAPHEIGHT as i16) - 20) * 8;
        self.background.set_scroll_pos((0i16, y_scroll));
        self.background.commit(self.vram);
        let mut oam_iter = self.oam.iter();
        oam_iter.next().unwrap().set(&self.player_obj);
        for (oam, obj) in oam_iter.take(self.tree_obj.len()).zip(self.tree_obj.iter()) {
            oam.set(obj);
        }
        self.update_hud();
    }
    fn update_hud(&mut self) {
        self.menumap.set_visible(true);
        self.menumap.clear(self.vram);

        let menu_width = 10u16;
        let menu_height = 4u16;
        let menu_base_x = 30 - menu_width;
        let menu_base_y = 20 - menu_height;

        for y in 0..menu_height {
            for x in 0..menu_width {
                let tile_setting = Self::get_window_tile_setting(x, y, menu_width, menu_height);
                self.menumap.set_tile(
                    self.vram,
                    (menu_base_x + x, menu_base_y + y),
                    &background::font.tiles,
                    tile_setting,
                )
            }
        }

        self.menumap.set_tile(self.vram, (menu_base_x + 1, menu_base_y + 1), &background::font.tiles, background::font.tile_settings[15]);
        self.menumap.set_tile(self.vram, (menu_base_x + 1, menu_base_y + 2), &background::font.tiles, background::font.tile_settings[15]);
        let wood_digits = self.state.inventory().item(ItemType::WoodPlank).digits();

        for (i, digit) in wood_digits.enumerate() {
            self.menumap.set_tile(self.vram, ((menu_base_x + 2 + i as u16), menu_base_y + 1), &background::font.tiles, background::font.tile_settings[digit as usize]);
            self.menumap.set_tile(self.vram, ((menu_base_x + 2 + i as u16), menu_base_y + 2), &background::font.tiles, background::font.tile_settings[16 + (digit as usize)]);
        }

        let clock_width = 9;
        let clock_height = 4;

        let clock_base_x = if self.state.player_obj().get_position().0 < 16 { 30 - clock_width } else { 2 };
        let clock_base_y = 1;

        for y in 0..clock_height {
            for x in 0..clock_width {
                let tile_setting = Self::get_window_tile_setting(x, y, clock_width, clock_height);
                self.menumap.set_tile(
                    self.vram,
                    (clock_base_x + x, clock_base_y + y),
                    &background::font.tiles,
                    tile_setting,
                );
            }
        }

        let time = (self.state.frame() / 35);
        let ss = time % 60;
        let mm = (time / 60) % 24;

        for i in 0..2 {
            let digit = (mm / (10_u16.pow(1-i)) as usize) % 10;
            self.menumap.set_tile(self.vram, (clock_base_x + 2u16 + (i as u16), clock_base_y + 1), &background::font.tiles, background::font.tile_settings[digit]);
            self.menumap.set_tile(self.vram, (clock_base_x + 2u16 + (i as u16), clock_base_y + 2), &background::font.tiles, background::font.tile_settings[16 + (digit)]);
        }

        for i in 0..2 {
            let digit = (ss / (10_u16.pow(1-i)) as usize) % 10;
            self.menumap.set_tile(self.vram, (clock_base_x + 5u16 + (i as u16), clock_base_y + 1), &background::font.tiles, background::font.tile_settings[digit]);
            self.menumap.set_tile(self.vram, (clock_base_x + 5u16 + (i as u16), clock_base_y + 2), &background::font.tiles, background::font.tile_settings[16 + (digit)]);
        }


        self.menumap.set_scroll_pos((4i16, 4i16));

        self.menumap.commit(self.vram);
    }

    fn get_window_tile_setting(x: u16, y: u16, w: u16, h: u16) -> TileSetting {
        let is_left_border = x == 0;
        let is_right_border = x == w - 1;
        let is_top_border = y == 0;
        let is_bottom_border = y == h - 1;
        let tile_id = match (is_left_border, is_right_border, is_top_border, is_bottom_border) {
            (true, false, true, false) => 10,
            (false, true, true, false) => 11,
            (true, false, false, true) => 26,
            (false, true, false, true) => 27,
            (true, false, false, false) => 12,
            (false, true, false, false) => 13,
            (false, false, true, false) => 28,
            (false, false, false, true) => 29,
            (false, false, false, false) => 30,
            other => panic!("Incompatible combinations! {:?}", other)
        };
        let tile_setting = background::font.tile_settings[tile_id];
        tile_setting
    }
}

pub const Y_SCROLL_THRESHOLD: i32 = 10;
pub const FRAME_SCALE: usize = 5;
pub const TREECOUNT: usize = 4;
pub const MAPHEIGHT: usize = 32;
pub static MAN_SPRITE: &Graphics = agb::include_aseprite!("man.aseprite");
pub static TREE_SPRITE: &Graphics = agb::include_aseprite!("tree.aseprite");