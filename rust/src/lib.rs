use glam::*;
use godot::prelude::*;

pub mod inner;

mod block;
mod decl;
mod entity;
mod item;
mod tile;

struct Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Extension {}

#[derive(GodotClass)]
#[class(no_init)]
struct PanicHook {}

#[godot_api]
impl PanicHook {
    #[func]
    fn open() {
        godot_print!("Set panic hook");

        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let location_msg;
            if let Some(location) = info.location() {
                location_msg = format!("file {} at line {}", location.file(), location.line());
            } else {
                location_msg = "unknown location".into();
            }

            let payload_msg;
            if let Some(s) = info.payload().downcast_ref::<&str>() {
                payload_msg = s.to_string();
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                payload_msg = s.clone();
            } else {
                payload_msg = "unknown panic".into();
            }

            godot_error!("[RUST] {}: {}", location_msg, payload_msg);
            hook(info);
        }));
    }

    #[func]
    fn close() {
        godot_print!("Unset panic hook");

        let _ = std::panic::take_hook();
    }
}

#[allow(dead_code)]
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
    inventory_player: u16,
}

type MutCell<T> = std::cell::RefCell<Option<T>>;
thread_local! { static CONTEXT: MutCell<decl::Context<Registry>> = Default::default(); }

#[derive(GodotClass)]
#[class(no_init)]
struct Root;

