pub use glam::*;
pub use geom::*;

pub mod dataflow;
pub mod view;

mod geom;

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

pub struct SpriteInfo {
    pub images: Vec<godot::obj::Gd<godot::classes::Image>>,
    pub step_tick: u16,
    pub is_loop: bool,
}

pub struct TileInfo {
    pub display_name: String,
    pub description: String,
    pub sprites: Vec<SpriteInfo>,
    pub collision: bool,
}

pub struct BlockInfo {
    pub display_name: String,
    pub description: String,
    pub sprites: Vec<SpriteInfo>,
    pub y_sorting: bool,
    pub size: IVec2,
    pub collision_rect: Option<Rect2>,
    pub rendering_rect: Rect2,
}

pub struct EntityInfo {
    pub display_name: String,
    pub description: String,
    pub sprites: Vec<SpriteInfo>,
    pub y_sorting: bool,
    pub collision_rect: Option<Rect2>,
    pub rendering_rect: Rect2,
}

pub struct BuildInfo {
    pub tile_shaders: Vec<godot::obj::Gd<godot::classes::Shader>>,
    pub block_shaders: Vec<godot::obj::Gd<godot::classes::Shader>>,
    pub entity_shaders: Vec<godot::obj::Gd<godot::classes::Shader>>,
    pub viewport: godot::obj::Gd<godot::classes::Viewport>,
}

type RegisterFn<T> = Box<dyn for<'a> FnOnce(&'a Registry) -> T>;

#[derive(Default)]
pub struct ContextBuilder {
    tiles: Vec<RegisterFn<TileInfo>>,
    blocks: Vec<RegisterFn<BlockInfo>>,
    entities: Vec<RegisterFn<EntityInfo>>,
    registry: Registry,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_tile<F>(&mut self, name: String, desc_fn: F) where F: FnOnce(&Registry) -> TileInfo + 'static,
    {
        self.tiles.push(Box::new(desc_fn));
        let id = (self.tiles.len() - 1) as u16;
        self.registry.set(name, id);
    }

    pub fn add_block<F>(&mut self, name: String, desc_fn: F) where F: FnOnce(&Registry) -> BlockInfo + 'static,
    {
        self.blocks.push(Box::new(desc_fn));
        let id = (self.blocks.len() - 1) as u16;
        self.registry.set(name, id);
    }

    pub fn add_entity<F>(&mut self, name: String, desc_fn: F) where F: FnOnce(&Registry) -> EntityInfo + 'static,
    {
        self.entities.push(Box::new(desc_fn));
        let id = (self.entities.len() - 1) as u16;
        self.registry.set(name, id);
    }

    pub fn build(self, info: BuildInfo) -> Context {
        let world = info
            .viewport
            .get_world_3d()
            .unwrap_or_else(|| panic!("Failed to get World3D from {}", info.viewport));

        // tile field
        let mut tiles = vec![];
        let mut tiles_view = vec![];
        for tile in self.tiles {
            let tile_info = tile(&self.registry);

            tiles.push(dataflow::TileInfo {
                display_name: tile_info.display_name,
                description: tile_info.description,
                collision: tile_info.collision,
            });

            let mut sprites = vec![];
            for sprite in tile_info.sprites {
                let mut images = vec![];
                for image in sprite.images {
                    images.push(image);
                }

                sprites.push(view::TileSpriteInfo {
                    images,
                    tick_per_image: sprite.step_tick,
                    is_loop: sprite.is_loop,
                });
            }

            tiles_view.push(view::TileInfo { sprites });
        }

        let tile_field_info = dataflow::TileFieldInfo { tiles };

        let mut tile_shaders = vec![];
        for shader in info.tile_shaders {
            tile_shaders.push(shader);
        }
        let tile_field_view = view::TileField::new(view::TileFieldInfo {
            tiles: tiles_view,
            shaders: tile_shaders,
            world: world.clone(),
        });

        // block field
        let mut blocks = vec![];
        let mut blocks_view = vec![];
        for block in self.blocks {
            let block_info = block(&self.registry);

            blocks.push(dataflow::BlockInfo {
                display_name: block_info.display_name,
                description: block_info.description,
                size: block_info.size,
                collision_rect: block_info.collision_rect,
                hint_rect: block_info.rendering_rect,
                y_sorting: block_info.y_sorting,
            });

            let mut sprites = vec![];
            for sprite in block_info.sprites {
                let mut images = vec![];
                for image in sprite.images {
                    images.push(image);
                }

                sprites.push(view::BlockSpriteInfo {
                    images,
                    ticks_per_image: sprite.step_tick,
                    is_loop: sprite.is_loop,
                });
            }

            blocks_view.push(view::BlockInfo {
                sprites,
                y_sorting: block_info.y_sorting,
                rendering_rect: block_info.rendering_rect,
            });
        }

        let block_field_info = dataflow::BlockFieldInfo { blocks };

        let mut block_shaders = vec![];
        for shader in info.block_shaders {
            block_shaders.push(shader);
        }
        let block_field_view = view::BlockField::new(view::BlockFieldInfo {
            blocks: blocks_view,
            shaders: block_shaders,
            world: world.clone(),
        });

        // entity filed
        let mut entities = vec![];
        let mut entities_view = vec![];
        for entity in self.entities {
            let entity_info = entity(&self.registry);

            entities.push(dataflow::EntityInfo {
                display_name: entity_info.display_name,
                description: entity_info.description,
                collision_rect: entity_info.collision_rect,
                hint_rect: entity_info.rendering_rect,
                y_sorting: entity_info.y_sorting,
            });

            let mut sprites = vec![];
            for image in entity_info.sprites {
                let mut images = vec![];
                for image in image.images {
                    images.push(image);
                }

                sprites.push(view::EntitySpriteInfo {
                    images,
                    ticks_per_image: image.step_tick,
                    is_loop: image.is_loop,
                });
            }

            entities_view.push(view::EntityInfo {
                sprites,
                y_sorting: entity_info.y_sorting,
                rendering_rect: entity_info.rendering_rect,
            });
        }

        let entity_field_info = dataflow::EntityFieldInfo { entities };

        let mut entity_shaders = vec![];
        for shader in info.entity_shaders {
            entity_shaders.push(shader);
        }
        let entity_field_view = view::EntityField::new(view::EntityFieldInfo {
            entities: entities_view,
            shaders: entity_shaders,
            world: world.clone(),
        });

        let dataflow = dataflow::Dataflow::new(dataflow::DataflowInfo {
            tile_field: tile_field_info,
            block_field: block_field_info,
            entity_field: entity_field_info,
        });

        Context {
            registry: self.registry,
            dataflow,
            tile_field_view,
            block_field_view,
            entity_field_view,
        }
    }
}

pub struct Context {
    pub registry: Registry,
    pub dataflow: dataflow::Dataflow,
    pub tile_field_view: view::TileField,
    pub block_field_view: view::BlockField,
    pub entity_field_view: view::EntityField,
}
