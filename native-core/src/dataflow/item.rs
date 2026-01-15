use super::*;

// item storage

pub type InventoryId = u32;
pub type SlotId = (InventoryId, u32);

#[derive(Debug, Clone)]
pub struct ItemInfo {
    pub display_name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct InventoryInfo {
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct ItemStorageInfo {
    pub items: Vec<ItemInfo>,
    pub inventories: Vec<InventoryInfo>,
}

#[derive(Debug, Clone)]
pub struct ItemArchetype {
    pub display_name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct InventoryArchetype {
    pub size: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemRenderState {
    pub variant: u8,
    pub tick: u32,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub archetype_id: u16,
    pub amount: u32,
    pub data: Box<dyn ItemData>,
    pub render_param: ItemRenderState,
}

#[derive(Debug, Clone, Default)]
pub struct Slot {
    pub version: u64,
    pub item: Option<Item>,
}

#[derive(Debug, Clone)]
pub struct Inventory {
    pub archetype_id: u16,
    pub size: u32,
    pub slots: Vec<Slot>,
    pub version: u64,
}

#[derive(Debug, Clone)]
pub struct ItemStorage {
    item_archetypes: Vec<ItemArchetype>,
    inventory_archetypes: Vec<InventoryArchetype>,
    inventories: slab::Slab<Inventory>,
}

impl ItemStorage {
    pub fn new(info: ItemStorageInfo) -> Self {
        let mut item_archetypes = vec![];
        for info in info.items {
            item_archetypes.push(ItemArchetype {
                display_name: info.display_name,
                description: info.description,
            });
        }

        let mut inventory_archetypes = vec![];
        for info in info.inventories {
            inventory_archetypes.push(InventoryArchetype {
                size: info.size,
            });
        }

        Self {
            item_archetypes,
            inventory_archetypes,
            inventories: Default::default(),
        }
    }

    // inventory

    pub fn insert_inventory(&mut self, archetype_id: u16) -> Result<InventoryId, ItemError> {
        let archetype = self
            .inventory_archetypes
            .get(archetype_id as usize)
            .ok_or(ItemError::InventoryInvalidId)?;

        let inventory = Inventory {
            archetype_id,
            size: archetype.size,
            slots: vec![Default::default(); archetype.size as usize],
            version: 0,
        };
        let inventory_id = self.inventories.insert(inventory) as u32;

        Ok(inventory_id)
    }

    pub fn remove_inventory(&mut self, inventory_id: InventoryId) -> Result<u16, ItemError> {
        let inventory = self
            .inventories
            .try_remove(inventory_id as usize)
            .ok_or(ItemError::InventoryNotFound)?;
        Ok(inventory.archetype_id)
    }

    pub fn get_inventory(&self, inventory_id: InventoryId) -> Result<&Inventory, ItemError> {
        self.inventories
            .get(inventory_id as usize)
            .ok_or(ItemError::InventoryNotFound)
    }

    // inventory + item

    pub fn push_item_to_inventory(&mut self, inventory_id: InventoryId, item: Item) -> Result<(), ItemError> {
        let inventory = self
            .inventories
            .get_mut(inventory_id as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        let slot = inventory
            .slots
            .iter_mut()
            .find(|slot| slot.item.is_none())
            .ok_or(ItemError::ItemConflict)?;

        let _ = slot.item.replace(item);
        slot.version += 1;
        inventory.version += 1;
        Ok(())
    }

    pub fn pop_item_from_inventory(&mut self, inventory_id: InventoryId) -> Result<Item, ItemError> {
        let inventory = self
            .inventories
            .get_mut(inventory_id as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        let slot = inventory
            .slots
            .iter_mut()
            .find(|slot| slot.item.is_some())
            .ok_or(ItemError::ItemNotFound)?;

        let item = slot.item.take().unwrap();
        slot.version += 1;
        inventory.version += 1;
        Ok(item)
    }

    // item

    pub fn insert_item(&mut self, slot_id: SlotId, item: Item) -> Result<(), ItemError> {
        let (inventory_id, local_id) = slot_id;

        let inventory = self
            .inventories
            .get_mut(inventory_id as usize)
            .ok_or(ItemError::InventoryNotFound)?;
        let slot = inventory
            .slots
            .get_mut(local_id as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        if slot.item.is_some() {
            return Err(ItemError::ItemNotFound);
        }

        let _ = slot.item.replace(item);
        slot.version += 1;
        inventory.version += 1;
        Ok(())
    }

    pub fn remove_item(&mut self, slot_id: SlotId) -> Result<Item, ItemError> {
        let (inventory_id, local_id) = slot_id;

        let inventory = self
            .inventories
            .get_mut(inventory_id as usize)
            .ok_or(ItemError::InventoryNotFound)?;
        let slot = inventory
            .slots
            .get_mut(local_id as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        if slot.item.is_none() {
            return Err(ItemError::ItemNotFound);
        }

        let item = slot.item.take().unwrap();

        slot.version += 1;
        inventory.version += 1;
        Ok(item)
    }

    pub fn modify_item(&mut self, slot_id: SlotId, f: impl FnOnce(&mut Item)) -> Result<(), ItemError> {
        let (inventory_id, local_id) = slot_id;

        let inventory = self
            .inventories
            .get_mut(inventory_id as usize)
            .ok_or(ItemError::InventoryNotFound)?;
        let slot = inventory
            .slots
            .get_mut(local_id as usize)
            .ok_or(ItemError::InventoryNotFound)?;
        let item = slot.item.as_mut().ok_or(ItemError::ItemNotFound)?;

        let mut new_item = item.clone();
        f(&mut new_item);

        if new_item.archetype_id != item.archetype_id {
            return Err(ItemError::ItemInvalidId);
        }

        if new_item.amount != item.amount || new_item.render_param != item.render_param {
            slot.version += 1;
            inventory.version += 1;
        }

        item.data = new_item.data;
        Ok(())
    }

    pub fn swap_item(&mut self, src_slot_id: SlotId, dst_slot_id: SlotId) -> Result<(), ItemError> {
        let src_item = self.remove_item(src_slot_id);
        let dst_item = self.remove_item(dst_slot_id);

        if let Ok(item) = dst_item {
            self.insert_item(src_slot_id, item)?;
        }
        if let Ok(item) = src_item {
            self.insert_item(dst_slot_id, item)?;
        }

        Ok(())
    }

    pub fn get_item(&self, slot_id: SlotId) -> Result<&Item, ItemError> {
        let (inventory_id, local_id) = slot_id;

        let inventory = self
            .inventories
            .get(inventory_id as usize)
            .ok_or(ItemError::InventoryNotFound)?;
        let slot = inventory
            .slots
            .get(local_id as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        if slot.item.is_none() {
            return Err(ItemError::ItemNotFound);
        }

        let item = slot.item.as_ref().ok_or(ItemError::ItemNotFound)?;
        Ok(item)
    }

    // archetype

    pub fn get_item_archetype(&self, archetype_id: u16) -> Result<&ItemArchetype, ItemError> {
        self.item_archetypes.get(archetype_id as usize).ok_or(ItemError::ItemInvalidId)
    }
}

// error handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemError {
    ItemNotFound,
    ItemConflict,
    ItemInvalidId,
    InventoryNotFound,
    InventoryInvalidId,
}

impl std::fmt::Display for ItemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ItemNotFound => write!(f, "not found item error"),
            Self::ItemConflict => write!(f, "conflict item error"),
            Self::ItemInvalidId => write!(f, "invalid id item error"),
            Self::InventoryNotFound => write!(f, "not found inventory error"),
            Self::InventoryInvalidId => write!(f, "invalid id inventory error"),
        }
    }
}

impl std::error::Error for ItemError {}

// tests
// TODO: minimize test code using by fn, macro, etc.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_size() {
        println!("Item: {}B", std::mem::size_of::<Item>());
    }
}
