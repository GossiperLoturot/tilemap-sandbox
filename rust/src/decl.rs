use glam::*;
use godot::prelude::*;

use crate::*;

pub struct ImageDescriptor {
    pub frames: Vec<Gd<godot::classes::Image>>,
    pub step_tick: u16,
    pub is_loop: bool,
}

pub struct TileDescriptor {
    pub name_text: String,
    pub desc_text: String,
    pub images: Vec<ImageDescriptor>,
    pub collision: bool,
    pub feature: Box<dyn inner::TileFeature>,
}

pub struct BlockDescriptor {
    pub name_text: String,
    pub desc_text: String,
    pub images: Vec<ImageDescriptor>,
    pub z_along_y: bool,
    pub size: IVec2,
    pub collision_size: Vec2,
    pub collision_offset: Vec2,
    pub rendering_size: Vec2,
    pub rendering_offset: Vec2,
    pub feature: Box<dyn inner::BlockFeature>,
}

pub struct EntityDescriptor {
    pub name_text: String,
    pub desc_text: String,
    pub images: Vec<ImageDescriptor>,
    pub z_along_y: bool,
    pub collision_size: Vec2,
    pub collision_offset: Vec2,
    pub rendering_size: Vec2,
    pub rendering_offset: Vec2,
    pub feature: Box<dyn inner::EntityFeature>,
}

pub struct ItemDescriptor {
    pub name_text: String,
    pub desc_text: String,
    pub images: Vec<ImageDescriptor>,
    pub feature: Box<dyn inner::ItemFeature>,
}

pub struct InventoryDescriptor {
    pub size: u32,
    pub callback: Callable,
}

pub struct BuildDescriptor {
    pub tile_shaders: Vec<Gd<godot::classes::Shader>>,
    pub block_shaders: Vec<Gd<godot::classes::Shader>>,
    pub entity_shaders: Vec<Gd<godot::classes::Shader>>,
    pub selection_shader: Gd<godot::classes::Shader>,
    pub viewport: Gd<godot::classes::Viewport>,
}

type RegisterFn<T, U> = Box<dyn for<'a> FnOnce(&'a T) -> U>;

