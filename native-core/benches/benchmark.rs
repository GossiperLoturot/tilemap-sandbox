use criterion::*;
use glam::*;

use native_core::*;

fn make_tile_field() -> dataflow::TileField {
    dataflow::TileField::new(dataflow::TileFieldInfo {
        tiles: vec![
            dataflow::TileInfo {
                display_name: "tile_0".into(),
                description: "tile_0_desc".into(),
                collision: true,
            },
            dataflow::TileInfo {
                display_name: "tile_1".into(),
                description: "tile_1_desc".into(),
                collision: true,
            },
        ],
    })
}

fn benchmark_tile(c: &mut Criterion) {
    c.bench_function("tile add", |b| {
        b.iter_custom(|iters| {
            let mut field = make_tile_field();

            // warm up
            for i in 0..iters {
                let _ = field
                    .insert(dataflow::Tile {
                        archetype_id: 0,
                        coord: IVec2::new(i as i32, 0),
                        ..Default::default()
                    })
                    .unwrap();
            }

            let instance = std::time::Instant::now();
            for i in 0..iters {
                let tile = std::hint::black_box(dataflow::Tile {
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
            let mut field = make_tile_field();

            let mut ids = vec![];
            for i in 0..iters {
                let id = field
                    .insert(dataflow::Tile {
                        archetype_id: 0,
                        coord: IVec2::new(i as i32, 0),
                        ..Default::default()
                    })
                    .unwrap();
                ids.push(id);
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
            let mut field = make_tile_field();

            let mut ids = vec![];
            for i in 0..iters {
                let id = field
                    .insert(dataflow::Tile {
                        archetype_id: 0,
                        coord: IVec2::new(i as i32, 0),
                        ..Default::default()
                    })
                    .unwrap();
                ids.push(id);
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
            let mut field = make_tile_field();

            let mut ids = vec![];
            for i in 0..iters {
                let id = field
                    .insert(dataflow::Tile {
                        archetype_id: 0,
                        coord: IVec2::new(i as i32, 0),
                        ..Default::default()
                    })
                    .unwrap();
                ids.push(id);
            }

            let instance = std::time::Instant::now();
            for id in ids {
                let f = std::hint::black_box(|tile_modify: &mut dataflow::TileModify| tile_modify.variant += 1);
                let result = field.modify(id, f).unwrap();
                std::hint::black_box(result);
            }
            instance.elapsed()
        });
    });

    let row: &[(&str, Box<dyn Fn(&dataflow::TileField, usize)>)] = &[
        ("tile find point", Box::new(|field, i| {
            let query = std::hint::black_box(IVec2::new(i as i32, 0));
            let result = field.find_with_point(query);
            std::hint::black_box(result);
        })),
        ("tile find rect", Box::new(|field, i| {
            let query = std::hint::black_box(IRect2::new(IVec2::ZERO, IVec2::new(i as i32, 0)));
            let result = field.find_with_rect(query).count();
            std::hint::black_box(result);
        })),
        ("tile find collision point", Box::new(|field, i| {
            let query = std::hint::black_box(Vec2::new(i as f32, 0.0));
            let result = field.find_with_collision_point(query).count();
            std::hint::black_box(result);
        })),
        ("tile find collision rect", Box::new(|field, i| {
            let query = std::hint::black_box(Rect2::new(Vec2::ZERO, Vec2::new(i as f32, 0.0)));
            let result = field.find_with_collision_rect(query).count();
            std::hint::black_box(result);
        })),
    ];
    for (name, f) in row {
        c.bench_function(name, |b| {
            b.iter_custom(|iters| {
                let mut field = make_tile_field();

                let mut ids = vec![];
                for i in 0..iters {
                    let id = field
                        .insert(dataflow::Tile {
                            archetype_id: 0,
                            coord: IVec2::new(i as i32, 0),
                            ..Default::default()
                        })
                        .unwrap();
                    ids.push(id);
                }

                let instance = std::time::Instant::now();
                for i in 0..iters {
                    f(&field, i as usize);
                }
                instance.elapsed()
            });
        });
    }
}

fn make_block_field() -> dataflow::BlockField {
    dataflow::BlockField::new(dataflow::BlockFieldInfo {
        blocks: vec![
            dataflow::BlockInfo {
                display_name: "block_0".into(),
                description: "block_0_desc".into(),
                size: IVec2::new(1, 1),
                collision_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                y_sorting: false,
            },
            dataflow::BlockInfo {
                display_name: "block_1".into(),
                description: "block_1_desc".into(),
                size: IVec2::new(1, 1),
                collision_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                y_sorting: false,
            },
        ],
    })
}

fn benchmark_block(c: &mut Criterion) {
    c.bench_function("block add", |b| {
        b.iter_custom(|iters| {
            let mut field = make_block_field();

            // warm up
            for i in 0..iters {
                let _ = field
                    .insert(dataflow::Block {
                        archetype_id: 0,
                        coord: IVec2::new(i as i32, 0),
                        ..Default::default()
                    })
                    .unwrap();
            }

            let instance = std::time::Instant::now();
            for i in 0..iters {
                let tile = std::hint::black_box(dataflow::Block {
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

    c.bench_function("block remove", |b| {
        b.iter_custom(|iters| {
            let mut field = make_block_field();

            let mut ids = vec![];
            for i in 0..iters {
                let id = field
                    .insert(dataflow::Block {
                        archetype_id: 0,
                        coord: IVec2::new(i as i32, 0),
                        ..Default::default()
                    })
                    .unwrap();
                ids.push(id);
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

    c.bench_function("block get", |b| {
        b.iter_custom(|iters| {
            let mut field = make_block_field();

            let mut ids = vec![];
            for i in 0..iters {
                let id = field
                    .insert(dataflow::Block {
                        archetype_id: 0,
                        coord: IVec2::new(i as i32, 0),
                        ..Default::default()
                    })
                    .unwrap();
                ids.push(id);
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

    c.bench_function("block modify", |b| {
        b.iter_custom(|iters| {
            let mut field = make_block_field();

            let mut ids = vec![];
            for i in 0..iters {
                let id = field
                    .insert(dataflow::Block {
                        archetype_id: 0,
                        coord: IVec2::new(i as i32, 0),
                        ..Default::default()
                    })
                    .unwrap();
                ids.push(id);
            }

            let instance = std::time::Instant::now();
            for id in ids {
                let f = std::hint::black_box(|block_modify: &mut dataflow::BlockModify| block_modify.variant += 1);
                let result = field.modify(id, f).unwrap();
                std::hint::black_box(result);
            }
            instance.elapsed()
        });
    });

    let row: &[(&str, Box<dyn Fn(&dataflow::BlockField, usize)>)] = &[
        ("block find point", Box::new(|field, i| {
            let query = std::hint::black_box(IVec2::new(i as i32, 0));
            let result = field.find_with_point(query);
            std::hint::black_box(result);
        })),
        ("block find rect", Box::new(|field, i| {
            let query = std::hint::black_box(IRect2::new(IVec2::ZERO, IVec2::new(i as i32, 0)));
            let result = field.find_with_rect(query).count();
            std::hint::black_box(result);
        })),
        ("block find collision point", Box::new(|field, i| {
            let query = std::hint::black_box(Vec2::new(i as f32, 0.0));
            let result = field.find_with_collision_point(query).count();
            std::hint::black_box(result);
        })),
        ("block find collision rect", Box::new(|field, i| {
            let query = std::hint::black_box(Rect2::new(Vec2::ZERO, Vec2::new(i as f32, 0.0)));
            let result = field.find_with_collision_rect(query).count();
            std::hint::black_box(result);
        })),
    ];
    for (name, f) in row {
        c.bench_function(name, |b| {
            b.iter_custom(|iters| {
                let mut field = make_block_field();

                let mut ids = vec![];
                for i in 0..iters {
                    let id = field
                        .insert(dataflow::Block {
                            archetype_id: 0,
                            coord: IVec2::new(i as i32, 0),
                            ..Default::default()
                        })
                        .unwrap();
                    ids.push(id);
                }

                let instance = std::time::Instant::now();
                for i in 0..iters {
                    f(&field, i as usize);
                }
                instance.elapsed()
            });
        });
    }
}

fn make_entity_field() -> dataflow::EntityField {
    dataflow::EntityField::new(dataflow::EntityFieldInfo {
        entities: vec![
            dataflow::EntityInfo {
                display_name: "entity_0".into(),
                description: "entity_0_desc".into(),
                collision_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                y_sorting: false,
            },
            dataflow::EntityInfo {
                display_name: "entity_1".into(),
                description: "entity_1_desc".into(),
                collision_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                y_sorting: false,
            },
        ],
    })
}

fn benchmark_entity(c: &mut Criterion) {
    c.bench_function("entity add", |b| {
        b.iter_custom(|iters| {
            let mut field = make_entity_field();

            let instance = std::time::Instant::now();
            for i in 0..iters {
                std::hint::black_box(
                    field
                        .insert(dataflow::Entity {
                            archetype_id: 0,
                            coord: Vec2::new(i as f32, 0.0),
                            ..Default::default()
                        })
                        .unwrap(),
                );
            }
            instance.elapsed()
        });
    });

    c.bench_function("entity remove", |b| {
        b.iter_custom(|iters| {
            let mut field = make_entity_field();

            let mut ids = vec![];
            for i in 0..iters {
                let id = field
                    .insert(dataflow::Entity {
                        archetype_id: 0,
                        coord: Vec2::new(i as f32, 0.0),
                        ..Default::default()
                    })
                    .unwrap();
                ids.push(id);
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

    c.bench_function("entity get", |b| {
        b.iter_custom(|iters| {
            let mut field = make_entity_field();

            let mut ids = vec![];
            for i in 0..iters {
                let id = field
                    .insert(dataflow::Entity {
                        archetype_id: 0,
                        coord: Vec2::new(i as f32, 0.0),
                        ..Default::default()
                    })
                    .unwrap();
                ids.push(id);
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

    c.bench_function("entity modify", |b| {
        b.iter_custom(|iters| {
            let mut field = make_entity_field();

            let mut ids = vec![];
            for i in 0..iters {
                let id = field
                    .insert(dataflow::Entity {
                        archetype_id: 0,
                        coord: Vec2::new(i as f32, 0.0),
                        ..Default::default()
                    })
                    .unwrap();
                ids.push(id);
            }

            let instance = std::time::Instant::now();
            for id in ids {
                let f = std::hint::black_box(|entity_modify: &mut dataflow::EntityModify| entity_modify.variant += 1);
                let result = field.modify(id, f).unwrap();
                std::hint::black_box(result);
            }
            instance.elapsed()
        });
    });

    let row: &[(&str, Box<dyn Fn(&dataflow::EntityField, usize)>)] = &[
        ("entity find collision point", Box::new(|field, i| {
            let query = std::hint::black_box(Vec2::new(i as f32, 0.0));
            let result = field.find_with_collision_point(query).count();
            std::hint::black_box(result);
        })),
        ("entity find collision rect", Box::new(|field, i| {
            let query = std::hint::black_box(Rect2::new(Vec2::ZERO, Vec2::new(i as f32, 0.0)));
            let result = field.find_with_collision_rect(query).count();
            std::hint::black_box(result);
        })),
        ("entity find hint point", Box::new(|field, i| {
            let query = std::hint::black_box(Vec2::new(i as f32, 0.0));
            let result = field.find_with_hint_point(query).count();
            std::hint::black_box(result);
        })),
        ("entity find hint rect", Box::new(|field, i| {
            let query = std::hint::black_box(Rect2::new(Vec2::ZERO, Vec2::new(i as f32, 0.0)));
            let result = field.find_with_hint_rect(query).count();
            std::hint::black_box(result);
        })),
    ];
    for (name, f) in row {
        c.bench_function(name, |b| {
            b.iter_custom(|iters| {
                let mut field = make_entity_field();

                let mut ids = vec![];
                for i in 0..iters {
                    let id = field
                        .insert(dataflow::Entity {
                            archetype_id: 0,
                            coord: Vec2::new(i as f32, 0.0),
                            ..Default::default()
                        })
                        .unwrap();
                    ids.push(id);
                }

                let instance = std::time::Instant::now();
                for i in 0..iters {
                    f(&field, i as usize);
                }
                instance.elapsed()
            });
        });
    }
}

criterion_group!(benches, benchmark_tile, benchmark_block, benchmark_entity);
criterion_main!(benches);
