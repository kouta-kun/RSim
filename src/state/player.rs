use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

pub struct PlayerObj {
    direction: Direction,
    position: (u8, u8),
}

impl PlayerObj {
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
}

impl PlayerObj {
    pub fn new() -> Self {
        Self {
            direction: Direction::UP,
            position: (0, 0),
        }
    }
}
