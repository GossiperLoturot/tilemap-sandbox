use glam::*;

// item storage

pub type InventoryId = u32;

pub type ItemId = (InventoryId, u32);

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
pub struct InventorySystemInfo {
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

#[derive(Debug, Clone, Default)]
pub struct ItemModify {
    pub variant: u8,
    pub tick: u32,
}

#[derive(Debug, Clone, Default)]
pub struct Item {
    pub archetype_id: u16,
    pub amount: u32,
    pub variant: u8,
    pub tick: u32,
}

#[derive(Debug, Clone, Default)]
pub struct Inventory {
    pub archetype_id: u16,
    pub size: u32,
    pub items: Vec<Option<Item>>,
    pub version: u64,
}

#[derive(Debug)]
pub struct InventorySystem {
    item_archetypes: Vec<ItemArchetype>,
    inventory_archetypes: Vec<InventoryArchetype>,
    inventories: slab::Slab<Inventory>,
}

impl InventorySystem {
    pub fn new(info: InventorySystemInfo) -> Self {
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
            items: vec![None; archetype.size as usize],
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

        let item_mut = inventory
            .items
            .iter_mut()
            .find(|item| item.is_none())
            .ok_or(ItemError::ItemConflict)?;

        let _ = item_mut.replace(item);
        inventory.version += 1;
        Ok(())
    }

    pub fn pop_item_from_inventory(&mut self, inventory_id: InventoryId) -> Result<Item, ItemError> {
        let inventory = self
            .inventories
            .get_mut(inventory_id as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        let item_mut = inventory
            .items
            .iter_mut()
            .find(|item| item.is_some())
            .ok_or(ItemError::ItemNotFound)?;

        let item = item_mut.take().unwrap();
        inventory.version += 1;
        Ok(item)
    }

    // item

    pub fn insert_item(&mut self, item_id: ItemId, item: Item) -> Result<(), ItemError> {
        let (inventory_id, local_id) = item_id;

        let inventory = self
            .inventories
            .get_mut(inventory_id as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        let item_mut = inventory
            .items
            .get_mut(local_id as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        if item_mut.is_some() {
            return Err(ItemError::ItemNotFound);
        }

        let _ = item_mut.replace(item);
        inventory.version += 1;
        Ok(())
    }

    pub fn remove_item(&mut self, item_id: ItemId) -> Result<Item, ItemError> {
        let (inventory_id, local_id) = item_id;

        let inventory = self
            .inventories
            .get_mut(inventory_id as usize)
            .ok_or(ItemError::InventoryNotFound)?;
        let item_mut = inventory
            .items
            .get_mut(local_id as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        if item_mut.is_none() {
            return Err(ItemError::ItemNotFound);
        }

        let item = item_mut.take().unwrap();
        inventory.version += 1;
        Ok(item)
    }

    pub fn modify_item(&mut self, item_id: ItemId, f: impl FnOnce(&mut ItemModify)) -> Result<(), ItemError> {
        let (inventory_id, local_id) = item_id;

        let inventory = self
            .inventories
            .get_mut(inventory_id as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        let item_mut = inventory
            .items
            .get_mut(local_id as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        let item = item_mut.as_mut().ok_or(ItemError::ItemNotFound)?;
        let mut item_modify = ItemModify { variant: item.variant, tick: item.tick };
        f(&mut item_modify);
        item.variant = item_modify.variant;
        item.tick = item_modify.tick;
        inventory.version += 1;
        Ok(())
    }

    pub fn swap_item(&mut self, from_item_id: ItemId, to_item_id: ItemId) -> Result<(), ItemError> {
        let from_item = self.remove_item(from_item_id);
        let to_item = self.remove_item(to_item_id);

        if let Ok(item) = to_item {
            self.insert_item(from_item_id, item)?;
        }
        if let Ok(item) = from_item {
            self.insert_item(to_item_id, item)?;
        }

        Ok(())
    }

    pub fn get_item(&self, item_id: ItemId) -> Result<&Item, ItemError> {
        let (inventory_id, local_id) = item_id;

        let inventory = self
            .inventories
            .get(inventory_id as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        let item_ref = inventory
            .items
            .get(local_id as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        if item_ref.is_none() {
            return Err(ItemError::ItemNotFound);
        }

        let item = item_ref.as_ref().ok_or(ItemError::ItemNotFound)?;
        Ok(item)
    }

    // archetype

    pub fn get_item_archetype(&self, archetype_id: u16) -> Result<&ItemArchetype, ItemError> {
        self.item_archetypes.get(archetype_id as usize).ok_or(ItemError::ItemInvalidId)
    }

    pub fn get_inventory_archetype(&self, archetype_id: u16) -> Result<&InventoryArchetype, ItemError> {
        self.inventory_archetypes.get(archetype_id as usize).ok_or(ItemError::InventoryInvalidId)
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
