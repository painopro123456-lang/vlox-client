use crate::world::Block;

pub const HOTBAR_SLOTS: usize = 4;
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
