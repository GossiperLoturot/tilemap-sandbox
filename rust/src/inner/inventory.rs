use crate::inner;

use super::*;

pub type InventoryKey = u32;

#[derive(Debug, Clone)]
pub struct Item {
    pub id: u32,
    pub amount: f32,
}

#[derive(Debug, Clone)]
pub struct Inventory {
    items: Vec<Option<Item>>,
}

impl Inventory {
    pub fn new(item_size: u32) -> Self {
        Self {
            items: vec![None; item_size as usize],
        }
    }

    pub fn get_size(&self) -> u32 {
        self.items.len() as u32
    }
}

// resource

#[derive(Debug, Clone)]
pub struct InventoryResource {
    inventories: slab::Slab<Inventory>,
}

impl InventoryResource {
    pub fn init(root: &mut inner::Root) -> Result<(), InventoryError> {
        let slf = Self {
            inventories: Default::default(),
        };
        root.resource_insert(slf)?;
        Ok(())
    }

    pub fn insert(
        root: &mut inner::Root,
        inventory: Inventory,
    ) -> Result<InventoryKey, InventoryError> {
        let slf = root.resource_get_mut::<Self>()?;
        let key = slf.inventories.insert(inventory) as u32;
        Ok(key)
    }

    pub fn remove(root: &mut inner::Root, key: InventoryKey) -> Result<Inventory, InventoryError> {
        let slf = root.resource_get_mut::<Self>()?;
        slf.inventories
            .try_remove(key as usize)
            .ok_or(InventoryError::NotFoundInventory)
    }
}

// error handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InventoryError {
    Resource(ResourceError),
    NotFoundInventory,
}

impl std::fmt::Display for InventoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resource(e) => e.fmt(f),
            Self::NotFoundInventory => write!(f, "not found inventory error"),
        }
    }
}

impl std::error::Error for InventoryError {}

impl From<ResourceError> for InventoryError {
    fn from(value: ResourceError) -> Self {
        Self::Resource(value)
    }
}