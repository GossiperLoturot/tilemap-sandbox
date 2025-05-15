use glam::*;
use godot::prelude::*;

use crate::dataflow;

pub struct ItemImageDescriptor {
    pub frames: Vec<Gd<godot::classes::Image>>,
    pub step_tick: u16,
    pub is_loop: bool,
}

pub struct ItemDescriptor {
    pub images: Vec<ItemImageDescriptor>,
}

pub struct InventoryDescriptor {
    pub callback: Callable,
}

pub struct ItemStorageDescriptor {
    pub items: Vec<ItemDescriptor>,
    pub inventories: Vec<InventoryDescriptor>,
}

struct ImageHead {
    start_texcoord_id: u32,
    end_texcoord_id: u32,
    step_tick: u16,
    is_loop: bool,
}

struct ItemProperty {}

struct InventoryProperty {
    pub callback: Callable,
}

pub struct ItemStorage {
    inventory_props: Vec<InventoryProperty>,
    image_heads: Vec<Vec<ImageHead>>,
    textures: Vec<Rid>,
    free_handles: Vec<Rid>,
}

impl ItemStorage {
    pub fn new(desc: ItemStorageDescriptor) -> Self {
        let mut rendering_server = godot::classes::RenderingServer::singleton();

        let mut free_handles = vec![];

        let mut item_props = vec![];
        let mut image_heads = vec![];
        let mut textures = vec![];
        for desc in desc.items {
            item_props.push(ItemProperty {});

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
                callback: desc.callback,
            });
        }

        Self {
            inventory_props,
            image_heads,
            textures,
            free_handles,
        }
    }

    pub fn open_inventory_by_tile(
        &self,
        dataflow: &dataflow::Dataflow,
        tile_key: dataflow::TileKey,
        f: impl FnOnce(&Callable, &dataflow::Inventory),
    ) -> Result<(), dataflow::DataflowError> {
        let inventory_key = dataflow
            .get_tile_inventory(tile_key)?
            .ok_or(dataflow::ItemError::InventoryNotFound)?;
        self.open_inventory(dataflow, inventory_key, f)?;
        Ok(())
    }

    pub fn open_inventory_by_block(
        &self,
        dataflow: &dataflow::Dataflow,
        block_key: dataflow::BlockKey,
        f: impl FnOnce(&Callable, &dataflow::Inventory),
    ) -> Result<(), dataflow::DataflowError> {
        let inventory_key = dataflow
            .get_block_inventory(block_key)?
            .ok_or(dataflow::ItemError::InventoryNotFound)?;
        self.open_inventory(dataflow, inventory_key, f)?;
        Ok(())
    }

    pub fn open_inventory_by_entity(
        &self,
        dataflow: &dataflow::Dataflow,
        tile_key: dataflow::TileKey,
        f: impl FnOnce(&Callable, &dataflow::Inventory),
    ) -> Result<(), dataflow::DataflowError> {
        let inventory_key = dataflow
            .get_inventory_by_entity(tile_key)?
            .ok_or(dataflow::ItemError::InventoryNotFound)?;
        self.open_inventory(dataflow, inventory_key, f)?;
        Ok(())
    }

    pub fn open_inventory(
        &self,
        dataflow: &dataflow::Dataflow,
        inventory_key: dataflow::InventoryKey,
        f: impl FnOnce(&Callable, &dataflow::Inventory),
    ) -> Result<(), dataflow::DataflowError> {
        let inventory = dataflow.get_inventory(inventory_key)?;
        let prop = self
            .inventory_props
            .get(inventory.id as usize)
            .ok_or(dataflow::ItemError::InventoryInvalidId)?;

        f(&prop.callback, inventory);

        Ok(())
    }

    pub fn draw_item(
        &self,
        dataflow: &dataflow::Dataflow,
        slot_key: dataflow::SlotKey,
        control_item: Gd<godot::classes::Control>,
    ) -> Result<(), dataflow::DataflowError> {
        let (inventory_key, local_key) = slot_key;
        let inventory = dataflow.get_inventory(inventory_key)?;
        let slot = inventory
            .slots
            .get(local_key as usize)
            .ok_or(dataflow::ItemError::ItemNotFound)?;

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
                let step_id = (dataflow.get_tick() as u32 - item.render_param.tick)
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

impl Drop for ItemStorage {
    fn drop(&mut self) {
        let mut rendering_server = godot::classes::RenderingServer::singleton();
        for free_handle in &self.free_handles {
            rendering_server.free_rid(*free_handle);
        }
    }
}
