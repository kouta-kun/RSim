use agb::display::tiled::{MapLoan, RegularMap, VRamManager};
use rand_xoshiro::SplitMix64;
use core::fmt::{Debug, Formatter, Write};
use core::mem::size_of;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use crate::gamemode;
use crate::gamemode::{MAPHEIGHT, TREECOUNT};

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct MapData {
    pub map: [u32; MAPHEIGHT],
    pub bridge: [u32; MAPHEIGHT],
    pub tree_positions: [(u16, u16, i8); TREECOUNT],
}

impl Debug for MapData {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for y in 0..MAPHEIGHT {
            for x in 0..size_of::<u32>() * 8 {
                f.write_char(if self.get_terrain_point((x, y)) { '*' } else { '#' })?
            }
            f.write_char('\n')?
        }

        Ok(())
    }
}

impl MapData {
    pub fn get_tree_positions(&self) -> &[(u16, u16, i8); TREECOUNT] {
        return &self.tree_positions;
    }
    pub fn get_tree_positions_mut(&mut self) -> &mut [(u16, u16, i8); TREECOUNT] {
        return &mut self.tree_positions;
    }
    pub fn copy_map_to_bg(&self, mut vram: &mut VRamManager, bg: &mut MapLoan<RegularMap>) {
        for y in 0..32u16 {
            for x in 0..32u16 {
                let is_water = self.get_terrain_point((x as usize, y as usize));
                let is_bridge = self.get_bridge_point((x as usize, y as usize));
                let tileid = match (is_water, is_bridge) {
                    (_, true) => 2,
                    (false, false) => 0,
                    (true, false) => 1,
                };
                bg.set_tile(
                    &mut vram, (x, y),
                    &gamemode::background::tiles.tiles,
                    gamemode::background::tiles.tile_settings[tileid],
                );
            }
        }
    }

    pub fn gen() -> Self {
        let mut mix64 = rand_xoshiro::SplitMix64::seed_from_u64(0x7af07af07af07af0u64);
        let mut points: [u8; 9] = [0; 9];
        let start_point: u8 =
            (15i8 + 6i8 - (mix64.gen::<u8>() % 12u8) as i8) as u8;
        let end_point: u8 =
            (15i8 + 6i8 - (mix64.gen::<u8>() % 12u8) as i8) as u8;
        points[0] = start_point;
        points[8] = end_point;

        let from = 0;
        let to = 8;

        Self::evaluate_midpoint(from, to, &mut points, &mut mix64);

        let mut data = Self {
            map: [0; MAPHEIGHT],
            bridge: [0; MAPHEIGHT],
            tree_positions: [(0, 0, -1); TREECOUNT],
        };
        data.draw_river(points);

        for i in 0..TREECOUNT {
            let (x, y) = loop {
                let x: u16 = mix64.gen::<u16>() % MAPHEIGHT as u16;
                let y: u16 = mix64.gen::<u16>() % 24;
                if !(data.get_terrain_point((x as usize, y as usize)) || data.has_tree((x, y))) {
                    break (x, y);
                }
            };
            data.tree_positions[i] = (x, y, 1);
            agb::println!("{} = ({}, {})", i, x, y);
        }

        data
    }

    fn _set_u32_matrix(values: &mut [u32; 32], point: (usize, usize), value: bool) {
        let (x, y) = point;
        if x >= 32 || y >= 32 {
            panic!("Out of bounds!!! {:?}", (x, y));
        }
        let mut row = &mut values[y];
        if value {
            *row |= 1 << x;
        } else {
            *row &= !(1 << x);
        }
    }

    pub fn set_terrain_point(&mut self, point: (usize, usize), value: bool) {
        return Self::_set_u32_matrix(&mut self.map, point, value);
    }

    pub fn set_bridge_point(&mut self, point: (usize, usize), value: bool) {
        return Self::_set_u32_matrix(&mut self.bridge, point, value);
    }

    fn _get_u32_matrix(values: &[u32; 32], point: (usize, usize)) -> bool {
        let (x, y) = point;
        let row = values[y];
        return (row & (1 << x)) > 0;
    }

    pub fn get_terrain_point(&self, point: (usize, usize)) -> bool {
        return Self::_get_u32_matrix(&self.map, point);
    }

    pub fn get_bridge_point(&self, point: (usize, usize)) -> bool {
        return Self::_get_u32_matrix(&self.bridge, point);
    }

    fn draw_river<const COUNT: usize>(&mut self, river_points: [u8; COUNT]) {
        for i in 0..COUNT - 1 {
            let x1 = river_points[i] as usize;
            let y1 = (MAPHEIGHT * i) / (COUNT - 1);

            let x2 = river_points[i + 1] as usize;
            let y2 = (MAPHEIGHT * (i + 1)) / (COUNT - 1);


            for (x, y) in bresenham::Bresenham::new((x1 as isize, y1 as isize), (x2 as isize, y2 as isize)) {
                self.set_terrain_point((x as usize, y as usize), true);
                self.set_terrain_point(((x - 1) as usize, y as usize), true);
                self.set_terrain_point(((x + 1) as usize, y as usize), true);
            }
        }
    }

    fn evaluate_midpoint<const COUNT: usize>(from: usize, to: usize, points: &mut [u8; COUNT], mix64: &mut SplitMix64) {
        if to - from <= 1 { return; }
        let xi: usize = (from + to) / 2;

        let av = points[from];
        let bv = points[to];
        let xv = (av + bv) / 2;

        let noise = 4i8 - (mix64.gen::<u8>() % 8) as i8;
        let nxv = ((xv as i8) + noise).clamp(0, 31) as u8;

        points[xi] = nxv;
        Self::evaluate_midpoint(from, xi, points, mix64);
        Self::evaluate_midpoint(xi, to, points, mix64);
    }
    pub fn has_tree(&self, point: (u16, u16)) -> bool {
        for i in 0..TREECOUNT {
            if self.tree_positions[i].2 > 0 && (self.tree_positions[i].0, self.tree_positions[i].1) == point {
                return true;
            }
        }
        return false;
    }
}
