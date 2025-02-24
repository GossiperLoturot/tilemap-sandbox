use glam::*;
use godot::prelude::*;

use crate::inner;

pub struct ImageDescriptor {
    frames: Vec<String>,
    step_tick: u16,
    is_loop: bool,
}

impl ImageDescriptor {
    pub fn new<S>(frames: Vec<S>, step_tick: u16, is_loop: bool) -> Self
    where
        S: Into<String>,
    {
        let mut new_frames = vec![];
        for frame in frames {
            new_frames.push(frame.into());
        }

        Self {
            frames: new_frames,
            step_tick,
            is_loop,
        }
    }

    pub fn single<S>(frame: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            frames: vec![frame.into()],
            step_tick: 0,
            is_loop: false,
        }
    }
}

pub struct TileDescripter<F> {
    images: Vec<ImageDescriptor>,
    collision: bool,
    feature: F,
}

impl<F> TileDescripter<F> {
    pub fn new(images: Vec<ImageDescriptor>, collision: bool, feature: F) -> Self {
        Self {
            images,
            collision,
            feature,
        }
    }

    pub fn single(image: ImageDescriptor, collision: bool, feature: F) -> Self {
        Self {
            images: vec![image],
            collision,
            feature,
        }
    }
}

pub struct BlockDescripter<F> {
    images: Vec<ImageDescriptor>,
    z_along_y: bool,
    size: IVec2,
    collision: [Vec2; 2],
    rendering: [Vec2; 2],
    feature: F,
}

impl<F> BlockDescripter<F> {
    pub fn new(
        images: Vec<ImageDescriptor>,
        z_along_y: bool,
        size: IVec2,
        collision: [Vec2; 2],
        rendering: [Vec2; 2],
        feature: F,
    ) -> Self {
        Self {
            images,
            z_along_y,
            size,
            collision,
            rendering,
            feature,
        }
    }

    pub fn single(
        image: ImageDescriptor,
        z_along_y: bool,
        size: IVec2,
        collision: [Vec2; 2],
        rendering: [Vec2; 2],
        feature: F,
    ) -> Self {
        Self {
            images: vec![image],
            z_along_y,
            size,
            collision,
            rendering,
            feature,
        }
    }
}

pub struct EntityDescripter<F> {
    images: Vec<ImageDescriptor>,
    z_along_y: bool,
    collision: [Vec2; 2],
    rendering: [Vec2; 2],
    feature: F,
}

impl<F> EntityDescripter<F> {
    pub fn new(
        images: Vec<ImageDescriptor>,
        z_along_y: bool,
        collision: [Vec2; 2],
        rendering: [Vec2; 2],
        feature: F,
    ) -> Self {
        Self {
            images,
            z_along_y,
            collision,
            rendering,
            feature,
        }
    }

    pub fn single(
        image: ImageDescriptor,
        z_along_y: bool,
        collision: [Vec2; 2],
        rendering: [Vec2; 2],
        feature: F,
    ) -> Self {
        Self {
            images: vec![image],
            z_along_y,
            collision,
            rendering,
            feature,
        }
    }
}

pub struct ItemDescriptor<F> {
    name_text: String,
    desc_text: String,
    image: ImageDescriptor,
    feature: F,
}

impl<F> ItemDescriptor<F> {
    pub fn new<S1, S2>(name_text: S1, desc_text: S2, image: ImageDescriptor, feature: F) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            name_text: name_text.into(),
            desc_text: desc_text.into(),
            image,
            feature,
        }
    }
}

pub enum GenRuleDescriptor {
    March(MarchGenRuleDescriptor),
    Spawn(SpawnGenRuleDescriptor),
}

pub struct MarchGenRuleDescriptor {
    prob: f32,
    gen_fn: inner::GenFn<IVec2>,
}

impl MarchGenRuleDescriptor {
    pub fn new<F>(prob: f32, gen_fn: F) -> Self
    where
        F: Fn(&mut inner::Root, IVec2) + 'static,
    {
        Self {
            prob,
            gen_fn: Box::new(gen_fn),
        }
    }
}

