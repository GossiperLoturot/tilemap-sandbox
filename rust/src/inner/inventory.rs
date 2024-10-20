use crate::inner;

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

    pub fn get_item_size(&self) -> u32 {
        self.items.len() as u32
    }
}

#[derive(Debug, Clone)]
pub struct InventoryResource {
    inventories: slab::Slab<Inventory>,
}

impl InventoryResource {
    pub fn init(root: &mut inner::Root) {
        let slf = Self {
            inventories: Default::default(),
        };
        root.resource_insert(slf).unwrap();
    }

    pub fn insert(root: &mut inner::Root, inventory: Inventory) -> InventoryKey {
        let slf = root.resource_get_mut::<Self>().unwrap();
        slf.inventories.insert(inventory) as u32
    }

    pub fn remove(root: &mut inner::Root, key: InventoryKey) -> Option<Inventory> {
        let slf = root.resource_get_mut::<Self>().unwrap();
        slf.inventories.try_remove(key as usize)
    }
}
