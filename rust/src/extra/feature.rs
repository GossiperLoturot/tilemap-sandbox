use crate::inner;

#[derive(Clone)]
pub struct Feature;

impl inner::Feature for Feature {
    type Tile = TileFeature;
    type Block = BlockFeature;
    type Entity = EntityFeature;
}

#[derive(Clone)]
pub struct TileFeature;

impl inner::TileFeature<Feature> for TileFeature {
    type Item = ();

    fn after_place(&self, _root: &mut inner::Root<Feature>, _key: inner::TileKey) {}
    fn before_break(&self, _root: &mut inner::Root<Feature>, _key: inner::TileKey) {}
    fn forward(&self, _root: &mut inner::Root<Feature>, _key: inner::TileKey) {}
}

#[derive(Clone)]
pub struct BlockFeature;

impl inner::BlockFeature<Feature> for BlockFeature {
    type Item = ();

    fn after_place(&self, _root: &mut inner::Root<Feature>, _key: inner::TileKey) {}
    fn before_break(&self, _root: &mut inner::Root<Feature>, _key: inner::TileKey) {}
    fn forward(&self, _root: &mut inner::Root<Feature>, _key: inner::TileKey) {}
}

#[derive(Clone)]
pub struct EntityFeature;

impl inner::EntityFeature<Feature> for EntityFeature {
    type Item = ();

    fn after_place(&self, _root: &mut inner::Root<Feature>, _key: inner::TileKey) {}
    fn before_break(&self, _root: &mut inner::Root<Feature>, _key: inner::TileKey) {}
    fn forward(&self, _root: &mut inner::Root<Feature>, _key: inner::TileKey) {}
}

#[derive(Clone)]
#[non_exhaustive]
pub enum GeneratorRule {
    MarchingRandom {
        seed: u32,
        probability: f32,
        id: u32,
    },
    MarchingFBM {
        seed: u32,
        probability: f32,
        scale: f32,
        id: u32,
    },
    SpawnRandom {
        seed: u32,
        probability: f32,
        id: u32,
    },
    SpawnRandomGroup {
        seed: u32,
        probability: f32,
        variance: f32,
        group_size: u32,
        id: u32,
    },
}

#[derive(Clone)]
pub struct Generator {
    chunk_size: u32,
    tile_rules: Vec<GeneratorRule>,
    block_rules: Vec<GeneratorRule>,
    entity_rules: Vec<GeneratorRule>,
    visit: ahash::AHashSet<inner::IVec2>,
}

impl Generator {
    pub fn new(
        chunk_size: u32,
        tile_rules: Vec<GeneratorRule>,
        block_rules: Vec<GeneratorRule>,
        entity_rules: Vec<GeneratorRule>,
    ) -> Self {
        Self {
            chunk_size,
            tile_rules,
            block_rules,
            entity_rules,
            visit: ahash::AHashSet::new(),
        }
    }

