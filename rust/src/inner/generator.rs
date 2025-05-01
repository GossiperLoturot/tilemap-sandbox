use crate::inner;

use super::*;

const CHUNK_SIZE: u32 = 32;

pub type PlaceFn<T> = Box<dyn Fn(&mut Root, T)>;

pub trait Generator: std::fmt::Debug {
    fn generate_chunk(&self, root: &mut inner::Root, chunk_location: IVec2);
}

// method for generating chunk

pub struct MarchGenerator {
    pub prob: f32,
    pub place_fn: PlaceFn<IVec2>,
}

impl Generator for MarchGenerator {
    fn generate_chunk(&self, root: &mut inner::Root, chunk_location: IVec2) {
        let rng = &mut rand::thread_rng();

        for y in 0..CHUNK_SIZE as i32 {
            for x in 0..CHUNK_SIZE as i32 {
                let location = IVec2::new(
                    chunk_location.x * CHUNK_SIZE as i32 + x,
                    chunk_location.y * CHUNK_SIZE as i32 + y,
                );

                let value = rand::Rng::gen_range(rng, 0.0..1.0);
                if self.prob < value {
                    continue;
                }

                (*self.place_fn)(root, location);
            }
        }
    }
}

impl std::fmt::Debug for MarchGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MarchGenerator")
            .field("prob", &self.prob)
            .field("place_fn", &"...")
            .finish()
    }
}

pub struct SpawnGenerator {
    pub prob: f32,
    pub place_fn: PlaceFn<Vec2>,
}

impl Generator for SpawnGenerator {
    fn generate_chunk(&self, root: &mut inner::Root, chunk_location: IVec2) {
        let rng = &mut rand::thread_rng();

        let size = (self.prob * (CHUNK_SIZE * CHUNK_SIZE) as f32) as i32;
        for _ in 0..size {
            let u = rand::Rng::gen_range(rng, 0.0..CHUNK_SIZE as f32);
            let v = rand::Rng::gen_range(rng, 0.0..CHUNK_SIZE as f32);

            let location = Vec2::new(
                (chunk_location.x * CHUNK_SIZE as i32) as f32 + u,
                (chunk_location.y * CHUNK_SIZE as i32) as f32 + v,
            );

            (*self.place_fn)(root, location)
        }
    }
}

impl std::fmt::Debug for SpawnGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpawnGenerator")
            .field("prob", &self.prob)
            .field("place_fn", &"...")
            .finish()
    }
}

#[derive(Debug)]
pub struct GeneratorResourceDescriptor {
    pub generators: Vec<Box<dyn Generator>>,
}

// resource

#[derive(Debug)]
pub struct GeneratorResource {
    generators: Vec<Box<dyn Generator>>,
    visited: ahash::AHashSet<IVec2>,
}

impl GeneratorResource {
    pub fn new(desc: GeneratorResourceDescriptor) -> Self {
        Self {
            generators: desc.generators,
            visited: ahash::AHashSet::new(),
        }
    }
}

// system

pub struct GeneratorSystem;

impl GeneratorSystem {
    pub fn generate(root: &mut inner::Root, min_rect: [Vec2; 2]) -> Result<(), RootError> {
        let resource = root.find_resources::<GeneratorResource>()?;
        let mut resource = resource.borrow_mut()?;

        let chunk_size = Vec2::splat(CHUNK_SIZE as f32);
        let min_rect = [
            min_rect[0].div_euclid(chunk_size).as_ivec2(),
            min_rect[1].div_euclid(chunk_size).as_ivec2(),
        ];

        for y in min_rect[0].y..=min_rect[1].y {
            for x in min_rect[0].x..=min_rect[1].x {
                let chunk_location = IVec2::new(x, y);

                if resource.visited.contains(&chunk_location) {
                    continue;
                }

                resource.visited.insert(chunk_location);

                for generator in &resource.generators {
                    generator.generate_chunk(root, chunk_location);
                }
            }
        }

        Ok(())
    }
}
