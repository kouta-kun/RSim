use core::mem;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum ItemType {
    WoodPlank,
    Fish,
}

impl ItemType {
    pub const fn variant_count() -> usize {
        mem::variant_count::<Self>()
    }
}

#[derive(Default, Copy, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Inventory {
    content: [u8; ItemType::variant_count()]
}

impl Inventory {
    pub fn item(&self, item_type: ItemType) -> &u8 {
        return &self.content[item_type as usize];
    }

    pub fn item_mut(&mut self, item_type: ItemType) -> &mut u8 {
        return &mut self.content[item_type as usize];
    }
}