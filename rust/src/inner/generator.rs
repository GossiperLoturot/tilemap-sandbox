use crate::inner;

// generator rule descriptors

#[derive(Clone)]
pub struct GeneratorRuleMarchingDescriptor {
    pub prob: f32,
    pub id: u32,
}

#[derive(Clone)]
pub struct GeneratorRuleSpawnDescriptor {
    pub prob: f32,
    pub id: u32,
}

#[derive(Clone)]
pub enum GeneratorRuleDescriptor {
    Marching(GeneratorRuleMarchingDescriptor),
    Spawn(GeneratorRuleSpawnDescriptor),
}

// generator descriptors

#[derive(Clone)]
pub struct GeneratorDescriptor {
    pub chunk_size: u32,
    pub tile_rules: Vec<GeneratorRuleDescriptor>,
    pub block_rules: Vec<GeneratorRuleDescriptor>,
    pub entity_rules: Vec<GeneratorRuleDescriptor>,
}

// generator rules

#[derive(Clone)]
struct GeneratorRuleMarching {
    prob: f32,
    id: u32,
}

impl GeneratorRuleMarching {
    pub fn new(desc: GeneratorRuleMarchingDescriptor) -> Self {
        Self {
            prob: desc.prob,
            id: desc.id,
        }
    }
}

#[derive(Clone)]
struct GeneratorRuleSpawn {
    prob: f32,
    id: u32,
}

impl GeneratorRuleSpawn {
    pub fn new(desc: GeneratorRuleSpawnDescriptor) -> Self {
        Self {
            prob: desc.prob,
            id: desc.id,
        }
    }
}

#[derive(Clone)]
enum GeneratorRule {
    Marching(GeneratorRuleMarching),
    Spawn(GeneratorRuleSpawn),
}

impl GeneratorRule {
    pub fn new(desc: GeneratorRuleDescriptor) -> Self {
        match desc {
            GeneratorRuleDescriptor::Marching(desc) => {
                let rule = GeneratorRuleMarching::new(desc);
                Self::Marching(rule)
            }
            GeneratorRuleDescriptor::Spawn(rule) => {
                let rule = GeneratorRuleSpawn::new(rule);
                Self::Spawn(rule)
            }
        }
    }
}

// generator

#[derive(Clone)]
pub struct Generator {
    chunk_size: u32,
    tile_rules: Vec<GeneratorRule>,
    block_rules: Vec<GeneratorRule>,
    entity_rules: Vec<GeneratorRule>,
    visit: ahash::AHashSet<inner::IVec2>,
}

impl Generator {
    pub fn init(root: &mut inner::Root, desc: GeneratorDescriptor) {
        let mut tile_rules = vec![];
        for rule in desc.tile_rules {
            tile_rules.push(GeneratorRule::new(rule));
        }

        let mut block_rules = vec![];
        for rule in desc.block_rules {
            block_rules.push(GeneratorRule::new(rule));
        }

        let mut entity_rules = vec![];
        for rule in desc.entity_rules {
            entity_rules.push(GeneratorRule::new(rule));
        }

        let generator = Self {
            chunk_size: desc.chunk_size,
            tile_rules,
            block_rules,
            entity_rules,
            visit: ahash::AHashSet::new(),
        };
        root.resource_insert(generator).unwrap();
    }

    pub fn generate_rect(root: &mut inner::Root, min_rect: [inner::Vec2; 2]) {
        let mut slf = root.resource_remove::<Generator>().unwrap();

        #[rustfmt::skip]
        let min_rect = [[
            min_rect[0][0].div_euclid(slf.chunk_size as f32) as i32,
            min_rect[0][1].div_euclid(slf.chunk_size as f32) as i32, ], [
            min_rect[1][0].div_euclid(slf.chunk_size as f32) as i32,
            min_rect[1][1].div_euclid(slf.chunk_size as f32) as i32,
        ]];

        for y in min_rect[0][1]..=min_rect[1][1] {
            for x in min_rect[0][0]..=min_rect[1][0] {
                let chunk_location = [x, y];

                if slf.visit.contains(&chunk_location) {
                    continue;
                }

                for rule in &slf.tile_rules {
                    match rule {
                        GeneratorRule::Marching(rule) => {
                            slf.tile_marching_generate_chunk(root, rule, chunk_location)
                        }
                        GeneratorRule::Spawn(rule) => {
                            slf.tile_spawn_generate_chunk(root, rule, chunk_location)
                        }
                    }
                }

                for rule in &slf.block_rules {
                    match rule {
                        GeneratorRule::Marching(rule) => {
                            slf.block_marching_generate_chunk(root, rule, chunk_location)
                        }
                        GeneratorRule::Spawn(rule) => {
                            slf.block_spawn_generate_chunk(root, rule, chunk_location)
                        }
                    }
                }

                for rule in &slf.entity_rules {
                    match rule {
                        GeneratorRule::Marching(rule) => {
                            slf.entity_marching_generate_chunk(root, rule, chunk_location)
                        }
                        GeneratorRule::Spawn(rule) => {
                            slf.entity_spawn_generate_chunk(root, rule, chunk_location)
                        }
                    }
                }

                slf.visit.insert(chunk_location);
            }
        }

        root.resource_insert(slf).unwrap();
    }

