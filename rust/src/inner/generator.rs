use crate::inner;

#[derive(Debug, Clone)]
pub struct GeneratorRuleMarching {
    pub prob: f32,
    pub id: u16,
}

#[derive(Debug, Clone)]
pub struct GeneratorRuleSpawn {
    pub prob: f32,
    pub id: u16,
}

#[derive(Debug, Clone)]
pub enum GeneratorRule {
    Marching(GeneratorRuleMarching),
    Spawn(GeneratorRuleSpawn),
}

#[derive(Debug, Clone)]
pub struct GeneratorResourceDescriptor {
    pub tile_rules: Vec<GeneratorRule>,
    pub block_rules: Vec<GeneratorRule>,
    pub entity_rules: Vec<GeneratorRule>,
}

// resource

#[derive(Debug, Clone)]
pub struct GeneratorResource {
    tile_rules: Vec<GeneratorRule>,
    block_rules: Vec<GeneratorRule>,
    entity_rules: Vec<GeneratorRule>,
    visit: ahash::AHashSet<inner::IVec2>,
}

impl GeneratorResource {
    const CHUNK_SIZE: u32 = 32;

    pub fn new(desc: GeneratorResourceDescriptor) -> Self {
        Self {
            tile_rules: desc.tile_rules,
            block_rules: desc.block_rules,
            entity_rules: desc.entity_rules,
            visit: ahash::AHashSet::new(),
        }
    }