pub struct SpawnGenRuleDescriptor {
    prob: f32,
    gen_fn: inner::GenFn<Vec2>,
}

impl SpawnGenRuleDescriptor {
    pub fn new<F>(prob: f32, gen_fn: F) -> Self
    where
        F: Fn(&mut inner::Root, Vec2) + 'static,
    {
        Self {
            prob,
            gen_fn: Box::new(gen_fn),
        }
    }
}

pub struct BuildDescriptor {
    tile_shaders: Vec<String>,
    block_shaders: Vec<String>,
    entity_shaders: Vec<String>,
    world: Gd<godot::classes::World3D>,
}

// builder

type RegFn<R, T> = Box<dyn FnOnce(R) -> T>;
type TileFeatureBox = Box<dyn inner::TileFeature>;
type BlockFeatureBox = Box<dyn inner::BlockFeature>;
type EntityFeatureBox = Box<dyn inner::EntityFeature>;
type ItemFeatureBox = Box<dyn inner::ItemFeature>;

pub struct DefineBuilder<R> {
    tiles: Vec<RegFn<R, TileDescripter<TileFeatureBox>>>,
    blocks: Vec<RegFn<R, BlockDescripter<BlockFeatureBox>>>,
    entities: Vec<RegFn<R, EntityDescripter<EntityFeatureBox>>>,
    items: Vec<RegFn<R, ItemDescriptor<ItemFeatureBox>>>,
    gen_rules: Vec<RegFn<R, GenRuleDescriptor>>,
    _phantom: std::marker::PhantomData<R>,
}

impl<R> DefineBuilder<R> {
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
        L: FnOnce(R) -> TileDescripter<F> + 'static,
        F: inner::TileFeature + 'static,
    {
        let desc_fn = |map| {
            let desc = desc_fn(map);
            TileDescripter::<TileFeatureBox>::new(
                desc.images,
                desc.collision,
                Box::new(desc.feature),
            )
        };
        self.tiles.push(Box::new(desc_fn));
        (self.tiles.len() - 1) as u16
    }

    pub fn add_block<F, L>(&mut self, desc_fn: L) -> u16
    where
        L: FnOnce(R) -> BlockDescripter<F> + 'static,
        F: inner::BlockFeature + 'static,
    {
        let desc_fn = |map| {
            let desc = desc_fn(map);
            BlockDescripter::<BlockFeatureBox>::new(
                desc.images,
                desc.z_along_y,
                desc.size,
                desc.collision,
                desc.rendering,
                Box::new(desc.feature),
            )
        };
        self.blocks.push(Box::new(desc_fn));
        (self.blocks.len() - 1) as u16
    }

    pub fn add_entity<F, L>(&mut self, desc_fn: L) -> u16
    where
        L: FnOnce(R) -> EntityDescripter<F> + 'static,
        F: inner::EntityFeature + 'static,
    {
        let desc_fn = |map: R| {
            let desc = desc_fn(map);
            EntityDescripter::<EntityFeatureBox>::new(
                desc.images,
                desc.z_along_y,
                desc.collision,
                desc.rendering,
                Box::new(desc.feature),
            )
        };
        self.entities.push(Box::new(desc_fn));
        (self.entities.len() - 1) as u16
    }

    pub fn add_item<F, L>(&mut self, desc_fn: L) -> u16
    where
        L: FnOnce(R) -> ItemDescriptor<F> + 'static,
        F: inner::ItemFeature + 'static,
    {
        let desc_fn = |map: R| {
            let desc = desc_fn(map);
            ItemDescriptor::<ItemFeatureBox>::new(
                desc.name_text,
                desc.desc_text,
                desc.image,
                Box::new(desc.feature),
            )
        };
        self.items.push(Box::new(desc_fn));
        (self.items.len() - 1) as u16
    }

