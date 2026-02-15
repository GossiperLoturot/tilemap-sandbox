use glam::*;
use native_core::*;

pub trait Generator {
    fn generate(&self, dataflow: &mut dataflow::Dataflow, broad_rect: IRect2);
}

// method for generating

pub type SampleFn<T> = Box<dyn Fn(&mut dataflow::Dataflow, T)>;

pub struct DiscreteGenerator {
    pub probability: f32,
    pub sample_fn: SampleFn<IVec2>,
}

impl Generator for DiscreteGenerator {
    fn generate(&self, dataflow: &mut dataflow::Dataflow, broad_rect: IRect2) {
        let rng = &mut rand::thread_rng();

        for y in broad_rect.min.y..broad_rect.max.y {
            for x in broad_rect.min.x..broad_rect.max.x {
                let coord = IVec2::new(x, y);

                let value = rand::Rng::gen_range(rng, 0.0..1.0);
                if self.probability < value {
                    continue;
                }

                (*self.sample_fn)(dataflow, coord);
            }
        }
    }
}

pub struct RandomGenerator {
    pub probability: f32,
    pub sample_fn: SampleFn<Vec2>,
}

impl Generator for RandomGenerator {
    fn generate(&self, dataflow: &mut dataflow::Dataflow, broad_rect: IRect2) {
        let rng = &mut rand::thread_rng();

        let generate_count = (broad_rect.volume() as f32 * self.probability) as i32;
        for _ in 0..generate_count {
            let x = rand::Rng::gen_range(rng, broad_rect.min.x as f32..broad_rect.max.x as f32);
            let y = rand::Rng::gen_range(rng, broad_rect.min.y as f32..broad_rect.max.y as f32);
            let coord = Vec2::new(x, y);

            (*self.sample_fn)(dataflow, coord)
        }
    }
}

pub struct GeneratorResourceDescriptor {
    pub generators: Vec<Box<dyn Generator>>,
}

// resource

struct GeneratorResource {
    generators: Vec<Box<dyn Generator>>,
    rect: Option<IRect2>,
    visited: ahash::AHashSet<IVec2>,
}

impl dataflow::Resource for GeneratorResource {}

// system

pub struct GeneratorSystem;

impl GeneratorSystem {
    const CHUNK_SIZE: u32 = 32;

    pub fn insert(dataflow: &mut dataflow::Dataflow, desc: GeneratorResourceDescriptor) -> Result<(), dataflow::DataflowError> {
        let resource = GeneratorResource {
            generators: desc.generators,
            rect: Default::default(),
            visited: Default::default(),
        };
        dataflow.insert_resources(resource)?;
        Ok(())
    }

    pub fn generate(dataflow: &mut dataflow::Dataflow, rect: Rect2) -> Result<(), dataflow::DataflowError> {
        let resource = dataflow.find_resources::<GeneratorResource>()?;
        let mut resource = resource.borrow_mut()?;

        let chunk_size = Vec2::splat(Self::CHUNK_SIZE as f32);
        let rect = IRect2::new(
            rect.min.div_euclid(chunk_size).as_ivec2(),
            rect.max.div_euclid(chunk_size).as_ivec2(),
        );

        if Some(rect) != resource.rect {
            for y in rect.min.y..=rect.max.y {
                for x in rect.min.x..=rect.max.x {
                    let chunk_coord = IVec2::new(x, y);

                    if resource.visited.contains(&chunk_coord) {
                        continue;
                    }

                    let rect = IRect2::new(
                        chunk_coord * Self::CHUNK_SIZE as i32,
                        chunk_coord * Self::CHUNK_SIZE as i32 + Self::CHUNK_SIZE as i32,
                    );
                    for generator in &resource.generators {
                        generator.generate(dataflow, rect);
                    }

                    resource.visited.insert(chunk_coord);
                }
            }

            resource.rect = Some(rect);
        }

        Ok(())
    }
}
