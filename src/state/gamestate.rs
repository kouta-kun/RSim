use agb::display::object::SpriteLoader;
use agb::display::tiled::{MapLoan, RegularMap, TiledMap, VRamManager};
use crate::state::inventory::Inventory;
use crate::state::map::MapData;
use crate::state::player::PlayerObj;
use crate::state::serialized::SerializedState;

pub struct GameState {
    map_data: MapData,
    player_obj: PlayerObj,
    inventory: Inventory,
}

impl<'obj> GameState {
    pub fn from_save_state(sprite_loader: &'obj mut SpriteLoader, state: SerializedState) -> Self {
        let mut player_obj = PlayerObj::new(sprite_loader);
        player_obj.set_position((state.player.0, state.player.1));
        player_obj.set_direction(state.player.2);
        let state = Self {
            map_data: state.map_data,
            inventory: state.inventory,
            player_obj,
        };
        state
    }

    pub fn new(sprite_loader: &'obj mut SpriteLoader) -> Self {
        let state = Self {
            map_data: MapData::gen(),
            player_obj: PlayerObj::new(sprite_loader),
            inventory: Inventory::default(),
        };
        state
    }

    pub fn upload<'a>(&'a mut self, vram: &mut VRamManager, background: &mut MapLoan<RegularMap>) {
        vram.set_background_palettes(crate::gamemode::background::PALETTES);
        self.map_data.copy_map_to_bg(vram, background);
        background.set_scroll_pos((0i16, 0i16));
        background.set_visible(true);
        background.commit(vram);
    }

    pub fn map_data(&self) -> &MapData {
        return &self.map_data;
    }
    pub fn map_data_mut(&mut self) -> &mut MapData {
        return &mut self.map_data;
    }

    pub fn player_obj(&self) -> &PlayerObj {
        return &self.player_obj;
    }

    pub fn player_obj_mut(&mut self) -> &mut PlayerObj {
        return &mut self.player_obj;
    }
    pub fn inventory(&self) -> &Inventory {
        return &self.inventory;
    }

    pub fn inventory_mut(&mut self) -> &mut Inventory {
        return &mut self.inventory;
    }
}