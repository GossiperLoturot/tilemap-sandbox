use glam::*;
use godot::prelude::*;

use crate::dataflow;

pub struct ItemSpriteInfo {
    pub images: Vec<Gd<godot::classes::Image>>,
    pub ticks_per_image: u16,
    pub is_loop: bool,
}

pub struct ItemInfo {
    pub sprites: Vec<ItemSpriteInfo>,
}

pub struct InventoryInfo {
    pub callback: Callable,
}

pub struct InventorySystemInfo {
    pub items: Vec<ItemInfo>,
    pub inventories: Vec<InventoryInfo>,
}

struct ImageAddress {
    start_index: u32,
    end_index: u32,
    ticks_per_images: u16,
    is_loop: bool,
}

struct InventoryRenderLayout {
    pub callback: Callable,
}

pub struct InventorySystem {
    inventory_layouts: Vec<InventoryRenderLayout>,
    sprite_addrs: Vec<Vec<ImageAddress>>,
    textures: Vec<Rid>,
    free_handles: Vec<Rid>,
}

impl InventorySystem {
    pub fn new(info: InventorySystemInfo) -> Self {
        let mut rendering_server = godot::classes::RenderingServer::singleton();

        let mut free_handles = vec![];

        let mut sprite_addrs = vec![];
        let mut images = vec![];
        for item in info.items {
            let mut sprite_addr = vec![];

            for sprite in item.sprites {
                if images.len() + sprite.images.len() >= i32::MAX as usize {
                    panic!("number of frame must be less than i32::MAX");
                }

                sprite_addr.push(ImageAddress {
                    start_index: images.len() as u32,
                    end_index: (images.len() + sprite.images.len()) as u32,
                    ticks_per_images: sprite.ticks_per_image,
                    is_loop: sprite.is_loop,
                });

                for image in sprite.images {
                    images.push(image);
                }
            }

            sprite_addrs.push(sprite_addr);
        }

        let mut inventory_layouts = vec![];
        for inventory in info.inventories {
            inventory_layouts.push(InventoryRenderLayout { callback: inventory.callback });
        }

        let mut textures = vec![];
        for image in images {
            let texture = rendering_server.texture_2d_create(&image);
            textures.push(texture);
            free_handles.push(texture);
        }

        Self {
            inventory_layouts,
            sprite_addrs,
            textures,
            free_handles,
        }
    }

    pub fn open_inventory_by_tile(
        &self,
        dataflow: &dataflow::Dataflow,
        tile_id: dataflow::TileId,
        f: impl FnOnce(&Callable, &dataflow::Inventory),
    ) -> Result<(), dataflow::DataflowError> {
        let inventory_id = dataflow
            .get_tile_inventory(tile_id)?
            .ok_or(dataflow::ItemError::InventoryNotFound)?;
        self.open_inventory(dataflow, inventory_id, f)?;
        Ok(())
    }

    pub fn open_inventory_by_block(
        &self,
        dataflow: &dataflow::Dataflow,
        block_id: dataflow::BlockId,
        f: impl FnOnce(&Callable, &dataflow::Inventory),
    ) -> Result<(), dataflow::DataflowError> {
        let inventory_id = dataflow
            .get_block_inventory(block_id)?
            .ok_or(dataflow::ItemError::InventoryNotFound)?;
        self.open_inventory(dataflow, inventory_id, f)?;
        Ok(())
    }

    pub fn open_inventory_by_entity(
        &self,
        dataflow: &dataflow::Dataflow,
        entity_id: dataflow::EntityId,
        f: impl FnOnce(&Callable, &dataflow::Inventory),
    ) -> Result<(), dataflow::DataflowError> {
        let inventory_id = dataflow
            .get_inventory_by_entity(entity_id)?
            .ok_or(dataflow::ItemError::InventoryNotFound)?;
        self.open_inventory(dataflow, inventory_id, f)?;
        Ok(())
    }

    pub fn open_inventory(
        &self,
        dataflow: &dataflow::Dataflow,
        inventory_key: dataflow::InventoryId,
        f: impl FnOnce(&Callable, &dataflow::Inventory),
    ) -> Result<(), dataflow::DataflowError> {
        let inventory = dataflow.get_inventory(inventory_key)?;
        let prop = self
            .inventory_layouts
            .get(inventory.archetype_id as usize)
            .ok_or(dataflow::ItemError::InventoryInvalidId)?;

        f(&prop.callback, inventory);

        Ok(())
    }

    pub fn draw_item(
        &self,
        dataflow: &dataflow::Dataflow,
        slot_id: dataflow::ItemId,
        control_item: Gd<godot::classes::Control>,
    ) -> Result<(), dataflow::DataflowError> {
        let (inventory_id, local_id) = slot_id;

        let inventory = dataflow.get_inventory(inventory_id)?;

        let item_ref = inventory
            .items
            .get(local_id as usize)
            .ok_or(dataflow::ItemError::ItemNotFound)?;

        // rendering

        let canvas_item = control_item.get_canvas_item();

        let mut rendering_server = godot::classes::RenderingServer::singleton();

        rendering_server.canvas_item_clear(canvas_item);

        if let Some(item) = item_ref {
            let rect = Rect2::new(Vector2::ZERO, control_item.get_size());

            let image_addr = &self.sprite_addrs[item.archetype_id as usize][item.variant as usize];

            let index = if image_addr.ticks_per_images == 0 {
                image_addr.start_index
            } else {
                let step_index = (dataflow.get_tick() as u32 - item.tick) / image_addr.ticks_per_images as u32;
                let step_len = image_addr.end_index - image_addr.start_index;
                let cycle = if image_addr.is_loop { step_index % step_len } else { u32::min(step_index, step_len - 1) };
                image_addr.start_index + cycle
            };

            let texture = self.textures[index as usize];

            rendering_server.canvas_item_add_texture_rect(canvas_item, rect, texture);
        }

        Ok(())
    }
}

impl Drop for InventorySystem {
    fn drop(&mut self) {
        let mut rendering_server = godot::classes::RenderingServer::singleton();
        for free_handle in &self.free_handles {
            rendering_server.free_rid(*free_handle);
        }
    }
}
