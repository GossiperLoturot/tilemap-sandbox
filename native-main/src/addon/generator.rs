use glam::*;
use native_core::dataflow::*;

pub trait Generator {
    fn generate_chunk(&self, dataflow: &mut Dataflow, rect: [IVec2; 2]);
}

// method for generating chunk

pub type PlaceFn<T> = Box<dyn Fn(&mut Dataflow, T)>;

pub struct MarchGenerator {
    pub prob: f32,
    pub place_fn: PlaceFn<IVec2>,
}

impl Generator for MarchGenerator {
    fn generate_chunk(&self, dataflow: &mut Dataflow, rect: [IVec2; 2]) {
        let rng = &mut rand::thread_rng();

        for y in rect[0].y..rect[1].y {
            for x in rect[0].x..rect[1].x {
                let location = IVec2::new(x, y);

                let value = rand::Rng::gen_range(rng, 0.0..1.0);
                if self.prob < value {
                    continue;
                }

                (*self.place_fn)(dataflow, location);
            }
        }
    }
}

pub struct SpawnGenerator {
    pub prob: f32,
    pub place_fn: PlaceFn<Vec2>,
}

impl Generator for SpawnGenerator {
    fn generate_chunk(&self, dataflow: &mut Dataflow, rect: [IVec2; 2]) {
        let rng = &mut rand::thread_rng();

        let area = (rect[1] - rect[0]).element_product();
        let generate_count = (area as f32 * self.prob) as i32;
        for _ in 0..generate_count {
            let x = rand::Rng::gen_range(rng, rect[0].x as f32..rect[1].x as f32);
            let y = rand::Rng::gen_range(rng, rect[0].y as f32..rect[1].y as f32);
            let location = Vec2::new(x, y);

            (*self.place_fn)(dataflow, location)
        }
    }
}

pub struct GeneratorResourceDescriptor {
    pub generators: Vec<Box<dyn Generator>>,
}

// resource

pub struct GeneratorResource {
    generators: Vec<Box<dyn Generator>>,
    min_rect: Option<[IVec2; 2]>,
    visited: ahash::AHashSet<IVec2>,
}

impl GeneratorResource {
    pub fn new(desc: GeneratorResourceDescriptor) -> Self {
        Self {
            generators: desc.generators,
            min_rect: Default::default(),
            visited: ahash::AHashSet::new(),
        }
    }
}

impl Resource for GeneratorResource {}

// system

pub struct GeneratorSystem;

impl GeneratorSystem {
    const CHUNK_SIZE: u32 = 32;

    pub fn generate(dataflow: &mut Dataflow, min_rect: [Vec2; 2]) -> Result<(), DataflowError> {
        let resource = dataflow.find_resources::<GeneratorResource>()?;
        let mut resource = resource.borrow_mut()?;

        let chunk_size = Vec2::splat(Self::CHUNK_SIZE as f32);
        let min_rect = [
            min_rect[0].div_euclid(chunk_size).as_ivec2(),
            min_rect[1].div_euclid(chunk_size).as_ivec2(),
        ];

        if Some(min_rect) != resource.min_rect {
            for y in min_rect[0].y..=min_rect[1].y {
                for x in min_rect[0].x..=min_rect[1].x {
                    let chunk_location = IVec2::new(x, y);

                    if resource.visited.contains(&chunk_location) {
                        continue;
                    }

                    let p0 = chunk_location * Self::CHUNK_SIZE as i32;
                    let p1 = (chunk_location + IVec2::ONE) * Self::CHUNK_SIZE as i32;
                    let rect = [p0, p1];
                    for generator in &resource.generators {
                        generator.generate_chunk(dataflow, rect);
                    }

                    resource.visited.insert(chunk_location);
                }
            }

            resource.min_rect = Some(min_rect);
        }

        Ok(())
    }
}
