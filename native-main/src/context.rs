use glam::*;
use godot::prelude::*;

use native_core as core;
use crate::addon;

#[derive(GodotClass)]
#[class(init, base=Object)]
pub struct Context {
    context: Option<core::Context>,
}

#[godot_api]
impl Context {
    #[func]
    fn open(&mut self, retrieve_callable: Callable) {
        let mut builder = core::ContextBuilder::new();

        // dirt tile
        builder.add_tile("tile_dirt".into(), |_, retriever| core::TileInfo {
            display_name: "Dirt".into(),
            description: Default::default(),
            sprites: vec![core::SpriteInfo {
                images: vec![retriever.load("image_tile_dirt")],
                step_tick: 0,
                is_loop: false,
            }],
            collision: false,
        });

        // grass tile
        builder.add_tile("tile_grass".into(), |_, retriever| core::TileInfo {
            display_name: "Grass".into(),
            description: Default::default(),
            sprites: vec![core::SpriteInfo {
                images: vec![retriever.load("image_tile_grass")],
                step_tick: 0,
                is_loop: false,
            }],
            collision: false,
        });

        // dandelion block
        builder.add_block("block_dandelion".into(), |_, retriever| core::BlockInfo {
            display_name: "Dandelion".into(),
            description: Default::default(),
            sprites: vec![core::SpriteInfo {
                images: vec![retriever.load("image_block_dandelion")],
                step_tick: 0,
                is_loop: false,
            }],
            y_sorting: false,
            size: IVec2::new(1, 1),
            collision_rect: None,
            rendering_rect: core::Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
        });

        // fallen leaves block
        builder.add_block("block_fallenleaves".into(), |_, retriever| core::BlockInfo {
            display_name: "Fallen Leaves".into(),
            description: Default::default(),
            sprites: vec![core::SpriteInfo {
                images: vec![retriever.load("image_block_fallenleaves")],
                step_tick: 0,
                is_loop: false,
            }],
            y_sorting: false,
            size: IVec2::new(1, 1),
            collision_rect: None,
            rendering_rect: core::Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
        });

        // mix grass block
        builder.add_block("block_mixgrass".into(), |_, retriever| core::BlockInfo {
            display_name: "Grass".into(),
            description: Default::default(),
            sprites: vec![core::SpriteInfo {
                images: vec![retriever.load("image_block_mixgrass")],
                step_tick: 0,
                is_loop: false,
            }],
            y_sorting: true,
            size: IVec2::new(1, 1),
            collision_rect: None,
            rendering_rect: core::Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
        });

        // mix pebbles block
        builder.add_block("block_mixpebbles".into(), |_, retriever| core::BlockInfo {
            display_name: "Pebbles".into(),
            description: Default::default(),
            sprites: vec![core::SpriteInfo {
                images: vec![retriever.load("image_block_mixpebbles")],
                step_tick: 0,
                is_loop: false,
            }],
            y_sorting: false,
            size: IVec2::new(1, 1),
            collision_rect: None,
            rendering_rect: core::Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
        });

        // oak tree block
        builder.add_block("block_oaktree".into(), |_, retriever| core::BlockInfo {
            display_name: "Oak Tree".into(),
            description: Default::default(),
            sprites: vec![core::SpriteInfo {
                images: vec![
                    retriever.load("image_block_oaktree0"),
                    retriever.load("image_block_oaktree1"),
                ],
                step_tick: 48,
                is_loop: true,
            }],
            y_sorting: true,
            size: IVec2::new(4, 2),
            collision_rect: Some(core::Rect2::new(Vec2::new(-0.5, 0.0), Vec2::new(0.5, 2.0))),
            rendering_rect: core::Rect2::new(Vec2::new(-2.0, 0.0), Vec2::new(2.0, 6.0)),
        });

        // player entity
        builder.add_entity("entity_player".into(), |_, retriever| core::EntityInfo {
            display_name: "Player".into(),
            description: Default::default(),
            sprites: vec![
                core::SpriteInfo {
                    images: vec![
                        retriever.load("image_entity_player_idle0"),
                        retriever.load("image_entity_player_idle1"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                core::SpriteInfo {
                    images: vec![
                        retriever.load("image_entity_player_idle0r"),
                        retriever.load("image_entity_player_idle1r"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                core::SpriteInfo {
                    images: vec![
                        retriever.load("image_entity_player_walk0"),
                        retriever.load("image_entity_player_idle0"),
                        retriever.load("image_entity_player_walk1"),
                        retriever.load("image_entity_player_idle1"),
                    ],
                    step_tick: 6,
                    is_loop: true,
                },
                core::SpriteInfo {
                    images: vec![
                        retriever.load("image_entity_player_walk0r"),
                        retriever.load("image_entity_player_idle0r"),
                        retriever.load("image_entity_player_walk1r"),
                        retriever.load("image_entity_player_idle1r"),
                    ],
                    step_tick: 6,
                    is_loop: true,
                },
            ],
            y_sorting: true,
            collision_rect: Some(core::Rect2::new(Vec2::new(-0.4, 0.1), Vec2::new(0.4, 0.9))),
            rendering_rect: core::Rect2::new(Vec2::new(-0.75, 0.0), Vec2::new(0.75, 2.25)),
        });

        // pig entity
        builder.add_entity("entity_pig".into(), |_, retriever| core::EntityInfo {
            display_name: "Pig".into(),
            description: Default::default(),
            sprites: vec![
                core::SpriteInfo {
                    images: vec![
                        retriever.load("image_entity_pig_idle0"),
                        retriever.load("image_entity_pig_idle1"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                core::SpriteInfo {
                    images: vec![
                        retriever.load("image_entity_pig_walk0"),
                        retriever.load("image_entity_pig_idle0"),
                        retriever.load("image_entity_pig_walk1"),
                        retriever.load("image_entity_pig_idle1"),
                    ],
                    step_tick: 12,
                    is_loop: true,
                },
            ],
            y_sorting: true,
            collision_rect: Some(core::Rect2::new(Vec2::new(-0.4, 0.1), Vec2::new(0.4, 0.9))),
            rendering_rect: core::Rect2::new(Vec2::new(-1.0, 0.0), Vec2::new(1.0, 2.0)),
        });

        // cow entity
        builder.add_entity("entity_cow".into(), |_, retriever| core::EntityInfo {
            display_name: "Cow".into(),
            description: Default::default(),
            sprites: vec![
                core::SpriteInfo {
                    images: vec![
                        retriever.load("image_entity_cow_idle0"),
                        retriever.load("image_entity_cow_idle1"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                core::SpriteInfo {
                    images: vec![
                        retriever.load("image_entity_cow_walk0"),
                        retriever.load("image_entity_cow_idle0"),
                        retriever.load("image_entity_cow_walk1"),
                        retriever.load("image_entity_cow_idle1"),
                    ],
                    step_tick: 12,
                    is_loop: true,
                },
            ],
            y_sorting: true,
            collision_rect: Some(core::Rect2::new(Vec2::new(-0.4, 0.1), Vec2::new(0.4, 0.9))),
            rendering_rect: core::Rect2::new(Vec2::new(-1.0, 0.0), Vec2::new(1.0, 2.0)),
        });

        // sheep entity
        builder.add_entity("entity_sheep".into(), |_, retriever| core::EntityInfo {
            display_name: "Sheep".into(),
            description: Default::default(),
            sprites: vec![
                core::SpriteInfo {
                    images: vec![
                        retriever.load("image_entity_sheep_idle0"),
                        retriever.load("image_entity_sheep_idle1"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                core::SpriteInfo {
                    images: vec![
                        retriever.load("image_entity_sheep_walk0"),
                        retriever.load("image_entity_sheep_idle0"),
                        retriever.load("image_entity_sheep_walk1"),
                        retriever.load("image_entity_sheep_idle1"),
                    ],
                    step_tick: 12,
                    is_loop: true,
                },
            ],
            y_sorting: true,
            collision_rect: Some(core::Rect2::new(Vec2::new(-0.4, 0.1), Vec2::new(0.4, 0.9))),
            rendering_rect: core::Rect2::new(Vec2::new(-1.0, 0.0), Vec2::new(1.0, 2.0)),
        });

        // chicken entity
        builder.add_entity("entity_chicken".into(), |_, retriever| core::EntityInfo {
            display_name: "Chicken".into(),
            description: Default::default(),
            sprites: vec![
                core::SpriteInfo {
                    images: vec![retriever.load("image_entity_chicken_idle")],
                    step_tick: 24,
                    is_loop: true,
                },
                core::SpriteInfo {
                    images: vec![
                        retriever.load("image_entity_chicken_walk"),
                        retriever.load("image_entity_chicken_idle"),
                    ],
                    step_tick: 12,
                    is_loop: true,
                },
            ],
            y_sorting: true,
            collision_rect: Some(core::Rect2::new(Vec2::new(-0.4, 0.1), Vec2::new(0.4, 0.9))),
            rendering_rect: core::Rect2::new(Vec2::new(-0.5, 0.0), Vec2::new(0.5, 1.0)),
        });

        // bird entity
        builder.add_entity("entity_bird".into(), |_, retriever| core::EntityInfo {
            display_name: "Bird".into(),
            description: Default::default(),
            sprites: vec![
                core::SpriteInfo {
                    images: vec![retriever.load("image_entity_bird_idle")],
                    step_tick: 24,
                    is_loop: true,
                },
                core::SpriteInfo {
                    images: vec![
                        retriever.load("image_entity_bird_walk"),
                        retriever.load("image_entity_bird_idle"),
                    ],
                    step_tick: 12,
                    is_loop: true,
                },
            ],
            y_sorting: true,
            collision_rect: Some(core::Rect2::new(Vec2::new(-0.4, 0.1), Vec2::new(0.4, 0.9))),
            rendering_rect: core::Rect2::new(Vec2::new(-0.5, 0.0), Vec2::new(0.5, 1.0)),
        });

        // package entity
        builder.add_entity("entity_package".into(), |_, retriever| core::EntityInfo {
            display_name: "Package".into(),
            description: Default::default(),
            sprites: vec![core::SpriteInfo {
                images: vec![retriever.load("image_entity_package")],
                step_tick: 0,
                is_loop: false,
            }],
            y_sorting: true,
            collision_rect: None,
            rendering_rect: core::Rect2::new(Vec2::new(-0.25, -0.25), Vec2::new(0.25, 0.25)),
        });

        // particle entity
        builder.add_entity("entity_particle".into(), |_, retriever| core::EntityInfo {
            display_name: "Particle".into(),
            description: Default::default(),
            sprites: vec![core::SpriteInfo {
                images: vec![
                    retriever.load("image_entity_particle0"),
                    retriever.load("image_entity_particle1"),
                ],
                step_tick: 4,
                is_loop: false,
            }],
            y_sorting: true,
            collision_rect: None,
            rendering_rect: core::Rect2::new(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0)),
        });

        // grass item
        builder.add_item("item_grass".into(), |_, retriever| core::ItemInfo {
            display_name: "Grass".into(),
            description: Default::default(),
            sprites: vec![core::SpriteInfo {
                images: vec![retriever.load("image_item_grass")],
                step_tick: 0,
                is_loop: false,
            }],
        });

        // fallen leaves item
        builder.add_item("item_fallenleaves".into(), |_, retriever| core::ItemInfo {
            display_name: "Fallen Leaves".into(),
            description: Default::default(),
            sprites: vec![core::SpriteInfo {
                images: vec![retriever.load("image_item_fallenleaves")],
                step_tick: 0,
                is_loop: false,
            }],
        });

        // mix pebbles item
        builder.add_item("item_mixpebbles".into(), |_, retriever| core::ItemInfo {
            display_name: "Pebbles".into(),
            description: Default::default(),
            sprites: vec![core::SpriteInfo {
                images: vec![retriever.load("image_item_mixpebbles")],
                step_tick: 0,
                is_loop: false,
            }],
        });

        // wood item
        builder.add_item("item_wood".into(), |_, retriever| core::ItemInfo {
            display_name: "Wood".into(),
            description: Default::default(),
            sprites: vec![core::SpriteInfo {
                images: vec![retriever.load("image_item_wood")],
                step_tick: 0,
                is_loop: false,
            }],
        });

        // build
        let retriever = core::Retriever::new(retrieve_callable);
        let desc = core::BuildInfo {
            tile_shaders: vec![retriever.load("shader_field")],
            block_shaders: vec![
                retriever.load("shader_field"),
                retriever.load("shader_field_shadow"),
            ],
            entity_shaders: vec![
                retriever.load("shader_field"),
                retriever.load("shader_field_shadow"),
            ],
            viewport: retriever.load("viewport"),
        };
        let mut context = builder.build(&retriever, desc);

        // field generator
        let desc = addon::GeneratorResourceDescriptor {
            generators: vec![
                Box::new(addon::DiscreteGenerator {
                    probability: 0.75,
                    sample_fn: {
                        let archetype_id = context.registry.get("tile_grass");
                        Box::new(move |dataflow, coord| {
                            let tile = core::dataflow::Tile { archetype_id, coord, ..Default::default() };
                            let _ = dataflow.insert_tile(tile);
                        })
                    }
                }),
                Box::new(addon::DiscreteGenerator {
                    probability: 1.0,
                    sample_fn: {
                        let archetype_id = context.registry.get("tile_dirt");
                        Box::new(move |dataflow, coord| {
                            let tile = core::dataflow::Tile { archetype_id, coord, ..Default::default() };
                            let _ = dataflow.insert_tile(tile);
                        })
                    }
                }),
                Box::new(addon::DiscreteGenerator {
                    probability: 0.01,
                    sample_fn: {
                        let archetype_id = context.registry.get("block_oaktree");
                        Box::new(move |dataflow, coord| {
                            let block = core::dataflow::Block { archetype_id, coord, ..Default::default() };
                            let _ = dataflow.insert_block(block);
                        })
                    }
                }),
                Box::new(addon::DiscreteGenerator {
                    probability: 0.1,
                    sample_fn: {
                        let archetype_id = context.registry.get("block_dandelion");
                        Box::new(move |dataflow, coord| {
                            let block = core::dataflow::Block { archetype_id, coord, ..Default::default() };
                            let _ = dataflow.insert_block(block);
                        })
                    }
                }),
                Box::new(addon::DiscreteGenerator {
                    probability: 0.1,
                    sample_fn: {
                        let archetype_id = context.registry.get("block_mixgrass");
                        Box::new(move |dataflow, coord| {
                            let block = core::dataflow::Block { archetype_id, coord, ..Default::default() };
                            let _ = dataflow.insert_block(block);
                        })
                    }
                }),
                Box::new(addon::RandomGenerator {
                    probability: 0.01,
                    sample_fn: {
                        let archetype_id = context.registry.get("entity_bird");
                        Box::new(move |dataflow, coord| {
                            let entity = core::dataflow::Entity { archetype_id, coord, ..Default::default() };
                            let _ = dataflow.insert_entity(entity);
                        })
                    }
                }),
            ]
        };
        addon::GeneratorSystem::insert(&mut context.dataflow, desc).unwrap();

        // player system
        addon::PlayerSystem::insert(&mut context.dataflow).unwrap();

        self.context = Some(context);
    }

    #[func]
    fn close(&mut self) {
        self.context = None;
    }

    #[func]
    fn spawn_player(&mut self) {
        let context = self.context.as_mut().unwrap();

        let entity = core::dataflow::Entity { archetype_id: context.registry.get("entity_player"), ..Default::default() };
        let entity_id = context.dataflow.insert_entity(entity).unwrap();
        addon::PlayerSystem::attach_entity(&mut context.dataflow, entity_id).unwrap();
    }

    // update system

    #[func]
    fn forward_time(&mut self, delta_secs: f64) {
        let context = self.context.as_mut().unwrap();

        let delta_secs = delta_secs as f32;
        context.dataflow.forward_time(delta_secs);

        addon::PlayerSystem::process(&mut context.dataflow, delta_secs).unwrap();
    }

    #[func]
    fn generate_field(&mut self, rect: Rect2) {
        let context = self.context.as_mut().unwrap();

        let position = Vec2::new(rect.position.x, rect.position.y);
        let size = Vec2::new(rect.size.x, rect.size.y);
        let rect = core::Rect2::new(position, position + size);
        addon::GeneratorSystem::generate(&mut context.dataflow, rect).unwrap();
    }

    #[func]
    fn push_input(&mut self, input: Vector2) {
        let context = self.context.as_mut().unwrap();

        let input = Vec2::new(input.x, input.y);
        addon::PlayerSystem::push_input(&mut context.dataflow, input).unwrap();
    }

    // draw

    #[func]
    fn draw_field(&mut self, rect: Rect2) {
        let context = self.context.as_mut().unwrap();

        let position = Vec2::new(rect.position.x, rect.position.y);
        let size = Vec2::new(rect.size.x, rect.size.y);
        let rect = core::Rect2::new(position, position + size);
        context.tile_field_view.update_view(&context.dataflow, rect);
        context.block_field_view.update_view(&context.dataflow, rect);
        context.entity_field_view.update_view(&context.dataflow, rect);
    }
}
