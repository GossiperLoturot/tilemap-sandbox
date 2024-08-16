use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tilemap_sandbox::inner::*;

fn benchmark(c: &mut Criterion) {
    c.bench_function("tile add", |b| {
        b.iter_custom(|iters| {
            let mut field = TileField::new(TileFieldDescriptor {
                chunk_size: 32,
                tiles: vec![
                    TileDescriptor { collision: false },
                    TileDescriptor { collision: false },
                ],
            });

            let instance = std::time::Instant::now();
            for i in 0..iters {
                black_box(field.insert(Tile::new(0, [i as i32, 0], 0)).unwrap());
            }
            instance.elapsed()
        });
    });

    c.bench_function("tile remove", |b| {
        b.iter_custom(|iters| {
            let mut field = TileField::new(TileFieldDescriptor {
                chunk_size: 32,
                tiles: vec![
                    TileDescriptor { collision: false },
                    TileDescriptor { collision: false },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(field.insert(Tile::new(0, [i as i32, 0], 0)).unwrap());
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
            let mut field = TileField::new(TileFieldDescriptor {
                chunk_size: 32,
                tiles: vec![
                    TileDescriptor { collision: false },
                    TileDescriptor { collision: false },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(field.insert(Tile::new(0, [i as i32, 0], 0)).unwrap());
            }

            let instance = std::time::Instant::now();
            for key in keys {
                black_box(field.get(key).unwrap());
            }
            instance.elapsed()
        });
    });

    c.bench_function("block add", |b| {
        b.iter_custom(|iters| {
            let mut field = BlockField::new(BlockFieldDescriptor {
                chunk_size: 32,
                blocks: vec![
                    BlockDescriptor {
                        size: [1, 1],
                        collision_size: [1.0, 1.0],
                        collision_offset: [0.0, 0.0],
                        hint_size: [1.0, 1.0],
                        hint_offset: [0.0, 0.0],
                    },
                    BlockDescriptor {
                        size: [1, 1],
                        collision_size: [1.0, 1.0],
                        collision_offset: [0.0, 0.0],
                        hint_size: [1.0, 1.0],
                        hint_offset: [0.0, 0.0],
                    },
                ],
            });

            let instance = std::time::Instant::now();
            for i in 0..iters {
                black_box(field.insert(Block::new(0, [i as i32, 0], 0)).unwrap());
            }
            instance.elapsed()
        });
    });

    c.bench_function("block remove", |b| {
        b.iter_custom(|iters| {
            let mut field = BlockField::new(BlockFieldDescriptor {
                chunk_size: 32,
                blocks: vec![
                    BlockDescriptor {
                        size: [1, 1],
                        collision_size: [1.0, 1.0],
                        collision_offset: [0.0, 0.0],
                        hint_size: [1.0, 1.0],
                        hint_offset: [0.0, 0.0],
                    },
                    BlockDescriptor {
                        size: [1, 1],
                        collision_size: [1.0, 1.0],
                        collision_offset: [0.0, 0.0],
                        hint_size: [1.0, 1.0],
                        hint_offset: [0.0, 0.0],
                    },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(field.insert(Block::new(0, [i as i32, 0], 0)).unwrap());
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
            let mut field = BlockField::new(BlockFieldDescriptor {
                chunk_size: 32,
                blocks: vec![
                    BlockDescriptor {
                        size: [1, 1],
                        collision_size: [1.0, 1.0],
                        collision_offset: [0.0, 0.0],
                        hint_size: [1.0, 1.0],
                        hint_offset: [0.0, 0.0],
                    },
                    BlockDescriptor {
                        size: [1, 1],
                        collision_size: [1.0, 1.0],
                        collision_offset: [0.0, 0.0],
                        hint_size: [1.0, 1.0],
                        hint_offset: [0.0, 0.0],
                    },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(field.insert(Block::new(0, [i as i32, 0], 0)).unwrap());
            }

            let instance = std::time::Instant::now();
            for key in keys {
                black_box(field.get(key).unwrap());
            }
            instance.elapsed()
        });
    });

    c.bench_function("entity add", |b| {
        b.iter_custom(|iters| {
            let mut field = EntityField::new(EntityFieldDescriptor {
                chunk_size: 32,
                entities: vec![
                    EntityDescriptor {
                        collision_size: [1.0, 1.0],
                        collision_offset: [0.0, 0.0],
                        hint_size: [1.0, 1.0],
                        hint_offset: [0.0, 0.0],
                    },
                    EntityDescriptor {
                        collision_size: [1.0, 1.0],
                        collision_offset: [0.0, 0.0],
                        hint_size: [1.0, 1.0],
                        hint_offset: [0.0, 0.0],
                    },
                ],
            });

            let instance = std::time::Instant::now();
            for i in 0..iters {
                black_box(field.insert(Entity::new(0, [i as f32, 0.0], 0)).unwrap());
            }
            instance.elapsed()
        });
    });

    c.bench_function("entity remove", |b| {
        b.iter_custom(|iters| {
            let mut field = EntityField::new(EntityFieldDescriptor {
                chunk_size: 32,
                entities: vec![
                    EntityDescriptor {
                        collision_size: [1.0, 1.0],
                        collision_offset: [0.0, 0.0],
                        hint_size: [1.0, 1.0],
                        hint_offset: [0.0, 0.0],
                    },
                    EntityDescriptor {
                        collision_size: [1.0, 1.0],
                        collision_offset: [0.0, 0.0],
                        hint_size: [1.0, 1.0],
                        hint_offset: [0.0, 0.0],
                    },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(field.insert(Entity::new(0, [i as f32, 0.0], 0)).unwrap());
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
            let mut field = EntityField::new(EntityFieldDescriptor {
                chunk_size: 32,
                entities: vec![
                    EntityDescriptor {
                        collision_size: [1.0, 1.0],
                        collision_offset: [0.0, 0.0],
                        hint_size: [1.0, 1.0],
                        hint_offset: [0.0, 0.0],
                    },
                    EntityDescriptor {
                        collision_size: [1.0, 1.0],
                        collision_offset: [0.0, 0.0],
                        hint_size: [1.0, 1.0],
                        hint_offset: [0.0, 0.0],
                    },
                ],
            });

            let mut keys = vec![];
            for i in 0..iters {
                keys.push(field.insert(Entity::new(0, [i as f32, 0.0], 0)).unwrap());
            }

            let instance = std::time::Instant::now();
            for key in keys {
                black_box(field.get(key).unwrap());
            }
            instance.elapsed()
        });
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
