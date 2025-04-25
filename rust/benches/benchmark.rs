use criterion::*;
use glam::*;
use tilemap_sandbox::inner::*;

fn benchmark(c: &mut Criterion) {
    c.bench_function("tile add", |b| {
        b.iter_custom(|iters| {
            let mut field: TileField = TileField::new(TileFieldDescriptor {
                tiles: vec![
                    TileDescriptor {
                        name_text: "tile_0".into(),
                        desc_text: "tile_0_desc".into(),
                        collision: true,
                    },
                    TileDescriptor {
                        name_text: "tile_0".into(),
                        desc_text: "tile_0_desc".into(),
                        collision: true,
                    },
                ],
            });

            let instance = std::time::Instant::now();
            for i in 0..iters {
                black_box(
                    field
                        .insert(Tile {
                            id: 0,
                            location: IVec2::new(i as i32, 0),
                            data: Default::default(),
                            render_param: Default::default(),
                        })
                        .unwrap(),
                );
            }
            instance.elapsed()
        });
    });

    c.bench_function("tile remove", |b| {
        b.iter_custom(|iters| {
            let mut field: TileField = TileField::new(TileFieldDescriptor {
                tiles: vec![
                    TileDescriptor {
                        name_text: "tile_0".into(),
                        desc_text: "tile_0_desc".into(),
                        collision: true,
                    },
                    TileDescriptor {
                        name_text: "tile_0".into(),
                        desc_text: "tile_0_desc".into(),
                        collision: true,
                    },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(
                    field
                        .insert(Tile {
                            id: 0,
                            location: IVec2::new(i as i32, 0),
                            data: Default::default(),
                            render_param: Default::default(),
                        })
                        .unwrap(),
                );
            }

            let instance = std::time::Instant::now();
            for key in keys {
                black_box(field.remove(key).unwrap());
            }
            instance.elapsed()
        });
    });

    c.bench_function("tile get", |b| {
        b.iter_custom(|iters| {
            let mut field: TileField = TileField::new(TileFieldDescriptor {
                tiles: vec![
                    TileDescriptor {
                        name_text: "tile_0".into(),
                        desc_text: "tile_0_desc".into(),
                        collision: true,
                    },
                    TileDescriptor {
                        name_text: "tile_0".into(),
                        desc_text: "tile_0_desc".into(),
                        collision: true,
                    },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(
                    field
                        .insert(Tile {
                            id: 0,
                            location: IVec2::new(i as i32, 0),
                            data: Default::default(),
                            render_param: Default::default(),
                        })
                        .unwrap(),
                );
            }

            let instance = std::time::Instant::now();
            for key in keys {
                black_box(field.get(key).unwrap());
            }
            instance.elapsed()
        });
    });

    c.bench_function("tile modify", |b| {
        b.iter_custom(|iters| {
            let mut field: TileField = TileField::new(TileFieldDescriptor {
                tiles: vec![
                    TileDescriptor {
                        name_text: "tile_0".into(),
                        desc_text: "tile_0_desc".into(),
                        collision: true,
                    },
                    TileDescriptor {
                        name_text: "tile_0".into(),
                        desc_text: "tile_0_desc".into(),
                        collision: true,
                    },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(
                    field
                        .insert(Tile {
                            id: 0,
                            location: IVec2::new(i as i32, 0),
                            data: Default::default(),
                            render_param: Default::default(),
                        })
                        .unwrap(),
                );
            }

            let instance = std::time::Instant::now();
            for key in keys {
                black_box(field.modify(key, |tile| tile.location[1] += 1).unwrap());
            }
            instance.elapsed()
        });
    });

    c.bench_function("block add", |b| {
        b.iter_custom(|iters| {
            let mut field: BlockField = BlockField::new(BlockFieldDescriptor {
                blocks: vec![
                    BlockDescriptor {
                        name_text: "block_0".into(),
                        desc_text: "block_0_desc".into(),
                        size: IVec2::new(1, 1),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        z_along_y: false,
                    },
                    BlockDescriptor {
                        name_text: "block_1".into(),
                        desc_text: "block_1_desc".into(),
                        size: IVec2::new(1, 1),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        z_along_y: false,
                    },
                ],
            });

            let instance = std::time::Instant::now();
            for i in 0..iters {
                black_box(
                    field
                        .insert(Block {
                            id: 0,
                            location: IVec2::new(i as i32, 0),
                            data: Default::default(),
                            render_param: Default::default(),
                        })
                        .unwrap(),
                );
            }
            instance.elapsed()
        });
    });

    c.bench_function("block remove", |b| {
        b.iter_custom(|iters| {
            let mut field: BlockField = BlockField::new(BlockFieldDescriptor {
                blocks: vec![
                    BlockDescriptor {
                        name_text: "block_0".into(),
                        desc_text: "block_0_desc".into(),
                        size: IVec2::new(1, 1),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        z_along_y: false,
                    },
                    BlockDescriptor {
                        name_text: "block_1".into(),
                        desc_text: "block_1_desc".into(),
                        size: IVec2::new(1, 1),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        z_along_y: false,
                    },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(
                    field
                        .insert(Block {
                            id: 0,
                            location: IVec2::new(i as i32, 0),
                            data: Default::default(),
                            render_param: Default::default(),
                        })
                        .unwrap(),
                );
            }

            let instance = std::time::Instant::now();
            for key in keys {
                black_box(field.remove(key).unwrap());
            }
            instance.elapsed()
        });
    });

    c.bench_function("block get", |b| {
        b.iter_custom(|iters| {
            let mut field: BlockField = BlockField::new(BlockFieldDescriptor {
                blocks: vec![
                    BlockDescriptor {
                        name_text: "block_0".into(),
                        desc_text: "block_0_desc".into(),
                        size: IVec2::new(1, 1),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        z_along_y: false,
                    },
                    BlockDescriptor {
                        name_text: "block_1".into(),
                        desc_text: "block_1_desc".into(),
                        size: IVec2::new(1, 1),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        z_along_y: false,
                    },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(
                    field
                        .insert(Block {
                            id: 0,
                            location: IVec2::new(i as i32, 0),
                            data: Default::default(),
                            render_param: Default::default(),
                        })
                        .unwrap(),
                );
            }

            let instance = std::time::Instant::now();
            for key in keys {
                black_box(field.get(key).unwrap());
            }
            instance.elapsed()
        });
    });

    c.bench_function("block modify", |b| {
        b.iter_custom(|iters| {
            let mut field: BlockField = BlockField::new(BlockFieldDescriptor {
                blocks: vec![
                    BlockDescriptor {
                        name_text: "block_0".into(),
                        desc_text: "block_0_desc".into(),
                        size: IVec2::new(1, 1),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        z_along_y: false,
                    },
                    BlockDescriptor {
                        name_text: "block_1".into(),
                        desc_text: "block_1_desc".into(),
                        size: IVec2::new(1, 1),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        z_along_y: false,
                    },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(
                    field
                        .insert(Block {
                            id: 0,
                            location: IVec2::new(i as i32, 0),
                            data: Default::default(),
                            render_param: Default::default(),
                        })
                        .unwrap(),
                );
            }

            let instance = std::time::Instant::now();
            for key in keys {
                black_box(field.modify(key, |block| block.location[1] += 1).unwrap());
            }
            instance.elapsed()
        });
    });

    c.bench_function("entity add", |b| {
        b.iter_custom(|iters| {
            let mut field: EntityField = EntityField::new(EntityFieldDescriptor {
                entities: vec![
                    EntityDescriptor {
                        name_text: "entity_0".into(),
                        desc_text: "entity_0_desc".into(),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        z_along_y: false,
                    },
                    EntityDescriptor {
                        name_text: "entity_1".into(),
                        desc_text: "entity_1_desc".into(),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        z_along_y: false,
                    },
                ],
            });

            let instance = std::time::Instant::now();
            for i in 0..iters {
                black_box(
                    field
                        .insert(Entity {
                            id: 0,
                            location: Vec2::new(i as f32, 0.0),
                            data: Default::default(),
                            render_param: Default::default(),
                        })
                        .unwrap(),
                );
            }
            instance.elapsed()
        });
    });

    c.bench_function("entity remove", |b| {
        b.iter_custom(|iters| {
            let mut field: EntityField = EntityField::new(EntityFieldDescriptor {
                entities: vec![
                    EntityDescriptor {
                        name_text: "entity_0".into(),
                        desc_text: "entity_0_desc".into(),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        z_along_y: false,
                    },
                    EntityDescriptor {
                        name_text: "entity_1".into(),
                        desc_text: "entity_1_desc".into(),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        z_along_y: false,
                    },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(
                    field
                        .insert(Entity {
                            id: 0,
                            location: Vec2::new(i as f32, 0.0),
                            data: Default::default(),
                            render_param: Default::default(),
                        })
                        .unwrap(),
                );
            }

            let instance = std::time::Instant::now();
            for key in keys {
                black_box(field.remove(key).unwrap());
            }
            instance.elapsed()
        });
    });

    c.bench_function("entity get", |b| {
        b.iter_custom(|iters| {
            let mut field: EntityField = EntityField::new(EntityFieldDescriptor {
                entities: vec![
                    EntityDescriptor {
                        name_text: "entity_0".into(),
                        desc_text: "entity_0_desc".into(),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        z_along_y: false,
                    },
                    EntityDescriptor {
                        name_text: "entity_1".into(),
                        desc_text: "entity_1_desc".into(),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        z_along_y: false,
                    },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(
                    field
                        .insert(Entity {
                            id: 0,
                            location: Vec2::new(i as f32, 0.0),
                            data: Default::default(),
                            render_param: Default::default(),
                        })
                        .unwrap(),
                );
            }

            let instance = std::time::Instant::now();
            for key in keys {
                black_box(field.get(key).unwrap());
            }
            instance.elapsed()
        });
    });

    c.bench_function("entity modify", |b| {
        b.iter_custom(|iters| {
            let mut field: EntityField = EntityField::new(EntityFieldDescriptor {
                entities: vec![
                    EntityDescriptor {
                        name_text: "entity_0".into(),
                        desc_text: "entity_0_desc".into(),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        z_along_y: false,
                    },
                    EntityDescriptor {
                        name_text: "entity_1".into(),
                        desc_text: "entity_1_desc".into(),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        z_along_y: false,
                    },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(
                    field
                        .insert(Entity {
                            id: 0,
                            location: Vec2::new(i as f32, 0.0),
                            data: Default::default(),
                            render_param: Default::default(),
                        })
                        .unwrap(),
                );
            }

            let instance = std::time::Instant::now();
            for key in keys {
                black_box(
                    field
                        .modify(key, |entity| entity.location[1] += 1.0)
                        .unwrap(),
                );
            }
            instance.elapsed()
        });
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
