use glam::*;
use godot::prelude::*;

pub mod inner;

mod block;
mod entity;
mod item;
mod reg;
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
    context: reg::Context,
}

#[godot_api]
impl Root {
    #[func]
    fn create(world: Gd<godot::classes::World3D>) -> Gd<Self> {
        let mut builder = reg::ContextBuilder::<Registry>::new();

        // tiles
        let tile_dirt = builder.add_tile(|_| {
            reg::TileDescripter::single(
                reg::ImageDescriptor::single("res://images/surface_dirt.webp"),
                false,
                (),
            )
        });
        let tile_grass = builder.add_tile(|_| {
            reg::TileDescripter::single(
                reg::ImageDescriptor::single("res://images/surface_grass.webp"),
                false,
                (),
            )
        });

        // blocks
        let block_dandelion = builder.add_block(|_| {
            reg::BlockDescripter::single(
                reg::ImageDescriptor::single("res://images/dandelion.webp"),
                false,
                IVec2::new(1, 1),
                [Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)],
                [Vec2::new(1.0, 1.0), Vec2::new(0.0, 0.0)],
                (),
            )
        });
        let block_fallen_leaves = builder.add_block(|_| {
            reg::BlockDescripter::single(
                reg::ImageDescriptor::single("res://images/fallen_leaves.webp"),
                false,
                IVec2::new(1, 1),
                [Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)],
                [Vec2::new(1.0, 1.0), Vec2::new(0.0, 0.0)],
                (),
            )
        });
        let block_mix_grass = builder.add_block(|_| {
            reg::BlockDescripter::single(
                reg::ImageDescriptor::single("res://images/mix_grass.webp"),
                false,
                IVec2::new(1, 1),
                [Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)],
                [Vec2::new(1.0, 1.0), Vec2::new(0.0, 0.0)],
                (),
            )
        });
        let block_mix_pebbles = builder.add_block(|_| {
            reg::BlockDescripter::single(
                reg::ImageDescriptor::single("res://images/mix_pebbles.webp"),
                false,
                IVec2::new(1, 1),
                [Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)],
                [Vec2::new(1.0, 1.0), Vec2::new(0.0, 0.0)],
                (),
            )
        });

        // entities
        let entity_player = builder.add_entity(|_| {
            reg::EntityDescripter::new(
                vec![
                    reg::ImageDescriptor::new(
                        vec![
                            "res://images/player_idle_0.webp",
                            "res://images/player_idle_1.webp",
                        ],
                        24,
                        true,
                    ),
                    reg::ImageDescriptor::new(
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
                [Vec2::new(0.8, 0.8), Vec2::new(-0.4, 0.1)],
                [Vec2::new(1.5, 2.25), Vec2::new(-0.75, 0.0)],
                inner::PlayerEntityFeature,
            )
        });
        let entity_pig = builder.add_entity(|_| {
            reg::EntityDescripter::new(
                vec![
                    reg::ImageDescriptor::new(
                        vec![
                            "res://images/pig_idle_0.webp",
                            "res://images/pig_idle_1.webp",
                        ],
                        24,
                        true,
                    ),
                    reg::ImageDescriptor::new(
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
                [Vec2::new(0.8, 0.8), Vec2::new(-0.4, 0.1)],
                [Vec2::new(2.0, 2.0), Vec2::new(-1.0, 0.0)],
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
            reg::EntityDescripter::new(
                vec![
                    reg::ImageDescriptor::new(
                        vec![
                            "res://images/cow_idle_0.webp",
                            "res://images/cow_idle_1.webp",
                        ],
                        24,
                        true,
                    ),
                    reg::ImageDescriptor::new(
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
                [Vec2::new(0.8, 0.8), Vec2::new(-0.4, 0.1)],
                [Vec2::new(2.0, 2.0), Vec2::new(-1.0, 0.0)],
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
            reg::EntityDescripter::new(
                vec![
                    reg::ImageDescriptor::new(
                        vec![
                            "res://images/sheep_idle_0.webp",
                            "res://images/sheep_idle_1.webp",
                        ],
                        24,
                        true,
                    ),
                    reg::ImageDescriptor::new(
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
                [Vec2::new(0.8, 0.8), Vec2::new(-0.4, 0.1)],
                [Vec2::new(2.0, 2.0), Vec2::new(-1.0, 0.0)],
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
            reg::EntityDescripter::new(
                vec![
                    reg::ImageDescriptor::single("res://images/chicken_idle.webp"),
                    reg::ImageDescriptor::new(
                        vec![
                            "res://images/chicken_walk.webp",
                            "res://images/chicken_idle.webp",
                        ],
                        12,
                        true,
                    ),
                ],
                true,
                [Vec2::new(0.8, 0.8), Vec2::new(-0.4, 0.1)],
                [Vec2::new(1.0, 1.0), Vec2::new(-0.5, 0.0)],
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
            reg::EntityDescripter::new(
                vec![
                    reg::ImageDescriptor::single("res://images/bird_idle.webp"),
                    reg::ImageDescriptor::new(
                        vec![
                            "res://images/chicken_walk.webp",
                            "res://images/chicken_idle.webp",
                        ],
                        12,
                        true,
                    ),
                ],
                true,
                [Vec2::new(0.8, 0.8), Vec2::new(-0.4, 0.1)],
                [Vec2::new(1.0, 1.0), Vec2::new(-0.5, 0.0)],
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
            reg::ItemDescriptor::new(
                "Package",
                "A package of items.",
                reg::ImageDescriptor::single("res://images/package.webp"),
                (),
            )
        });

        // gen rule
        builder.add_gen_rule(|reg| {
            let id = reg.tile_grass;
            reg::GenRuleDescriptor::March(reg::MarchGenRuleDescriptor::new(
                0.5,
                move |root, location| {
                    let tile = inner::Tile {
                        id,
                        location,
                        data: Default::default(),
                        render_param: Default::default(),
                    };
                    let _ = root.tile_insert(tile);
                },
            ))
        });
        builder.add_gen_rule(|reg| {
            let id = reg.tile_dirt;
            reg::GenRuleDescriptor::March(reg::MarchGenRuleDescriptor::new(
                1.0,
                move |root, location| {
                    let tile = inner::Tile {
                        id,
                        location,
                        data: Default::default(),
                        render_param: Default::default(),
                    };
                    let _ = root.tile_insert(tile);
                },
            ))
        });
        builder.add_gen_rule(|reg| {
            let id = reg.tile_dirt;
            reg::GenRuleDescriptor::Spawn(reg::SpawnGenRuleDescriptor::new(
                0.05,
                move |root, location| {
                    let tile = inner::Block {
                        id,
                        location: location.as_ivec2(),
                        data: Default::default(),
                        render_param: Default::default(),
                    };
                    let _ = root.block_insert(tile);
                },
            ))
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
        let desc = reg::BuildDescriptor {
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
        let context = builder.build(&register, &desc);
        Gd::from_object(Self { context })
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
            id: 0,
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
