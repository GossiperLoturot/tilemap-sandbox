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
    fn set_hook() {
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
}

#[derive(GodotClass)]
#[class(no_init)]
struct Root {
    context: decl::Context<Registry>,
}

#[godot_api]
impl Root {
    #[func]
    fn create(world: Gd<godot::classes::World3D>) -> Gd<Self> {
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
            feature: (),
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
            feature: (),
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
            feature: (),
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
            feature: (),
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
            feature: (),
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
            feature: (),
        });

        // entities
        let entity_player = builder.add_entity(|_| decl::EntityDescriptor {
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
            feature: inner::PlayerEntityFeature,
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
            feature: inner::AnimalEntityFeature {
                min_rest_secs: 0.0,
                max_rest_secs: 10.0,
                min_distance: 0.0,
                max_distance: 10.0,
                speed: 1.0,
                idle_variant: 0,
                walk_variant: 1,
            },
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
            feature: inner::AnimalEntityFeature {
                min_rest_secs: 0.0,
                max_rest_secs: 10.0,
                min_distance: 0.0,
                max_distance: 10.0,
                speed: 1.0,
                idle_variant: 0,
                walk_variant: 1,
            },
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
            feature: inner::AnimalEntityFeature {
                min_rest_secs: 0.0,
                max_rest_secs: 10.0,
                min_distance: 0.0,
                max_distance: 10.0,
                speed: 1.0,
                idle_variant: 0,
                walk_variant: 1,
            },
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
            feature: inner::AnimalEntityFeature {
                min_rest_secs: 0.0,
                max_rest_secs: 10.0,
                min_distance: 0.0,
                max_distance: 10.0,
                speed: 1.0,
                idle_variant: 0,
                walk_variant: 1,
            },
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
            feature: inner::AnimalEntityFeature {
                min_rest_secs: 0.0,
                max_rest_secs: 10.0,
                min_distance: 0.0,
                max_distance: 10.0,
                speed: 1.0,
                idle_variant: 0,
                walk_variant: 1,
            },
        });

        // item
        let item_package = builder.add_item(|_| decl::ItemDescriptor {
            name_text: "Package".into(),
            desc_text: "A package of items.".into(),
            image: decl::ImageDescriptor {
                frames: vec!["res://images/package.webp".into()],
                step_tick: 0,
                is_loop: false,
            },
            feature: (),
        });

        // gen rule
        builder.add_gen_rule(|reg| {
            let id = reg.tile_grass;
            decl::GenRuleDescriptor::March(decl::MarchGenRuleDescriptor {
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
            })
        });
        builder.add_gen_rule(|reg| {
            let id = reg.tile_dirt;
            decl::GenRuleDescriptor::March(decl::MarchGenRuleDescriptor {
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
            })
        });
        builder.add_gen_rule(|reg| {
            let id = reg.block_dandelion;
            decl::GenRuleDescriptor::Spawn(decl::SpawnGenRuleDescriptor {
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
            })
        });
        builder.add_gen_rule(|reg| {
            let id = reg.entity_bird;
            decl::GenRuleDescriptor::Spawn(decl::SpawnGenRuleDescriptor {
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
            })
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
        };
        let context = builder.build(register, desc);
        Gd::from_object(Self { context })
    }

    #[func]
    fn tile_get_name_text(&self, location: Vector2i) -> String {
        let location = IVec2::new(location.x, location.y);
        let key = self.context.root.tile_get_by_point(location).unwrap();
        let name_text = self.context.root.tile_get_name_text(key).unwrap();
        name_text.into()
    }

    #[func]
    fn tile_get_desc_text(&self, location: Vector2i) -> String {
        let location = IVec2::new(location.x, location.y);
        let key = self.context.root.tile_get_by_point(location).unwrap();
        let name_text = self.context.root.tile_get_name_text(key).unwrap();
        name_text.into()
    }

    #[func]
    fn time_forward(&mut self, delta_secs: f32) {
        self.context.root.time_forward(delta_secs);
    }

    #[func]
    fn forwarder_exec_rect(&mut self, min_rect: Rect2, delta_secs: f32) {
        let position = Vec2::new(min_rect.position.x, min_rect.position.y);
        let size = Vec2::new(min_rect.size.x, min_rect.size.y);
        let min_rect = [position, position + size];

        self.context
            .root
            .forwarder_exec_rect(min_rect, delta_secs)
            .unwrap();
    }

    #[func]
    fn gen_exec_rect(&mut self, min_rect: Rect2) {
        let position = Vec2::new(min_rect.position.x, min_rect.position.y);
        let size = Vec2::new(min_rect.size.x, min_rect.size.y);
        let min_rect = [position, position + size];

        self.context.root.gen_exec_rect(min_rect).unwrap();
    }

    #[func]
    fn player_spawn(&mut self, location: Vector2) {
        let location = Vec2::new(location.x, location.y);
        let entity = inner::Entity {
            id: self.context.registry.entity_player,
            location,
            data: Default::default(),
            render_param: Default::default(),
        };
        self.context.root.entity_insert(entity).unwrap();
    }

    #[func]
    fn player_insert_input(&mut self, input: Vector2) {
        let input = Vec2::new(input.x, input.y);
        self.context.root.player_insert_input(input).unwrap();
    }

    #[func]
    fn player_get_current_location(&mut self) -> Vector2 {
        let location = self.context.root.player_get_current_location().unwrap();
        Vector2::new(location[0], location[1])
    }

    #[func]
    fn update_view(&mut self, min_rect: Rect2) {
        let position = Vec2::new(min_rect.position.x, min_rect.position.y);
        let size = Vec2::new(min_rect.size.x, min_rect.size.y);
        let min_rect = [position, position + size];

        self.context
            .tile_field
            .update_view(&self.context.root, min_rect);
        self.context
            .block_field
            .update_view(&self.context.root, min_rect);
        self.context
            .entity_field
            .update_view(&self.context.root, min_rect);
    }
}
