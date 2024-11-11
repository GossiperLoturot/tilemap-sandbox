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
    pub items: Vec<Option<Item>>,
}

impl Inventory {
    pub fn new(item_size: u32) -> Self {
        Self {
            version: 0,
            items: vec![None; item_size as usize],
        }
    }

    pub fn get_size(&self) -> u32 {
        self.items.len() as u32
    }

    pub fn insert(&mut self, index: u32, item: Item) -> Result<(), ItemError> {
        let target = self
            .items
            .get_mut(index as usize)
            .ok_or(ItemError::InvalidIndex)?;

        if target.is_some() {
            return Err(ItemError::NotFoundItem);
        }

        let _ = std::mem::replace(target, Some(item));

        self.version += 1;
        Ok(())
    }

    pub fn remove(&mut self, index: u32) -> Result<Item, ItemError> {
        let target = self
            .items
            .get_mut(index as usize)
            .ok_or(ItemError::InvalidIndex)?;

        if target.is_none() {
            return Err(ItemError::NotFoundItem);
        }

        let item = std::mem::replace(target, None).unwrap();

        self.version += 1;
        Ok(item)
    }

    pub fn modify(&mut self, index: u32, f: impl FnOnce(&mut Item)) -> Result<(), ItemError> {
        let target = self
            .items
            .get_mut(index as usize)
            .ok_or(ItemError::InvalidIndex)?;
        let item = target.as_mut().ok_or(ItemError::NotFoundItem)?;

        let mut new_item = item.clone();
        f(&mut new_item);

        if new_item.id != item.id {
            return Err(ItemError::InvalidId);
        }

        if new_item.amount != item.amount {
            self.version += 1;
        }

        if new_item.render_param != item.render_param {
            self.version += 1;
        }

        item.data = new_item.data;

        Ok(())
    }

    pub fn get(&self, index: u32) -> Result<&Item, ItemError> {
        let target = self
            .items
            .get(index as usize)
            .ok_or(ItemError::InvalidIndex)?;
        target.as_ref().ok_or(ItemError::NotFoundItem)
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

    pub fn insert(&mut self, inventory: Inventory) -> Result<InventoryKey, ItemError> {
        let key = self.inventories.insert(inventory) as u32;
        Ok(key)
    }

    pub fn remove(&mut self, key: InventoryKey) -> Result<Inventory, ItemError> {
        self.inventories
            .try_remove(key as usize)
            .ok_or(ItemError::NotFoundInventory)
    }

    pub fn modify(
        &mut self,
        key: InventoryKey,
        f: impl FnOnce(&mut Inventory),
    ) -> Result<InventoryKey, ItemError> {
        let inventory = self
            .inventories
            .get_mut(key as usize)
            .ok_or(ItemError::NotFoundInventory)?;

        let mut new_inventory = inventory.clone();
        f(&mut new_inventory);

        inventory.items = new_inventory.items;

        Ok(key)
    }

    pub fn get(&self, key: InventoryKey) -> Result<&Inventory, ItemError> {
        self.inventories
            .get(key as usize)
            .ok_or(ItemError::NotFoundInventory)
    }
}

// error handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemError {
    NotFoundInventory,
    NotFoundItem,
    InvalidId,
    InvalidIndex,
}

impl std::fmt::Display for ItemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFoundInventory => write!(f, "not found inventory error"),
            Self::NotFoundItem => write!(f, "not found item error"),
            Self::InvalidId => write!(f, "invalid id error"),
            Self::InvalidIndex => write!(f, "invalid index error"),
        }
    }
}

impl std::error::Error for ItemError {}
