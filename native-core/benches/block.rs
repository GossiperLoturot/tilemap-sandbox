use criterion::*;
use glam::*;

use native_core::*;

fn make_block_field() -> dataflow::BlockField {
    dataflow::BlockField::new(dataflow::BlockFieldInfo {
        blocks: vec![
            dataflow::BlockInfo {
                display_name: "block_0".into(),
                description: "block_0_desc".into(),
                size: IVec2::new(1, 1),
                collision_rect: Some(Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0))),
                hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                y_sorting: false,
            },
            dataflow::BlockInfo {
                display_name: "block_1".into(),
                description: "block_1_desc".into(),
                size: IVec2::new(1, 1),
                collision_rect: Some(Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0))),
                hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                y_sorting: false,
            },
        ],
    })
}

pub fn benchmark_block(c: &mut Criterion) {
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
                let result = field.modify_variant(id, std::hint::black_box(1)).unwrap();
                std::hint::black_box(result);
            }
            instance.elapsed()
        });
    });

    c.bench_function("block move", |b| {
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
                let new_coord = std::hint::black_box(IVec2::new(i as i32, (i % 16) as i32));
                let result = field.r#move(ids[i as usize], new_coord).unwrap();
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
            let query = std::hint::black_box(IRect2::new(IVec2::new(i as i32 - 100, 0), IVec2::new(i as i32, 100)));
            let result = field.find_with_rect(query).count();
            std::hint::black_box(result);
        })),
        ("block find collision point", Box::new(|field, i| {
            let query = std::hint::black_box(Vec2::new(i as f32, 0.0));
            let result = field.find_with_collision_point(query).count();
            std::hint::black_box(result);
        })),
        ("block find collision rect", Box::new(|field, i| {
            let query = std::hint::black_box(Rect2::new(Vec2::new(i as f32 - 100.0, 0.0), Vec2::new(i as f32, 100.0)));
            let result = field.find_with_collision_rect(query).count();
            std::hint::black_box(result);
        })),
        ("block find hint point", Box::new(|field, i| {
            let query = std::hint::black_box(Vec2::new(i as f32, 0.0));
            let result = field.find_with_hint_point(query).count();
            std::hint::black_box(result);
        })),
        ("block find hint rect", Box::new(|field, i| {
            let query = std::hint::black_box(Rect2::new(Vec2::new(i as f32 - 100.0, 0.0), Vec2::new(i as f32, 100.0)));
            let result = field.find_with_hint_rect(query).count();
            std::hint::black_box(result);
        })),
    ];
    for (name, f) in row {
        c.bench_function(name, |b| {
            b.iter_custom(|iters| {
                let mut field = make_block_field();

                let mut ids = vec![];
                for x in 0..iters {
                    for y in 0..100 {
                        let id = field
                            .insert(dataflow::Block {
                                archetype_id: 0,
                                coord: IVec2::new(x as i32, y as i32),
                                ..Default::default()
                            })
                            .unwrap();
                        ids.push(id);
                    }
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
