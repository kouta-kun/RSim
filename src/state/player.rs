use agb::fixnum::{Num, Vector2D};
use agb::display::affine::AffineMatrix;
use agb::display::object::{AffineMatrixInstance, AffineMode, ObjectUnmanaged, SpriteLoader, SpriteVram};
use agb::display::Priority;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use crate::gamemode::MAN_SPRITE;
use crate::gamemode::{FRAME_SCALE, MAPHEIGHT, Y_SCROLL_THRESHOLD};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

pub struct PlayerObj {
    sprites: Vec<SpriteVram>,
    object: ObjectUnmanaged,
    frame: usize,
    direction: Direction,
    position: (u8, u8),
}

impl PlayerObj {
    pub fn frame(&self) -> usize {
        self.frame
    }
}

impl PlayerObj {
    pub fn get_object(&self) -> &ObjectUnmanaged {
        return &self.object;
    }
    pub fn get_direction(&self) -> Direction {
        return self.direction;
    }

    pub fn get_position(&self) -> (u8, u8) {
        return self.position;
    }

    pub fn set_direction(&mut self, new_direction: Direction) {
        self.direction = new_direction;
    }

    pub fn move_direction(&self, direction: Direction) -> (u8, u8) {
        let (x, y) = self.position;
        match direction {
            Direction::UP => (x, y.saturating_sub(1)),
            Direction::DOWN => (x, (y + 1).clamp(0, 31)),
            Direction::LEFT => (x.saturating_sub(1), y),
            Direction::RIGHT => ((x + 1).clamp(0, 31), y),
        }
    }

    pub fn set_position(&mut self, newpos: (u8, u8)) {
        agb::println!("Player @ {:?}", newpos);
        self.position = newpos;
    }

    pub fn step(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        self.object.set_sprite(self.sprites[(self.frame / FRAME_SCALE) % self.sprites.len()].clone());
        let angle: Num<i32, 8> = agb::fixnum::num!(0.25) * match self.direction {
            Direction::UP => 0,
            Direction::LEFT => 1,
            Direction::DOWN => 2,
            Direction::RIGHT => 3,
        };
        let matrix = AffineMatrix::from_rotation(angle);
        self.object.set_affine_matrix(AffineMatrixInstance::new(matrix.to_object_wrapping()));
        let (ox, oy) = self.position;
        let (ox, oy) = (ox as i32, oy as i32);
        let oy = oy - (if oy > Y_SCROLL_THRESHOLD {
            oy - Y_SCROLL_THRESHOLD
        } else { 0 }).min((MAPHEIGHT as i32) - 20);
        let pos: Vector2D<i32> = Vector2D::new(ox * 8 - 4, oy * 8 - 4);
        self.object.set_position(pos);
        self.object.set_priority(Priority::P2);
        self.object.show_affine(AffineMode::Affine);
    }
}

impl PlayerObj {
    pub fn new(sprite_loader: &mut SpriteLoader) -> Self {
        let sprites = MAN_SPRITE.sprites().iter().map(|sprite| sprite_loader.get_vram_sprite(sprite)).collect::<Vec<_>>();
        let mut object = ObjectUnmanaged::new(sprites[0].clone());
        object.set_x(100);
        object.set_y(100);
        Self {
            frame: 0,
            object,
            sprites,
            direction: Direction::UP,
            position: (0, 0),
        }
    }
}
