use glam::*;
use godot::prelude::*;

use crate::inner;

pub type InventoryFn =
    Box<dyn Fn(Gd<godot::classes::Node>, Gd<godot::classes::Node>, inner::InventoryKey)>;

pub(crate) struct ItemImageDescriptor {
    pub frames: Vec<Gd<godot::classes::Image>>,
    pub step_tick: u16,
    pub is_loop: bool,
}

pub(crate) struct ItemDescriptor {
    pub name_text: String,
    pub desc_text: String,
    pub images: Vec<ItemImageDescriptor>,
}

pub(crate) struct InventoryDescriptor {
    pub scene: Gd<godot::classes::PackedScene>,
    pub callback: InventoryFn,
}

pub(crate) struct ItemStoreDescriptor {
    pub items: Vec<ItemDescriptor>,
    pub inventories: Vec<InventoryDescriptor>,
    pub ui: Gd<godot::classes::Node>,
}

struct ImageHead {
    start_texcoord_id: u32,
    end_texcoord_id: u32,
    step_tick: u16,
    is_loop: bool,
}

struct ItemProperty {
    name_text: String,
    desc_text: String,
}

struct InventoryProperty {
    pub scene: Gd<godot::classes::PackedScene>,
    pub callback: InventoryFn,
}

pub(crate) struct ItemStore {
    item_props: Vec<ItemProperty>,
    inventory_props: Vec<InventoryProperty>,
    ui: Gd<godot::classes::Node>,
    image_heads: Vec<Vec<ImageHead>>,
    textures: Vec<Rid>,
    free_handles: Vec<Rid>,
}

impl ItemStore {
    pub fn new(desc: ItemStoreDescriptor) -> Self {
        let mut rendering_server = godot::classes::RenderingServer::singleton();

        let mut free_handles = vec![];

        let mut item_props = vec![];
        let mut image_heads = vec![];
        let mut textures = vec![];
        for desc in desc.items {
            item_props.push(ItemProperty {
                name_text: desc.name_text.clone(),
                desc_text: desc.desc_text.clone(),
            });

            let mut sub_image_heads = vec![];

            for image in desc.images {
                if textures.len() + image.frames.len() >= i32::MAX as usize {
                    panic!("number of frame must be less than i32::MAX");
                }

                sub_image_heads.push(ImageHead {
                    start_texcoord_id: textures.len() as u32,
                    end_texcoord_id: (textures.len() + image.frames.len()) as u32,
                    step_tick: image.step_tick,
                    is_loop: image.is_loop,
                });

                for frame in image.frames {
                    let texture = rendering_server.texture_2d_create(&frame);
                    free_handles.push(texture);

                    textures.push(texture);
                }
            }

            image_heads.push(sub_image_heads);
        }

        let mut inventory_props = vec![];
        for desc in desc.inventories {
            inventory_props.push(InventoryProperty {
                scene: desc.scene.clone(),
                callback: desc.callback,
            });
        }

        Self {
            ui: desc.ui,
            item_props,
            inventory_props,
            image_heads,
            textures,
            free_handles,
        }
    }

    pub fn open_inventory_by_tile(
        &mut self,
        root: &inner::Root,
        tile_key: inner::TileKey,
    ) -> Result<(), inner::RootError> {
        let inventory_key = root
            .tile_get_inventory(tile_key)?
            .ok_or(inner::ItemError::InventoryNotFound)?;
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
            .ok_or(inner::ItemError::InventoryNotFound)?;
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
            .ok_or(inner::ItemError::InventoryNotFound)?;
        self.open_inventory(root, inventory_key)?;
        Ok(())
    }

    pub fn open_inventory(
        &mut self,
        root: &inner::Root,
        inventory_key: inner::InventoryKey,
    ) -> Result<(), inner::ItemError> {
        let inventory = root.item_get_inventory(inventory_key)?;
        let prop = self
            .inventory_props
            .get(inventory.id as usize)
            .ok_or(inner::ItemError::InventoryInvalidId)?;

        let instance = prop
            .scene
            .instantiate()
            .expect("Failed to instantiate inventory");
        self.ui.add_child(&instance);

        // invoke after method
        (*prop.callback)(self.ui.clone(), instance, inventory_key);

        Ok(())
    }

