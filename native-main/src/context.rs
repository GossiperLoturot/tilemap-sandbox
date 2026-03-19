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
    fn open(&mut self, viewport: Gd<godot::classes::Viewport>) {
        let mut builder = core::ContextBuilder::new();

        // dirt tile
        builder.add_tile("tile_dirt".into(), |_| core::TileInfo {
            display_name: "Dirt".into(),
            description: Default::default(),
            sprites: vec![core::SpriteInfo {
                images: vec![load("res://images/dirt.webp")],
                step_tick: 0,
                is_loop: false,
            }],
            collision: false,
        });

        // grass tile
        builder.add_tile("tile_grass".into(), |_| core::TileInfo {
            display_name: "Grass".into(),
            description: Default::default(),
            sprites: vec![core::SpriteInfo {
                images: vec![load("res://images/grass.webp")],
                step_tick: 0,
                is_loop: false,
            }],
            collision: false,
        });

        // dandelion block
        builder.add_block("block_dandelion".into(), |_| core::BlockInfo {
            display_name: "Dandelion".into(),
            description: Default::default(),
            sprites: vec![core::SpriteInfo {
                images: vec![load("res://images/dandelion.webp")],
                step_tick: 0,
                is_loop: false,
            }],
            y_sorting: false,
            size: IVec2::new(1, 1),
            collision_rect: None,
            rendering_rect: core::Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
        });

        // fallen leaves block
        builder.add_block("block_fallenleaves".into(), |_| core::BlockInfo {
            display_name: "Fallen Leaves".into(),
            description: Default::default(),
            sprites: vec![core::SpriteInfo {
                images: vec![load("res://images/fallenleaves.webp")],
                step_tick: 0,
                is_loop: false,
            }],
            y_sorting: false,
            size: IVec2::new(1, 1),
            collision_rect: None,
            rendering_rect: core::Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
        });

        // mix grass block
        builder.add_block("block_mixgrass".into(), |_| core::BlockInfo {
            display_name: "Grass".into(),
            description: Default::default(),
            sprites: vec![core::SpriteInfo {
                images: vec![load("res://images/mixgrass.webp")],
                step_tick: 0,
                is_loop: false,
            }],
            y_sorting: true,
            size: IVec2::new(1, 1),
            collision_rect: None,
            rendering_rect: core::Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
        });

        // mix pebbles block
        builder.add_block("block_mixpebbles".into(), |_| core::BlockInfo {
            display_name: "Pebbles".into(),
            description: Default::default(),
            sprites: vec![core::SpriteInfo {
                images: vec![load("res://images/mixpebbles.webp")],
                step_tick: 0,
                is_loop: false,
            }],
            y_sorting: false,
            size: IVec2::new(1, 1),
            collision_rect: None,
            rendering_rect: core::Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
        });

        // oak tree block
        builder.add_block("block_oaktree".into(), |_| core::BlockInfo {
            display_name: "Oak Tree".into(),
            description: Default::default(),
            sprites: vec![core::SpriteInfo {
                images: vec![
                    load("res://images/oaktree_0.webp"),
                    load("res://images/oaktree_1.webp"),
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
        builder.add_entity("entity_player".into(), |_| core::EntityInfo {
            display_name: "Player".into(),
            description: Default::default(),
            sprites: vec![
                core::SpriteInfo {
                    images: vec![
                        load("res://images/player_idle0.webp"),
                        load("res://images/player_idle1.webp"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                core::SpriteInfo {
                    images: vec![
                        load("res://images/player_idle0r.webp"),
                        load("res://images/player_idle1r.webp"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                core::SpriteInfo {
                    images: vec![
                        load("res://images/player_idle0.webp"),
                        load("res://images/player_walk0.webp"),
                        load("res://images/player_idle1.webp"),
                        load("res://images/player_walk1.webp"),
                    ],
                    step_tick: 6,
                    is_loop: true,
                },
                core::SpriteInfo {
                    images: vec![
                        load("res://images/player_idle0r.webp"),
                        load("res://images/player_walk0r.webp"),
                        load("res://images/player_idle1r.webp"),
                        load("res://images/player_walk1r.webp"),
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
        builder.add_entity("entity_pig".into(), |_| core::EntityInfo {
            display_name: "Pig".into(),
            description: Default::default(),
            sprites: vec![
                core::SpriteInfo {
                    images: vec![
                        load("res://images/pig_idle0.webp"),
                        load("res://images/pig_idle1.webp"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                core::SpriteInfo {
                    images: vec![
                        load("res://images/pig_walk0.webp"),
                        load("res://images/pig_idle0.webp"),
                        load("res://images/pig_walk1.webp"),
                        load("res://images/pig_idle1.webp"),
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
        builder.add_entity("entity_cow".into(), |_| core::EntityInfo {
            display_name: "Cow".into(),
            description: Default::default(),
            sprites: vec![
                core::SpriteInfo {
                    images: vec![
                        load("res://images/cow_idle0.webp"),
                        load("res://images/cow_idle1.webp"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                core::SpriteInfo {
                    images: vec![
                        load("res://images/cow_walk0.webp"),
                        load("res://images/cow_idle0.webp"),
                        load("res://images/cow_walk1.webp"),
                        load("res://images/cow_idle1.webp"),
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
        builder.add_entity("entity_sheep".into(), |_| core::EntityInfo {
            display_name: "Sheep".into(),
            description: Default::default(),
            sprites: vec![
                core::SpriteInfo {
                    images: vec![
                        load("res://images/sheep_idle0.webp"),
                        load("res://images/sheep_idle1.webp"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                core::SpriteInfo {
                    images: vec![
                        load("res://images/sheep_walk0.webp"),
                        load("res://images/sheep_idle0.webp"),
                        load("res://images/sheep_walk1.webp"),
                        load("res://images/sheep_idle1.webp"),
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
        builder.add_entity("entity_chicken".into(), |_| core::EntityInfo {
            display_name: "Chicken".into(),
            description: Default::default(),
            sprites: vec![
                core::SpriteInfo {
                    images: vec![
                        load("res://images/chicken_idle.webp"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                core::SpriteInfo {
                    images: vec![
                        load("res://images/chicken_walk.webp"),
                        load("res://images/chicken_idle.webp"),
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
        builder.add_entity("entity_bird".into(), |_| core::EntityInfo {
            display_name: "Bird".into(),
            description: Default::default(),
            sprites: vec![
                core::SpriteInfo {
                    images: vec![
                        load("res://images/bird_idle.webp"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                core::SpriteInfo {
                    images: vec![
                        load("res://images/bird_walk.webp"),
                        load("res://images/bird_idle.webp"),
                    ],
                    step_tick: 12,
                    is_loop: true,
                },
            ],
            y_sorting: true,
            collision_rect: Some(core::Rect2::new(Vec2::new(-0.4, 0.1), Vec2::new(0.4, 0.9))),
            rendering_rect: core::Rect2::new(Vec2::new(-0.5, 0.0), Vec2::new(0.5, 1.0)),
        });

        // build
        let desc = core::BuildInfo {
            tile_shaders: vec![
                load("res://shaders/field.gdshader"),
            ],
            block_shaders: vec![
                load("res://shaders/field.gdshader"),
                load("res://shaders/field_shadow.gdshader"),
            ],
            entity_shaders: vec![
                load("res://shaders/field.gdshader"),
                load("res://shaders/field_shadow.gdshader"),
            ],
            viewport,
        };
        let mut context = builder.build(desc);

        // field generator
        let desc = addon::GeneratorResourceDescriptor {
            generators: vec![
                Box::new(addon::DiscreteGenerator {
                    probability: 0.75,
                    sample_fn: {
                        let archetype_id = context.registry.get("tile_grass");
                        move |dataflow, coord| {
                            let tile = core::dataflow::Tile { archetype_id, coord, ..Default::default() };
                            let _ = dataflow.insert_tile(tile);
                        }
                    }
                }),
                Box::new(addon::DiscreteGenerator {
                    probability: 1.0,
                    sample_fn: {
                        let archetype_id = context.registry.get("tile_dirt");
                        move |dataflow, coord| {
                            let tile = core::dataflow::Tile { archetype_id, coord, ..Default::default() };
                            let _ = dataflow.insert_tile(tile);
                        }
                    }
                }),
                Box::new(addon::DiscreteGenerator {
                    probability: 0.01,
                    sample_fn: {
                        let archetype_id = context.registry.get("block_oaktree");
                        move |dataflow, coord| {
                            let block = core::dataflow::Block { archetype_id, coord, ..Default::default() };
                            let _ = dataflow.insert_block(block);
                        }
                    }
                }),
                Box::new(addon::DiscreteGenerator {
                    probability: 0.1,
                    sample_fn: {
                        let archetype_id = context.registry.get("block_dandelion");
                        move |dataflow, coord| {
                            let block = core::dataflow::Block { archetype_id, coord, ..Default::default() };
                            let _ = dataflow.insert_block(block);
                        }
                    }
                }),
                Box::new(addon::DiscreteGenerator {
                    probability: 0.1,
                    sample_fn: {
                        let archetype_id = context.registry.get("block_mixgrass");
                        move |dataflow, coord| {
                            let block = core::dataflow::Block { archetype_id, coord, ..Default::default() };
                            let _ = dataflow.insert_block(block);
                        }
                    }
                }),
                Box::new(addon::RandomGenerator {
                    probability: 0.01,
                    sample_fn: {
                        let archetype_id = context.registry.get("entity_bird");
                        move |dataflow, coord| {
                            let entity = core::dataflow::Entity { archetype_id, coord, ..Default::default() };
                            let _ = dataflow.insert_entity(entity);
                        }
                    }
                }),
            ]
        };
        addon::GeneratorSystem::insert(&mut context.dataflow, desc).unwrap();

        // player system
        addon::PlayerSystem::insert(&mut context.dataflow).unwrap();
        // animal system
        addon::AnimalSystem::insert(&mut context.dataflow).unwrap();

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

    #[func]
    fn spawn_animal(&mut self, position: Vector2) {
        let context = self.context.as_mut().unwrap();

        let coord = Vec2::new(position.x, position.y);
        let entity = core::dataflow::Entity { archetype_id: context.registry.get("entity_bird"), coord, ..Default::default() };
        let entity_id = context.dataflow.insert_entity(entity).unwrap();
        addon::AnimalSystem::attach_entity(&mut context.dataflow, entity_id).unwrap();
    }

    // update system

    #[func]
    fn forward_time(&mut self, delta_secs: f64) {
        let context = self.context.as_mut().unwrap();

        let delta_secs = delta_secs as f32;
        context.dataflow.forward_time(delta_secs);

        // player system
        addon::PlayerSystem::process(&mut context.dataflow, delta_secs).unwrap();
        // animal sysyem
        addon::AnimalSystem::process(&mut context.dataflow, delta_secs).unwrap();
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
    fn queue_input(&mut self, input: Vector2) {
        let context = self.context.as_mut().unwrap();

        let input = Vec2::new(input.x, input.y);
        addon::PlayerSystem::queue_input(&mut context.dataflow, input).unwrap();
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
