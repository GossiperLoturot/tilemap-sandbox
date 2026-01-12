use std::rc::Rc;

use glam::*;
use native_core::dataflow::*;

use super::*;

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
pub struct AnimalEntityFeatureSet {
    pub min_rest_secs: f32,
    pub max_rest_secs: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub speed: f32,
    pub idle_variant: u8,
    pub walk_variant: u8,
}

impl FeatureSet for AnimalEntityFeatureSet {
    fn attach_set(&self, b: &mut FeatureSetBuilder) -> Result<(), FeatureError> {
        let slf = Rc::new(self.clone());
        b.insert::<Rc<dyn FieldFeature<Key = EntityId>>>(slf.clone())?;
        b.insert::<Rc<dyn ForwardFeature<Key = EntityId>>>(slf.clone())?;
        Ok(())
    }
}

impl FieldFeature for AnimalEntityFeatureSet {
    type Key = EntityId;

    fn after_place(&self, dataflow: &mut Dataflow, key: Self::Key) {
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

    fn before_break(&self, _dataflow: &mut Dataflow, _key: Self::Key) {}
}

impl ForwardFeature for AnimalEntityFeatureSet {
    type Key = EntityId;

    fn forward(&self, dataflow: &mut Dataflow, key: EntityId, delta_secs: f32) {
        let mut entity = dataflow.get_entity(key).unwrap().clone();

        let data = entity.data.downcast_mut::<AnimalEntityData>().unwrap();

        let mut rng = rand::thread_rng();
        match data.state {
            AnimalEntityDataState::Init => {
                data.state = AnimalEntityDataState::WaitStart;
            }
            AnimalEntityDataState::WaitStart => {
                entity.render_state.variant = self.idle_variant;
                entity.render_state.tick = dataflow.get_tick() as u32;

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
                entity.render_state.variant = self.walk_variant;
                entity.render_state.tick = dataflow.get_tick() as u32;

                let angle = rand::Rng::gen_range(&mut rng, 0.0..std::f32::consts::PI * 2.0);
                let distance = rand::Rng::gen_range(&mut rng, data.min_distance..data.max_distance);
                let destination = entity.coord + Vec2::from_angle(angle) * distance;
                data.state = AnimalEntityDataState::Trip(destination);
            }
            AnimalEntityDataState::Trip(destination) => {
                if entity.coord != destination {
                    let difference = destination - entity.coord;
                    let distance = difference.length();
                    let direction = difference / distance;
                    let velocity = distance.min(data.speed * delta_secs);
                    let location = entity.coord + direction * velocity;

                    if intersection_guard(dataflow, key, location).unwrap() {
                        data.state = AnimalEntityDataState::WaitStart;
                    } else {
                        entity.coord = location;
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
    entity_key: EntityId,
    new_location: Vec2,
) -> Result<bool, DataflowError> {
    let entity = dataflow.get_entity(entity_key)?;
    let base_rect = dataflow.get_entity_base_collision_rect(entity.archetype_id)?;

    #[rustfmt::skip]
    let rect = [
        new_location + base_rect[0],
        new_location + base_rect[1],
    ];

    // TODO: enable tile collision check
    // if dataflow.has_tile_by_collision_rect(rect) {
    //     return Ok(true);
    // }

    // TODO: enable block collision check
    // if dataflow.has_block_by_collision_rect(rect) {
    //     return Ok(true);
    // }

    let intersect = dataflow
        .get_entity_ids_by_collision_rect(rect)
        .any(|other_key| other_key != entity_key);
    Ok(intersect)
}