    pub fn generate_chunk(root: &mut inner::Root<Feature>, min_rect: [inner::Vec2; 2]) {
        use noise::NoiseFn;

        // TODO: Add manual taking ownership function by using closure
        let mut slf = root.resource_remove::<Generator>().unwrap();
        let chunk_size = slf.chunk_size as i32;

        #[rustfmt::skip]
        let min_rect = [[
            min_rect[0][0].div_euclid(chunk_size as f32) as i32,
            min_rect[0][1].div_euclid(chunk_size as f32) as i32, ], [
            min_rect[1][0].div_euclid(chunk_size as f32) as i32,
            min_rect[1][1].div_euclid(chunk_size as f32) as i32,
        ]];

        for y in min_rect[0][1]..=min_rect[1][1] {
            for x in min_rect[0][0]..=min_rect[1][0] {
                let chunk_location = [x, y];

                if slf.visit.contains(&chunk_location) {
                    continue;
                }

                // TODO: Move to a new module and separate functions enclosed by match arms
                for rule in &slf.tile_rules {
                    match rule.clone() {
                        GeneratorRule::MarchingRandom {
                            seed,
                            probability,
                            id,
                        } => {
                            let noise = noise::Simplex::new(seed);

                            for v in 0..chunk_size {
                                for u in 0..chunk_size {
                                    let location = [x * chunk_size + u, y * chunk_size + v];
                                    let value =
                                        noise.get([location[0] as f64, location[1] as f64]) as f32;
                                    let value = value * 0.5 + 0.5;

                                    if probability < value {
                                        continue;
                                    }

                                    let _ = root.tile_insert(inner::Tile {
                                        id,
                                        location,
                                        variant: Default::default(),
                                        data: Default::default(),
                                    });
                                }
                            }
                        }
                        GeneratorRule::MarchingFBM {
                            seed,
                            probability,
                            scale,
                            id,
                        } => {
                            let noise = noise::Fbm::<noise::Simplex>::new(seed);

                            for v in 0..chunk_size {
                                for u in 0..chunk_size {
                                    let location = [x * chunk_size + u, y * chunk_size + v];
                                    let value = noise.get([
                                        (location[0] as f32 * scale) as f64,
                                        (location[1] as f32 * scale) as f64,
                                    ]) as f32;
                                    let value = value * 0.5 + 0.5;

                                    if probability < value {
                                        continue;
                                    }

                                    let _ = root.tile_insert(inner::Tile {
                                        id,
                                        location,
                                        variant: Default::default(),
                                        data: Default::default(),
                                    });
                                }
                            }
                        }
                        GeneratorRule::SpawnRandom {
                            seed,
                            probability,
                            id,
                        } => {
                            // TODO: Change to hash function
                            let noise = noise::Simplex::new(seed);

                            let size = (probability / (chunk_size * chunk_size) as f32) as i32;
                            for i in 1..=size {
                                let u = noise.get([x as f64, y as f64, i as f64]);
                                let v = noise.get([x as f64, y as f64, -i as f64]);
                                let location = [
                                    ((x as f32 + u as f32) * chunk_size as f32) as i32,
                                    ((y as f32 + v as f32) * chunk_size as f32) as i32,
                                ];

                                let _ = root.tile_insert(inner::Tile {
                                    id,
                                    location,
                                    variant: Default::default(),
                                    data: Default::default(),
                                });
                            }
                        }
                        GeneratorRule::SpawnRandomGroup {
                            seed,
                            probability,
                            variance,
                            group_size,
                            id,
                        } => {
                            // TODO: Change to hash function
                            let noise = noise::Simplex::new(seed);

                            let size = (probability / (chunk_size * chunk_size) as f32) as i32;
                            for i in 1..=size {
                                let u = noise.get([x as f64, y as f64, i as f64]);
                                let v = noise.get([x as f64, y as f64, -i as f64]);
                                let location = [
                                    ((x as f32 + u as f32) * chunk_size as f32) as i32,
                                    ((y as f32 + v as f32) * chunk_size as f32) as i32,
                                ];

                                for j in 0..group_size {
                                    let s = noise.get([
                                        x as f64,
                                        y as f64,
                                        (i as f32 + j as f32 / group_size as f32) as f64,
                                    ]) as f32;
                                    let t = noise.get([
                                        x as f64,
                                        y as f64,
                                        -(i as f32 + j as f32 / group_size as f32) as f64,
                                    ]) as f32;
                                    let location = [
                                        location[0] + (s * variance) as i32,
                                        location[1] + (t * variance) as i32,
                                    ];
                                    let _ = root.tile_insert(inner::Tile {
                                        id,
                                        location,
                                        variant: Default::default(),
                                        data: Default::default(),
                                    });
                                }
                            }
                        }
                    }
                }

                slf.visit.insert(chunk_location);
            }
        }

        root.resource_insert(slf).unwrap();
    }
}
