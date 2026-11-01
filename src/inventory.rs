use crate::world::Block;
use std::io::{self, Read, Write};

pub const HOTBAR_SLOTS: usize = 9;
pub const MAX_STACK: u8 = 64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ItemStack {
    pub block: Option<Block>,
    pub count: u8,
}

impl ItemStack {
    pub const fn is_empty(self) -> bool {
        self.block.is_none() || self.count == 0
    }
}

#[derive(Clone, Debug)]
pub struct Inventory {
    slots: [ItemStack; HOTBAR_SLOTS],
    selected: usize,
}

impl Default for Inventory {
    fn default() -> Self {
        Self { slots: [ItemStack::default(); HOTBAR_SLOTS], selected: 0 }
    }
}

impl Inventory {
    pub fn selected_slot(&self) -> usize { self.selected }

    pub fn select(&mut self, slot: usize) {
        if slot < HOTBAR_SLOTS { self.selected = slot; }
    }

    pub fn selected_block(&self) -> Option<Block> {
        let stack = self.slots[self.selected];
        (!stack.is_empty()).then_some(stack.block).flatten()
    }

    pub fn stack(&self, slot: usize) -> Option<ItemStack> {
        self.slots.get(slot).copied()
    }

    pub fn add(&mut self, block: Block) -> bool {
        if block == Block::Air { return false; }
        if let Some(stack) = self.slots.iter_mut().find(|stack| stack.block == Some(block) && stack.count < MAX_STACK) {
            stack.count += 1;
            return true;
        }
        if let Some(stack) = self.slots.iter_mut().find(|stack| stack.is_empty()) {
            *stack = ItemStack { block: Some(block), count: 1 };
            return true;
        }
        false
    }

    pub fn save<W: Write>(&self, mut writer: W) -> io::Result<()> {
        writer.write_all(b"VLOXINV1")?;
        writer.write_all(&[self.selected as u8])?;
        for stack in self.slots {
            let id = stack.block.map_or(0, |block| block as u8);
            writer.write_all(&[id, stack.count])?;
        }
        Ok(())
    }

    pub fn load<R: Read>(mut reader: R) -> io::Result<Self> {
        let mut header = [0_u8; 8];
        reader.read_exact(&mut header)?;
        if &header != b"VLOXINV1" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid VLOX inventory header"));
        }
        let mut selected = [0_u8; 1];
        reader.read_exact(&mut selected)?;
        if selected[0] as usize >= HOTBAR_SLOTS {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid selected hotbar slot"));
        }

        let mut inventory = Self::default();
        inventory.selected = selected[0] as usize;
        for stack in &mut inventory.slots {
            let mut bytes = [0_u8; 2];
            reader.read_exact(&mut bytes)?;
            let block = match bytes[0] {
                0 => None,
                1 => Some(Block::Grass),
                2 => Some(Block::Dirt),
                3 => Some(Block::Stone),
                4 => Some(Block::Sand),
                5 => Some(Block::OakLog),
                6 => Some(Block::OakLeaves),
                7 => Some(Block::CoalOre),
                8 => Some(Block::IronOre),
                _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "unknown inventory block ID")),
            };
            if bytes[1] > MAX_STACK || (block.is_none() && bytes[1] != 0) || (block.is_some() && bytes[1] == 0) {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid inventory stack count"));
            }
            *stack = ItemStack { block, count: bytes[1] };
        }
        Ok(inventory)
    }

    pub fn consume_selected(&mut self) -> bool {
        let stack = &mut self.slots[self.selected];
        if stack.is_empty() { return false; }
        stack.count -= 1;
        if stack.count == 0 { *stack = ItemStack::default(); }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mining_pickups_stack_to_sixty_four() {
        let mut inventory = Inventory::default();
        for _ in 0..65 { assert!(inventory.add(Block::Dirt)); }
        assert_eq!(inventory.stack(0).unwrap().count, 64);
        assert_eq!(inventory.stack(1).unwrap().count, 1);
    }

    #[test]
    fn placing_consumes_one_item_and_clears_empty_stack() {
        let mut inventory = Inventory::default();
        assert!(inventory.add(Block::Stone));
        assert!(inventory.consume_selected());
        assert!(inventory.stack(0).unwrap().is_empty());
        assert!(!inventory.consume_selected());
    }

    #[test]
    fn inventory_save_load_preserves_stacks_and_selection() {
        let mut inventory = Inventory::default();
        inventory.add(Block::OakLog);
        inventory.add(Block::OakLog);
        inventory.add(Block::Sand);
        inventory.select(2);
        let mut bytes = Vec::new();
        inventory.save(&mut bytes).unwrap();
        let loaded = Inventory::load(bytes.as_slice()).unwrap();
        assert_eq!(loaded.selected_slot(), 2);
        assert_eq!(loaded.stack(0).unwrap().count, 2);
        assert_eq!(loaded.stack(0).unwrap().block, Some(Block::OakLog));
        assert_eq!(loaded.stack(1).unwrap().block, Some(Block::Sand));
    }

    #[test]
    fn air_cannot_be_picked_up() {
        let mut inventory = Inventory::default();
        assert!(!inventory.add(Block::Air));
    }

    #[test]
    fn selected_slot_is_bounded() {
        let mut inventory = Inventory::default();
        inventory.select(3);
        inventory.select(99);
        assert_eq!(inventory.selected_slot(), 3);
    }
}
