use glam::*;
use godot::prelude::*;

pub mod inner;

mod block;
mod decl;
mod entity;
mod item;
mod selection;
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
    block_fallenleaves: u16,
    block_mixgrass: u16,
    block_mixpebbles: u16,
    entity_player: u16,
    entity_pig: u16,
    entity_cow: u16,
    entity_sheep: u16,
    entity_chicken: u16,
    entity_bird: u16,
    item_package: u16,
    inventory_player: u16,
}

impl inner::Resource for Registry {}

struct Retriever {
    retrieve_callable: Callable,
}

impl Retriever {
    fn new(retrieve_callable: Callable) -> Self {
        Self { retrieve_callable }
    }

    fn load<T: FromGodot>(&self, name: &str) -> T {
        let ret = self.retrieve_callable.call(&[name.to_variant()]);
        ret.to()
    }
}

type Body = decl::Context;
thread_local! { static CONTEXT: std::cell::RefCell<Option<Body>> = Default::default(); }

#[derive(GodotClass)]
#[class(no_init)]
struct Root;

#[godot_api]
impl Root {
    #[func]
    fn open(retrieve_callable: Callable) {
        let mut builder = decl::ContextBuilder::<(&Registry, &Retriever)>::new();

        // dirt tile
        let tile_dirt = builder.add_tile(|(_, retriever)| decl::TileDescriptor {
            name_text: "Dirt".into(),
            desc_text: "".into(),
            images: vec![decl::ImageDescriptor {
                frames: vec![retriever.load("image_tile_dirt")],
                step_tick: 0,
                is_loop: false,
            }],
            collision: false,
            feature: Box::new(()),
        });

        // grass tile
        let tile_grass = builder.add_tile(|(_, retriever)| decl::TileDescriptor {
            name_text: "Grass".into(),
            desc_text: "".into(),
            images: vec![decl::ImageDescriptor {
                frames: vec![retriever.load("image_tile_grass")],
                step_tick: 0,
                is_loop: false,
            }],
            collision: false,
            feature: Box::new(()),
        });

        // dandelion block
        let block_dandelion = builder.add_block(|(_, retriever)| decl::BlockDescriptor {
            name_text: "Dandelion".into(),
            desc_text: "".into(),
            images: vec![decl::ImageDescriptor {
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
            feature: Box::new(()),
        });

        // fallen leaves block
        let block_fallenleaves = builder.add_block(|(_, retriever)| decl::BlockDescriptor {
            name_text: "Fallen Leaves".into(),
            desc_text: "".into(),
            images: vec![decl::ImageDescriptor {
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
            feature: Box::new(()),
        });

        // mix grass block
        let block_mixgrass = builder.add_block(|(_, retriever)| decl::BlockDescriptor {
            name_text: "Grass".into(),
            desc_text: "".into(),
            images: vec![decl::ImageDescriptor {
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
            feature: Box::new(()),
        });

        // mix pebbles block
        let block_mixpebbles = builder.add_block(|(_, retriever)| decl::BlockDescriptor {
            name_text: "Pebbles".into(),
            desc_text: "".into(),
            images: vec![decl::ImageDescriptor {
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
            feature: Box::new(()),
        });

        // player entity
        let entity_player = builder.add_entity(|(registry, retriever)| decl::EntityDescriptor {
            name_text: "Player".into(),
            desc_text: "".into(),
            images: vec![
                decl::ImageDescriptor {
                    frames: vec![
                        retriever.load("image_entity_player_idle0"),
                        retriever.load("image_entity_player_idle1"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                decl::ImageDescriptor {
                    frames: vec![
                        retriever.load("image_entity_player_walk0"),
                        retriever.load("image_entity_player_idle0"),
                        retriever.load("image_entity_player_walk1"),
                        retriever.load("image_entity_player_idle1"),
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
                inventory_id: registry.inventory_player,
            }),
        });

        // pig entity
        let entity_pig = builder.add_entity(|(_, retriever)| decl::EntityDescriptor {
            name_text: "Pig".into(),
            desc_text: "".into(),
            images: vec![
                decl::ImageDescriptor {
                    frames: vec![
                        retriever.load("image_entity_pig_idle0"),
                        retriever.load("image_entity_pig_idle1"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                decl::ImageDescriptor {
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

        // cow entity
        let entity_cow = builder.add_entity(|(_, retriever)| decl::EntityDescriptor {
            name_text: "Cow".into(),
            desc_text: "".into(),
            images: vec![
                decl::ImageDescriptor {
                    frames: vec![
                        retriever.load("image_entity_cow_idle0"),
                        retriever.load("image_entity_cow_idle1"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                decl::ImageDescriptor {
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

        // sheep entity
        let entity_sheep = builder.add_entity(|(_, retriever)| decl::EntityDescriptor {
            name_text: "Sheep".into(),
            desc_text: "".into(),
            images: vec![
                decl::ImageDescriptor {
                    frames: vec![
                        retriever.load("image_entity_sheep_idle0"),
                        retriever.load("image_entity_sheep_idle1"),
                    ],
                    step_tick: 24,
                    is_loop: true,
                },
                decl::ImageDescriptor {
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

        // chicken entity
        let entity_chicken = builder.add_entity(|(_, retriever)| decl::EntityDescriptor {
            name_text: "Chicken".into(),
            desc_text: "".into(),
            images: vec![
                decl::ImageDescriptor {
                    frames: vec![retriever.load("image_entity_chicken_idle")],
                    step_tick: 24,
                    is_loop: true,
                },
                decl::ImageDescriptor {
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

        // bird entity
        let entity_bird = builder.add_entity(|(_, retriever)| decl::EntityDescriptor {
            name_text: "Bird".into(),
            desc_text: "".into(),
            images: vec![
                decl::ImageDescriptor {
                    frames: vec![retriever.load("image_entity_bird_idle")],
                    step_tick: 24,
                    is_loop: true,
                },
                decl::ImageDescriptor {
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

        // package item
        let item_package = builder.add_item(|(_, retriever)| decl::ItemDescriptor {
            name_text: "Package".into(),
            desc_text: "A package of items.".into(),
            images: vec![decl::ImageDescriptor {
                frames: vec![retriever.load("image_item_package")],
                step_tick: 0,
                is_loop: false,
            }],
            feature: Box::new(()),
        });

        // player inventory
        let inventory_player = builder.add_inventory(|(_, retriever)| decl::InventoryDescriptor {
            size: 32,
            callback: retriever.load("callable_inventory_player"),
        });

        let registry = Registry {
            tile_dirt,
            tile_grass,
            block_dandelion,
            block_fallenleaves,
            block_mixgrass,
            block_mixpebbles,
            entity_player,
            entity_pig,
            entity_cow,
            entity_sheep,
            entity_chicken,
            entity_bird,
            item_package,
            inventory_player,
        };
        let retriever = Retriever::new(retrieve_callable);
        let desc = decl::BuildDescriptor {
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
        let mut context = builder.build((&registry, &retriever), desc);

        // registry
        context.root.insert_resources(registry).unwrap();

        // generator system
        let desc = inner::GeneratorResourceDescriptor {
            generators: vec![
                Box::new(inner::MarchGenerator {
                    prob: 0.5,
                    place_fn: Box::new(move |root, location| {
                        let tile = inner::Tile {
                            id: tile_grass,
                            location,
                            data: Default::default(),
                            render_param: Default::default(),
                        };
                        let _ = root.insert_tile(tile);
                    }),
                }),
                Box::new(inner::MarchGenerator {
                    prob: 1.0,
                    place_fn: Box::new(move |root, location| {
                        let tile = inner::Tile {
                            id: tile_dirt,
                            location,
                            data: Default::default(),
                            render_param: Default::default(),
                        };
                        let _ = root.insert_tile(tile);
                    }),
                }),
                Box::new(inner::SpawnGenerator {
                    prob: 0.05,
                    place_fn: Box::new(move |root, location| {
                        let block = inner::Block {
                            id: block_dandelion,
                            location: location.as_ivec2(),
                            data: Default::default(),
                            render_param: Default::default(),
                        };
                        let _ = root.insert_block(block);
                    }),
                }),
                Box::new(inner::SpawnGenerator {
                    prob: 0.05,
                    place_fn: Box::new(move |root, location| {
                        let entity = inner::Entity {
                            id: entity_bird,
                            location,
                            data: Default::default(),
                            render_param: Default::default(),
                        };
                        let _ = root.insert_entity(entity);
                    }),
                }),
            ],
        };
        let resource = inner::GeneratorResource::new(desc);
        context.root.insert_resources(resource).unwrap();

        // player system
        let resource = inner::PlayerResource::new();
        context.root.insert_resources(resource).unwrap();

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

            context.root.forward_time(delta_secs);
        })
    }

    #[func]
    fn forwarde_rect(min_rect: Rect2, delta_secs: f32) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let position = Vec2::new(min_rect.position.x, min_rect.position.y);
            let size = Vec2::new(min_rect.size.x, min_rect.size.y);
            let min_rect = [position, position + size];
            inner::ForwarderSystem::forward(&mut context.root, min_rect, delta_secs).unwrap();
        })
    }

    #[func]
    fn generate_rect(min_rect: Rect2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let position = Vec2::new(min_rect.position.x, min_rect.position.y);
            let size = Vec2::new(min_rect.size.x, min_rect.size.y);
            let min_rect = [position, position + size];
            inner::GeneratorSystem::generate(&mut context.root, min_rect).unwrap();
        })
    }

    #[func]
    fn spawn_player(location: Vector2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let registry = context.root.find_resources::<Registry>().unwrap();
            let registry = registry.borrow().unwrap();

            let location = Vec2::new(location.x, location.y);
            let entity = inner::Entity {
                id: registry.entity_player,
                location,
                data: Default::default(),
                render_param: Default::default(),
            };
            let entity_key = context.root.insert_entity(entity).unwrap();

            // for inventory and item rendering test
            let inventory_key = context
                .root
                .get_inventory_by_entity(entity_key)
                .unwrap()
                .unwrap();
            let item = inner::Item {
                id: registry.item_package,
                amount: 1,
                data: Default::default(),
                render_param: Default::default(),
            };
            context.root.push_item(inventory_key, item).unwrap();
        })
    }

    #[func]
    fn push_player_input(input: Vector2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let input = Vec2::new(input.x, input.y);
            inner::PlayerSystem::push_input(&mut context.root, input).unwrap();
        })
    }

    #[func]
    fn get_player_location() -> Vector2 {
        CONTEXT.with_borrow(|context| {
            let context = context.as_ref().unwrap();

            let location = inner::PlayerSystem::get_location(&context.root).unwrap();
            Vector2::new(location.x, location.y)
        })
    }

    #[func]
    fn open_player_inventory() {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            if let Ok(inventory_key) = inner::PlayerSystem::get_inventory_key(&context.root) {
                let _ = context
                    .item_storage
                    .open_inventory(&context.root, inventory_key);
            }
        })
    }

    #[func]
    fn open_inventory_by_tile(location: Vector2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let location = Vec2::new(location.x, location.y).as_ivec2();
            if let Some(tile) = context.root.get_tile_by_point(location) {
                let _ = context
                    .item_storage
                    .open_inventory_by_tile(&context.root, tile);
            }
        })
    }

    #[func]
    fn open_inventory_by_block(location: Vector2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let location = Vec2::new(location.x, location.y);
            if let Some(block) = context.root.get_block_by_hint_point(location).next() {
                let _ = context
                    .item_storage
                    .open_inventory_by_block(&context.root, block);
            }
        })
    }

    #[func]
    fn open_inventory_by_entity(location: Vector2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let location = Vec2::new(location.x, location.y);
            if let Some(entity) = context.root.get_entity_by_hint_point(location).next() {
                let _ = context
                    .item_storage
                    .open_inventory_by_entity(&context.root, entity);
            }
        })
    }

    #[func]
    fn has_item(inventory_key: u32, local_key: u32) -> bool {
        CONTEXT.with_borrow(|context| {
            let context = context.as_ref().unwrap();

            let slot_key = (inventory_key, local_key);
            context
                .item_storage
                .has_item(&context.root, slot_key)
                .unwrap()
        })
    }

    #[func]
    fn get_item_amount(inventory_key: u32, local_key: u32) -> u32 {
        CONTEXT.with_borrow(|context| {
            let context = context.as_ref().unwrap();

            let slot_key = (inventory_key, local_key);

            context
                .item_storage
                .get_item_amount(&context.root, slot_key)
                .unwrap()
        })
    }

    #[func]
    fn get_item_name_text(inventory_key: u32, local_key: u32) -> GString {
        CONTEXT.with_borrow(|context| {
            let context = context.as_ref().unwrap();

            let slot_key = (inventory_key, local_key);
            let text = context
                .item_storage
                .get_item_name_text(&context.root, slot_key)
                .unwrap();
            text.into()
        })
    }

    #[func]
    fn get_item_desc_text(inventory_key: u32, local_key: u32) -> GString {
        CONTEXT.with_borrow(|context| {
            let context = context.as_ref().unwrap();

            let slot_key = (inventory_key, local_key);
            let text = context
                .item_storage
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

            let src_item = context.root.remove_item(src_slot_key);
            let dst_item = context.root.remove_item(dst_slot_key);

            if let Ok(item) = dst_item {
                let _ = context.root.insert_item(src_slot_key, item);
            }
            if let Ok(item) = src_item {
                let _ = context.root.insert_item(dst_slot_key, item);
            }
        })
    }

    #[func]
    fn draw_item(inventory_key: u32, local_key: u32, control_item: Gd<godot::classes::Control>) {
        CONTEXT.with_borrow(|context| {
            let context = context.as_ref().unwrap();

            let slot_key = (inventory_key, local_key);
            context
                .item_storage
                .draw_item(&context.root, slot_key, control_item)
                .unwrap();
        })
    }

    #[func]
    fn set_selection(location: Vector2) {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            let point = Vec2::new(location.x, location.y).floor().as_ivec2();
            let tiles = context
                .root
                .get_tile_by_point(point)
                .into_iter()
                .collect::<Vec<_>>();

            let point = Vec2::new(location.x, location.y);
            let blocks = context
                .root
                .get_block_by_hint_point(point)
                .collect::<Vec<_>>();

            let point = Vec2::new(location.x, location.y);
            let entities = context
                .root
                .get_entity_by_hint_point(point)
                .collect::<Vec<_>>();

            context
                .selection
                .update_view(&context.root, &tiles, &blocks, &entities);
        })
    }

    #[func]
    fn get_selection_size() -> u32 {
        todo!()
    }

    #[func]
    fn get_selection_name_text() -> GString {
        todo!()
    }

    #[func]
    fn clear_selection() {
        CONTEXT.with_borrow_mut(|context| {
            let context = context.as_mut().unwrap();

            context.selection.update_view(&context.root, &[], &[], &[]);
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
