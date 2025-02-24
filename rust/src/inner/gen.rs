use crate::inner;

use super::*;

type GenFn<T> = std::rc::Rc<dyn Fn(&mut Root, T)>;

#[derive(Clone)]
pub struct MarchGenRule {
    pub prob: f32,
    pub gen_fn: GenFn<IVec2>,
}

impl std::fmt::Debug for MarchGenRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MarchGenRule")
            .field("prob", &self.prob)
            .field("gen_fn", &"...")
            .finish()
    }
}

#[derive(Clone)]
pub struct SpawnGenRule {
    pub prob: f32,
    pub gen_fn: GenFn<Vec2>,
}

impl std::fmt::Debug for SpawnGenRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpawnGenRule")
            .field("prob", &self.prob)
            .field("gen_fn", &"...")
            .finish()
    }
}

#[derive(Debug, Clone)]
pub enum GenRule {
    March(MarchGenRule),
    Spawn(SpawnGenRule),
}

#[derive(Debug, Clone)]
pub struct GenResourceDescriptor {
    pub gen_rules: Vec<GenRule>,
}

// resource

#[derive(Debug, Clone)]
pub struct GenResource {
    gen_rules: Vec<GenRule>,
    visit: ahash::AHashSet<IVec2>,
}

impl GenResource {
    const CHUNK_SIZE: u32 = 32;

    pub fn new(desc: GenResourceDescriptor) -> Self {
        Self {
            gen_rules: desc.gen_rules,
            visit: ahash::AHashSet::new(),
        }
    }

    pub fn exec_rect(
        &mut self,
        root: &mut inner::Root,
        min_rect: [Vec2; 2],
    ) -> Result<(), RootError> {
        let chunk_size = Vec2::splat(Self::CHUNK_SIZE as f32);
        let min_rect = [
            min_rect[0].div_euclid(chunk_size).as_ivec2(),
            min_rect[1].div_euclid(chunk_size).as_ivec2(),
        ];

        let rng = &mut rand::rng();
        for y in min_rect[0].y..=min_rect[1].y {
            for x in min_rect[0].x..=min_rect[1].x {
                let chunk_location = IVec2::new(x, y);

                if self.visit.contains(&chunk_location) {
                    continue;
                }

                self.visit.insert(chunk_location);

                for rule in &self.gen_rules {
                    match rule {
                        GenRule::March(rule) => {
                            self.march_gen_chunk(root, rule, chunk_location, rng);
                        }
                        GenRule::Spawn(rule) => {
                            self.spawn_gen_chunk(root, rule, chunk_location, rng);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn march_gen_chunk(
        &self,
        root: &mut inner::Root,
        rule: &MarchGenRule,
        chunk_location: IVec2,
        rng: &mut impl rand::Rng,
    ) {
        for y in 0..Self::CHUNK_SIZE as i32 {
            for x in 0..Self::CHUNK_SIZE as i32 {
                let location = IVec2::new(
                    chunk_location[0] * Self::CHUNK_SIZE as i32 + x,
                    chunk_location[1] * Self::CHUNK_SIZE as i32 + y,
                );

                let value = rand::Rng::random_range(rng, 0.0..1.0);
                if rule.prob < value {
                    continue;
                }

                (*rule.gen_fn)(root, location);
            }
        }
    }

    fn spawn_gen_chunk(
        &self,
        root: &mut inner::Root,
        rule: &SpawnGenRule,
        chunk_location: IVec2,
        rng: &mut impl rand::Rng,
    ) {
        let size = (rule.prob * (Self::CHUNK_SIZE * Self::CHUNK_SIZE) as f32) as i32;
        for _ in 0..size {
            let u = rand::Rng::random_range(rng, 0.0..Self::CHUNK_SIZE as f32);
            let v = rand::Rng::random_range(rng, 0.0..Self::CHUNK_SIZE as f32);

            let location = Vec2::new(
                (chunk_location.x * Self::CHUNK_SIZE as i32) as f32 + u,
                (chunk_location.y * Self::CHUNK_SIZE as i32) as f32 + v,
            );

            (*rule.gen_fn)(root, location)
        }
    }
}
