use criterion::*;
use glam::*;

use native_core::*;

fn make_entity_field() -> dataflow::EntityField {
    dataflow::EntityField::new(dataflow::EntityFieldInfo {
        entities: vec![
            dataflow::EntityInfo {
                display_name: "entity_0".into(),
                description: "entity_0_desc".into(),
                collision_rect: Some(Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0))),
                hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                y_sorting: false,
            },
            dataflow::EntityInfo {
                display_name: "entity_1".into(),
                description: "entity_1_desc".into(),
                collision_rect: Some(Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0))),
                hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                y_sorting: false,
            },
        ],
    })
}

pub fn benchmark_entity(c: &mut Criterion) {
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
                let result = field.modify_variant(id, std::hint::black_box(1)).unwrap();
                std::hint::black_box(result);
            }
            instance.elapsed()
        });
    });

    c.bench_function("entity move", |b| {
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
                let new_coord = std::hint::black_box(Vec2::new(i as f32, (i % 16) as f32));
                let result = field.r#move(ids[i as usize], new_coord).unwrap();
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
            let query = std::hint::black_box(Rect2::new(Vec2::new(i as f32 - 100.0, 0.0), Vec2::new(i as f32, 100.0)));
            let result = field.find_with_collision_rect(query).count();
            std::hint::black_box(result);
        })),
        ("entity find hint point", Box::new(|field, i| {
            let query = std::hint::black_box(Vec2::new(i as f32, 0.0));
            let result = field.find_with_hint_point(query).count();
            std::hint::black_box(result);
        })),
        ("entity find hint rect", Box::new(|field, i| {
            let query = std::hint::black_box(Rect2::new(Vec2::new(i as f32 - 100.0, 0.0), Vec2::new(i as f32, 100.0)));
            let result = field.find_with_hint_rect(query).count();
            std::hint::black_box(result);
        })),
    ];
    for (name, f) in row {
        c.bench_function(name, |b| {
            b.iter_custom(|iters| {
                let mut field = make_entity_field();

                let mut ids = vec![];
                for x in 0..iters {
                    for y in 0..100 {
                        let id = field
                            .insert(dataflow::Entity {
                                archetype_id: 0,
                                coord: Vec2::new(x as f32, y as f32),
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

