use glam::*;
use godot::prelude::*;

use crate::*;

pub struct ImageDescriptor {
    pub frames: Vec<String>,
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
    pub scene: String,
}

pub struct GenRuleDescriptor {
    pub gen_rule: Box<dyn inner::GenRule>,
}

pub struct BuildDescriptor {
    pub tile_shaders: Vec<String>,
    pub block_shaders: Vec<String>,
    pub entity_shaders: Vec<String>,
    pub world: Gd<godot::classes::World3D>,
    pub node: Gd<godot::classes::Node>,
}

type RegFn<R, T> = Box<dyn for<'a> FnOnce(&'a R) -> T>;

pub struct ContextBuilder<R> {
    tiles: Vec<RegFn<R, TileDescriptor>>,
    blocks: Vec<RegFn<R, BlockDescriptor>>,
    entities: Vec<RegFn<R, EntityDescriptor>>,
    items: Vec<RegFn<R, ItemDescriptor>>,
    inventories: Vec<RegFn<R, InventoryDescriptor>>,
    gen_rules: Vec<RegFn<R, GenRuleDescriptor>>,
    _phantom: std::marker::PhantomData<R>,
}

impl<R> ContextBuilder<R> {
    pub fn new() -> Self {
        Self {
            tiles: Default::default(),
            blocks: Default::default(),
            entities: Default::default(),
            items: Default::default(),
            inventories: Default::default(),
            gen_rules: Default::default(),
            _phantom: Default::default(),
        }
    }

    pub fn add_tile<L>(&mut self, desc_fn: L) -> u16
    where
        L: FnOnce(&R) -> TileDescriptor + 'static,
    {
        self.tiles.push(Box::new(desc_fn));
        (self.tiles.len() - 1) as u16
    }

    pub fn add_block<L>(&mut self, desc_fn: L) -> u16
    where
        L: FnOnce(&R) -> BlockDescriptor + 'static,
    {
        self.blocks.push(Box::new(desc_fn));
        (self.blocks.len() - 1) as u16
    }

    pub fn add_entity<L>(&mut self, desc_fn: L) -> u16
    where
        L: FnOnce(&R) -> EntityDescriptor + 'static,
    {
        self.entities.push(Box::new(desc_fn));
        (self.entities.len() - 1) as u16
    }

    pub fn add_item<L>(&mut self, desc_fn: L) -> u16
    where
        L: FnOnce(&R) -> ItemDescriptor + 'static,
    {
        self.items.push(Box::new(desc_fn));
        (self.items.len() - 1) as u16
    }

    pub fn add_inventory<L>(&mut self, desc_fn: L) -> u16
    where
        L: FnOnce(&R) -> InventoryDescriptor + 'static,
    {
        self.inventories.push(Box::new(desc_fn));
        (self.inventories.len() - 1) as u16
    }

    pub fn add_gen_rule<L>(&mut self, desc_fn: L) -> u16
    where
        L: FnOnce(&R) -> GenRuleDescriptor + 'static,
    {
        self.gen_rules.push(Box::new(desc_fn));
        (self.gen_rules.len() - 1) as u16
    }

