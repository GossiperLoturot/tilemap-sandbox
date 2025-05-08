use glam::*;
use godot::prelude::*;

pub mod dataflow;
pub mod view;

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
    pub fn new() -> Self {
        Default::default()
    }

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
    pub feature: Box<dyn dataflow::FeatureRow>,
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
    pub feature: Box<dyn dataflow::FeatureRow>,
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
    pub feature: Box<dyn dataflow::FeatureRow>,
}

pub struct ItemDescriptor {
    pub display_name: String,
    pub description: String,
    pub images: Vec<ImageDescriptor>,
    pub feature: Box<dyn dataflow::FeatureRow>,
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

#[derive(Default)]
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
        Default::default()
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

        // feature
        let mut matrix_builder = dataflow::FeatureMatrixBuilder::default();

        // tile field
        let mut tiles = vec![];
        let mut tiles_view = vec![];
        for tile in self.tiles {
            let desc = tile(&self.registry, retriever);

            let mut row = matrix_builder.insert_row();
            desc.feature.create_row(&mut row).unwrap();

            tiles.push(dataflow::TileDescriptor {
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

                images.push(view::TileImageDescriptor {
                    frames,
                    step_tick: image.step_tick,
                    is_loop: image.is_loop,
                });
            }

            tiles_view.push(view::TileDescriptor { images });
        }

        let tile_field_desc = dataflow::TileFieldDescriptor { tiles };

        let mut tile_shaders = vec![];
        for shader in desc.tile_shaders {
            tile_shaders.push(shader);
        }
        let tile_field_view = view::TileField::new(view::TileFieldDescriptor {
            tiles: tiles_view,
            shaders: tile_shaders,
            world: world.clone(),
        });

        // block field
        let mut blocks = vec![];
        let mut blocks_view = vec![];
        for block in self.blocks {
            let desc = block(&self.registry, retriever);

            let mut row = matrix_builder.insert_row();
            desc.feature.create_row(&mut row).unwrap();

            blocks.push(dataflow::BlockDescriptor {
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

                images.push(view::BlockImageDescriptor {
                    frames,
                    step_tick: image.step_tick,
                    is_loop: image.is_loop,
                });
            }

            blocks_view.push(view::BlockDescriptor {
                images,
                z_along_y: desc.z_along_y,
                rendering_size: desc.rendering_size,
                rendering_offset: desc.rendering_offset,
            });
        }

        let block_field_desc = dataflow::BlockFieldDescriptor { blocks };

        let mut block_shaders = vec![];
        for shader in desc.block_shaders {
            block_shaders.push(shader);
        }
        let block_field_view = view::BlockField::new(view::BlockFieldDescriptor {
            blocks: blocks_view,
            shaders: block_shaders,
            world: world.clone(),
        });

        // entity filed
        let mut entities = vec![];
        let mut entities_view = vec![];
        for entity in self.entities {
            let desc = entity(&self.registry, retriever);

            let mut row = matrix_builder.insert_row();
            desc.feature.create_row(&mut row).unwrap();

            entities.push(dataflow::EntityDescriptor {
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

                images.push(view::EntityImageDescriptor {
                    frames,
                    step_tick: image.step_tick,
                    is_loop: image.is_loop,
                });
            }

            entities_view.push(view::EntityDescriptor {
                images,
                z_along_y: desc.z_along_y,
                rendering_size: desc.rendering_size,
                rendering_offset: desc.rendering_offset,
            });
        }

        let entity_field_desc = dataflow::EntityFieldDescriptor { entities };

        let mut entity_shaders = vec![];
        for shader in desc.entity_shaders {
            entity_shaders.push(shader);
        }
        let entity_field_view = view::EntityField::new(view::EntityFieldDescriptor {
            entities: entities_view,
            shaders: entity_shaders,
            world: world.clone(),
        });

        // item field
        let mut items = vec![];
        let mut items_view = vec![];
        for item in self.items {
            let desc = item(&self.registry, retriever);

            let mut row = matrix_builder.insert_row();
            desc.feature.create_row(&mut row).unwrap();

            items.push(dataflow::ItemDescriptor {
                display_name: desc.display_name,
                description: desc.description,
            });

            let mut images = vec![];
            for image in desc.images {
                let mut frames = vec![];
                for image in image.frames {
                    frames.push(image);
                }

                images.push(view::ItemImageDescriptor {
                    frames,
                    step_tick: image.step_tick,
                    is_loop: image.is_loop,
                });
            }

            items_view.push(view::ItemDescriptor { images });
        }

        let mut inventories = vec![];
        let mut inventories_view = vec![];
        for inventory in self.inventories {
            let desc = inventory(&self.registry, retriever);

            inventories.push(dataflow::InventoryDescriptor { size: desc.size });

            inventories_view.push(view::InventoryDescriptor {
                callback: desc.callback,
            });
        }

        let item_storage_desc = dataflow::ItemStorageDescriptor { items, inventories };

        let item_storage_view = view::ItemStorage::new(view::ItemStorageDescriptor {
            items: items_view,
            inventories: inventories_view,
        });

        let selection_view = view::Selection::new(view::SelectionDescriptor {
            shader: desc.selection_shader,
            world: world.clone(),
        });

        let dataflow = dataflow::Dataflow::new(dataflow::DataflowDescriptor {
            tile_field: tile_field_desc,
            block_field: block_field_desc,
            entity_field: entity_field_desc,
            item_storage: item_storage_desc,
            matrix: matrix_builder,
        });

        Context {
            registry: self.registry,
            dataflow,
            tile_field_view,
            block_field_view,
            entity_field_view,
            item_storage_view,
            selection_view,
        }
    }
}

pub struct Context {
    pub registry: Registry,
    pub dataflow: dataflow::Dataflow,
    pub tile_field_view: view::TileField,
    pub block_field_view: view::BlockField,
    pub entity_field_view: view::EntityField,
    pub item_storage_view: view::ItemStorage,
    pub selection_view: view::Selection,
}
