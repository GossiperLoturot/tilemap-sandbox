use glam::*;
use godot::prelude::*;
use native_core as core;

use crate::addon;

type Wrap<T> = std::cell::RefCell<Option<T>>;
thread_local! { static CONTEXT: Wrap<core::Context> = Default::default(); }

#[derive(GodotClass)]
#[class(no_init)]
struct Context;

#[godot_api]
impl Context {
    #[func]
    fn open(retrieve_callable: Callable) {
        let mut builder = core::ContextBuilder::new();

        // dirt tile
        builder.add_tile("tile_dirt".into(), |_, retriever| core::TileDescriptor {
            display_name: "Dirt".into(),
            description: "".into(),
            images: vec![core::ImageDescriptor {
                frames: vec![retriever.load("image_tile_dirt")],
                step_tick: 0,
                is_loop: false,
            }],
            collision: false,
            feature_set: Box::new(()),
        });

        // grass tile
        builder.add_tile("tile_grass".into(), |_, retriever| core::TileDescriptor {
            display_name: "Grass".into(),
            description: "".into(),
            images: vec![core::ImageDescriptor {
                frames: vec![retriever.load("image_tile_grass")],
                step_tick: 0,
                is_loop: false,
            }],
            collision: false,
            feature_set: Box::new(()),
        });

        // dandelion block
        builder.add_block("block_dandelion".into(), |_, retriever| {
            core::BlockDescriptor {
                display_name: "Dandelion".into(),
                description: "".into(),
                images: vec![core::ImageDescriptor {
                    frames: vec![retriever.load("image_block_dandelion")],
                    step_tick: 0,
                    is_loop: false,
                }],
                z_along_y: false,
                size: IVec2::new(1, 1),
                collision_size: Vec2::new(0.0, 0.0),
                collision_offset: Vec2::new(0.0, 0.0),
                rendering_size: Vec2::new(1.0, 1.0),
                rendering_offset: Vec2::new(0.0, 0.0),
                feature_set: Box::new(()),
            }
        });

        // fallen leaves block
        builder.add_block("block_fallenleaves".into(), |_, retriever| {
            core::BlockDescriptor {
                display_name: "Fallen Leaves".into(),
                description: "".into(),
                images: vec![core::ImageDescriptor {
                    frames: vec![retriever.load("image_block_fallenleaves")],
                    step_tick: 0,
                    is_loop: false,
                }],
                z_along_y: false,
                size: IVec2::new(1, 1),
                collision_size: Vec2::new(0.0, 0.0),
                collision_offset: Vec2::new(0.0, 0.0),
                rendering_size: Vec2::new(1.0, 1.0),
                rendering_offset: Vec2::new(0.0, 0.0),
                feature_set: Box::new(()),
            }
        });

        // mix grass block
        builder.add_block("block_mixgrass".into(), |_, retriever| {
            core::BlockDescriptor {
                display_name: "Grass".into(),
                description: "".into(),
                images: vec![core::ImageDescriptor {
                    frames: vec![retriever.load("image_block_mixgrass")],
                    step_tick: 0,
                    is_loop: false,
                }],
                z_along_y: false,
                size: IVec2::new(1, 1),
                collision_size: Vec2::new(0.0, 0.0),
                collision_offset: Vec2::new(0.0, 0.0),
                rendering_size: Vec2::new(1.0, 1.0),
                rendering_offset: Vec2::new(0.0, 0.0),
                feature_set: Box::new(()),
            }
        });

        // mix pebbles block
        builder.add_block("block_mixpebbles".into(), |_, retriever| {
            core::BlockDescriptor {
                display_name: "Pebbles".into(),
                description: "".into(),
                images: vec![core::ImageDescriptor {
                    frames: vec![retriever.load("image_block_mixpebbles")],
                    step_tick: 0,
                    is_loop: false,
                }],
                z_along_y: false,
                size: IVec2::new(1, 1),
                collision_size: Vec2::new(0.0, 0.0),
                collision_offset: Vec2::new(0.0, 0.0),
                rendering_size: Vec2::new(1.0, 1.0),
                rendering_offset: Vec2::new(0.0, 0.0),
                feature_set: Box::new(()),
            }
        });

        // player entity
        builder.add_entity("entity_player".into(), |registry, retriever| {
            core::EntityDescriptor {
                display_name: "Player".into(),
                description: "".into(),
                images: vec![
                    core::ImageDescriptor {
                        frames: vec![
                            retriever.load("image_entity_player_idle0"),
                            retriever.load("image_entity_player_idle1"),
                        ],
                        step_tick: 24,
                        is_loop: true,
                    },
                    core::ImageDescriptor {
                        frames: vec![
                            retriever.load("image_entity_player_idle0r"),
                            retriever.load("image_entity_player_idle1r"),
                        ],
                        step_tick: 24,
                        is_loop: true,
                    },
                    core::ImageDescriptor {
                        frames: vec![
                            retriever.load("image_entity_player_walk0"),
                            retriever.load("image_entity_player_idle0"),
                            retriever.load("image_entity_player_walk1"),
                            retriever.load("image_entity_player_idle1"),
                        ],
                        step_tick: 6,
                        is_loop: true,
                    },
                    core::ImageDescriptor {
                        frames: vec![
                            retriever.load("image_entity_player_walk0r"),
                            retriever.load("image_entity_player_idle0r"),
                            retriever.load("image_entity_player_walk1r"),
                            retriever.load("image_entity_player_idle1r"),
                        ],
                        step_tick: 6,
                        is_loop: true,
                    },
                ],
                z_along_y: true,
                collision_size: Vec2::new(0.8, 0.8),
                collision_offset: Vec2::new(-0.4, 0.1),
                rendering_size: Vec2::new(1.5, 2.25),
                rendering_offset: Vec2::new(-0.75, 0.0),
                feature_set: Box::new(addon::PlayerEntityFeatureSet {
                    move_speed: 3.0,
                    inventory_id: registry.get("inventory_player"),
                }),
            }
        });

        // pig entity
        builder.add_entity("entity_pig".into(), |_, retriever| core::EntityDescriptor {
            display_name: "Pig".into(),
            description: "".into(),
            images: vec![
                core::ImageDescriptor {
                    frames: vec![
                        retriever.load("image_entity_pig_idle0"),
                        retriever.load("image_entity_pig_idle1"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                core::ImageDescriptor {
                    frames: vec![
                        retriever.load("image_entity_pig_walk0"),
                        retriever.load("image_entity_pig_idle0"),
                        retriever.load("image_entity_pig_walk1"),
                        retriever.load("image_entity_pig_idle1"),
                    ],
                    step_tick: 12,
                    is_loop: true,
                },
            ],
            z_along_y: true,
            collision_size: Vec2::new(0.8, 0.8),
            collision_offset: Vec2::new(-0.4, 0.1),
            rendering_size: Vec2::new(2.0, 2.0),
            rendering_offset: Vec2::new(-1.0, 0.0),
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
        builder.add_entity("entity_cow".into(), |_, retriever| core::EntityDescriptor {
            display_name: "Cow".into(),
            description: "".into(),
            images: vec![
                core::ImageDescriptor {
                    frames: vec![
                        retriever.load("image_entity_cow_idle0"),
                        retriever.load("image_entity_cow_idle1"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                core::ImageDescriptor {
                    frames: vec![
                        retriever.load("image_entity_cow_walk0"),
                        retriever.load("image_entity_cow_idle0"),
                        retriever.load("image_entity_cow_walk1"),
                        retriever.load("image_entity_cow_idle1"),
                    ],
                    step_tick: 12,
                    is_loop: true,
                },
            ],
            z_along_y: true,
            collision_size: Vec2::new(0.8, 0.8),
            collision_offset: Vec2::new(-0.4, 0.1),
            rendering_size: Vec2::new(2.0, 2.0),
            rendering_offset: Vec2::new(-1.0, 0.0),
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
            core::EntityDescriptor {
                display_name: "Sheep".into(),
                description: "".into(),
                images: vec![
                    core::ImageDescriptor {
                        frames: vec![
                            retriever.load("image_entity_sheep_idle0"),
                            retriever.load("image_entity_sheep_idle1"),
                        ],
                        step_tick: 24,
                        is_loop: true,
                    },
                    core::ImageDescriptor {
                        frames: vec![
                            retriever.load("image_entity_sheep_walk0"),
                            retriever.load("image_entity_sheep_idle0"),
                            retriever.load("image_entity_sheep_walk1"),
                            retriever.load("image_entity_sheep_idle1"),
                        ],
                        step_tick: 12,
                        is_loop: true,
                    },
                ],
                z_along_y: true,
                collision_size: Vec2::new(0.8, 0.8),
                collision_offset: Vec2::new(-0.4, 0.1),
                rendering_size: Vec2::new(2.0, 2.0),
                rendering_offset: Vec2::new(-1.0, 0.0),
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
            core::EntityDescriptor {
                display_name: "Chicken".into(),
                description: "".into(),
                images: vec![
                    core::ImageDescriptor {
                        frames: vec![retriever.load("image_entity_chicken_idle")],
                        step_tick: 24,
                        is_loop: true,
                    },
                    core::ImageDescriptor {
                        frames: vec![
                            retriever.load("image_entity_chicken_walk"),
                            retriever.load("image_entity_chicken_idle"),
                        ],
                        step_tick: 12,
                        is_loop: true,
                    },
                ],
                z_along_y: true,
                collision_size: Vec2::new(0.8, 0.8),
                collision_offset: Vec2::new(-0.4, 0.1),
                rendering_size: Vec2::new(1.0, 1.0),
                rendering_offset: Vec2::new(-0.5, 0.0),
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
            core::EntityDescriptor {
                display_name: "Bird".into(),
                description: "".into(),
                images: vec![
                    core::ImageDescriptor {
                        frames: vec![retriever.load("image_entity_bird_idle")],
                        step_tick: 24,
                        is_loop: true,
                    },
                    core::ImageDescriptor {
                        frames: vec![
                            retriever.load("image_entity_bird_walk"),
                            retriever.load("image_entity_bird_idle"),
                        ],
                        step_tick: 12,
                        is_loop: true,
                    },
                ],
                z_along_y: true,
                collision_size: Vec2::new(0.8, 0.8),
                collision_offset: Vec2::new(-0.4, 0.1),
                rendering_size: Vec2::new(1.0, 1.0),
                rendering_offset: Vec2::new(-0.5, 0.0),
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

        // package item
        builder.add_item("item_package".into(), |_, retriever| core::ItemDescriptor {
            display_name: "Package".into(),
            description: "A package of items.".into(),
            images: vec![core::ImageDescriptor {
                frames: vec![retriever.load("image_item_package")],
                step_tick: 0,
                is_loop: false,
            }],
            feature_set: Box::new(()),
        });

        // player inventory
        builder.add_inventory("inventory_player".into(), |_, retriever| {
            core::InventoryDescriptor {
                size: 32,
                callback: retriever.load("callable_inventory_player"),
            }
        });

        let retriever = core::Retriever::new(retrieve_callable);
        let desc = core::BuildDescriptor {
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
                    let id = context.registry.get("tile_grass");
                    Box::new(addon::MarchGenerator {
                        prob: 0.5,
                        place_fn: Box::new(move |dataflow, location| {
                            let tile = core::dataflow::Tile {
                                id,
                                location,
                                data: Default::default(),
                                render_param: Default::default(),
                            };
                            let _ = dataflow.insert_tile(tile);
                        }),
                    })
                },
                {
                    let id = context.registry.get("tile_dirt");
                    Box::new(addon::MarchGenerator {
                        prob: 1.0,
                        place_fn: Box::new(move |dataflow, location| {
                            let tile = core::dataflow::Tile {
                                id,
                                location,
                                data: Default::default(),
                                render_param: Default::default(),
                            };
                            let _ = dataflow.insert_tile(tile);
                        }),
                    })
                },
                {
                    let id = context.registry.get("block_dandelion");
                    Box::new(addon::SpawnGenerator {
                        prob: 0.05,
                        place_fn: Box::new(move |dataflow, location| {
                            let block = core::dataflow::Block {
                                id,
                                location: location.as_ivec2(),
                                data: Default::default(),
                                render_param: Default::default(),
                            };
                            let _ = dataflow.insert_block(block);
                        }),
                    })
                },
                {
                    let id = context.registry.get("entity_bird");
                    Box::new(addon::SpawnGenerator {
                        prob: 0.05,
                        place_fn: Box::new(move |dataflow, location| {
                            let entity = core::dataflow::Entity {
                                id,
                                location,
                                data: Default::default(),
                                render_param: Default::default(),
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

        CONTEXT.set(Some(context));
    }

    #[func]
    fn close() {
        CONTEXT.take();
    }

    #[func]
    fn forward_time(delta_secs: f32) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            context.dataflow.forward_time(delta_secs);
        })
    }

    #[func]
    fn forwarde_rect(min_rect: Rect2, delta_secs: f32) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let position = Vec2::new(min_rect.position.x, min_rect.position.y);
            let size = Vec2::new(min_rect.size.x, min_rect.size.y);
            let min_rect = [position, position + size];
            addon::ForwarderSystem::forward(&mut context.dataflow, min_rect, delta_secs).unwrap();
        })
    }

    #[func]
    fn generate_rect(min_rect: Rect2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let position = Vec2::new(min_rect.position.x, min_rect.position.y);
            let size = Vec2::new(min_rect.size.x, min_rect.size.y);
            let min_rect = [position, position + size];
            addon::GeneratorSystem::generate(&mut context.dataflow, min_rect).unwrap();
        })
    }

    #[func]
    fn spawn_player(location: Vector2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let location = Vec2::new(location.x, location.y);
            let entity = core::dataflow::Entity {
                id: context.registry.get("entity_player"),
                location,
                data: Default::default(),
                render_param: Default::default(),
            };
            context.dataflow.insert_entity(entity).unwrap();
        })
    }

    #[func]
    fn push_player_input(input: Vector2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let input = Vec2::new(input.x, input.y);
            addon::PlayerSystem::push_input(&mut context.dataflow, input).unwrap();
        })
    }

    #[func]
    fn get_player_location() -> Vector2 {
        CONTEXT.with_borrow(|context| {
            let context = context.as_ref().unwrap();

            let location = addon::PlayerSystem::get_location(&context.dataflow).unwrap();
            Vector2::new(location.x, location.y)
        })
    }

    #[func]
    fn open_player_inventory() {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            if let Ok(inventory_key) = addon::PlayerSystem::get_inventory_key(&context.dataflow) {
                let _ = context
                    .item_storage_view
                    .open_inventory(&context.dataflow, inventory_key);
            }
        })
    }

    #[func]
    fn has_item(inventory_key: u32, local_key: u32) -> bool {
        CONTEXT.with_borrow(|context| {
            let context = context.as_ref().unwrap();

            let slot_key = (inventory_key, local_key);
            context.dataflow.get_item(slot_key).is_ok()
        })
    }

    #[func]
    fn get_item_amount(inventory_key: u32, local_key: u32) -> u32 {
        CONTEXT.with_borrow(|context| {
            let context = context.as_ref().unwrap();

            let slot_key = (inventory_key, local_key);

            let item = context.dataflow.get_item(slot_key).unwrap();
            item.amount
        })
    }

    #[func]
    fn get_item_display_name(inventory_key: u32, local_key: u32) -> GString {
        CONTEXT.with_borrow(|context| {
            let context = context.as_ref().unwrap();

            let slot_key = (inventory_key, local_key);
            let text = context.dataflow.get_item_display_name(slot_key).unwrap();
            text.into()
        })
    }

    #[func]
    fn get_item_description(inventory_key: u32, local_key: u32) -> GString {
        CONTEXT.with_borrow(|context| {
            let context = context.as_ref().unwrap();

            let slot_key = (inventory_key, local_key);
            let text = context.dataflow.get_item_description(slot_key).unwrap();
            text.into()
        })
    }

    #[func]
    fn swap_item(
        src_inventory_key: u32,
        src_local_key: u32,
        dst_inventory_key: u32,
        dst_local_key: u32,
    ) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let src_slot_key = (src_inventory_key, src_local_key);
            let dst_slot_key = (dst_inventory_key, dst_local_key);
            context
                .dataflow
                .swap_item(src_slot_key, dst_slot_key)
                .unwrap();
        })
    }

    #[func]
    fn draw_item(inventory_key: u32, local_key: u32, control_item: Gd<godot::classes::Control>) {
        CONTEXT.with_borrow(|context| {
            let context = context.as_ref().unwrap();

            let slot_key = (inventory_key, local_key);
            context
                .item_storage_view
                .draw_item(&context.dataflow, slot_key, control_item)
                .unwrap();
        })
    }

    #[func]
    fn set_selection(location: Vector2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let point = Vec2::new(location.x, location.y).floor().as_ivec2();
            let tiles = context
                .dataflow
                .get_tile_key_by_point(point)
                .into_iter()
                .collect::<Vec<_>>();

            let point = Vec2::new(location.x, location.y);
            let blocks = context
                .dataflow
                .get_block_keys_by_hint_point(point)
                .collect::<Vec<_>>();

            let point = Vec2::new(location.x, location.y);
            let entities = context
                .dataflow
                .get_entity_keys_by_hint_point(point)
                .collect::<Vec<_>>();

            context
                .selection_view
                .update_view(&context.dataflow, &tiles, &blocks, &entities);
        })
    }

    #[func]
    fn clear_selection() {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            context
                .selection_view
                .update_view(&context.dataflow, &[], &[], &[]);
        })
    }

    #[func]
    fn get_selection_size() -> u32 {
        Default::default()
    }

    #[func]
    fn get_selection_display_name() -> GString {
        Default::default()
    }

    #[func]
    fn get_selection_description() -> GString {
        Default::default()
    }

    #[func]
    fn update_view(min_rect: Rect2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

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
        })
    }
}