    pub fn add_gen_rule<L>(&mut self, desc_fn: L) -> u16
    where
        L: FnOnce(R) -> GenRuleDescriptor + 'static,
    {
        self.gen_rules.push(Box::new(desc_fn));
        (self.gen_rules.len() - 1) as u16
    }

    pub fn build(self, desc: BuildDescriptor, registry: R) -> Define {
        // base
        let base = {
            let mut tile_features = vec![];
            let tile_field = {
                let mut tiles = vec![];
                for tile in self.tiles {
                    let desc = (*tile)(registry);

                    tiles.push(inner::TileDescriptor {
                        collision: desc.collision,
                    });

                    tile_features.push(desc.feature);
                }

                inner::TileFieldDescriptor { tiles }
            };
            let tile_features = tile_features.into();

            let mut block_features = vec![];
            let block_field = {
                let mut blocks = vec![];
                for block in self.blocks {
                    let desc = (*block)(registry);

                    blocks.push(inner::BlockDescriptor {
                        size: desc.size,
                        collision_size: desc.collision[0],
                        collision_offset: desc.collision[1],
                        hint_size: desc.rendering[0],
                        hint_offset: desc.rendering[1],
                    });

                    block_features.push(desc.feature);
                }

                inner::BlockFieldDescriptor { blocks }
            };
            let block_features = block_features.into();

            let mut entity_features = vec![];
            let entity_field = {
                let mut entities = vec![];
                for entity in self.entities.iter() {
                    let desc = (*entity)(registry);

                    entities.push(inner::EntityDescriptor {
                        collision_size: desc.collision[0],
                        collision_offset: desc.collision[1],
                        hint_size: desc.rendering[0],
                        hint_offset: desc.rendering[1],
                    });

                    entity_features.push(desc.feature);
                }

                inner::EntityFieldDescriptor { entities }
            };
            let entity_features = entity_features.into();

            let mut item_features = vec![];
            let item_store = {
                let mut items = vec![];
                for item in self.items.iter() {
                    let desc = (*item)(registry);

                    items.push(inner::ItemDescriptor {
                        name_text: desc.name_text,
                    });

                    item_features.push(desc.feature);
                }

                inner::ItemStoreDescriptor { items }
            };
            let item_features = item_features.into();

            let gen_resource = {
                let mut gen_rules = vec![];
                for gen_rule in self.gen_rules.iter() {
                    let desc = (*gen_rule)(registry);

                    let gen_rule = match desc {
                        GenRuleDescriptor::March(desc) => {
                            inner::GenRule::March(inner::MarchGenRule {
                                prob: desc.prob,
                                gen_fn: desc.gen_fn,
                            })
                        }
                        GenRuleDescriptor::Spawn(desc) => {
                            inner::GenRule::Spawn(inner::SpawnGenRule {
                                prob: desc.prob,
                                gen_fn: desc.gen_fn,
                            })
                        }
                    };

                    gen_rules.push(gen_rule);
                }

                inner::GenResourceDescriptor { gen_rules }
            };

            inner::Root::new(inner::RootDescriptor {
                tile_field,
                block_field,
                entity_field,
                item_store,

                tile_features,
                block_features,
                entity_features,
                item_features,

                gen_resource,
            })
        };

        // tile field renderer
        let tile_field = {
            let mut tiles = vec![];
            for tile in self.tiles.iter() {
                let tile = tile.bind();

                let mut images = vec![];
                for image in tile.images.iter_shared() {
                    let image = image.bind();

                    let mut frames = vec![];
                    for image in image.frames.iter_shared() {
                        frames.push(image);
                    }

                    images.push(tile::TileImageDescriptor {
                        frames,
                        step_tick: image.step_tick,
                        is_loop: image.is_loop,
                    });
                }

                tiles.push(tile::TileDescriptor { images });
            }

            let mut tile_shaders = vec![];
            for shader in desc.shaders.iter_shared() {
                tile_shaders.push(shader);
            }

            tile::TileField::new(tile::TileFieldDescriptor {
                tiles,
                shaders: tile_shaders,
                world: desc.world.clone(),
            })
        };

        // block field renderer
        let block_field = {
            let desc = desc.bind();
            let desc = desc.block_field.bind();

            let mut blocks = vec![];
            for block in desc.blocks.iter_shared() {
                let block = block.bind();

                let mut images = vec![];
                for image in block.images.iter_shared() {
                    let image = image.bind();

                    let mut frames = vec![];
                    for image in image.frames.iter_shared() {
                        frames.push(image);
                    }

                    images.push(block::BlockImageDescriptor {
                        frames,
                        step_tick: image.step_tick,
                        is_loop: image.is_loop,
                    });
                }

                blocks.push(block::BlockDescriptor {
                    images,
                    z_along_y: block.z_along_y,
                    rendering_size: Vec2::new(block.rendering_size.x, block.rendering_size.y),
                    rendering_offset: Vec2::new(block.rendering_offset.x, block.rendering_offset.y),
                });
            }

            let mut block_shaders = vec![];
            for shader in desc.shaders.iter_shared() {
                block_shaders.push(shader);
            }

            block::BlockField::new(block::BlockFieldDescriptor {
                blocks,
                shaders: block_shaders,
                world: desc.world.clone(),
            })
        };

        // entity field renderer
        let entity_field = {
            let desc = desc.bind();
            let desc = desc.entity_field.bind();

            let mut entities = vec![];
            for entity in desc.entities.iter_shared() {
                let entity = entity.bind();

                let mut images = vec![];
                for image in entity.images.iter_shared() {
                    let image = image.bind();

                    let mut frames = vec![];
                    for image in image.frames.iter_shared() {
                        frames.push(image);
                    }

                    images.push(entity::EntityImageDescriptor {
                        frames,
                        step_tick: image.step_tick,
                        is_loop: image.is_loop,
                    });
                }

                entities.push(entity::EntityDescriptor {
                    images,
                    z_along_y: entity.z_along_y,
                    rendering_size: Vec2::new(entity.rendering_size.x, entity.rendering_size.y),
                    rendering_offset: Vec2::new(
                        entity.rendering_offset.x,
                        entity.rendering_offset.y,
                    ),
                });
            }

            let mut entity_shaders = vec![];
            for shader in desc.shaders.iter_shared() {
                entity_shaders.push(shader.clone());
            }

            entity::EntityField::new(entity::EntityFieldDescriptor {
                entities,
                shaders: entity_shaders,
                world: desc.world.clone(),
            })
        };

        // item store renderer
        let item_store = {
            let desc = desc.bind();
            let desc = desc.item_store.bind();

            let mut items = vec![];
            for item in desc.items.iter_shared() {
                let item = item.bind();

                items.push(item::ItemDescriptor {
                    name_text: item.name_text.clone(),
                    desc_text: item.desc_text.clone(),
                    image: item.image.clone(),
                });
            }

            item::ItemStore::new(item::ItemStoreDescriptor { items })
        };

        Gd::from_object(Root {
            base,
            tile_field,
            block_field,
            entity_field,
            item_store,
        })
    }
}

