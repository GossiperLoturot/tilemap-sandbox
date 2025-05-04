use glam::*;
use godot::prelude::*;

use crate::*;

// retriever for loading resources

pub struct Retriever {
    retrieve_callable: Callable,
}

impl Retriever {
    pub fn new(retrieve_callable: Callable) -> Self {
        Self { retrieve_callable }
    }

    pub fn load<T>(&self, name: &str) -> T
    where
        T: FromGodot,
    {
        if self.retrieve_callable.is_null() {
            panic!("Retriever callable is null");
        }
        if self.retrieve_callable.get_argument_count() < 1 {
            panic!("Retriever callable must have at least one argument");
        }
        let ret = self.retrieve_callable.call(&[name.to_variant()]);

        match ret.try_to() {
            Ok(value) => {
                // successfully converted to the expected type
                value
            }
            Err(e) => {
                let found_type = ret.get_type().as_str();
                let expected_type = std::any::type_name::<T>();
                panic!(
                    "Failed to load resource {}: Expected type is {}, but found type is {}.\n{}",
                    name, found_type, expected_type, e
                );
            }
        }
    }
}

// registry

#[derive(Default)]
pub struct Registry {
    storage: ahash::AHashMap<String, u16>,
}

impl Registry {
    pub fn set(&mut self, name: String, value: u16) {
        if self.storage.contains_key(&name) {
            panic!("Name {} already exists", name);
        }

        self.storage.insert(name, value);
    }

    pub fn get(&self, name: &str) -> u16 {
        match self.storage.get(name) {
            Some(value) => {
                // successfully get the value
                *value
            }
            None => {
                panic!("Name {} not found", name);
            }
        }
    }
}

// descriptor for building the context

pub struct ImageDescriptor {
    pub frames: Vec<Gd<godot::classes::Image>>,
    pub step_tick: u16,
    pub is_loop: bool,
}

pub struct TileDescriptor {
    pub display_name: String,
    pub description: String,
    pub images: Vec<ImageDescriptor>,
    pub collision: bool,
    pub feature: Box<dyn inner::TileFeature>,
}

pub struct BlockDescriptor {
    pub display_name: String,
    pub description: String,
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
    pub display_name: String,
    pub description: String,
    pub images: Vec<ImageDescriptor>,
    pub z_along_y: bool,
    pub collision_size: Vec2,
    pub collision_offset: Vec2,
    pub rendering_size: Vec2,
    pub rendering_offset: Vec2,
    pub feature: Box<dyn inner::EntityFeature>,
}

pub struct ItemDescriptor {
    pub display_name: String,
    pub description: String,
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

type RegisterFn<T> = Box<dyn for<'a> FnOnce(&'a Registry, &'a Retriever) -> T>;
pub struct ContextBuilder {
    tiles: Vec<RegisterFn<TileDescriptor>>,
    blocks: Vec<RegisterFn<BlockDescriptor>>,
    entities: Vec<RegisterFn<EntityDescriptor>>,
    items: Vec<RegisterFn<ItemDescriptor>>,
    inventories: Vec<RegisterFn<InventoryDescriptor>>,
    registry: Registry,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            tiles: Default::default(),
            blocks: Default::default(),
            entities: Default::default(),
            items: Default::default(),
            inventories: Default::default(),
            registry: Default::default(),
        }
    }

    pub fn add_tile<F>(&mut self, name: String, desc_fn: F)
    where
        F: FnOnce(&Registry, &Retriever) -> TileDescriptor + 'static,
    {
        self.tiles.push(Box::new(desc_fn));
        let id = (self.tiles.len() - 1) as u16;
        self.registry.set(name, id);
    }

    pub fn add_block<F>(&mut self, name: String, desc_fn: F)
    where
        F: FnOnce(&Registry, &Retriever) -> BlockDescriptor + 'static,
    {
        self.blocks.push(Box::new(desc_fn));
        let id = (self.blocks.len() - 1) as u16;
        self.registry.set(name, id);
    }

    pub fn add_entity<F>(&mut self, name: String, desc_fn: F)
    where
        F: FnOnce(&Registry, &Retriever) -> EntityDescriptor + 'static,
    {
        self.entities.push(Box::new(desc_fn));
        let id = (self.entities.len() - 1) as u16;
        self.registry.set(name, id);
    }

    pub fn add_item<F>(&mut self, name: String, desc_fn: F)
    where
        F: FnOnce(&Registry, &Retriever) -> ItemDescriptor + 'static,
    {
        self.items.push(Box::new(desc_fn));
        let id = (self.items.len() - 1) as u16;
        self.registry.set(name, id);
    }

    pub fn add_inventory<F>(&mut self, name: String, desc_fn: F)
    where
        F: FnOnce(&Registry, &Retriever) -> InventoryDescriptor + 'static,
    {
        self.inventories.push(Box::new(desc_fn));
        let id = (self.inventories.len() - 1) as u16;
        self.registry.set(name, id);
    }

    pub fn build(self, retriever: &Retriever, desc: BuildDescriptor) -> Context {
        let world = desc
            .viewport
            .get_world_3d()
            .unwrap_or_else(|| panic!("Failed to get World3D from {}", desc.viewport));

        // tile field
        let mut tile_features = vec![];
        let mut tiles = vec![];
        let mut tiles_view = vec![];
        for tile in self.tiles {
            let desc = tile(&self.registry, retriever);

            tile_features.push(desc.feature);

            tiles.push(inner::TileDescriptor {
                display_name: desc.display_name,
                description: desc.description,
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
            let desc = block(&self.registry, retriever);

            block_features.push(desc.feature);

            blocks.push(inner::BlockDescriptor {
                display_name: desc.display_name,
                description: desc.description,
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
            let desc = entity(&self.registry, retriever);

            entity_features.push(desc.feature);

            entities.push(inner::EntityDescriptor {
                display_name: desc.display_name,
                description: desc.description,
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
            let desc = item(&self.registry, retriever);

            item_features.push(desc.feature);

            items.push(inner::ItemDescriptor {
                display_name: desc.display_name,
                description: desc.description,
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

            items_view.push(item::ItemDescriptor { images });
        }

        let mut inventories = vec![];
        let mut inventories_view = vec![];
        for inventory in self.inventories {
            let desc = inventory(&self.registry, retriever);

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
            registry: self.registry,
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
    pub registry: Registry,
}
