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
pub struct Generator {
    chunk_size: u32,
    visit: ahash::AHashSet<inner::IVec2>,
}

impl Generator {
    pub fn new(chunk_size: u32) -> Self {
        Self {
            chunk_size,
            visit: ahash::AHashSet::new(),
        }
    }

    pub fn generate_chunk(root: &mut inner::Root<Feature>, min_rect: [inner::Vec2; 2]) {
        let slf = root.resource_get::<Generator>().unwrap();
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

                let slf = root.resource_get::<Generator>().unwrap();
                if slf.visit.contains(&chunk_location) {
                    continue;
                }

                for v in 0..chunk_size {
                    for u in 0..chunk_size {
                        let id = rand::Rng::gen_range(&mut rand::thread_rng(), 0..=1);
                        let location = [x * chunk_size + u, y * chunk_size + v];
                        let _ = root.tile_insert(inner::Tile {
                            id,
                            location,
                            variant: Default::default(),
                            data: Default::default(),
                        });
                    }
                }

                for _ in 0..16 {
                    let id = rand::Rng::gen_range(&mut rand::thread_rng(), 0..=3);
                    let u = rand::Rng::gen_range(&mut rand::thread_rng(), 0..chunk_size);
                    let v = rand::Rng::gen_range(&mut rand::thread_rng(), 0..chunk_size);
                    let location = [x * chunk_size + u, y * chunk_size + v];
                    let _ = root.block_insert(inner::Block {
                        id,
                        location,
                        variant: Default::default(),
                        data: Default::default(),
                    });
                }

                for _ in 0..16 {
                    let id = rand::Rng::gen_range(&mut rand::thread_rng(), 1..=5);
                    let u = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..chunk_size as f32);
                    let v = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..chunk_size as f32);
                    let location = [
                        x as f32 * chunk_size as f32 + u,
                        y as f32 * chunk_size as f32 + v,
                    ];
                    let _ = root.entity_insert(inner::Entity {
                        id,
                        location,
                        variant: Default::default(),
                        data: Default::default(),
                    });
                }

                let slf = root.resource_get_mut::<Generator>().unwrap();
                slf.visit.insert(chunk_location);
            }
        }
    }
}
