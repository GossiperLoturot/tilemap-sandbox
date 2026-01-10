use criterion::*;
use glam::*;
use native_core::dataflow::*;

fn benchmark(c: &mut Criterion) {
    c.bench_function("tile add", |b| {
        b.iter_custom(|iters| {
            let mut field: TileField = TileField::new(TileFieldInfo {
                tiles: vec![
                    TileInfo {
                        display_name: "tile_0".into(),
                        description: "tile_0_desc".into(),
                        collision: true,
                    },
                    TileInfo {
                        display_name: "tile_0".into(),
                        description: "tile_0_desc".into(),
                        collision: true,
                    },
                ],
            });

            // warm up
            for i in 0..iters {
                std::hint::black_box(
                    field
                        .insert(Tile {
                            archetype_id: 0,
                            coord: IVec2::new(i as i32, 0),
                            ..Default::default()
                        })
                        .unwrap(),
                );
            }

            let instance = std::time::Instant::now();
            for i in 0..iters {
                let tile = std::hint::black_box(Tile {
                    archetype_id: 0,
                    coord: IVec2::new(i as i32, 1),
                    ..Default::default()
                });
                let result = field.insert(tile).unwrap();
                std::hint::black_box(result);
            }
            instance.elapsed()
        });
    });

    c.bench_function("tile remove", |b| {
        b.iter_custom(|iters| {
            let mut field: TileField = TileField::new(TileFieldInfo {
                tiles: vec![
                    TileInfo {
                        display_name: "tile_0".into(),
                        description: "tile_0_desc".into(),
                        collision: true,
                    },
                    TileInfo {
                        display_name: "tile_0".into(),
                        description: "tile_0_desc".into(),
                        collision: true,
                    },
                ],
            });

            let mut ids = vec![];
            for i in 0..iters {
                ids.push(
                    field
                        .insert(Tile {
                            archetype_id: 0,
                            coord: IVec2::new(i as i32, 0),
                            ..Default::default()
                        })
                        .unwrap(),
                );
            }

            let instance = std::time::Instant::now();
            for id in ids {
                let id = std::hint::black_box(id);
                let result = field.remove(id).unwrap();
                std::hint::black_box(result);
            }
            instance.elapsed()
        });
    });

    c.bench_function("tile get", |b| {
        b.iter_custom(|iters| {
            let mut field: TileField = TileField::new(TileFieldInfo {
                tiles: vec![
                    TileInfo {
                        display_name: "tile_0".into(),
                        description: "tile_0_desc".into(),
                        collision: true,
                    },
                    TileInfo {
                        display_name: "tile_0".into(),
                        description: "tile_0_desc".into(),
                        collision: true,
                    },
                ],
            });

            let mut ids = vec![];
            for i in 0..iters {
                ids.push(
                    field
                        .insert(Tile {
                            archetype_id: 0,
                            coord: IVec2::new(i as i32, 0),
                            ..Default::default()
                        })
                        .unwrap(),
                );
            }

            let instance = std::time::Instant::now();
            for id in ids {
                let id = std::hint::black_box(id);
                let result = field.get(id).unwrap();
                std::hint::black_box(result);
            }
            instance.elapsed()
        });
    });

    c.bench_function("tile modify", |b| {
        b.iter_custom(|iters| {
            let mut field: TileField = TileField::new(TileFieldInfo {
                tiles: vec![
                    TileInfo {
                        display_name: "tile_0".into(),
                        description: "tile_0_desc".into(),
                        collision: true,
                    },
                    TileInfo {
                        display_name: "tile_0".into(),
                        description: "tile_0_desc".into(),
                        collision: true,
                    },
                ],
            });

            let mut ids = vec![];
            for i in 0..iters {
                ids.push(
                    field
                        .insert(Tile {
                            archetype_id: 0,
                            coord: IVec2::new(i as i32, 0),
                            ..Default::default()
                        })
                        .unwrap(),
                );
            }

            let instance = std::time::Instant::now();
            for id in ids {
                let f = std::hint::black_box(|render_state: &mut TileRenderState| render_state.variant += 1);
                let result = field.modify(id, f).unwrap();
                std::hint::black_box(result);
            }
            instance.elapsed()
        });
    });

    let row: &[(&str, Box<dyn Fn(&TileField, usize)>)] = &[
        ("tile find point", Box::new(|field, i| {
            let query = std::hint::black_box(IVec2::new(i as i32, 0));
            let result = field.find_with_point(query);
            std::hint::black_box(result);
        })),
        ("tile find rect", Box::new(|field, i| {
            let query = std::hint::black_box([IVec2::ZERO, IVec2::new(i as i32, 0)]);
            let result = field.find_with_rect(query).count();
            std::hint::black_box(result);
        })),
        ("tile find rect first", Box::new(|field, i| {
            let query = std::hint::black_box([IVec2::ZERO, IVec2::new(i as i32, 0)]);
            let result = field.find_with_rect(query).next();
            std::hint::black_box(result);
        })),
        ("tile find collision point", Box::new(|field, i| {
            let query = std::hint::black_box(Vec2::new(i as f32, 0.0));
            let result = field.find_with_collision_point(query);
            std::hint::black_box(result);
        })),
        ("tile find collision rect", Box::new(|field, i| {
            let query = std::hint::black_box([Vec2::ZERO, Vec2::new(i as f32, 0.0)]);
            let result = field.find_with_collision_rect(query).count();
            std::hint::black_box(result);
        })),
        ("tile find collision rect first", Box::new(|field, i| {
            let query = std::hint::black_box([Vec2::ZERO, Vec2::new(i as f32, 0.0)]);
            let result = field.find_with_collision_rect(query).next();
            std::hint::black_box(result);
        })),
    ];
    for (name, f) in row {
        c.bench_function(name, |b| {
            b.iter_custom(|iters| {
                let mut field: TileField = TileField::new(TileFieldInfo {
                    tiles: vec![
                        TileInfo {
                            display_name: "tile_0".into(),
                            description: "tile_0_desc".into(),
                            collision: true,
                        },
                        TileInfo {
                            display_name: "tile_0".into(),
                            description: "tile_0_desc".into(),
                            collision: true,
                        },
                    ],
                });

                let mut ids = vec![];
                for i in 0..iters {
                    ids.push(
                        field
                            .insert(Tile {
                                archetype_id: 0,
                                coord: IVec2::new(i as i32, 0),
                                ..Default::default()
                            })
                            .unwrap(),
                    );
                }

                let instance = std::time::Instant::now();
                for i in 0..iters {
                    f(&field, i as usize);
                }
                instance.elapsed()
            });
        });
    }

    c.bench_function("block add", |b| {
        b.iter_custom(|iters| {
            let mut field: BlockField = BlockField::new(BlockFieldInfo {
                blocks: vec![
                    BlockInfo {
                        display_name: "block_0".into(),
                        description: "block_0_desc".into(),
                        size: IVec2::new(1, 1),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        y_sorting: false,
                    },
                    BlockInfo {
                        display_name: "block_1".into(),
                        description: "block_1_desc".into(),
                        size: IVec2::new(1, 1),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        y_sorting: false,
                    },
                ],
            });

            let instance = std::time::Instant::now();
            for i in 0..iters {
                std::hint::black_box(
                    field
                        .insert(Block {
                            archetype_id: 0,
                            coord: IVec2::new(i as i32, 0),
                            data: Default::default(),
                            render_state: Default::default(),
                        })
                        .unwrap(),
                );
            }
            instance.elapsed()
        });
    });

    c.bench_function("block remove", |b| {
        b.iter_custom(|iters| {
            let mut field: BlockField = BlockField::new(BlockFieldInfo {
                blocks: vec![
                    BlockInfo {
                        display_name: "block_0".into(),
                        description: "block_0_desc".into(),
                        size: IVec2::new(1, 1),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        y_sorting: false,
                    },
                    BlockInfo {
                        display_name: "block_1".into(),
                        description: "block_1_desc".into(),
                        size: IVec2::new(1, 1),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        y_sorting: false,
                    },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(
                    field
                        .insert(Block {
                            archetype_id: 0,
                            coord: IVec2::new(i as i32, 0),
                            data: Default::default(),
                            render_state: Default::default(),
                        })
                        .unwrap(),
                );
            }

            let instance = std::time::Instant::now();
            for key in keys {
                std::hint::black_box(field.remove(key).unwrap());
            }
            instance.elapsed()
        });
    });

    c.bench_function("block get", |b| {
        b.iter_custom(|iters| {
            let mut field: BlockField = BlockField::new(BlockFieldInfo {
                blocks: vec![
                    BlockInfo {
                        display_name: "block_0".into(),
                        description: "block_0_desc".into(),
                        size: IVec2::new(1, 1),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        y_sorting: false,
                    },
                    BlockInfo {
                        display_name: "block_1".into(),
                        description: "block_1_desc".into(),
                        size: IVec2::new(1, 1),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        y_sorting: false,
                    },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(
                    field
                        .insert(Block {
                            archetype_id: 0,
                            coord: IVec2::new(i as i32, 0),
                            data: Default::default(),
                            render_state: Default::default(),
                        })
                        .unwrap(),
                );
            }

            let instance = std::time::Instant::now();
            for key in keys {
                std::hint::black_box(field.get(key).unwrap());
            }
            instance.elapsed()
        });
    });

    c.bench_function("block modify", |b| {
        b.iter_custom(|iters| {
            let mut field: BlockField = BlockField::new(BlockFieldInfo {
                blocks: vec![
                    BlockInfo {
                        display_name: "block_0".into(),
                        description: "block_0_desc".into(),
                        size: IVec2::new(1, 1),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        y_sorting: false,
                    },
                    BlockInfo {
                        display_name: "block_1".into(),
                        description: "block_1_desc".into(),
                        size: IVec2::new(1, 1),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        y_sorting: false,
                    },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(
                    field
                        .insert(Block {
                            archetype_id: 0,
                            coord: IVec2::new(i as i32, 0),
                            data: Default::default(),
                            render_state: Default::default(),
                        })
                        .unwrap(),
                );
            }

            let instance = std::time::Instant::now();
            for key in keys {
                std::hint::black_box(field.modify(key, |block| block.coord[1] += 1).unwrap());
            }
            instance.elapsed()
        });
    });

    c.bench_function("entity add", |b| {
        b.iter_custom(|iters| {
            let mut field: EntityField = EntityField::new(EntityFieldInfo {
                entities: vec![
                    EntityInfo {
                        display_name: "entity_0".into(),
                        description: "entity_0_desc".into(),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        y_sorting: false,
                    },
                    EntityInfo {
                        display_name: "entity_1".into(),
                        description: "entity_1_desc".into(),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        y_sorting: false,
                    },
                ],
            });

            let instance = std::time::Instant::now();
            for i in 0..iters {
                std::hint::black_box(
                    field
                        .insert(Entity {
                            archetype_id: 0,
                            coord: Vec2::new(i as f32, 0.0),
                            data: Default::default(),
                            render_state: Default::default(),
                        })
                        .unwrap(),
                );
            }
            instance.elapsed()
        });
    });

    c.bench_function("entity remove", |b| {
        b.iter_custom(|iters| {
            let mut field: EntityField = EntityField::new(EntityFieldInfo {
                entities: vec![
                    EntityInfo {
                        display_name: "entity_0".into(),
                        description: "entity_0_desc".into(),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        y_sorting: false,
                    },
                    EntityInfo {
                        display_name: "entity_1".into(),
                        description: "entity_1_desc".into(),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        y_sorting: false,
                    },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(
                    field
                        .insert(Entity {
                            archetype_id: 0,
                            coord: Vec2::new(i as f32, 0.0),
                            data: Default::default(),
                            render_state: Default::default(),
                        })
                        .unwrap(),
                );
            }

            let instance = std::time::Instant::now();
            for key in keys {
                std::hint::black_box(field.remove(key).unwrap());
            }
            instance.elapsed()
        });
    });

    c.bench_function("entity get", |b| {
        b.iter_custom(|iters| {
            let mut field: EntityField = EntityField::new(EntityFieldInfo {
                entities: vec![
                    EntityInfo {
                        display_name: "entity_0".into(),
                        description: "entity_0_desc".into(),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        y_sorting: false,
                    },
                    EntityInfo {
                        display_name: "entity_1".into(),
                        description: "entity_1_desc".into(),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        y_sorting: false,
                    },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(
                    field
                        .insert(Entity {
                            archetype_id: 0,
                            coord: Vec2::new(i as f32, 0.0),
                            data: Default::default(),
                            render_state: Default::default(),
                        })
                        .unwrap(),
                );
            }

            let instance = std::time::Instant::now();
            for key in keys {
                std::hint::black_box(field.get(key).unwrap());
            }
            instance.elapsed()
        });
    });

    c.bench_function("entity modify", |b| {
        b.iter_custom(|iters| {
            let mut field: EntityField = EntityField::new(EntityFieldInfo {
                entities: vec![
                    EntityInfo {
                        display_name: "entity_0".into(),
                        description: "entity_0_desc".into(),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        y_sorting: false,
                    },
                    EntityInfo {
                        display_name: "entity_1".into(),
                        description: "entity_1_desc".into(),
                        collision_size: Vec2::new(1.0, 1.0),
                        collision_offset: Vec2::new(0.0, 0.0),
                        hint_size: Vec2::new(1.0, 1.0),
                        hint_offset: Vec2::new(0.0, 0.0),
                        y_sorting: false,
                    },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(
                    field
                        .insert(Entity {
                            archetype_id: 0,
                            coord: Vec2::new(i as f32, 0.0),
                            data: Default::default(),
                            render_state: Default::default(),
                        })
                        .unwrap(),
                );
            }

            let instance = std::time::Instant::now();
            for key in keys {
                std::hint::black_box(
                    field
                        .modify(key, |entity| entity.coord[1] += 1.0)
                        .unwrap(),
                );
            }
            instance.elapsed()
        });
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