pub struct ContextBuilder<T> {
    tiles: Vec<RegisterFn<T, TileDescriptor>>,
    blocks: Vec<RegisterFn<T, BlockDescriptor>>,
    entities: Vec<RegisterFn<T, EntityDescriptor>>,
    items: Vec<RegisterFn<T, ItemDescriptor>>,
    inventories: Vec<RegisterFn<T, InventoryDescriptor>>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ContextBuilder<T> {
    pub fn new() -> Self {
        Self {
            tiles: Default::default(),
            blocks: Default::default(),
            entities: Default::default(),
            items: Default::default(),
            inventories: Default::default(),
            _phantom: Default::default(),
        }
    }

    pub fn add_tile<F>(&mut self, desc_fn: F) -> u16
    where
        F: FnOnce(&T) -> TileDescriptor + 'static,
    {
        self.tiles.push(Box::new(desc_fn));
        (self.tiles.len() - 1) as u16
    }

    pub fn add_block<F>(&mut self, desc_fn: F) -> u16
    where
        F: FnOnce(&T) -> BlockDescriptor + 'static,
    {
        self.blocks.push(Box::new(desc_fn));
        (self.blocks.len() - 1) as u16
    }

    pub fn add_entity<F>(&mut self, desc_fn: F) -> u16
    where
        F: FnOnce(&T) -> EntityDescriptor + 'static,
    {
        self.entities.push(Box::new(desc_fn));
        (self.entities.len() - 1) as u16
    }

    pub fn add_item<F>(&mut self, desc_fn: F) -> u16
    where
        F: FnOnce(&T) -> ItemDescriptor + 'static,
    {
        self.items.push(Box::new(desc_fn));
        (self.items.len() - 1) as u16
    }

    pub fn add_inventory<F>(&mut self, desc_fn: F) -> u16
    where
        F: FnOnce(&T) -> InventoryDescriptor + 'static,
    {
        self.inventories.push(Box::new(desc_fn));
        (self.inventories.len() - 1) as u16
    }

    pub fn build(self, args: T, desc: BuildDescriptor) -> Context {
        let world = desc.viewport.get_world_3d().unwrap();

        // tile field
        let mut tile_features = vec![];
        let mut tiles = vec![];
        let mut tiles_view = vec![];
        for tile in self.tiles {
            let desc = tile(&args);

            tile_features.push(desc.feature);

            tiles.push(inner::TileDescriptor {
                name_text: desc.name_text,
                desc_text: desc.desc_text,
                collision: desc.collision,
            });

            let mut images = vec![];
            for image in desc.images {
                let mut frames = vec![];
                for image in image.frames {
                    frames.push(image);
                }

                images.push(tile::TileImageDescriptor {
                    frames,
                    step_tick: image.step_tick,
                    is_loop: image.is_loop,
                });
            }

            tiles_view.push(tile::TileDescriptor { images });
        }

        let tile_field_desc = inner::TileFieldDescriptor { tiles };

        let mut tile_shaders = vec![];
        for shader in desc.tile_shaders {
            tile_shaders.push(shader);
        }
        let tile_field_view = tile::TileField::new(tile::TileFieldDescriptor {
            tiles: tiles_view,
            shaders: tile_shaders,
            world: world.clone(),
        });

        // block field
        let mut block_features = vec![];
        let mut blocks = vec![];
        let mut blocks_view = vec![];
        for block in self.blocks {
            let desc = block(&args);

            block_features.push(desc.feature);

            blocks.push(inner::BlockDescriptor {
                name_text: desc.name_text,
                desc_text: desc.desc_text,
                size: desc.size,
                collision_size: desc.collision_size,
                collision_offset: desc.collision_offset,
                hint_size: desc.rendering_size,
                hint_offset: desc.rendering_offset,
                z_along_y: desc.z_along_y,
            });

            let mut images = vec![];
            for image in desc.images {
                let mut frames = vec![];
                for image in image.frames {
                    frames.push(image);
                }

                images.push(block::BlockImageDescriptor {
                    frames,
                    step_tick: image.step_tick,
                    is_loop: image.is_loop,
                });
            }

            blocks_view.push(block::BlockDescriptor {
                images,
                z_along_y: desc.z_along_y,
                rendering_size: desc.rendering_size,
                rendering_offset: desc.rendering_offset,
            });
        }

        let block_field_desc = inner::BlockFieldDescriptor { blocks };

        let mut block_shaders = vec![];
        for shader in desc.block_shaders {
            block_shaders.push(shader);
        }
        let block_field_view = block::BlockField::new(block::BlockFieldDescriptor {
            blocks: blocks_view,
            shaders: block_shaders,
            world: world.clone(),
        });

        // entity filed
        let mut entity_features = vec![];
        let mut entities = vec![];
        let mut entities_view = vec![];
        for entity in self.entities {
            let desc = entity(&args);

            entity_features.push(desc.feature);

            entities.push(inner::EntityDescriptor {
                name_text: desc.name_text,
                desc_text: desc.desc_text,
                collision_size: desc.collision_size,
                collision_offset: desc.collision_offset,
                hint_size: desc.rendering_size,
                hint_offset: desc.rendering_offset,
                z_along_y: desc.z_along_y,
            });

            let mut images = vec![];
            for image in desc.images {
                let mut frames = vec![];
                for image in image.frames {
                    frames.push(image);
                }

                images.push(entity::EntityImageDescriptor {
                    frames,
                    step_tick: image.step_tick,
                    is_loop: image.is_loop,
                });
            }

            entities_view.push(entity::EntityDescriptor {
                images,
                z_along_y: desc.z_along_y,
                rendering_size: desc.rendering_size,
                rendering_offset: desc.rendering_offset,
            });
        }

        let entity_field_desc = inner::EntityFieldDescriptor { entities };

        let mut entity_shaders = vec![];
        for shader in desc.entity_shaders {
            entity_shaders.push(shader);
        }
        let entity_field_view = entity::EntityField::new(entity::EntityFieldDescriptor {
            entities: entities_view,
            shaders: entity_shaders,
            world: world.clone(),
        });

        // item field
        let mut item_features = vec![];
        let mut items = vec![];
        let mut items_view = vec![];
        for item in self.items {
            let desc = item(&args);

            item_features.push(desc.feature);

            items.push(inner::ItemDescriptor {
                name_text: desc.name_text.clone(),
            });

            let mut images = vec![];
            for image in desc.images {
                let mut frames = vec![];
                for image in image.frames {
                    frames.push(image);
                }

                images.push(item::ItemImageDescriptor {
                    frames,
                    step_tick: image.step_tick,
                    is_loop: image.is_loop,
                });
            }

            items_view.push(item::ItemDescriptor {
                name_text: desc.name_text.clone(),
                desc_text: desc.desc_text.clone(),
                images,
            });
        }

        let mut inventories = vec![];
        let mut inventories_view = vec![];
        for inventory in self.inventories {
            let desc = inventory(&args);

            inventories.push(inner::InventoryDescriptor { size: desc.size });

            inventories_view.push(item::InventoryDescriptor {
                callback: desc.callback,
            });
        }

        let item_storage_desc = inner::ItemStorageDescriptor { items, inventories };

        let item_storage_view = item::ItemStorage::new(item::ItemStorageDescriptor {
            items: items_view,
            inventories: inventories_view,
        });

        let selection_view = selection::Selection::new(selection::SelectionDescriptor {
            shader: desc.selection_shader,
            world: world.clone(),
        });

        let root = inner::Root::new(inner::RootDescriptor {
            tile_field: tile_field_desc,
            block_field: block_field_desc,
            entity_field: entity_field_desc,
            item_storage: item_storage_desc,

            tile_features: tile_features.into(),
            block_features: block_features.into(),
            entity_features: entity_features.into(),
            item_features: item_features.into(),
        });

        Context {
            root,
            tile_field: tile_field_view,
            block_field: block_field_view,
            entity_field: entity_field_view,
            item_storage: item_storage_view,
            selection: selection_view,
        }
    }
}

pub struct Context {
    pub root: inner::Root,
    pub tile_field: tile::TileField,
    pub block_field: block::BlockField,
    pub entity_field: entity::EntityField,
    pub item_storage: item::ItemStorage,
    pub selection: selection::Selection,
}
