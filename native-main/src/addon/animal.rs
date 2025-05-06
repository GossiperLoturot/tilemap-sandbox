use glam::*;
use native_core::dataflow::*;

#[derive(Debug, Clone)]
pub enum AnimalEntityDataState {
    Init,
    WaitStart,
    Wait(f32),
    TripStart,
    Trip(Vec2),
}

#[derive(Debug, Clone)]
pub struct AnimalEntityData {
    pub min_rest_secs: f32,
    pub max_rest_secs: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub speed: f32,
    pub state: AnimalEntityDataState,
}

impl EntityData for AnimalEntityData {}

#[derive(Debug, Clone)]
pub struct AnimalEntityFeature {
    pub min_rest_secs: f32,
    pub max_rest_secs: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub speed: f32,
    pub idle_variant: u8,
    pub walk_variant: u8,
}

impl EntityFeature for AnimalEntityFeature {
    fn after_place(&self, dataflow: &mut Dataflow, key: EntityKey) {
        dataflow
            .modify_entity(key, |entity| {
                entity.data = Box::new(AnimalEntityData {
                    min_rest_secs: self.min_rest_secs,
                    max_rest_secs: self.max_rest_secs,
                    min_distance: self.min_distance,
                    max_distance: self.max_distance,
                    speed: self.speed,
                    state: AnimalEntityDataState::Init,
                });
            })
            .unwrap();
    }

    fn forward(&self, dataflow: &mut Dataflow, key: EntityKey, delta_secs: f32) {
        let mut entity = dataflow.get_entity(key).unwrap().clone();

        let data = entity.data.downcast_mut::<AnimalEntityData>().unwrap();

        let mut rng = rand::thread_rng();
        match data.state {
            AnimalEntityDataState::Init => {
                data.state = AnimalEntityDataState::WaitStart;
            }
            AnimalEntityDataState::WaitStart => {
                entity.render_param.variant = self.idle_variant;
                entity.render_param.tick = dataflow.get_tick() as u32;

                let secs = rand::Rng::gen_range(&mut rng, data.min_rest_secs..data.max_rest_secs);
                data.state = AnimalEntityDataState::Wait(secs);
            }
            AnimalEntityDataState::Wait(secs) => {
                if secs <= 0.0 {
                    data.state = AnimalEntityDataState::TripStart;
                } else {
                    let new_secs = secs - delta_secs;
                    data.state = AnimalEntityDataState::Wait(new_secs);
                }
            }
            AnimalEntityDataState::TripStart => {
                entity.render_param.variant = self.walk_variant;
                entity.render_param.tick = dataflow.get_tick() as u32;

                let angle = rand::Rng::gen_range(&mut rng, 0.0..std::f32::consts::PI * 2.0);
                let distance = rand::Rng::gen_range(&mut rng, data.min_distance..data.max_distance);
                let destination = entity.location + Vec2::from_angle(angle) * distance;
                data.state = AnimalEntityDataState::Trip(destination);
            }
            AnimalEntityDataState::Trip(destination) => {
                if entity.location != destination {
                    let difference = destination - entity.location;
                    let distance = difference.length();
                    let direction = difference / distance;
                    let velocity = distance.min(data.speed * delta_secs);
                    let location = entity.location + direction * velocity;

                    if intersection_guard(dataflow, key, location).unwrap() {
                        data.state = AnimalEntityDataState::WaitStart;
                    } else {
                        entity.location = location;
                    }
                } else {
                    data.state = AnimalEntityDataState::WaitStart;
                }
            }
        }

        dataflow.modify_entity(key, move |e| *e = entity).unwrap();
    }
}

// intersection guard
// DUPLICATE: src/inner/player.rs
fn intersection_guard(
    dataflow: &mut Dataflow,
    entity_key: EntityKey,
    new_location: Vec2,
) -> Result<bool, DataflowError> {
    let entity = dataflow.get_entity(entity_key)?;
    let base_rect = dataflow.get_entity_base_collision_rect(entity.id)?;

    #[rustfmt::skip]
    let rect = [
        new_location + base_rect[0],
        new_location + base_rect[1],
    ];

    if dataflow.has_tile_by_collision_rect(rect) {
        return Ok(true);
    }

    if dataflow.has_block_by_collision_rect(rect) {
        return Ok(true);
    }

    let intersect = dataflow
        .get_entity_by_collision_rect(rect)
        .any(|other_key| other_key != entity_key);
    Ok(intersect)
}