    fn tile_marching_generate_chunk(
        &self,
        root: &mut inner::Root,
        rule: &GeneratorRuleMarching,
        chunk_location: inner::IVec2,
    ) {
        let mut rng = rand::thread_rng();

        for y in 0..self.chunk_size as i32 {
            for x in 0..self.chunk_size as i32 {
                let location = [
                    chunk_location[0] * self.chunk_size as i32 + x,
                    chunk_location[1] * self.chunk_size as i32 + y,
                ];

                let value = rand::Rng::gen_range(&mut rng, 0.0..1.0);
                if rule.prob < value {
                    continue;
                }

                let _ = root.tile_insert(inner::Tile {
                    id: rule.id as u16,
                    location,
                    variant: Default::default(),
                    tick: Default::default(),
                    data: Default::default(),
                });
            }
        }
    }

    fn tile_spawn_generate_chunk(
        &self,
        root: &mut inner::Root,
        rule: &GeneratorRuleSpawn,
        chunk_location: inner::IVec2,
    ) {
        let mut rng = rand::thread_rng();

        let x = chunk_location[0];
        let y = chunk_location[1];

        let size = (rule.prob * (self.chunk_size * self.chunk_size) as f32) as i32;
        for _ in 0..size {
            let u = rand::Rng::gen_range(&mut rng, 0..self.chunk_size as i32);
            let v = rand::Rng::gen_range(&mut rng, 0..self.chunk_size as i32);

            let location = [
                x * self.chunk_size as i32 + u,
                y * self.chunk_size as i32 + v,
            ];

            let _ = root.tile_insert(inner::Tile {
                id: rule.id as u16,
                location,
                variant: Default::default(),
                tick: Default::default(),
                data: Default::default(),
            });
        }
    }

    fn block_marching_generate_chunk(
        &self,
        root: &mut inner::Root,
        rule: &GeneratorRuleMarching,
        chunk_location: inner::IVec2,
    ) {
        let mut rng = rand::thread_rng();

        for y in 0..self.chunk_size as i32 {
            for x in 0..self.chunk_size as i32 {
                let location = [
                    chunk_location[0] * self.chunk_size as i32 + x,
                    chunk_location[1] * self.chunk_size as i32 + y,
                ];

                let value = rand::Rng::gen_range(&mut rng, 0.0..1.0);
                if rule.prob < value {
                    continue;
                }

                let _ = root.block_insert(inner::Block {
                    id: rule.id as u16,
                    location,
                    variant: Default::default(),
                    tick: Default::default(),
                    data: Default::default(),
                });
            }
        }
    }

    fn block_spawn_generate_chunk(
        &self,
        root: &mut inner::Root,
        rule: &GeneratorRuleSpawn,
        chunk_location: inner::IVec2,
    ) {
        let mut rng = rand::thread_rng();

        let x = chunk_location[0];
        let y = chunk_location[1];

        let size = (rule.prob * (self.chunk_size * self.chunk_size) as f32) as i32;
        for _ in 0..size {
            let u = rand::Rng::gen_range(&mut rng, 0..self.chunk_size as i32);
            let v = rand::Rng::gen_range(&mut rng, 0..self.chunk_size as i32);

            let location = [
                x * self.chunk_size as i32 + u,
                y * self.chunk_size as i32 + v,
            ];

            let _ = root.block_insert(inner::Block {
                id: rule.id as u16,
                location,
                variant: Default::default(),
                tick: Default::default(),
                data: Default::default(),
            });
        }
    }

    fn entity_marching_generate_chunk(
        &self,
        root: &mut inner::Root,
        rule: &GeneratorRuleMarching,
        chunk_location: inner::IVec2,
    ) {
        let mut rng = rand::thread_rng();

        for y in 0..self.chunk_size as i32 {
            for x in 0..self.chunk_size as i32 {
                let location = [
                    (chunk_location[0] * self.chunk_size as i32 + x) as f32,
                    (chunk_location[1] * self.chunk_size as i32 + y) as f32,
                ];

                let value = rand::Rng::gen_range(&mut rng, 0.0..1.0);
                if rule.prob < value {
                    continue;
                }

                let _ = root.entity_insert(inner::Entity {
                    id: rule.id,
                    location,
                    variant: Default::default(),
                    data: Default::default(),
                });
            }
        }
    }

    fn entity_spawn_generate_chunk(
        &self,
        root: &mut inner::Root,
        rule: &GeneratorRuleSpawn,
        chunk_location: inner::IVec2,
    ) {
        let mut rng = rand::thread_rng();

        let x = chunk_location[0];
        let y = chunk_location[1];

        let size = (rule.prob * (self.chunk_size * self.chunk_size) as f32) as i32;
        for _ in 0..size {
            let u = rand::Rng::gen_range(&mut rng, 0..self.chunk_size as i32);
            let v = rand::Rng::gen_range(&mut rng, 0..self.chunk_size as i32);

            let location = [
                (x * self.chunk_size as i32) as f32 + u as f32,
                (y * self.chunk_size as i32) as f32 + v as f32,
            ];

            let _ = root.entity_insert(inner::Entity {
                id: rule.id,
                location,
                variant: Default::default(),
                data: Default::default(),
            });
        }
    }
}