#[godot_api]
impl Root {
    #[func]
    fn open(world: Gd<godot::classes::World3D>, node: Gd<godot::classes::Node>) {
        let mut builder = decl::ContextBuilder::<Registry>::new();

        // tiles
        let tile_dirt = builder.add_tile(|_| decl::TileDescriptor {
            name_text: "Dirt".into(),
            desc_text: "".into(),
            images: vec![decl::ImageDescriptor {
                frames: vec!["res://images/surface_dirt.webp".into()],
                step_tick: 0,
                is_loop: false,
            }],
            collision: false,
            feature: Box::new(()),
        });
        let tile_grass = builder.add_tile(|_| decl::TileDescriptor {
            name_text: "Grass".into(),
            desc_text: "".into(),
            images: vec![decl::ImageDescriptor {
                frames: vec!["res://images/surface_grass.webp".into()],
                step_tick: 0,
                is_loop: false,
            }],
            collision: false,
            feature: Box::new(()),
        });

        // blocks
        let block_dandelion = builder.add_block(|_| decl::BlockDescriptor {
            images: vec![decl::ImageDescriptor {
                frames: vec!["res://images/dandelion.webp".into()],
                step_tick: 0,
                is_loop: false,
            }],
            z_along_y: false,
            size: IVec2::new(1, 1),
            collision_size: Vec2::new(0.0, 0.0),
            collision_offset: Vec2::new(0.0, 0.0),
            rendering_size: Vec2::new(1.0, 1.0),
            rendering_offset: Vec2::new(0.0, 0.0),
            feature: Box::new(()),
        });
        let block_fallen_leaves = builder.add_block(|_| decl::BlockDescriptor {
            images: vec![decl::ImageDescriptor {
                frames: vec!["res://images/fallen_leaves.webp".into()],
                step_tick: 0,
                is_loop: false,
            }],
            z_along_y: false,
            size: IVec2::new(1, 1),
            collision_size: Vec2::new(0.0, 0.0),
            collision_offset: Vec2::new(0.0, 0.0),
            rendering_size: Vec2::new(1.0, 1.0),
            rendering_offset: Vec2::new(0.0, 0.0),
            feature: Box::new(()),
        });
        let block_mix_grass = builder.add_block(|_| decl::BlockDescriptor {
            images: vec![decl::ImageDescriptor {
                frames: vec!["res://images/mix_grass.webp".into()],
                step_tick: 0,
                is_loop: false,
            }],
            z_along_y: false,
            size: IVec2::new(1, 1),
            collision_size: Vec2::new(0.0, 0.0),
            collision_offset: Vec2::new(0.0, 0.0),
            rendering_size: Vec2::new(1.0, 1.0),
            rendering_offset: Vec2::new(0.0, 0.0),
            feature: Box::new(()),
        });
        let block_mix_pebbles = builder.add_block(|_| decl::BlockDescriptor {
            images: vec![decl::ImageDescriptor {
                frames: vec!["res://images/mix_pebbles.webp".into()],
                step_tick: 0,
                is_loop: false,
            }],
            z_along_y: false,
            size: IVec2::new(1, 1),
            collision_size: Vec2::new(0.0, 0.0),
            collision_offset: Vec2::new(0.0, 0.0),
            rendering_size: Vec2::new(1.0, 1.0),
            rendering_offset: Vec2::new(0.0, 0.0),
            feature: Box::new(()),
        });

        // entities
        let entity_player = builder.add_entity(|reg| decl::EntityDescriptor {
            images: vec![
                decl::ImageDescriptor {
                    frames: vec![
                        "res://images/player_idle_0.webp".into(),
                        "res://images/player_idle_1.webp".into(),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                decl::ImageDescriptor {
                    frames: vec![
                        "res://images/player_walk_0.webp".into(),
                        "res://images/player_idle_0.webp".into(),
                        "res://images/player_walk_1.webp".into(),
                        "res://images/player_idle_1.webp".into(),
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
            feature: Box::new(inner::PlayerEntityFeature {
                move_speed: 3.0,
                inventory_id: reg.inventory_player,
            }),
        });
        let entity_pig = builder.add_entity(|_| decl::EntityDescriptor {
            images: vec![
                decl::ImageDescriptor {
                    frames: vec![
                        "res://images/pig_idle_0.webp".into(),
                        "res://images/pig_idle_1.webp".into(),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                decl::ImageDescriptor {
                    frames: vec![
                        "res://images/pig_walk_0.webp".into(),
                        "res://images/pig_idle_0.webp".into(),
                        "res://images/pig_walk_1.webp".into(),
                        "res://images/pig_idle_1.webp".into(),
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
            feature: Box::new(inner::AnimalEntityFeature {
                min_rest_secs: 0.0,
                max_rest_secs: 10.0,
                min_distance: 0.0,
                max_distance: 10.0,
                speed: 1.0,
                idle_variant: 0,
                walk_variant: 1,
            }),
        });
        let entity_cow = builder.add_entity(|_| decl::EntityDescriptor {
            images: vec![
                decl::ImageDescriptor {
                    frames: vec![
                        "res://images/cow_idle_0.webp".into(),
                        "res://images/cow_idle_1.webp".into(),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                decl::ImageDescriptor {
                    frames: vec![
                        "res://images/cow_walk_0.webp".into(),
                        "res://images/cow_idle_0.webp".into(),
                        "res://images/cow_walk_1.webp".into(),
                        "res://images/cow_idle_1.webp".into(),
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
            feature: Box::new(inner::AnimalEntityFeature {
                min_rest_secs: 0.0,
                max_rest_secs: 10.0,
                min_distance: 0.0,
                max_distance: 10.0,
                speed: 1.0,
                idle_variant: 0,
                walk_variant: 1,
            }),
        });
        let entity_sheep = builder.add_entity(|_| decl::EntityDescriptor {
            images: vec![
                decl::ImageDescriptor {
                    frames: vec![
                        "res://images/sheep_idle_0.webp".into(),
                        "res://images/sheep_idle_1.webp".into(),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                decl::ImageDescriptor {
                    frames: vec![
                        "res://images/sheep_walk_0.webp".into(),
                        "res://images/sheep_idle_0.webp".into(),
                        "res://images/sheep_walk_1.webp".into(),
                        "res://images/sheep_idle_1.webp".into(),
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
            feature: Box::new(inner::AnimalEntityFeature {
                min_rest_secs: 0.0,
                max_rest_secs: 10.0,
                min_distance: 0.0,
                max_distance: 10.0,
                speed: 1.0,
                idle_variant: 0,
                walk_variant: 1,
            }),
        });
        let entity_chicken = builder.add_entity(|_| decl::EntityDescriptor {
            images: vec![
                decl::ImageDescriptor {
                    frames: vec!["res://images/chicken_idle.webp".into()],
                    step_tick: 24,
                    is_loop: true,
                },
                decl::ImageDescriptor {
                    frames: vec![
                        "res://images/chicken_walk.webp".into(),
                        "res://images/chicken_idle.webp".into(),
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
            feature: Box::new(inner::AnimalEntityFeature {
                min_rest_secs: 0.0,
                max_rest_secs: 10.0,
                min_distance: 0.0,
                max_distance: 10.0,
                speed: 1.0,
                idle_variant: 0,
                walk_variant: 1,
            }),
        });
        let entity_bird = builder.add_entity(|_| decl::EntityDescriptor {
            images: vec![
                decl::ImageDescriptor {
                    frames: vec!["res://images/bird_idle.webp".into()],
                    step_tick: 24,
                    is_loop: true,
                },
                decl::ImageDescriptor {
                    frames: vec![
                        "res://images/bird_walk.webp".into(),
                        "res://images/bird_idle.webp".into(),
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
            feature: Box::new(inner::AnimalEntityFeature {
                min_rest_secs: 0.0,
                max_rest_secs: 10.0,
                min_distance: 0.0,
                max_distance: 10.0,
                speed: 1.0,
                idle_variant: 0,
                walk_variant: 1,
            }),
        });

        // item
        let item_package = builder.add_item(|_| decl::ItemDescriptor {
            name_text: "Package".into(),
            desc_text: "A package of items.".into(),
            images: vec![decl::ImageDescriptor {
                frames: vec!["res://images/package.webp".into()],
                step_tick: 0,
                is_loop: false,
            }],
            feature: Box::new(()),
        });

        // inventory
        let inventory_player = builder.add_inventory(|_| decl::InventoryDescriptor {
            size: 32,
            scene: "res://scenes/inventory_player.tscn".into(),
        });

        // gen rule
        builder.add_gen_rule(|reg| {
            let id = reg.tile_grass;
            let gen_rule = inner::MarchGenRule {
                prob: 0.5,
                gen_fn: Box::new(move |root, location| {
                    let tile = inner::Tile {
                        id,
                        location,
                        data: Default::default(),
                        render_param: Default::default(),
                    };
                    let _ = root.tile_insert(tile);
                }),
            };
            decl::GenRuleDescriptor {
                gen_rule: Box::new(gen_rule),
            }
        });
        builder.add_gen_rule(|reg| {
            let id = reg.tile_dirt;
            let gen_rule = inner::MarchGenRule {
                prob: 1.0,
                gen_fn: Box::new(move |root, location| {
                    let tile = inner::Tile {
                        id,
                        location,
                        data: Default::default(),
                        render_param: Default::default(),
                    };
                    let _ = root.tile_insert(tile);
                }),
            };
            decl::GenRuleDescriptor {
                gen_rule: Box::new(gen_rule),
            }
        });
        builder.add_gen_rule(|reg| {
            let id = reg.block_dandelion;
            let gen_rule = inner::SpawnGenRule {
                prob: 0.05,
                gen_fn: Box::new(move |root, location| {
                    let block = inner::Block {
                        id,
                        location: location.as_ivec2(),
                        data: Default::default(),
                        render_param: Default::default(),
                    };
                    let _ = root.block_insert(block);
                }),
            };
            decl::GenRuleDescriptor {
                gen_rule: Box::new(gen_rule),
            }
        });
        builder.add_gen_rule(|reg| {
            let id = reg.entity_bird;
            let gen_rule = inner::SpawnGenRule {
                prob: 0.05,
                gen_fn: Box::new(move |root, location| {
                    let entity = inner::Entity {
                        id,
                        location,
                        data: Default::default(),
                        render_param: Default::default(),
                    };
                    let _ = root.entity_insert(entity);
                }),
            };
            decl::GenRuleDescriptor {
                gen_rule: Box::new(gen_rule),
            }
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
            inventory_player,
        };
        let desc = decl::BuildDescriptor {
            tile_shaders: vec!["res://shaders/field.gdshader".into()],
            block_shaders: vec![
                "res://shaders/field.gdshader".into(),
                "res://shaders/field_shadow.gdshader".into(),
            ],
            entity_shaders: vec![
                "res://shaders/field.gdshader".into(),
                "res://shaders/field_shadow.gdshader".into(),
            ],
            world,
            node,
        };
        let context = builder.build(register, desc);
        CONTEXT.set(Some(context));
    }

    #[func]
    fn close() {
        CONTEXT.take();
    }

    #[func]
    fn time_forward(delta_secs: f32) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            context.root.time_forward(delta_secs);
        })
    }

    #[func]
    fn forwarder_exec_rect(min_rect: Rect2, delta_secs: f32) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let position = Vec2::new(min_rect.position.x, min_rect.position.y);
            let size = Vec2::new(min_rect.size.x, min_rect.size.y);
            let min_rect = [position, position + size];
            context
                .root
                .forwarder_exec_rect(min_rect, delta_secs)
                .unwrap();
        })
    }

    #[func]
    fn gen_exec_rect(min_rect: Rect2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let position = Vec2::new(min_rect.position.x, min_rect.position.y);
            let size = Vec2::new(min_rect.size.x, min_rect.size.y);
            let min_rect = [position, position + size];
            context.root.gen_exec_rect(min_rect).unwrap();
        })
    }

    #[func]
    fn player_spawn(location: Vector2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let location = Vec2::new(location.x, location.y);
            let entity = inner::Entity {
                id: context.registry.entity_player,
                location,
                data: Default::default(),
                render_param: Default::default(),
            };
            let entity_key = context.root.entity_insert(entity).unwrap();

            // for inventory and item rendering test
            let inventory_key = context
                .root
                .entity_get_inventory(entity_key)
                .unwrap()
                .unwrap();
            let item = inner::Item {
                id: context.registry.item_package,
                amount: 1,
                data: Default::default(),
                render_param: Default::default(),
            };
            context.root.item_push_item(inventory_key, item).unwrap();
        })
    }

    #[func]
    fn player_push_input(input: Vector2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let input = Vec2::new(input.x, input.y);
            context.root.player_push_input(input).unwrap();
        })
    }

    #[func]
    fn player_get_location() -> Vector2 {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let location = context.root.player_get_location().unwrap();
            Vector2::new(location[0], location[1])
        })
    }

    #[func]
    fn item_open_inventory_by_tile(location: Vector2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let location = Vec2::new(location.x, location.y).as_ivec2();
            let tile = context.root.tile_get_by_point(location).unwrap();
            context
                .item_store
                .open_inventory_by_tile(&context.root, tile)
                .unwrap();
        })
    }

    #[func]
    fn item_open_inventory_by_block(location: Vector2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let location = Vec2::new(location.x, location.y);
            let block = context
                .root
                .block_get_by_hint_point(location)
                .next()
                .unwrap();
            context
                .item_store
                .open_inventory_by_block(&context.root, block)
                .unwrap();
        })
    }

    #[func]
    fn item_open_inventory_by_entity(location: Vector2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let location = Vec2::new(location.x, location.y);
            let entity = context
                .root
                .entity_get_by_hint_point(location)
                .next()
                .unwrap();
            context
                .item_store
                .open_inventory_by_entity(&context.root, entity)
                .unwrap();
        })
    }

    #[func]
    fn item_draw_view(
        inventory_key: u32,
        local_key: u32,
        control_item: Gd<godot::classes::Control>,
    ) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let slot_key = (inventory_key, local_key);
            context
                .item_store
                .draw_view(&context.root, slot_key, control_item)
                .unwrap();
        })
    }

    #[func]
    fn update_view(min_rect: Rect2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let position = Vec2::new(min_rect.position.x, min_rect.position.y);
            let size = Vec2::new(min_rect.size.x, min_rect.size.y);
            let min_rect = [position, position + size];
            context.tile_field.update_view(&context.root, min_rect);
            context.block_field.update_view(&context.root, min_rect);
            context.entity_field.update_view(&context.root, min_rect);
        })
    }
}
