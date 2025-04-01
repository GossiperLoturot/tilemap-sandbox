use godot::prelude::*;

use crate::inner;

pub(crate) struct ItemImageDescriptor {
    pub frames: Vec<Gd<godot::classes::Image>>,
    pub step_tick: u16,
    pub is_loop: bool,
}

pub(crate) struct ItemDescriptor {
    pub name_text: String,
    pub desc_text: String,
    pub image: ItemImageDescriptor,
}

pub(crate) struct InventoryDescriptor {
    pub scene: Gd<PackedScene>,
}

pub(crate) struct ItemStoreDescriptor {
    pub items: Vec<ItemDescriptor>,
    pub inventories: Vec<InventoryDescriptor>,
    pub node: Gd<godot::classes::Node>,
}

struct ItemProperty {
    name_text: String,
    desc_text: String,
    image: ItemImageDescriptor,
}

struct InventoryProperty {
    scene: Gd<PackedScene>,
}

#[derive(GodotClass)]
#[class(no_init)]
pub(crate) struct ItemStore {
    item_props: Vec<ItemProperty>,
    inventory_props: Vec<InventoryProperty>,
    node: Gd<godot::classes::Node>,
}

impl ItemStore {
    pub fn new(desc: ItemStoreDescriptor) -> Self {
        let mut item_props = vec![];
        for desc in desc.items {
            item_props.push(ItemProperty {
                name_text: desc.name_text,
                desc_text: desc.desc_text,
                image: desc.image,
            });
        }

        let mut inventory_props = vec![];
        for desc in desc.inventories {
            inventory_props.push(InventoryProperty { scene: desc.scene });
        }

        Self {
            node: desc.node,
            item_props,
            inventory_props,
        }
    }

    pub fn open_inventory_by_tile(
        &mut self,
        root: &inner::Root,
        tile_key: inner::TileKey,
    ) -> Result<(), inner::RootError> {
        let inventory_key = root
            .tile_get_inventory(tile_key)?
            .expect("Tile does not have inventory");
        self.open_inventory(root, inventory_key)?;
        Ok(())
    }

    pub fn open_inventory_by_block(
        &mut self,
        root: &inner::Root,
        block_key: inner::BlockKey,
    ) -> Result<(), inner::RootError> {
        let inventory_key = root
            .block_get_inventory(block_key)?
            .expect("Block does not have inventory");
        self.open_inventory(root, inventory_key)?;
        Ok(())
    }

    pub fn open_inventory_by_entity(
        &mut self,
        root: &inner::Root,
        tile_key: inner::TileKey,
    ) -> Result<(), inner::RootError> {
        let inventory_key = root
            .entity_get_inventory(tile_key)?
            .expect("Entity does not have inventory");
        self.open_inventory(root, inventory_key)?;
        Ok(())
    }

    fn open_inventory(
        &mut self,
        root: &inner::Root,
        inventory_key: inner::InventoryKey,
    ) -> Result<(), inner::ItemError> {
        let inventory = root.item_get_inventory(inventory_key)?;
        let prop = self
            .inventory_props
            .get(inventory.id as usize)
            .ok_or(inner::ItemError::InventoryInvalidId)?;

        let mut inventory_node = prop
            .scene
            .instantiate()
            .expect("Failed to instantiate inventory");
        self.node.add_child(&inventory_node);

        // invoke after method
        inventory_node.call("set_inventory_key", &[inventory_key.to_variant()]);

        Ok(())
    }
}
