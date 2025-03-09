use glam::*;
use godot::prelude::*;

use crate::*;

pub struct ImageDescriptor {
    pub frames: Vec<String>,
    pub step_tick: u16,
    pub is_loop: bool,
}

pub struct TileDescriptor<F> {
    pub images: Vec<ImageDescriptor>,
    pub collision: bool,
    pub feature: F,
}

pub struct BlockDescriptor<F> {
    pub images: Vec<ImageDescriptor>,
    pub z_along_y: bool,
    pub size: IVec2,
    pub collision_size: Vec2,
    pub collision_offset: Vec2,
    pub rendering_size: Vec2,
    pub rendering_offset: Vec2,
    pub feature: F,
}

pub struct EntityDescriptor<F> {
    pub images: Vec<ImageDescriptor>,
    pub z_along_y: bool,
    pub collision_size: Vec2,
    pub collision_offset: Vec2,
    pub rendering_size: Vec2,
    pub rendering_offset: Vec2,
    pub feature: F,
}

pub struct ItemDescriptor<F> {
    pub name_text: String,
    pub desc_text: String,
    pub image: ImageDescriptor,
    pub feature: F,
}

pub enum GenRuleDescriptor {
    March(MarchGenRuleDescriptor),
    Spawn(SpawnGenRuleDescriptor),
}

pub struct MarchGenRuleDescriptor {
    pub prob: f32,
    pub gen_fn: inner::GenFn<IVec2>,
}

pub struct SpawnGenRuleDescriptor {
    pub prob: f32,
    pub gen_fn: inner::GenFn<Vec2>,
}

pub struct BuildDescriptor {
    pub tile_shaders: Vec<String>,
    pub block_shaders: Vec<String>,
    pub entity_shaders: Vec<String>,
    pub world: Gd<godot::classes::World3D>,
}

type RegFn<R, T> = Box<dyn for<'a> FnOnce(&'a R) -> T>;
type TileFeatureBox = Box<dyn inner::TileFeature>;
type BlockFeatureBox = Box<dyn inner::BlockFeature>;
type EntityFeatureBox = Box<dyn inner::EntityFeature>;
type ItemFeatureBox = Box<dyn inner::ItemFeature>;

pub struct ContextBuilder<R> {
    tiles: Vec<RegFn<R, TileDescriptor<TileFeatureBox>>>,
    blocks: Vec<RegFn<R, BlockDescriptor<BlockFeatureBox>>>,
    entities: Vec<RegFn<R, EntityDescriptor<EntityFeatureBox>>>,
    items: Vec<RegFn<R, ItemDescriptor<ItemFeatureBox>>>,
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
            gen_rules: Default::default(),
            _phantom: Default::default(),
        }
    }

    pub fn add_tile<F, L>(&mut self, desc_fn: L) -> u16
    where
        L: FnOnce(&R) -> TileDescriptor<F> + 'static,
        F: inner::TileFeature + 'static,
    {
        let desc_fn: RegFn<R, TileDescriptor<TileFeatureBox>> = Box::new(|map| {
            let desc = desc_fn(map);
            TileDescriptor {
                images: desc.images,
                collision: desc.collision,
                feature: Box::new(desc.feature),
            }
        });
        self.tiles.push(desc_fn);
        (self.tiles.len() - 1) as u16
    }

    pub fn add_block<F, L>(&mut self, desc_fn: L) -> u16
    where
        L: FnOnce(&R) -> BlockDescriptor<F> + 'static,
        F: inner::BlockFeature + 'static,
    {
        let desc_fn: RegFn<R, BlockDescriptor<BlockFeatureBox>> = Box::new(|map| {
            let desc = desc_fn(map);
            BlockDescriptor {
                images: desc.images,
                z_along_y: desc.z_along_y,
                size: desc.size,
                collision_size: desc.collision_size,
                collision_offset: desc.collision_offset,
                rendering_size: desc.rendering_size,
                rendering_offset: desc.rendering_offset,
                feature: Box::new(desc.feature),
            }
        });
        self.blocks.push(desc_fn);
        (self.blocks.len() - 1) as u16
    }

    pub fn add_entity<F, L>(&mut self, desc_fn: L) -> u16
    where
        L: FnOnce(&R) -> EntityDescriptor<F> + 'static,
        F: inner::EntityFeature + 'static,
    {
        let desc_fn: RegFn<R, EntityDescriptor<EntityFeatureBox>> = Box::new(|map| {
            let desc = desc_fn(map);
            EntityDescriptor {
                images: desc.images,
                z_along_y: desc.z_along_y,
                collision_size: desc.collision_size,
                collision_offset: desc.collision_offset,
                rendering_size: desc.rendering_size,
                rendering_offset: desc.rendering_offset,
                feature: Box::new(desc.feature),
            }
        });
        self.entities.push(desc_fn);
        (self.entities.len() - 1) as u16
    }

    pub fn add_item<F, L>(&mut self, desc_fn: L) -> u16
    where
        L: FnOnce(&R) -> ItemDescriptor<F> + 'static,
        F: inner::ItemFeature + 'static,
    {
        let desc_fn: RegFn<R, ItemDescriptor<ItemFeatureBox>> = Box::new(|map| {
            let desc = desc_fn(map);
            ItemDescriptor {
                name_text: desc.name_text,
                desc_text: desc.desc_text,
                image: desc.image,
                feature: Box::new(desc.feature),
            }
        });
        self.items.push(desc_fn);
        (self.items.len() - 1) as u16
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

            // TODO: image
            let mut frames = vec![];
            for image in desc.image.frames {
                let image = load::<godot::classes::Image>(&image);
                frames.push(image);
            }

            let image = item::ItemImageDescriptor {
                frames,
                step_tick: desc.image.step_tick,
                is_loop: desc.image.is_loop,
            };

            items_view.push(item::ItemDescriptor {
                name_text: desc.name_text.clone(),
                desc_text: desc.desc_text.clone(),
                image,
            });
        }

        let item_store_desc = inner::ItemStoreDescriptor { items };

        let item_store_view = item::ItemStore::new(item::ItemStoreDescriptor { items: items_view });

        // gen rules
        let mut gen_rules = vec![];
        for gen_rule in self.gen_rules {
            let desc = gen_rule(&registry);

            let gen_rule = match desc {
                GenRuleDescriptor::March(desc) => inner::GenRule::March(inner::MarchGenRule {
                    prob: desc.prob,
                    gen_fn: desc.gen_fn,
                }),
                GenRuleDescriptor::Spawn(desc) => inner::GenRule::Spawn(inner::SpawnGenRule {
                    prob: desc.prob,
                    gen_fn: desc.gen_fn,
                }),
            };

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
