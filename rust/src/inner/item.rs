use super::*;

// item entity

#[derive(Debug, Clone)]
pub struct ItemEntityData {
    pub item: Item,
}

#[derive(Debug, Clone)]
pub struct ItemEntityFeature;

impl EntityFeature for ItemEntityFeature {}

// item store

pub type InventoryKey = u32;
pub type SlotKey = (InventoryKey, u32);

#[derive(Debug, Clone)]
pub struct ItemDescriptor {
    pub name_text: String,
}

#[derive(Debug, Clone)]
pub struct ItemStoreDescriptor {
    pub items: Vec<ItemDescriptor>,
}

#[derive(Debug, Clone)]
struct ItemProperty {
    name_text: String,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: u16,
    pub amount: u32,
    pub data: Box<dyn ItemData>,
    pub render_param: ItemRenderParam,
}

#[derive(Debug, Clone, Default)]
pub struct Slot {
    pub version: u64,
    pub item: Option<Item>,
}

#[derive(Debug, Clone)]
pub struct Inventory {
    pub version: u64,
    pub slots: Vec<Slot>,
}

#[derive(Debug, Clone)]
pub struct ItemStore {
    props: Vec<ItemProperty>,
    inventories: slab::Slab<Inventory>,
}

impl ItemStore {
    pub fn new(desc: ItemStoreDescriptor) -> Self {
        let mut props = vec![];
        for item in desc.items {
            props.push(ItemProperty {
                name_text: item.name_text,
            });
        }

        Self {
            props,
            inventories: Default::default(),
        }
    }

    pub fn alloc_inventory(&mut self, slot_size: u32) -> Result<InventoryKey, ItemError> {
        let inventory = Inventory {
            version: 0,
            slots: vec![Default::default(); slot_size as usize],
        };
        let inventory_key = self.inventories.insert(inventory) as u32;
        Ok(inventory_key)
    }

    pub fn free_inventory(&mut self, inventory_key: InventoryKey) -> Result<(), ItemError> {
        let _ = self
            .inventories
            .try_remove(inventory_key as usize)
            .ok_or(ItemError::InventoryNotFound)?;
        Ok(())
    }

    pub fn get_inventory(&self, inventory_key: InventoryKey) -> Result<&Inventory, ItemError> {
        self.inventories
            .get(inventory_key as usize)
            .ok_or(ItemError::InventoryNotFound)
    }

    pub fn push_item(&mut self, inventory_key: InventoryKey, item: Item) -> Result<(), ItemError> {
        let inventory = self
            .inventories
            .get_mut(inventory_key as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        let slot = inventory
            .slots
            .iter_mut()
            .find(|slot| slot.item.is_none())
            .ok_or(ItemError::ItemConflict)?;

        let _ = std::mem::replace(&mut slot.item, Some(item));
        slot.version += 1;
        inventory.version += 1;
        Ok(())
    }

    pub fn pop_item(&mut self, inventory_key: InventoryKey) -> Result<Item, ItemError> {
        let inventory = self
            .inventories
            .get_mut(inventory_key as usize)
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

    pub fn search_item(
        &self,
        inventory_key: InventoryKey,
        text: &str,
    ) -> Result<Vec<SlotKey>, ItemError> {
        let inventory = self
            .inventories
            .get(inventory_key as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        let mut slot_keys = vec![];
        for local_key in 0..inventory.slots.len() {
            let slot = &inventory.slots.get(local_key).unwrap();

            let Some(item) = &slot.item else {
                continue;
            };

            let other_text = &self
                .props
                .get(item.id as usize)
                .ok_or(ItemError::ItemInvalidId)?
                .name_text;
            if other_text.contains(text) || text.contains(other_text) {
                let slot_key = (inventory_key, local_key as u32);
                slot_keys.push(slot_key);
            }
        }

        Ok(slot_keys)
    }

    pub fn insert_item(&mut self, slot_key: SlotKey, item: Item) -> Result<(), ItemError> {
        let (inventory_key, local_key) = slot_key;

        let inventory = self
            .inventories
            .get_mut(inventory_key as usize)
            .ok_or(ItemError::InventoryNotFound)?;
        let slot = inventory
            .slots
            .get_mut(local_key as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        if slot.item.is_some() {
            return Err(ItemError::ItemNotFound);
        }

        let _ = std::mem::replace(&mut slot.item, Some(item));
        slot.version += 1;
        inventory.version += 1;
        Ok(())
    }

    pub fn remove_item(&mut self, slot_key: SlotKey) -> Result<Item, ItemError> {
        let (inventory_key, local_key) = slot_key;

        let inventory = self
            .inventories
            .get_mut(inventory_key as usize)
            .ok_or(ItemError::InventoryNotFound)?;
        let slot = inventory
            .slots
            .get_mut(local_key as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        if slot.item.is_none() {
            return Err(ItemError::ItemNotFound);
        }

        let item = slot.item.take().unwrap();

        slot.version += 1;
        inventory.version += 1;
        Ok(item)
    }

    pub fn modify_item(
        &mut self,
        slot_key: SlotKey,
        f: impl FnOnce(&mut Item),
    ) -> Result<(), ItemError> {
        let (inventory_key, local_key) = slot_key;

        let inventory = self
            .inventories
            .get_mut(inventory_key as usize)
            .ok_or(ItemError::InventoryNotFound)?;
        let slot = inventory
            .slots
            .get_mut(local_key as usize)
            .ok_or(ItemError::InventoryNotFound)?;
        let item = slot.item.as_mut().ok_or(ItemError::ItemNotFound)?;

        let mut new_item = item.clone();
        f(&mut new_item);

        if new_item.id != item.id {
            return Err(ItemError::ItemInvalidId);
        }

        if new_item.amount != item.amount || new_item.render_param != item.render_param {
            slot.version += 1;
            inventory.version += 1;
        }

        item.data = new_item.data;
        Ok(())
    }

    pub fn get_item(&self, slot_key: SlotKey) -> Result<&Item, ItemError> {
        let (inventory_key, local_key) = slot_key;

        let inventory = self
            .inventories
            .get(inventory_key as usize)
            .ok_or(ItemError::InventoryNotFound)?;
        let slot = inventory
            .slots
            .get(local_key as usize)
            .ok_or(ItemError::InventoryNotFound)?;

        if slot.item.is_none() {
            return Err(ItemError::ItemNotFound);
        }

        let item = slot.item.as_ref().ok_or(ItemError::ItemNotFound)?;
        Ok(item)
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
