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
    pub slot_node_glob: String,
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
    slot_node_glob: String,
}

struct Inventory {
    inventory_node: Gd<godot::classes::Node>,
    slot_nodes: Array<Gd<godot::classes::Node>>,
}

#[derive(GodotClass)]
#[class(no_init)]
pub(crate) struct ItemStore {
    item_props: Vec<ItemProperty>,
    inventory_props: Vec<InventoryProperty>,
    node: Gd<godot::classes::Node>,

    inventories: slab::Slab<Inventory>,
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
            inventory_props.push(InventoryProperty {
                scene: desc.scene,
                slot_node_glob: desc.slot_node_glob,
            });
        }

        Self {
            node: desc.node,
            item_props,
            inventory_props,
            inventories: slab::Slab::new(),
        }
    }

    pub fn open_inventory_by_entity(
        &mut self,
        root: &inner::Root,
        key: inner::TileKey,
    ) -> Result<u32, inner::RootError> {
        let inventory_key = root
            .entity_get_inventory(key)?
            .expect("Entity does not have inventory");
        let inventory = root.item_get_inventory(inventory_key)?;
        let prop = self
            .inventory_props
            .get(inventory.id as usize)
            .ok_or(inner::ItemError::InventoryInvalidId)?;

        let key = self.inventories.vacant_key() as u32;

        let inventory_node = prop
            .scene
            .instantiate()
            .expect("Failed to instantiate inventory");
        self.node.add_child(&inventory_node);

        let slot_nodes = inventory_node.find_children(&prop.slot_node_glob);

        let inventory = Inventory {
            inventory_node,
            slot_nodes,
        };
        self.inventories.insert(inventory);

        Ok(key)
    }

    pub fn close_inventory(&mut self, key: u32) -> Result<(), inner::ItemError> {
        let inventory = self
            .inventories
            .try_remove(key as usize)
            .ok_or(inner::ItemError::InventoryNotFound)?;
        inventory.inventory_node.free();
        Ok(())
    }
}