// define

struct Registry {
    tile_dirt: u16,
    tile_grass: u16,
    block_dandelion: u16,
    block_fallen_leaves: u16,
    block_mix_grass: u16,
    block_mix_pebbles: u16,
    entity_player: u16,
    entity_pig: u16,
    entity_cow: u16,
    entity_sheep: u16,
    entity_chicken: u16,
    entity_bird: u16,
    item_package: u16,
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct Define {}

#[godot_api]
impl Define {
    #[func]
    pub fn create() -> Gd<Self> {
        let mut builder = DefineBuilder::<Registry>::new();

        // tiles
        let tile_dirt = builder.add_tile(|_| {
            TileDescripter::single(
                ImageDescriptor::single("res://images/surface_dirt.webp"),
                false,
                (),
            )
        });
        let tile_grass = builder.add_tile(|_| {
            TileDescripter::single(
                ImageDescriptor::single("res://images/surface_grass.webp"),
                false,
                (),
            )
        });

        // blocks
        let block_dandelion = builder.add_block(|_| {
            BlockDescripter::single(
                ImageDescriptor::single("res://images/dandelion.webp"),
                false,
                IVec2::new(1, 1),
                [Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)],
                [Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)],
                (),
            )
        });
        let block_fallen_leaves = builder.add_block(|_| {
            BlockDescripter::single(
                ImageDescriptor::single("res://images/fallen_leaves.webp"),
                false,
                IVec2::new(1, 1),
                [Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)],
                [Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)],
                (),
            )
        });
        let block_mix_grass = builder.add_block(|_| {
            BlockDescripter::single(
                ImageDescriptor::single("res://images/mix_grass.webp"),
                false,
                IVec2::new(1, 1),
                [Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)],
                [Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)],
                (),
            )
        });
        let block_mix_pebbles = builder.add_block(|_| {
            BlockDescripter::single(
                ImageDescriptor::single("res://images/mix_pebbles.webp"),
                false,
                IVec2::new(1, 1),
                [Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)],
                [Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)],
                (),
            )
        });

        // entities
        let entity_player = builder.add_entity(|_| {
            EntityDescripter::new(
                vec![
                    ImageDescriptor::new(
                        vec![
                            "res://images/player_idle_0.webp",
                            "res://images/player_idle_1.webp",
                        ],
                        24,
                        true,
                    ),
                    ImageDescriptor::new(
                        vec![
                            "res://images/player_walk_0.webp",
                            "res://images/player_idle_0.webp",
                            "res://images/player_walk_1.webp",
                            "res://images/player_idle_1.webp",
                        ],
                        6,
                        true,
                    ),
                ],
                true,
                [Vec2::new(-0.4, 0.1), Vec2::new(0.4, 0.9)],
                [Vec2::new(-0.75, 0.0), Vec2::new(0.75, 2.25)],
                inner::PlayerEntityFeature,
            )
        });
        let entity_pig = builder.add_entity(|_| {
            EntityDescripter::new(
                vec![
                    ImageDescriptor::new(
                        vec![
                            "res://images/pig_idle_0.webp",
                            "res://images/pig_idle_1.webp",
                        ],
                        24,
                        true,
                    ),
                    ImageDescriptor::new(
                        vec![
                            "res://images/pig_walk_0.webp",
                            "res://images/pig_idle_0.webp",
                            "res://images/pig_walk_1.webp",
                            "res://images/pig_idle_1.webp",
                        ],
                        12,
                        true,
                    ),
                ],
                true,
                [Vec2::new(-0.4, 0.1), Vec2::new(0.4, 0.9)],
                [Vec2::new(-1.0, 0.0), Vec2::new(1.0, 2.0)],
                inner::AnimalEntityFeature {
                    min_rest_secs: 0.0,
                    max_rest_secs: 10.0,
                    min_distance: 0.0,
                    max_distance: 10.0,
                    speed: 1.0,
                    idle_variant: 0,
                    walk_variant: 1,
                },
            )
        });
        let entity_cow = builder.add_entity(|_| {
            EntityDescripter::new(
                vec![
                    ImageDescriptor::new(
                        vec![
                            "res://images/cow_idle_0.webp",
                            "res://images/cow_idle_1.webp",
                        ],
                        24,
                        true,
                    ),
                    ImageDescriptor::new(
                        vec![
                            "res://images/cow_walk_0.webp",
                            "res://images/cow_idle_0.webp",
                            "res://images/cow_walk_1.webp",
                            "res://images/cow_idle_1.webp",
                        ],
                        12,
                        true,
                    ),
                ],
                true,
                [Vec2::new(-0.4, 0.1), Vec2::new(0.4, 0.9)],
                [Vec2::new(-1.0, 0.0), Vec2::new(1.0, 2.0)],
                inner::AnimalEntityFeature {
                    min_rest_secs: 0.0,
                    max_rest_secs: 10.0,
                    min_distance: 0.0,
                    max_distance: 10.0,
                    speed: 1.0,
                    idle_variant: 0,
                    walk_variant: 1,
                },
            )
        });
        let entity_sheep = builder.add_entity(|_| {
            EntityDescripter::new(
                vec![
                    ImageDescriptor::new(
                        vec![
                            "res://images/sheep_idle_0.webp",
                            "res://images/sheep_idle_1.webp",
                        ],
                        24,
                        true,
                    ),
                    ImageDescriptor::new(
                        vec![
                            "res://images/sheep_walk_0.webp",
                            "res://images/sheep_idle_0.webp",
                            "res://images/sheep_walk_1.webp",
                            "res://images/sheep_idle_1.webp",
                        ],
                        12,
                        true,
                    ),
                ],
                true,
                [Vec2::new(-0.4, 0.1), Vec2::new(0.4, 0.9)],
                [Vec2::new(-1.0, 0.0), Vec2::new(1.0, 2.0)],
                inner::AnimalEntityFeature {
                    min_rest_secs: 0.0,
                    max_rest_secs: 10.0,
                    min_distance: 0.0,
                    max_distance: 10.0,
                    speed: 1.0,
                    idle_variant: 0,
                    walk_variant: 1,
                },
            )
        });
        let entity_chicken = builder.add_entity(|_| {
            EntityDescripter::new(
                vec![
                    ImageDescriptor::single("res://images/chicken_idle.webp"),
                    ImageDescriptor::new(
                        vec![
                            "res://images/chicken_walk.webp",
                            "res://images/chicken_idle.webp",
                        ],
                        12,
                        true,
                    ),
                ],
                true,
                [Vec2::new(-0.4, 0.1), Vec2::new(0.4, 0.9)],
                [Vec2::new(-0.5, 0.0), Vec2::new(0.5, 1.0)],
                inner::AnimalEntityFeature {
                    min_rest_secs: 0.0,
                    max_rest_secs: 10.0,
                    min_distance: 0.0,
                    max_distance: 10.0,
                    speed: 1.0,
                    idle_variant: 0,
                    walk_variant: 1,
                },
            )
        });
        let entity_bird = builder.add_entity(|_| {
            EntityDescripter::new(
                vec![
                    ImageDescriptor::single("res://images/bird_idle.webp"),
                    ImageDescriptor::new(
                        vec![
                            "res://images/chicken_walk.webp",
                            "res://images/chicken_idle.webp",
                        ],
                        12,
                        true,
                    ),
                ],
                true,
                [Vec2::new(-0.4, 0.1), Vec2::new(0.4, 0.9)],
                [Vec2::new(-0.5, 0.0), Vec2::new(0.5, 1.0)],
                inner::AnimalEntityFeature {
                    min_rest_secs: 0.0,
                    max_rest_secs: 10.0,
                    min_distance: 0.0,
                    max_distance: 10.0,
                    speed: 1.0,
                    idle_variant: 0,
                    walk_variant: 1,
                },
            )
        });

        // item
        let item_package = builder.add_item(|_| {
            ItemDescriptor::new(
                "Package",
                "A package of items.",
                ImageDescriptor::single("res://images/package.webp"),
                (),
            )
        });

        // gen rule
        builder.add_gen_rule(|reg| {
            GenRuleDescriptor::March(MarchGenRuleDescriptor::new(1.0, move |root, location| {
                let tile = inner::Tile {
                    id: reg.tile_dirt,
                    location,
                    data: Default::default(),
                    render_param: Default::default(),
                };
                let _ = root.tile_insert(tile);
            }))
        });

        let register = Registry {
            tile_dirt,
            tile_grass,
            block_dandelion,
            block_fallen_leaves,
            block_mix_grass,
            block_mix_pebbles,
            entity_player,
            entity_pig,
            entity_cow,
            entity_sheep,
            entity_chicken,
            entity_bird,
            item_package,
        };
        Gd::from_object(builder.build(register))
    }
}
