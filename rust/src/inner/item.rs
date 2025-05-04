use super::*;

// item entity

#[derive(Debug, Clone)]
pub struct ItemEntityData {
    pub item: Item,
}

#[derive(Debug, Clone)]
pub struct ItemEntityFeature;

impl EntityFeature for ItemEntityFeature {}

// item storage

pub type InventoryKey = u32;
pub type SlotKey = (InventoryKey, u32);

#[derive(Debug, Clone)]
pub struct ItemDescriptor {
    pub display_name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct InventoryDescriptor {
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct ItemStorageDescriptor {
    pub items: Vec<ItemDescriptor>,
    pub inventories: Vec<InventoryDescriptor>,
}

#[derive(Debug, Clone)]
struct ItemProperty {
    display_name: String,
    description: String,
}

#[derive(Debug, Clone)]
struct InventoryProperty {
    size: u32,
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
    pub id: u16,
    pub size: u32,
    pub slots: Vec<Slot>,
    pub version: u64,
}

#[derive(Debug, Clone)]
pub struct ItemStorage {
    item_props: Vec<ItemProperty>,
    inventory_props: Vec<InventoryProperty>,
    inventories: slab::Slab<Inventory>,
}

impl ItemStorage {
    pub fn new(desc: ItemStorageDescriptor) -> Self {
        let mut item_props = vec![];
        for item in desc.items {
            item_props.push(ItemProperty {
                display_name: item.display_name,
                description: item.description,
            });
        }

        let mut inventory_props = vec![];
        for inventory in desc.inventories {
            inventory_props.push(InventoryProperty {
                size: inventory.size,
            });
        }

        Self {
            item_props,
            inventory_props,
            inventories: Default::default(),
        }
    }

    // inventory

    pub fn insert_inventory(&mut self, id: u16) -> Result<InventoryKey, ItemError> {
        let props = self
            .inventory_props
            .get(id as usize)
            .ok_or(ItemError::InventoryInvalidId)?;

        let inventory = Inventory {
            id,
            size: props.size,
            slots: vec![Default::default(); props.size as usize],
            version: 0,
        };
        let inventory_key = self.inventories.insert(inventory) as u32;

        Ok(inventory_key)
    }

    pub fn remove_inventory(&mut self, inventory_key: InventoryKey) -> Result<u16, ItemError> {
        let inventory = self
            .inventories
            .try_remove(inventory_key as usize)
            .ok_or(ItemError::InventoryNotFound)?;
        Ok(inventory.id)
    }

    pub fn get_inventory(&self, inventory_key: InventoryKey) -> Result<&Inventory, ItemError> {
        self.inventories
            .get(inventory_key as usize)
            .ok_or(ItemError::InventoryNotFound)
    }

    // inventory + item

    pub fn push_item_to_inventory(
        &mut self,
        inventory_key: InventoryKey,
        item: Item,
    ) -> Result<(), ItemError> {
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

    pub fn pop_item_from_inventory(
        &mut self,
        inventory_key: InventoryKey,
    ) -> Result<Item, ItemError> {
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

    pub fn search_item_in_inventory(
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
                .item_props
                .get(item.id as usize)
                .ok_or(ItemError::ItemInvalidId)?
                .display_name;
            if other_text.contains(text) || text.contains(other_text) {
                let slot_key = (inventory_key, local_key as u32);
                slot_keys.push(slot_key);
            }
        }

        Ok(slot_keys)
    }

    // item

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

    pub fn swap_item(
        &mut self,
        src_slot_key: SlotKey,
        dst_slot_key: SlotKey,
    ) -> Result<(), ItemError> {
        let src_item = self.remove_item(src_slot_key);
        let dst_item = self.remove_item(dst_slot_key);

        if let Ok(item) = dst_item {
            self.insert_item(src_slot_key, item)?;
        }
        if let Ok(item) = src_item {
            self.insert_item(dst_slot_key, item)?;
        }

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

    // item property

    pub fn get_item_display_name(&self, key: SlotKey) -> Result<&str, ItemError> {
        let item = self.get_item(key)?;
        let prop = self.item_props.get(item.id as usize).unwrap();
        Ok(&prop.display_name)
    }

    pub fn get_item_description(&self, key: SlotKey) -> Result<&str, ItemError> {
        let item = self.get_item(key)?;
        let prop = self.item_props.get(item.id as usize).unwrap();
        Ok(&prop.description)
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
