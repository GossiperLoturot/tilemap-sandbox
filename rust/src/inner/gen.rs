use crate::inner;

use super::*;

const CHUNK_SIZE: u32 = 32;

pub type GenFn<T> = Box<dyn Fn(&mut Root, T)>;

pub trait GenRule: std::fmt::Debug {
    fn gen_chunk(&self, root: &mut inner::Root, chunk_location: IVec2);
}

// rules

pub struct MarchGenRule {
    pub prob: f32,
    pub gen_fn: GenFn<IVec2>,
}

impl GenRule for MarchGenRule {
    fn gen_chunk(&self, root: &mut inner::Root, chunk_location: IVec2) {
        let rng = &mut rand::thread_rng();

        for y in 0..CHUNK_SIZE as i32 {
            for x in 0..CHUNK_SIZE as i32 {
                let location = IVec2::new(
                    chunk_location[0] * CHUNK_SIZE as i32 + x,
                    chunk_location[1] * CHUNK_SIZE as i32 + y,
                );

                let value = rand::Rng::gen_range(rng, 0.0..1.0);
                if self.prob < value {
                    continue;
                }

                (*self.gen_fn)(root, location);
            }
        }
    }
}

impl std::fmt::Debug for MarchGenRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MarchGenRule")
            .field("prob", &self.prob)
            .field("gen_fn", &"...")
            .finish()
    }
}

pub struct SpawnGenRule {
    pub prob: f32,
    pub gen_fn: GenFn<Vec2>,
}

impl GenRule for SpawnGenRule {
    fn gen_chunk(&self, root: &mut inner::Root, chunk_location: IVec2) {
        let rng = &mut rand::thread_rng();

        let size = (self.prob * (CHUNK_SIZE * CHUNK_SIZE) as f32) as i32;
        for _ in 0..size {
            let u = rand::Rng::gen_range(rng, 0.0..CHUNK_SIZE as f32);
            let v = rand::Rng::gen_range(rng, 0.0..CHUNK_SIZE as f32);

            let location = Vec2::new(
                (chunk_location.x * CHUNK_SIZE as i32) as f32 + u,
                (chunk_location.y * CHUNK_SIZE as i32) as f32 + v,
            );

            (*self.gen_fn)(root, location)
        }
    }
}

impl std::fmt::Debug for SpawnGenRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpawnGenRule")
            .field("prob", &self.prob)
            .field("gen_fn", &"...")
            .finish()
    }
}

#[derive(Debug)]
pub struct GenResourceDescriptor {
    pub gen_rules: Vec<Box<dyn GenRule>>,
}

// resource

#[derive(Debug)]
pub struct GenResource {
    gen_rules: Vec<Box<dyn GenRule>>,
    visit: ahash::AHashSet<IVec2>,
}

impl GenResource {
    pub fn new(desc: GenResourceDescriptor) -> Self {
        Self {
            gen_rules: desc.gen_rules,
            visit: ahash::AHashSet::new(),
        }
    }
}

// system

pub struct GenSystem;

impl GenSystem {
    pub fn exec_rect(root: &mut inner::Root, min_rect: [Vec2; 2]) -> Result<(), RootError> {
        let resource = root.find_resources::<GenResource>()?;
        let mut resource = resource.borrow_mut()?;

        let chunk_size = Vec2::splat(CHUNK_SIZE as f32);
        let min_rect = [
            min_rect[0].div_euclid(chunk_size).as_ivec2(),
            min_rect[1].div_euclid(chunk_size).as_ivec2(),
        ];

        for y in min_rect[0].y..=min_rect[1].y {
            for x in min_rect[0].x..=min_rect[1].x {
                let chunk_location = IVec2::new(x, y);

                if resource.visit.contains(&chunk_location) {
                    continue;
                }

                resource.visit.insert(chunk_location);

                for gen_rule in &resource.gen_rules {
                    gen_rule.gen_chunk(root, chunk_location);
                }
            }
        }

        Ok(())
    }
}
