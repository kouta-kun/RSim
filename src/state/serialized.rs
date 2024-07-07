use serde::{Deserialize, Serialize};
use crate::state::inventory::Inventory;
use crate::state::map::MapData;
use crate::state::player::Direction;

#[derive(Serialize, Deserialize)]
pub struct SerializedState {
    pub map_data: MapData,
    pub inventory: Inventory,
    pub player: (u8, u8, Direction),
}