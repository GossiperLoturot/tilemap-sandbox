use glam::*;
use godot::prelude::*;

pub mod inner;

mod block;
mod decl;
mod entity;
mod item;
mod pick;
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
    fn open(world: Gd<godot::classes::World3D>, ui: Gd<godot::classes::Node>) {
        let mut builder = decl::ContextBuilder::<Registry>::new();

        // tiles
        let tile_dirt = builder.add_tile(|_| decl::TileDescriptor {
            name_text: "Dirt".into(),
            desc_text: "".into(),
            images: vec![decl::ImageDescriptor {
                frames: vec![load("res://images/surface_dirt.webp")],
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
                frames: vec![load("res://images/surface_grass.webp")],
                step_tick: 0,
                is_loop: false,
            }],
            collision: false,
            feature: Box::new(()),
        });

        // blocks
        let block_dandelion = builder.add_block(|_| decl::BlockDescriptor {
            name_text: "Dandelion".into(),
            desc_text: "".into(),
            images: vec![decl::ImageDescriptor {
                frames: vec![load("res://images/dandelion.webp")],
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
            name_text: "Fallen Leaves".into(),
            desc_text: "".into(),
            images: vec![decl::ImageDescriptor {
                frames: vec![load("res://images/fallen_leaves.webp")],
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
            name_text: "Grass".into(),
            desc_text: "".into(),
            images: vec![decl::ImageDescriptor {
                frames: vec![load("res://images/mix_grass.webp")],
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
            name_text: "Pebbles".into(),
            desc_text: "".into(),
            images: vec![decl::ImageDescriptor {
                frames: vec![load("res://images/mix_pebbles.webp")],
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
            name_text: "Player".into(),
            desc_text: "".into(),
            images: vec![
                decl::ImageDescriptor {
                    frames: vec![
                        load("res://images/player_idle_0.webp"),
                        load("res://images/player_idle_1.webp"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                decl::ImageDescriptor {
                    frames: vec![
                        load("res://images/player_walk_0.webp"),
                        load("res://images/player_idle_0.webp"),
                        load("res://images/player_walk_1.webp"),
                        load("res://images/player_idle_1.webp"),
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
            name_text: "Pig".into(),
            desc_text: "".into(),
            images: vec![
                decl::ImageDescriptor {
                    frames: vec![
                        load("res://images/pig_idle_0.webp"),
                        load("res://images/pig_idle_1.webp"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                decl::ImageDescriptor {
                    frames: vec![
                        load("res://images/pig_walk_0.webp"),
                        load("res://images/pig_idle_0.webp"),
                        load("res://images/pig_walk_1.webp"),
                        load("res://images/pig_idle_1.webp"),
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
            name_text: "Cow".into(),
            desc_text: "".into(),
            images: vec![
                decl::ImageDescriptor {
                    frames: vec![
                        load("res://images/cow_idle_0.webp"),
                        load("res://images/cow_idle_1.webp"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                decl::ImageDescriptor {
                    frames: vec![
                        load("res://images/cow_walk_0.webp"),
                        load("res://images/cow_idle_0.webp"),
                        load("res://images/cow_walk_1.webp"),
                        load("res://images/cow_idle_1.webp"),
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
            name_text: "Sheep".into(),
            desc_text: "".into(),
            images: vec![
                decl::ImageDescriptor {
                    frames: vec![
                        load("res://images/sheep_idle_0.webp"),
                        load("res://images/sheep_idle_1.webp"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                decl::ImageDescriptor {
                    frames: vec![
                        load("res://images/sheep_walk_0.webp"),
                        load("res://images/sheep_idle_0.webp"),
                        load("res://images/sheep_walk_1.webp"),
                        load("res://images/sheep_idle_1.webp"),
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
            name_text: "Chicken".into(),
            desc_text: "".into(),
            images: vec![
                decl::ImageDescriptor {
                    frames: vec![load("res://images/chicken_idle.webp")],
                    step_tick: 24,
                    is_loop: true,
                },
                decl::ImageDescriptor {
                    frames: vec![
                        load("res://images/chicken_walk.webp"),
                        load("res://images/chicken_idle.webp"),
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
            name_text: "Bird".into(),
            desc_text: "".into(),
            images: vec![
                decl::ImageDescriptor {
                    frames: vec![load("res://images/bird_idle.webp")],
                    step_tick: 24,
                    is_loop: true,
                },
                decl::ImageDescriptor {
                    frames: vec![
                        load("res://images/bird_walk.webp"),
                        load("res://images/bird_idle.webp"),
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
                frames: vec![load("res://images/package.webp")],
                step_tick: 0,
                is_loop: false,
            }],
            feature: Box::new(()),
        });

        // inventory
        let inventory_player = builder.add_inventory(|_| decl::InventoryDescriptor {
            size: 32,
            scene: load("res://scenes/inventory_player.tscn"),
            callback: Box::new(|ui, mut instance, key| {
                instance.call("change_inventory", &[ui.to_variant(), key.to_variant()]);
            }),
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
            tile_shaders: vec![load("res://shaders/field.gdshader")],
            block_shaders: vec![
                load("res://shaders/field.gdshader"),
                load("res://shaders/field_shadow.gdshader"),
            ],
            entity_shaders: vec![
                load("res://shaders/field.gdshader"),
                load("res://shaders/field_shadow.gdshader"),
            ],
            pick_shader: load("res://shaders/pick.gdshader"),
            world,
            ui,
        };
        let mut context = builder.build(register, desc);

        // register gen system
        let desc = inner::GenResourceDescriptor {
            gen_rules: vec![
                Box::new(inner::MarchGenRule {
                    prob: 0.5,
                    gen_fn: Box::new(move |root, location| {
                        let tile = inner::Tile {
                            id: tile_grass,
                            location,
                            data: Default::default(),
                            render_param: Default::default(),
                        };
                        let _ = root.tile_insert(tile);
                    }),
                }),
                Box::new(inner::MarchGenRule {
                    prob: 1.0,
                    gen_fn: Box::new(move |root, location| {
                        let tile = inner::Tile {
                            id: tile_dirt,
                            location,
                            data: Default::default(),
                            render_param: Default::default(),
                        };
                        let _ = root.tile_insert(tile);
                    }),
                }),
                Box::new(inner::SpawnGenRule {
                    prob: 0.05,
                    gen_fn: Box::new(move |root, location| {
                        let block = inner::Block {
                            id: block_dandelion,
                            location: location.as_ivec2(),
                            data: Default::default(),
                            render_param: Default::default(),
                        };
                        let _ = root.block_insert(block);
                    }),
                }),
                Box::new(inner::SpawnGenRule {
                    prob: 0.05,
                    gen_fn: Box::new(move |root, location| {
                        let entity = inner::Entity {
                            id: entity_bird,
                            location,
                            data: Default::default(),
                            render_param: Default::default(),
                        };
                        let _ = root.entity_insert(entity);
                    }),
                }),
            ],
        };
        let resource = inner::GenResource::new(desc);
        context.root.insert_resources(resource).unwrap();

        // register player system
        let resource = inner::PlayerResource::new();
        context.root.insert_resources(resource).unwrap();

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
            inner::ForwarderSystem::exec_rect(&mut context.root, min_rect, delta_secs).unwrap();
        })
    }

    #[func]
    fn gen_exec_rect(min_rect: Rect2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let position = Vec2::new(min_rect.position.x, min_rect.position.y);
            let size = Vec2::new(min_rect.size.x, min_rect.size.y);
            let min_rect = [position, position + size];
            inner::GenSystem::exec_rect(&mut context.root, min_rect).unwrap();
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
            inner::PlayerSystem::push_input(&mut context.root, input).unwrap();
        })
    }

    #[func]
    fn player_get_location() -> Vector2 {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let location = inner::PlayerSystem::get_location(&context.root).unwrap();
            Vector2::new(location[0], location[1])
        })
    }

    #[func]
    fn open_inventory_by_tile(location: Vector2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let location = Vec2::new(location.x, location.y).as_ivec2();
            if let Some(tile) = context.root.tile_get_by_point(location) {
                let _ = context
                    .item_store
                    .open_inventory_by_tile(&context.root, tile);
            }
        })
    }

    #[func]
    fn open_inventory_by_block(location: Vector2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let location = Vec2::new(location.x, location.y);
            if let Some(block) = context.root.block_get_by_hint_point(location).next() {
                let _ = context
                    .item_store
                    .open_inventory_by_block(&context.root, block);
            }
        })
    }

    #[func]
    fn open_inventory_by_entity(location: Vector2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let location = Vec2::new(location.x, location.y);
            if let Some(entity) = context.root.entity_get_by_hint_point(location).next() {
                let _ = context
                    .item_store
                    .open_inventory_by_entity(&context.root, entity);
            }
        })
    }

    #[func]
    fn open_inventory_player() {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            if let Ok(inventory_key) = inner::PlayerSystem::get_inventory_key(&context.root) {
                let _ = context
                    .item_store
                    .open_inventory(&context.root, inventory_key);
            }
        })
    }

    #[func]
    fn has_item(inventory_key: u32, local_key: u32) -> bool {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let slot_key = (inventory_key, local_key);
            context
                .item_store
                .has_item(&context.root, slot_key)
                .unwrap()
        })
    }

    #[func]
    fn get_item_amount(inventory_key: u32, local_key: u32) -> u32 {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let slot_key = (inventory_key, local_key);

            context
                .item_store
                .get_item_amount(&context.root, slot_key)
                .unwrap()
        })
    }

    #[func]
    fn get_item_name_text(inventory_key: u32, local_key: u32) -> GString {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let slot_key = (inventory_key, local_key);
            let text = context
                .item_store
                .get_item_name_text(&context.root, slot_key)
                .unwrap();
            text.into()
        })
    }

    #[func]
    fn get_item_desc_text(inventory_key: u32, local_key: u32) -> GString {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let slot_key = (inventory_key, local_key);
            let text = context
                .item_store
                .get_item_desc_text(&context.root, slot_key)
                .unwrap();
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

            let src_item = context.root.item_remove_item(src_slot_key);
            let dst_item = context.root.item_remove_item(dst_slot_key);

            if let Ok(item) = dst_item {
                let _ = context.root.item_insert_item(src_slot_key, item);
            }
            if let Ok(item) = src_item {
                let _ = context.root.item_insert_item(dst_slot_key, item);
            }
        })
    }

    #[func]
    fn draw_item(inventory_key: u32, local_key: u32, control_item: Gd<godot::classes::Control>) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let slot_key = (inventory_key, local_key);
            context
                .item_store
                .draw_item(&context.root, slot_key, control_item)
                .unwrap();
        })
    }

    #[func]
    fn get_pick_size(location: Vector2) -> u32 {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let point = Vec2::new(location.x, location.y).floor().as_ivec2();
            let tiles = context
                .root
                .tile_get_by_point(point)
                .into_iter()
                .collect::<Vec<_>>();
            let point = Vec2::new(location.x, location.y);
            let blocks = context
                .root
                .block_get_by_hint_point(point)
                .collect::<Vec<_>>();
            let point = Vec2::new(location.x, location.y);
            let entity = context
                .root
                .entity_get_by_hint_point(point)
                .collect::<Vec<_>>();

            (tiles.len() + blocks.len() + entity.len()) as u32
        })
    }

    #[func]
    fn get_pick_name_text(location: Vector2, key: u32) -> GString {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let point = Vec2::new(location.x, location.y).floor().as_ivec2();
            let tiles = context
                .root
                .tile_get_by_point(point)
                .into_iter()
                .collect::<Vec<_>>();
            let point = Vec2::new(location.x, location.y);
            let blocks = context
                .root
                .block_get_by_hint_point(point)
                .collect::<Vec<_>>();
            let point = Vec2::new(location.x, location.y);
            let entities = context
                .root
                .entity_get_by_hint_point(point)
                .collect::<Vec<_>>();

            let (lb, ub) = (0, tiles.len() as u32);
            if key < ub {
                let tile = tiles[(key - lb) as usize];
                let name = context.root.tile_get_name_text(tile).unwrap();
                return name.into();
            }

            let (lb, ub) = (ub, ub + blocks.len() as u32);
            if key < ub {
                let block = blocks[(key - lb) as usize];
                let name = context.root.block_get_name_text(block).unwrap();
                return name.into();
            }

            let (lb, ub) = (ub, ub + entities.len() as u32);
            if key < ub {
                let entity = entities[(key - lb) as usize];
                let name = context.root.entity_get_name_text(entity).unwrap();
                return name.into();
            }

            panic!("key out of range");
        })
    }

    #[func]
    fn set_pick(location: Vector2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let point = Vec2::new(location.x, location.y).floor().as_ivec2();
            let tiles = context
                .root
                .tile_get_by_point(point)
                .into_iter()
                .collect::<Vec<_>>();

            let point = Vec2::new(location.x, location.y);
            let blocks = context
                .root
                .block_get_by_hint_point(point)
                .collect::<Vec<_>>();

            let point = Vec2::new(location.x, location.y);
            let entities = context
                .root
                .entity_get_by_hint_point(point)
                .collect::<Vec<_>>();

            context
                .pick
                .update_view(&context.root, &tiles, &blocks, &entities);
        })
    }

    #[func]
    fn clear_pick() {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            context.pick.update_view(&context.root, &[], &[], &[]);
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