    pub fn build(self, registry: R, desc: BuildDescriptor) -> Context<R> {
        // tile field
        let mut tile_features = vec![];
        let mut tiles = vec![];
        let mut tiles_view = vec![];
        for tile in self.tiles {
            let desc = tile(&registry);

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
                    let image = load::<godot::classes::Image>(&image);
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
        for shader in &desc.tile_shaders {
            let shader = load::<godot::classes::Shader>(shader);
            tile_shaders.push(shader);
        }
        let tile_field_view = tile::TileField::new(tile::TileFieldDescriptor {
            tiles: tiles_view,
            shaders: tile_shaders,
            world: desc.world.clone(),
        });

        // block field
        let mut block_features = vec![];
        let mut blocks = vec![];
        let mut blocks_view = vec![];
        for block in self.blocks {
            let desc = block(&registry);

            block_features.push(desc.feature);

            blocks.push(inner::BlockDescriptor {
                name_text: desc.name_text,
                desc_text: desc.desc_text,
                size: desc.size,
                collision_size: desc.collision_size,
                collision_offset: desc.collision_offset,
                hint_size: desc.rendering_size,
                hint_offset: desc.rendering_offset,
            });

            let mut images = vec![];
            for image in desc.images {
                let mut frames = vec![];
                for image in image.frames {
                    let image = load::<godot::classes::Image>(&image);
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
        for shader in &desc.block_shaders {
            let shader = load::<godot::classes::Shader>(shader);
            block_shaders.push(shader);
        }
        let block_field_view = block::BlockField::new(block::BlockFieldDescriptor {
            blocks: blocks_view,
            shaders: block_shaders,
            world: desc.world.clone(),
        });

        // entity filed
        let mut entity_features = vec![];
        let mut entities = vec![];
        let mut entities_view = vec![];
        for entity in self.entities {
            let desc = entity(&registry);

            entity_features.push(desc.feature);

            entities.push(inner::EntityDescriptor {
                name_text: desc.name_text,
                desc_text: desc.desc_text,
                collision_size: desc.collision_size,
                collision_offset: desc.collision_offset,
                hint_size: desc.rendering_size,
                hint_offset: desc.rendering_offset,
            });

            let mut images = vec![];
            for image in desc.images {
                let mut frames = vec![];
                for image in image.frames {
                    let image = load::<godot::classes::Image>(&image);
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
        for shader in &desc.entity_shaders {
            let shader = load::<godot::classes::Shader>(shader);
            entity_shaders.push(shader);
        }
        let entity_field_view = entity::EntityField::new(entity::EntityFieldDescriptor {
            entities: entities_view,
            shaders: entity_shaders,
            world: desc.world.clone(),
        });

        // item field
        let mut item_features = vec![];
        let mut items = vec![];
        let mut items_view = vec![];
        for item in self.items {
            let desc = item(&registry);

            item_features.push(desc.feature);

            items.push(inner::ItemDescriptor {
                name_text: desc.name_text.clone(),
            });

            let mut images = vec![];
            for image in desc.images {
                let mut frames = vec![];
                for image in image.frames {
                    let image = load::<godot::classes::Image>(&image);
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
            let desc = inventory(&registry);

            let scene = load::<godot::classes::PackedScene>(&desc.scene);

            inventories.push(inner::InventoryDescriptor { size: desc.size });

            inventories_view.push(item::InventoryDescriptor {
                scene: scene.clone(),
            });
        }

        let item_store_desc = inner::ItemStoreDescriptor { items, inventories };

        let item_store_view = item::ItemStore::new(item::ItemStoreDescriptor {
            items: items_view,
            inventories: inventories_view,
            node: desc.node,
        });

        // gen rules
        let mut gen_rules = vec![];
        for gen_rule in self.gen_rules {
            let desc = gen_rule(&registry);

            let gen_rule = desc.gen_rule;

            gen_rules.push(gen_rule);
        }

        let gen_resource_desc = inner::GenResourceDescriptor { gen_rules };

        let root = inner::Root::new(inner::RootDescriptor {
            tile_field: tile_field_desc,
            block_field: block_field_desc,
            entity_field: entity_field_desc,
            item_store: item_store_desc,

            tile_features: tile_features.into(),
            block_features: block_features.into(),
            entity_features: entity_features.into(),
            item_features: item_features.into(),

            gen_resource: gen_resource_desc,
        });
        Context {
            root,
            tile_field: tile_field_view,
            block_field: block_field_view,
            entity_field: entity_field_view,
            item_store: item_store_view,
            registry,
        }
    }
}

pub struct Context<R> {
    pub root: inner::Root,
    pub tile_field: tile::TileField,
    pub block_field: block::BlockField,
    pub entity_field: entity::EntityField,
    pub item_store: item::ItemStore,
    pub registry: R,
}
