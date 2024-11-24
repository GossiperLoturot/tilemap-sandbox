use super::*;

pub type InventoryKey = u32;

#[derive(Debug, Clone)]
pub struct ItemDescriptor {}

#[derive(Debug, Clone)]
pub struct ItemStoreDescriptor {
    pub items: Vec<ItemDescriptor>,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: u32,
    pub amount: u32,
    pub data: Vec<Option<ItemData>>,
    pub render_param: ItemRenderParam,
}

#[derive(Debug, Clone)]
pub struct Inventory {
    pub version: u64,
    pub slots: Vec<Option<Item>>,
}

impl Inventory {
    pub fn new(item_size: u32) -> Self {
        Self {
            version: 0,
            slots: vec![None; item_size as usize],
        }
    }

    pub fn slot_size(&self) -> u32 {
        self.slots.len() as u32
    }

    pub fn insert(&mut self, index: u32, item: Item) -> Result<(), ItemError> {
        let slot = self
            .slots
            .get_mut(index as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        if slot.is_some() {
            return Err(ItemError::ItemNotFound);
        }

        let _ = std::mem::replace(slot, Some(item));
        self.version += 1;
        Ok(())
    }

    pub fn remove(&mut self, index: u32) -> Result<Item, ItemError> {
        let slot = self
            .slots
            .get_mut(index as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        if slot.is_none() {
            return Err(ItemError::ItemNotFound);
        }

        let item = slot.take().unwrap();

        self.version += 1;
        Ok(item)
    }

    pub fn modify_item(&mut self, index: u32, f: impl FnOnce(&mut Item)) -> Result<(), ItemError> {
        let slot = self
            .slots
            .get_mut(index as usize)
            .ok_or(ItemError::InventoryNotFound)?;
        let item = slot.as_mut().ok_or(ItemError::ItemNotFound)?;

        let mut new_item = item.clone();
        f(&mut new_item);

        if new_item.id != item.id {
            return Err(ItemError::ItemInvalidId);
        }

        if new_item.amount != item.amount || new_item.render_param != item.render_param {
            self.version += 1;
        }

        item.data = new_item.data;
        Ok(())
    }

    pub fn get_item(&self, index: u32) -> Result<&Item, ItemError> {
        let slot = self
            .slots
            .get(index as usize)
            .ok_or(ItemError::InventoryNotFound)?;
        slot.as_ref().ok_or(ItemError::ItemNotFound)
    }
}

#[derive(Debug, Clone)]
pub struct ItemStore {
    inventories: slab::Slab<Inventory>,
}

impl ItemStore {
    pub fn new(desc: ItemStoreDescriptor) -> Self {
        Self {
            inventories: Default::default(),
        }
    }

    pub fn insert_inventory(&mut self, inventory: Inventory) -> Result<InventoryKey, ItemError> {
        let inventory_key = self.inventories.insert(inventory) as u32;
        Ok(inventory_key)
    }

    pub fn remove_inventory(
        &mut self,
        inventory_key: InventoryKey,
    ) -> Result<Inventory, ItemError> {
        self.inventories
            .try_remove(inventory_key as usize)
            .ok_or(ItemError::InventoryNotFound)
    }

    pub fn modify_inventory(
        &mut self,
        inventory_key: InventoryKey,
        f: impl FnOnce(&mut Inventory),
    ) -> Result<InventoryKey, ItemError> {
        let inventory = self
            .inventories
            .get_mut(inventory_key as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        let mut new_inventory = inventory.clone();
        f(&mut new_inventory);

        inventory.slots = new_inventory.slots;

        Ok(inventory_key)
    }

    pub fn get_inventory(&self, inventory_key: InventoryKey) -> Result<&Inventory, ItemError> {
        self.inventories
            .get(inventory_key as usize)
            .ok_or(ItemError::InventoryNotFound)
    }
}

// error handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemError {
    ItemNotFound,
    ItemConflict,
    ItemInvalidId,
    InventoryNotFound,
}

impl std::fmt::Display for ItemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ItemNotFound => write!(f, "not found item error"),
            Self::ItemConflict => write!(f, "conflict item error"),
            Self::ItemInvalidId => write!(f, "invalid id error"),
            Self::InventoryNotFound => write!(f, "not found inventory error"),
        }
    }
}

impl std::error::Error for ItemError {}