    pub fn has_item(
        &mut self,
        root: &inner::Root,
        slot_key: inner::SlotKey,
    ) -> Result<bool, inner::ItemError> {
        let (inventory_key, local_key) = slot_key;
        let inventory = root.item_get_inventory(inventory_key)?;
        let slot = inventory
            .slots
            .get(local_key as usize)
            .ok_or(inner::ItemError::ItemNotFound)?;
        Ok(slot.item.is_some())
    }

    pub fn get_item_amount(
        &mut self,
        root: &inner::Root,
        slot_key: inner::SlotKey,
    ) -> Result<u32, inner::ItemError> {
        let (inventory_key, local_key) = slot_key;
        let inventory = root.item_get_inventory(inventory_key)?;
        let slot = inventory
            .slots
            .get(local_key as usize)
            .ok_or(inner::ItemError::ItemNotFound)?;
        let item = slot.item.as_ref().ok_or(inner::ItemError::ItemNotFound)?;

        Ok(item.amount)
    }

    pub fn get_item_name_text(
        &mut self,
        root: &inner::Root,
        slot_key: inner::SlotKey,
    ) -> Result<String, inner::ItemError> {
        let (inventory_key, local_key) = slot_key;
        let inventory = root.item_get_inventory(inventory_key)?;
        let slot = inventory
            .slots
            .get(local_key as usize)
            .ok_or(inner::ItemError::ItemNotFound)?;
        let item = slot.item.as_ref().ok_or(inner::ItemError::ItemNotFound)?;

        let item_prop = self
            .item_props
            .get(item.id as usize)
            .ok_or(inner::ItemError::ItemInvalidId)?;
        Ok(item_prop.name_text.clone())
    }

    pub fn get_item_desc_text(
        &mut self,
        root: &inner::Root,
        slot_key: inner::SlotKey,
    ) -> Result<String, inner::ItemError> {
        let (inventory_key, local_key) = slot_key;
        let inventory = root.item_get_inventory(inventory_key)?;
        let slot = inventory
            .slots
            .get(local_key as usize)
            .ok_or(inner::ItemError::ItemNotFound)?;
        let item = slot.item.as_ref().ok_or(inner::ItemError::ItemNotFound)?;

        let item_prop = self
            .item_props
            .get(item.id as usize)
            .ok_or(inner::ItemError::ItemInvalidId)?;
        Ok(item_prop.desc_text.clone())
    }

    pub fn draw_item(
        &self,
        root: &inner::Root,
        slot_key: inner::SlotKey,
        control_item: Gd<godot::classes::Control>,
    ) -> Result<(), inner::ItemError> {
        let (inventory_key, local_key) = slot_key;
        let inventory = root.item_get_inventory(inventory_key)?;
        let slot = inventory
            .slots
            .get(local_key as usize)
            .ok_or(inner::ItemError::ItemNotFound)?;

        // rendering

        let canvas_item = control_item.get_canvas_item();

        let mut rendering_server = godot::classes::RenderingServer::singleton();
        rendering_server.canvas_item_clear(canvas_item);

        if let Some(item) = &slot.item {
            let rect = Rect2::new(Vector2::ZERO, control_item.get_size());

            let image_head =
                &self.image_heads[item.id as usize][item.render_param.variant as usize];
            let texcoord_id = if image_head.step_tick == 0 {
                image_head.start_texcoord_id
            } else {
                let step_id = (root.time_tick() as u32 - item.render_param.tick)
                    / image_head.step_tick as u32;
                let step_size = image_head.end_texcoord_id - image_head.start_texcoord_id;
                if image_head.is_loop {
                    image_head.start_texcoord_id + (step_id % step_size)
                } else {
                    image_head.start_texcoord_id + u32::min(step_id, step_size - 1)
                }
            };
            let texture = self.textures[texcoord_id as usize];

            rendering_server.canvas_item_add_texture_rect(canvas_item, rect, texture);
        }

        Ok(())
    }
}

impl Drop for ItemStore {
    fn drop(&mut self) {
        let mut rendering_server = godot::classes::RenderingServer::singleton();
        for free_handle in &self.free_handles {
            rendering_server.free_rid(*free_handle);
        }
    }
}
