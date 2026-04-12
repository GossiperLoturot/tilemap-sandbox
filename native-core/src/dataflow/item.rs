pub type InventoryId = u64;

#[derive(Debug, Clone)]
pub struct ItemInfo {
    pub display_name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ItemStorageInfo {
    pub items: Vec<ItemInfo>,
}

#[derive(Debug, Clone)]
pub struct ItemArchetype {
    pub display_name: String,
    pub description: String,
}

#[derive(Debug, Clone, Default)]
pub struct Item {
    pub amount: u32,
    pub archetype_id: u16,
}

#[derive(Debug, Clone, Default)]
pub struct Inventory {
    pub max_variety: u32,
    pub max_stack: u32,
}

#[derive(Debug)]
pub struct ItemPage {
    pub version: u64,
    pub id: InventoryId,
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub struct ItemStorage {
    archetypes: Vec<ItemArchetype>,

    id_index: slab::Slab<u32>,
    pages: Vec<ItemPage>,
    inventories: Vec<Inventory>,
}

impl ItemStorage {
    pub fn new(info: ItemStorageInfo) -> Self {
        let mut archetypes = vec![];

        for item in info.items {
            archetypes.push(ItemArchetype {
                display_name: item.display_name,
                description: item.description,
            });
        }

        Self {
            archetypes,
            id_index: Default::default(),
            pages: Default::default(),
            inventories: Default::default(),
        }
    }

    // inventory

    pub fn insert_inventory(&mut self, inventory: Inventory) -> Result<InventoryId, ItemError> {
        let inventory_id = self.id_index.vacant_key() as u64;

        assert!(self.pages.len() <= u32::MAX as usize, "capacity overflow");
        let page_id = self.pages.len() as u32;
        self.pages.push(ItemPage {
            version: Default::default(),
            id: inventory_id,
            items: Default::default(),
        });
        self.inventories.push(inventory);
        self.id_index.insert(page_id);

        Ok(inventory_id)
    }

    pub fn remove_inventory(&mut self, inventory_id: InventoryId) -> Result<Inventory, ItemError> {
        let page_id = self.id_index.try_remove(inventory_id as usize).ok_or(ItemError::InventoryNotFound)?;

        let _ = self.pages.swap_remove(page_id as usize);
        let inventory = self.inventories.swap_remove(page_id as usize);

        if let Some(page) = self.pages.get(page_id as usize) {
            *self.id_index.get_mut(page.id as usize).unwrap() = page_id;
        }

        Ok(inventory)
    }

    #[inline]
    pub fn get_inventory(&mut self, inventory_id: InventoryId) -> Result<&Inventory, ItemError> {
        let page_id = *self.id_index.get(inventory_id as usize).ok_or(ItemError::InventoryNotFound)?;

        let inventory = self.inventories.get(page_id as usize).unwrap();

        Ok(inventory)
    }

    // item

    pub fn insert(&mut self, inventory_id: InventoryId, item: Item) -> Result<(), ItemError> {
        self.archetypes.get(item.archetype_id as usize).ok_or(ItemError::ItemInvalidId)?;
        let page_id = *self.id_index.get(inventory_id as usize).ok_or(ItemError::InventoryNotFound)?;

        let inventory = self.inventories.get(page_id as usize).unwrap();
        if item.amount > inventory.max_stack {
            return Err(ItemError::ItemConflict);
        }

        let page = self.pages.get_mut(page_id as usize).unwrap();
        if let Some(item_) = page.items.iter_mut().find(|v| v.archetype_id == item.archetype_id) {
            if item_.amount + item.amount > inventory.max_stack {
                return Err(ItemError::ItemConflict);
            }
            item_.amount += item.amount;
        } else {
            if (page.items.len() as u32) + 1 > inventory.max_variety {
                return Err(ItemError::ItemConflict);
            }
            page.items.push(item);
        }
        page.version += 1;
        Ok(())
    }

    pub fn remove(&mut self, inventory_id: InventoryId, item: Item) -> Result<(), ItemError> {
        self.archetypes.get(item.archetype_id as usize).ok_or(ItemError::ItemInvalidId)?;
        let page_id = *self.id_index.get(inventory_id as usize).ok_or(ItemError::InventoryNotFound)?;

        let page = self.pages.get_mut(page_id as usize).unwrap();
        if let Some(local_id) = page.items.iter().position(|v| v.archetype_id == item.archetype_id) {
            let item_ = page.items.get_mut(local_id).unwrap();
            if item_.amount < item.amount {
                return Err(ItemError::ItemConflict);
            }
            item_.amount -= item.amount;
            if item_.amount == 0 {
                page.items.swap_remove(local_id);
            }
        } else {
            return Err(ItemError::ItemConflict);
        }

        page.version += 1;
        Ok(())
    }

    // archetype

    #[inline]
    pub fn get_archetype(&self, archetype_id: u16) -> Result<&ItemArchetype, ItemError> {
        self.archetypes.get(archetype_id as usize).ok_or(ItemError::ItemInvalidId)
    }

    // transfer page data

    #[inline]
    pub fn get_page(&self, inventory_id: InventoryId) -> Result<&ItemPage, ItemError> {
        let page_id = *self.id_index.get(inventory_id as usize).ok_or(ItemError::InventoryNotFound)?;
        let page = self.pages.get(page_id as usize).unwrap();
        Ok(page)
    }
}

// error handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemError {
    ItemNotFound,
    ItemConflict,
    ItemInvalidId,

    InventoryNotFound,
    InventoryConflict,
    InventoryInvalidId,
}

impl std::fmt::Display for ItemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ItemNotFound => write!(f, "not found item error"),
            Self::ItemConflict => write!(f, "conflict item error"),
            Self::ItemInvalidId => write!(f, "invalid id item error"),

            Self::InventoryNotFound => write!(f, "not found inventory error"),
            Self::InventoryConflict => write!(f, "conflict inventory error"),
            Self::InventoryInvalidId => write!(f, "invalid id inventory error"),
        }
    }
}

impl std::error::Error for ItemError {}