    pub fn exec_rect(
        &mut self,
        root: &mut inner::Root,
        min_rect: [inner::Vec2; 2],
    ) -> Result<(), GeneratorError> {
        #[rustfmt::skip]
        let min_rect = [[
            min_rect[0][0].div_euclid(Self::CHUNK_SIZE as f32) as i32,
            min_rect[0][1].div_euclid(Self::CHUNK_SIZE as f32) as i32, ], [
            min_rect[1][0].div_euclid(Self::CHUNK_SIZE as f32) as i32,
            min_rect[1][1].div_euclid(Self::CHUNK_SIZE as f32) as i32,
        ]];

        let rng = &mut rand::thread_rng();
        for y in min_rect[0][1]..=min_rect[1][1] {
            for x in min_rect[0][0]..=min_rect[1][0] {
                let chunk_location = [x, y];

                if self.visit.contains(&chunk_location) {
                    continue;
                }

                self.visit.insert(chunk_location);

                for rule in &self.tile_rules {
                    match rule {
                        GeneratorRule::Marching(rule) => {
                            self.gen_tile_chunk_by_marching(root, rule, chunk_location, rng);
                        }
                        GeneratorRule::Spawn(rule) => {
                            self.gen_tile_chunk_by_spawn(root, rule, chunk_location, rng);
                        }
                    }
                }

                for rule in &self.block_rules {
                    match rule {
                        GeneratorRule::Marching(rule) => {
                            self.gen_block_chunk_by_marching(root, rule, chunk_location, rng);
                        }
                        GeneratorRule::Spawn(rule) => {
                            self.gen_block_chunk_by_spawn(root, rule, chunk_location, rng);
                        }
                    }
                }

                for rule in &self.entity_rules {
                    match rule {
                        GeneratorRule::Marching(rule) => {
                            self.gen_entity_chunk_by_marching(root, rule, chunk_location, rng);
                        }
                        GeneratorRule::Spawn(rule) => {
                            self.gen_entity_chunk_by_spawn(root, rule, chunk_location, rng);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn gen_tile_chunk_by_marching(
        &self,
        root: &mut inner::Root,
        rule: &GeneratorRuleMarching,
        chunk_location: inner::IVec2,
        rng: &mut impl rand::Rng,
    ) {
        for y in 0..Self::CHUNK_SIZE as i32 {
            for x in 0..Self::CHUNK_SIZE as i32 {
                let location = [
                    chunk_location[0] * Self::CHUNK_SIZE as i32 + x,
                    chunk_location[1] * Self::CHUNK_SIZE as i32 + y,
                ];

                let value = rand::Rng::gen_range(rng, 0.0..1.0);
                if rule.prob < value {
                    continue;
                }

                let _ = root.tile_insert(inner::Tile {
                    id: rule.id,
                    location,
                    data: Default::default(),
                    render_param: Default::default(),
                });
            }
        }
    }

    fn gen_tile_chunk_by_spawn(
        &self,
        root: &mut inner::Root,
        rule: &GeneratorRuleSpawn,
        chunk_location: inner::IVec2,
        rng: &mut impl rand::Rng,
    ) {
        let x = chunk_location[0];
        let y = chunk_location[1];

        let size = (rule.prob * (Self::CHUNK_SIZE * Self::CHUNK_SIZE) as f32) as i32;
        for _ in 0..size {
            let u = rand::Rng::gen_range(rng, 0..Self::CHUNK_SIZE as i32);
            let v = rand::Rng::gen_range(rng, 0..Self::CHUNK_SIZE as i32);

            let location = [
                x * Self::CHUNK_SIZE as i32 + u,
                y * Self::CHUNK_SIZE as i32 + v,
            ];

            let _ = root.tile_insert(inner::Tile {
                id: rule.id,
                location,
                data: Default::default(),
                render_param: Default::default(),
            });
        }
    }

    fn gen_block_chunk_by_marching(
        &self,
        root: &mut inner::Root,
        rule: &GeneratorRuleMarching,
        chunk_location: inner::IVec2,
        rng: &mut impl rand::Rng,
    ) {
        for y in 0..Self::CHUNK_SIZE as i32 {
            for x in 0..Self::CHUNK_SIZE as i32 {
                let location = [
                    chunk_location[0] * Self::CHUNK_SIZE as i32 + x,
                    chunk_location[1] * Self::CHUNK_SIZE as i32 + y,
                ];

                let value = rand::Rng::gen_range(rng, 0.0..1.0);
                if rule.prob < value {
                    continue;
                }

                let _ = root.block_insert(inner::Block {
                    id: rule.id,
                    location,
                    data: Default::default(),
                    render_param: Default::default(),
                });
            }
        }
    }

    fn gen_block_chunk_by_spawn(
        &self,
        root: &mut inner::Root,
        rule: &GeneratorRuleSpawn,
        chunk_location: inner::IVec2,
        rng: &mut impl rand::Rng,
    ) {
        let x = chunk_location[0];
        let y = chunk_location[1];

        let size = (rule.prob * (Self::CHUNK_SIZE * Self::CHUNK_SIZE) as f32) as i32;
        for _ in 0..size {
            let u = rand::Rng::gen_range(rng, 0..Self::CHUNK_SIZE as i32);
            let v = rand::Rng::gen_range(rng, 0..Self::CHUNK_SIZE as i32);

            let location = [
                x * Self::CHUNK_SIZE as i32 + u,
                y * Self::CHUNK_SIZE as i32 + v,
            ];

            let _ = root.block_insert(inner::Block {
                id: rule.id,
                location,
                data: Default::default(),
                render_param: Default::default(),
            });
        }
    }

    fn gen_entity_chunk_by_marching(
        &self,
        root: &mut inner::Root,
        rule: &GeneratorRuleMarching,
        chunk_location: inner::IVec2,
        rng: &mut impl rand::Rng,
    ) {
        for y in 0..Self::CHUNK_SIZE as i32 {
            for x in 0..Self::CHUNK_SIZE as i32 {
                let location = [
                    (chunk_location[0] * Self::CHUNK_SIZE as i32 + x) as f32,
                    (chunk_location[1] * Self::CHUNK_SIZE as i32 + y) as f32,
                ];

                let value = rand::Rng::gen_range(rng, 0.0..1.0);
                if rule.prob < value {
                    continue;
                }

                let _ = root.entity_insert(inner::Entity {
                    id: rule.id,
                    location,
                    data: Default::default(),
                    render_param: Default::default(),
                });
            }
        }
    }

    fn gen_entity_chunk_by_spawn(
        &self,
        root: &mut inner::Root,
        rule: &GeneratorRuleSpawn,
        chunk_location: inner::IVec2,
        rng: &mut impl rand::Rng,
    ) {
        let x = chunk_location[0];
        let y = chunk_location[1];

        let size = (rule.prob * (Self::CHUNK_SIZE * Self::CHUNK_SIZE) as f32) as i32;
        for _ in 0..size {
            let u = rand::Rng::gen_range(rng, 0..Self::CHUNK_SIZE as i32);
            let v = rand::Rng::gen_range(rng, 0..Self::CHUNK_SIZE as i32);

            let location = [
                (x * Self::CHUNK_SIZE as i32) as f32 + u as f32,
                (y * Self::CHUNK_SIZE as i32) as f32 + v as f32,
            ];

            let _ = root.entity_insert(inner::Entity {
                id: rule.id,
                location,
                data: Default::default(),
                render_param: Default::default(),
            });
        }
    }
}

// error handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeneratorError {
    NotScoped,
}

impl std::fmt::Display for GeneratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotScoped => write!(f, "not scoped error"),
        }
    }
}
