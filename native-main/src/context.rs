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
            description: "".into(),
            sprites: vec![core::SpriteInfo {
                images: vec![retriever.load("image_tile_dirt")],
                step_tick: 0,
                is_loop: false,
            }],
            collision: false,
            feature_set: Box::new(()),
        });

        // grass tile
        builder.add_tile("tile_grass".into(), |registry, retriever| {
            core::TileInfo {
                display_name: "Grass".into(),
                description: "".into(),
                sprites: vec![core::SpriteInfo {
                    images: vec![retriever.load("image_tile_grass")],
                    step_tick: 0,
                    is_loop: false,
                }],
                collision: false,
                feature_set: Box::new(addon::BreakFeatureSet {
                    item_entity_id: registry.get("entity_package"),
                    item_id: registry.get("item_grass"),
                }),
            }
        });

        // dandelion block
        builder.add_block("block_dandelion".into(), |_, retriever| {
            core::BlockInfo {
                display_name: "Dandelion".into(),
                description: "".into(),
                sprites: vec![core::SpriteInfo {
                    images: vec![retriever.load("image_block_dandelion")],
                    step_tick: 0,
                    is_loop: false,
                }],
                y_sorting: false,
                size: IVec2::new(1, 1),
                collision_rect: core::geom::Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)),
                rendering_rect: core::geom::Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                feature_set: Box::new(()),
            }
        });

        // fallen leaves block
        builder.add_block("block_fallenleaves".into(), |registry, retriever| {
            core::BlockInfo {
                display_name: "Fallen Leaves".into(),
                description: "".into(),
                sprites: vec![core::SpriteInfo {
                    images: vec![retriever.load("image_block_fallenleaves")],
                    step_tick: 0,
                    is_loop: false,
                }],
                y_sorting: false,
                size: IVec2::new(1, 1),
                collision_rect: core::geom::Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)),
                rendering_rect: core::geom::Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                feature_set: Box::new(addon::BreakFeatureSet {
                    item_entity_id: registry.get("entity_package"),
                    item_id: registry.get("item_fallenleaves"),
                }),
            }
        });

        // mix grass block
        builder.add_block("block_mixgrass".into(), |_, retriever| {
            core::BlockInfo {
                display_name: "Grass".into(),
                description: "".into(),
                sprites: vec![core::SpriteInfo {
                    images: vec![retriever.load("image_block_mixgrass")],
                    step_tick: 0,
                    is_loop: false,
                }],
                y_sorting: false,
                size: IVec2::new(1, 1),
                collision_rect: core::geom::Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)),
                rendering_rect: core::geom::Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                feature_set: Box::new(()),
            }
        });

        // mix pebbles block
        builder.add_block("block_mixpebbles".into(), |registry, retriever| {
            core::BlockInfo {
                display_name: "Pebbles".into(),
                description: "".into(),
                sprites: vec![core::SpriteInfo {
                    images: vec![retriever.load("image_block_mixpebbles")],
                    step_tick: 0,
                    is_loop: false,
                }],
                y_sorting: false,
                size: IVec2::new(1, 1),
                collision_rect: core::geom::Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)),
                rendering_rect: core::geom::Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                feature_set: Box::new(addon::BreakFeatureSet {
                    item_entity_id: registry.get("entity_package"),
                    item_id: registry.get("item_mixpebbles"),
                }),
            }
        });

        // oak tree block
        builder.add_block("block_oaktree".into(), |registry, retriever| {
            core::BlockInfo {
                display_name: "Oak Tree".into(),
                description: "".into(),
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
                collision_rect: core::geom::Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)),
                rendering_rect: core::geom::Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                feature_set: Box::new(addon::BreakFeatureSet {
                    item_entity_id: registry.get("entity_package"),
                    item_id: registry.get("item_wood"),
                }),
            }
        });

        // player entity
        builder.add_entity("entity_player".into(), |registry, retriever| {
            core::EntityInfo {
                display_name: "Player".into(),
                description: "".into(),
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
                collision_rect: core::geom::Rect2::new(Vec2::new(-0.4, 0.1), Vec2::new(0.8, 0.8)),
                rendering_rect: core::geom::Rect2::new(Vec2::new(-0.75, 0.0), Vec2::new(1.5, 2.25)),
                feature_set: Box::new(addon::PlayerEntityFeatureSet {
                    move_speed: 3.0,
                    inventory_id: registry.get("inventory_player"),
                }),
            }
        });

        // pig entity
        builder.add_entity("entity_pig".into(), |_, retriever| core::EntityInfo {
            display_name: "Pig".into(),
            description: "".into(),
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
            collision_rect: core::geom::Rect2::new(Vec2::new(-0.4, 0.1), Vec2::new(0.8, 0.8)),
            rendering_rect: core::geom::Rect2::new(Vec2::new(-1.0, 0.0), Vec2::new(2.0, 2.0)),
            feature_set: Box::new(addon::AnimalEntityFeatureSet {
                min_rest_secs: 0.0,
                max_rest_secs: 10.0,
                min_distance: 0.0,
                max_distance: 10.0,
                speed: 1.0,
                idle_variant: 0,
                walk_variant: 1,
            }),
        });

        // cow entity
        builder.add_entity("entity_cow".into(), |_, retriever| core::EntityInfo {
            display_name: "Cow".into(),
            description: "".into(),
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
            collision_rect: core::geom::Rect2::new(Vec2::new(-0.4, 0.1), Vec2::new(0.8, 0.8)),
            rendering_rect: core::geom::Rect2::new(Vec2::new(-1.0, 0.0), Vec2::new(2.0, 2.0)),
            feature_set: Box::new(addon::AnimalEntityFeatureSet {
                min_rest_secs: 0.0,
                max_rest_secs: 10.0,
                min_distance: 0.0,
                max_distance: 10.0,
                speed: 1.0,
                idle_variant: 0,
                walk_variant: 1,
            }),
        });

        // sheep entity
        builder.add_entity("entity_sheep".into(), |_, retriever| {
            core::EntityInfo {
                display_name: "Sheep".into(),
                description: "".into(),
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
                collision_rect: core::geom::Rect2::new(Vec2::new(-0.4, 0.1), Vec2::new(0.8, 0.8)),
                rendering_rect: core::geom::Rect2::new(Vec2::new(-1.0, 0.0), Vec2::new(2.0, 2.0)),
                feature_set: Box::new(addon::AnimalEntityFeatureSet {
                    min_rest_secs: 0.0,
                    max_rest_secs: 10.0,
                    min_distance: 0.0,
                    max_distance: 10.0,
                    speed: 1.0,
                    idle_variant: 0,
                    walk_variant: 1,
                }),
            }
        });

        // chicken entity
        builder.add_entity("entity_chicken".into(), |_, retriever| {
            core::EntityInfo {
                display_name: "Chicken".into(),
                description: "".into(),
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
                collision_rect: core::geom::Rect2::new(Vec2::new(-0.4, 0.1), Vec2::new(0.8, 0.8)),
                rendering_rect: core::geom::Rect2::new(Vec2::new(-0.5, 0.0), Vec2::new(1.0, 1.0)),
                feature_set: Box::new(addon::AnimalEntityFeatureSet {
                    min_rest_secs: 0.0,
                    max_rest_secs: 10.0,
                    min_distance: 0.0,
                    max_distance: 10.0,
                    speed: 1.0,
                    idle_variant: 0,
                    walk_variant: 1,
                }),
            }
        });

        // bird entity
        builder.add_entity("entity_bird".into(), |_, retriever| {
            core::EntityInfo {
                display_name: "Bird".into(),
                description: "".into(),
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
                collision_rect: core::geom::Rect2::new(Vec2::new(-0.4, 0.1), Vec2::new(0.8, 0.8)),
                rendering_rect: core::geom::Rect2::new(Vec2::new(-0.5, 0.0), Vec2::new(1.0, 1.0)),
                feature_set: Box::new(addon::AnimalEntityFeatureSet {
                    min_rest_secs: 0.0,
                    max_rest_secs: 10.0,
                    min_distance: 0.0,
                    max_distance: 10.0,
                    speed: 1.0,
                    idle_variant: 0,
                    walk_variant: 1,
                }),
            }
        });

        // package entity
        builder.add_entity("entity_package".into(), |_, retriever| {
            core::EntityInfo {
                display_name: "Package".into(),
                description: "".into(),
                sprites: vec![core::SpriteInfo {
                    images: vec![retriever.load("image_entity_package")],
                    step_tick: 0,
                    is_loop: false,
                }],
                y_sorting: true,
                collision_rect: core::geom::Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)),
                rendering_rect: core::geom::Rect2::new(Vec2::new(-0.25, -0.25), Vec2::new(0.5, 0.5)),
                feature_set: Box::new(addon::ItemEntityFeatureSet {}),
            }
        });

        // particle entity
        builder.add_entity("entity_particle".into(), |_, retriever| {
            core::EntityInfo {
                display_name: "Particle".into(),
                description: "".into(),
                sprites: vec![core::SpriteInfo {
                    images: vec![
                        retriever.load("image_entity_particle0"),
                        retriever.load("image_entity_particle1"),
                    ],
                    step_tick: 4,
                    is_loop: false,
                }],
                y_sorting: true,
                collision_rect: core::geom::Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)),
                rendering_rect: core::geom::Rect2::new(Vec2::new(-1.0, -1.0), Vec2::new(2.0, 2.0)),
                feature_set: Box::new(addon::ParticleEntityFeatureSet {}),
            }
        });

        // grass item
        builder.add_item("item_grass".into(), |_, retriever| core::ItemInfo {
            display_name: "Grass".into(),
            description: "".into(),
            sprites: vec![core::SpriteInfo {
                images: vec![retriever.load("image_item_grass")],
                step_tick: 0,
                is_loop: false,
            }],
            feature_set: Box::new(()),
        });

        // fallen leaves item
        builder.add_item("item_fallenleaves".into(), |_, retriever| {
            core::ItemInfo {
                display_name: "Fallen Leaves".into(),
                description: "".into(),
                sprites: vec![core::SpriteInfo {
                    images: vec![retriever.load("image_item_fallenleaves")],
                    step_tick: 0,
                    is_loop: false,
                }],
                feature_set: Box::new(()),
            }
        });

        // mix pebbles item
        builder.add_item("item_mixpebbles".into(), |_, retriever| {
            core::ItemInfo {
                display_name: "Pebbles".into(),
                description: "".into(),
                sprites: vec![core::SpriteInfo {
                    images: vec![retriever.load("image_item_mixpebbles")],
                    step_tick: 0,
                    is_loop: false,
                }],
                feature_set: Box::new(()),
            }
        });

        // wood item
        builder.add_item("item_wood".into(), |_, retriever| core::ItemInfo {
            display_name: "Wood".into(),
            description: "".into(),
            sprites: vec![core::SpriteInfo {
                images: vec![retriever.load("image_item_wood")],
                step_tick: 0,
                is_loop: false,
            }],
            feature_set: Box::new(()),
        });

        // player inventory
        builder.add_inventory("inventory_player".into(), |_, retriever| {
            core::InventoryInfo {
                size: 32,
                callback: retriever.load("callable_inventory_player"),
            }
        });

        // global inventory
        builder.add_inventory("inventory_global".into(), |_, retriever| {
            core::InventoryInfo {
                size: 0,
                callback: retriever.load("callable_inventory_global"),
            }
        });

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
            selection_shader: retriever.load("shader_selection"),
            viewport: retriever.load("viewport"),
        };
        let mut context = builder.build(&retriever, desc);

        // generator system
        let desc = addon::GeneratorResourceDescriptor {
            generators: vec![
                {
                    let archetype_id = context.registry.get("tile_grass");
                    Box::new(addon::MarchGenerator {
                        prob: 0.5,
                        place_fn: Box::new(move |dataflow, coord| {
                            let tile = core::dataflow::Tile {
                                archetype_id,
                                coord,
                                ..Default::default()
                            };
                            let _ = dataflow.insert_tile(tile);
                        }),
                    })
                },
                {
                    let archetype_id = context.registry.get("tile_dirt");
                    Box::new(addon::MarchGenerator {
                        prob: 1.0,
                        place_fn: Box::new(move |dataflow, coord| {
                            let tile = core::dataflow::Tile {
                                archetype_id,
                                coord,
                                ..Default::default()
                            };
                            let _ = dataflow.insert_tile(tile);
                        }),
                    })
                },
                {
                    let archetype_id = context.registry.get("block_oaktree");
                    Box::new(addon::SpawnGenerator {
                        prob: 0.01,
                        place_fn: Box::new(move |dataflow, coord| {
                            let block = core::dataflow::Block {
                                archetype_id,
                                coord: coord.as_ivec2(),
                                ..Default::default()
                            };
                            let _ = dataflow.insert_block(block);
                        }),
                    })
                },
                {
                    let archetype_id = context.registry.get("block_dandelion");
                    Box::new(addon::SpawnGenerator {
                        prob: 0.05,
                        place_fn: Box::new(move |dataflow, coord| {
                            let block = core::dataflow::Block {
                                archetype_id,
                                coord: coord.as_ivec2(),
                                ..Default::default()
                            };
                            let _ = dataflow.insert_block(block);
                        }),
                    })
                },
                {
                    let archetype_id = context.registry.get("block_fallenleaves");
                    Box::new(addon::SpawnGenerator {
                        prob: 0.05,
                        place_fn: Box::new(move |dataflow, coord| {
                            let block = core::dataflow::Block {
                                archetype_id,
                                coord: coord.as_ivec2(),
                                ..Default::default()
                            };
                            let _ = dataflow.insert_block(block);
                        }),
                    })
                },
                {
                    let archetype_id = context.registry.get("block_mixpebbles");
                    Box::new(addon::SpawnGenerator {
                        prob: 0.05,
                        place_fn: Box::new(move |dataflow, coord| {
                            let block = core::dataflow::Block {
                                archetype_id,
                                coord: coord.as_ivec2(),
                                ..Default::default()
                            };
                            let _ = dataflow.insert_block(block);
                        }),
                    })
                },
                {
                    let archetype_id = context.registry.get("entity_bird");
                    Box::new(addon::SpawnGenerator {
                        prob: 0.01,
                        place_fn: Box::new(move |dataflow, coord| {
                            let entity = core::dataflow::Entity {
                                archetype_id,
                                coord,
                                ..Default::default()
                            };
                            let _ = dataflow.insert_entity(entity);
                        }),
                    })
                },
            ],
        };
        let resource = addon::GeneratorResource::new(desc);
        context.dataflow.insert_resources(resource).unwrap();

        // player system
        let resource = addon::PlayerResource::new();
        context.dataflow.insert_resources(resource).unwrap();

        // global inventory
        let resource = addon::GlobalInventoryResource::new();
        context.dataflow.insert_resources(resource).unwrap();
        addon::GlobalInventorySystem::insert_inventory(
            &mut context.dataflow,
            context.registry.get("inventory_global"),
        )
        .unwrap();

        // breakable system
        let resource = addon::BreakableResource {
            default_tile_id: context.registry.get("tile_dirt"),
            id: context.registry.get("entity_particle"),
        };
        context.dataflow.insert_resources(resource).unwrap();

        self.context = Some(context);
    }

    #[func]
    fn close(&mut self) {
        self.context = None;
    }

    // update system

    #[func]
    fn forward_time(&mut self, delta_secs: f64) {
        let context = self.context.as_mut().unwrap();

        let delta_secs = delta_secs as f32;
        context.dataflow.forward_time(delta_secs);
    }

    #[func]
    fn forwarde_rect(&mut self, min_rect: Rect2, delta_secs: f64) {
        let context = self.context.as_mut().unwrap();

        let position = Vec2::new(min_rect.position.x, min_rect.position.y);
        let size = Vec2::new(min_rect.size.x, min_rect.size.y);
        let min_rect = [position, position + size];
        let delta_secs = delta_secs as f32;
        addon::ForwarderSystem::forward(&mut context.dataflow, min_rect, delta_secs).unwrap();
    }

    #[func]
    fn generate_rect(&mut self, min_rect: Rect2) {
        let context = self.context.as_mut().unwrap();

        let position = Vec2::new(min_rect.position.x, min_rect.position.y);
        let size = Vec2::new(min_rect.size.x, min_rect.size.y);
        let min_rect = [position, position + size];
        addon::GeneratorSystem::generate(&mut context.dataflow, min_rect).unwrap();
    }

    // player

    #[func]
    fn spawn_player(&mut self, coord: Vector2) {
        let context = self.context.as_mut().unwrap();

        let coord = Vec2::new(coord.x, coord.y);
        let entity = core::dataflow::Entity {
            archetype_id: context.registry.get("entity_player"),
            coord,
            ..Default::default()
        };
        context.dataflow.insert_entity(entity).unwrap();
    }

    #[func]
    fn push_player_input(&mut self, input: Vector2) {
        let context = self.context.as_mut().unwrap();

        let input = Vec2::new(input.x, input.y);
        addon::PlayerSystem::push_input(&mut context.dataflow, input).unwrap();
    }

    #[func]
    fn get_player_location(&self) -> Vector2 {
        let context = self.context.as_ref().unwrap();

        let location = addon::PlayerSystem::get_location(&context.dataflow).unwrap();
        Vector2::new(location.x, location.y)
    }

    #[func]
    fn open_player_inventory(&mut self) {
        let context = self.context.as_mut().unwrap();

        if let Ok(inventory_id) = addon::PlayerSystem::get_inventory_key(&context.dataflow) {
            let _ = context.inventory_system_view.open_inventory(
                &context.dataflow,
                inventory_id,
                |callable, inventory| {
                    let mut item_ids = Array::<Gd<SlotKey>>::new();
                    for (local_id, _) in inventory.items.iter().enumerate() {
                        let item_id = (inventory_id, local_id as u32);
                        item_ids.push(&Gd::from_object(SlotKey { inner: item_id }));
                    }
                    callable.call(&[item_ids.to_variant()]);
                },
            );
        }
    }

    // global inventory

    #[func]
    fn open_global_inventory(&mut self) {
        let context = self.context.as_mut().unwrap();

        if let Ok(inventory_id) = addon::GlobalInventorySystem::get_inventory_key(&context.dataflow) {
            let _ = context.inventory_system_view.open_inventory(
                &context.dataflow,
                inventory_id,
                |callable, inventory| {
                    let mut item_ids = Array::<Gd<SlotKey>>::new();
                    for (local_id, _) in inventory.items.iter().enumerate() {
                        let item_id = (inventory_id, local_id as u32);
                        item_ids.push(&Gd::from_object(SlotKey { inner: item_id }));
                    }
                    callable.call(&[item_ids.to_variant()]);
                },
            );
        }
    }

    // inventory

    #[func]
    fn get_slot(&self, slot_key: Gd<SlotKey>) -> Option<Gd<SlotResult>> {
        let context = self.context.as_ref().unwrap();

        let slot_key = **slot_key.bind();
        let item = context.dataflow.get_item(slot_key).ok()?;
        let archetype = context.dataflow.get_item_archetype(item.archetype_id).ok()?;

        Some(Gd::from_object(SlotResult {
            amount: item.amount as i64,
            display_name: GString::from(&archetype.display_name),
            description: GString::from(&archetype.description),
        }))
    }

    #[func]
    fn swap_slot(&mut self, src_slot_key: Gd<SlotKey>, dst_slot_key: Gd<SlotKey>) {
        let context = self.context.as_mut().unwrap();

        let src_slot_key = **src_slot_key.bind();
        let dst_slot_key = **dst_slot_key.bind();
        context
            .dataflow
            .swap_item(src_slot_key, dst_slot_key)
            .unwrap();
    }

    #[func]
    fn draw_slot(&mut self, slot_key: Gd<SlotKey>, control_item: Gd<godot::classes::Control>) {
        let context = self.context.as_ref().unwrap();

        let slot_key = **slot_key.bind();
        context
            .inventory_system_view
            .draw_item(&context.dataflow, slot_key, control_item)
            .unwrap();
    }

    // field

    #[func]
    fn find_tile(&self, location: Vector2) -> Option<Gd<TileKey>> {
        let context = self.context.as_ref().unwrap();

        let point = Vec2::new(location.x, location.y).floor().as_ivec2();
        context
            .dataflow
            .find_tile_with_point(point)
            .map(|key| Gd::from_object(TileKey { inner: key }))
    }

    #[func]
    fn get_tile(&self, tile_key: Gd<TileKey>) -> Gd<TileResult> {
        let context = self.context.as_ref().unwrap();

        let k = **tile_key.bind();
        let tile = context.dataflow.get_tile(k).unwrap();
        let archetype = context.dataflow.get_tile_archetype(tile.archetype_id).unwrap();
        Gd::from_object(TileResult {
            display_name: GString::from(&archetype.display_name),
            description: GString::from(&archetype.description),
        })
    }

    #[func]
    fn find_block(&self, location: Vector2, offset: i64) -> Option<Gd<BlockKey>> {
        let context = self.context.as_ref().unwrap();

        let point = Vec2::new(location.x, location.y);
        let keys = context
            .dataflow
            .find_block_with_hint_point(point)
            .collect::<Vec<_>>();
        if !keys.is_empty() {
            let index = (offset as usize).div_euclid(keys.len());
            Some(Gd::from_object(BlockKey { inner: keys[index] }))
        } else {
            None
        }
    }

    #[func]
    fn get_block(&self, block_key: Gd<BlockKey>) -> Gd<BlockResult> {
        let context = self.context.as_ref().unwrap();

        let k = **block_key.bind();
        let block = context.dataflow.get_block(k).unwrap();
        let archetype = context.dataflow.get_tile_archetype(block.archetype_id).unwrap();
        Gd::from_object(BlockResult {
            display_name: GString::from(&archetype.display_name),
            description: GString::from(&archetype.description),
        })
    }

    #[func]
    fn find_entity(&self, location: Vector2, offset: i64) -> Option<Gd<EntityKey>> {
        let context = self.context.as_ref().unwrap();

        let point = Vec2::new(location.x, location.y);
        let keys = context
            .dataflow
            .find_entity_with_hint_point(point)
            .collect::<Vec<_>>();
        if !keys.is_empty() {
            let index = (offset as usize).div_euclid(keys.len());
            Some(Gd::from_object(EntityKey { inner: keys[index] }))
        } else {
            None
        }
    }

    #[func]
    fn get_entity(&self, entity_key: Gd<EntityKey>) -> Gd<EntityResult> {
        let context = self.context.as_ref().unwrap();

        let k = **entity_key.bind();
        let entity = context.dataflow.get_entity(k).unwrap();
        let archetype = context.dataflow.get_entity_archetype(entity.archetype_id).unwrap();
        Gd::from_object(EntityResult {
            display_name: GString::from(&archetype.display_name),
            description: GString::from(&archetype.description),
        })
    }

    #[func]
    fn draw_selection_none(&mut self) {
        let context = self.context.as_mut().unwrap();

        context
            .selection_view
            .update_view(&context.dataflow, &[], &[], &[]);
    }

    #[func]
    fn draw_selection_tile(&mut self, tile_key: Gd<TileKey>) {
        let context = self.context.as_mut().unwrap();

        let k = **tile_key.bind();
        context
            .selection_view
            .update_view(&context.dataflow, &[k], &[], &[]);
    }

    #[func]
    fn draw_selection_block(&mut self, block_key: Gd<BlockKey>) {
        let context = self.context.as_mut().unwrap();

        let k = **block_key.bind();
        context
            .selection_view
            .update_view(&context.dataflow, &[], &[k], &[]);
    }

    #[func]
    fn draw_selection_entity(&mut self, entity_key: Gd<EntityKey>) {
        let context = self.context.as_mut().unwrap();

        let k = **entity_key.bind();
        context
            .selection_view
            .update_view(&context.dataflow, &[], &[], &[k]);
    }

    #[func]
    fn break_tile(&mut self, tile_key: Gd<TileKey>) {
        let context = self.context.as_mut().unwrap();

        let k = **tile_key.bind();
        // addon::BreakableSystem::break_tile(&mut context.dataflow, k).unwrap();
    }

    #[func]
    fn break_block(&mut self, block_key: Gd<BlockKey>) {
        let context = self.context.as_mut().unwrap();

        let k = **block_key.bind();
        // addon::BreakableSystem::break_block(&mut context.dataflow, k).unwrap();
    }

    #[func]
    fn break_entity(&mut self, entity_key: Gd<EntityKey>) {
        let context = self.context.as_mut().unwrap();

        let k = **entity_key.bind();
        // addon::BreakableSystem::break_entity(&mut context.dataflow, k).unwrap();
    }

    #[func]
    fn draw_field(&mut self, min_rect: Rect2) {
        let context = self.context.as_mut().unwrap();

        let position = Vec2::new(min_rect.position.x, min_rect.position.y);
        let size = Vec2::new(min_rect.size.x, min_rect.size.y);
        let min_rect = [position, position + size];
        context
            .tile_field_view
            .update_view(&context.dataflow, min_rect);
        context
            .block_field_view
            .update_view(&context.dataflow, min_rect);
        context
            .entity_field_view
            .update_view(&context.dataflow, min_rect);
    }
}

// interface types

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct SlotKey {
    inner: core::dataflow::ItemId,
}

impl std::ops::Deref for SlotKey {
    type Target = core::dataflow::ItemId;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct SlotResult {
    #[var]
    amount: i64,
    #[var]
    display_name: GString,
    #[var]
    description: GString,
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct TileKey {
    inner: core::dataflow::TileId,
}

impl std::ops::Deref for TileKey {
    type Target = core::dataflow::TileId;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct TileResult {
    #[var]
    display_name: GString,
    #[var]
    description: GString,
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct BlockKey {
    inner: core::dataflow::BlockId,
}

impl std::ops::Deref for BlockKey {
    type Target = core::dataflow::BlockId;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct BlockResult {
    #[var]
    display_name: GString,
    #[var]
    description: GString,
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct EntityKey {
    inner: core::dataflow::EntityId,
}

impl std::ops::Deref for EntityKey {
    type Target = core::dataflow::EntityId;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct EntityResult {
    #[var]
    display_name: GString,
    #[var]
    description: GString,
}
