use glam::*;
use native_core::*;

const IDLE_VARIANT: u8 = 0;
const WALK_VARIANT: u8 = 1;

// resource

#[derive(Debug)]
pub enum AnimalDataState {
    Init,
    WaitStart,
    Wait(f32),
    TripStart,
    Trip(Vec2),
}

#[derive(Debug)]
pub struct AnimalData {
    pub entity_id: dataflow::EntityId,
    pub min_rest_secs: f32,
    pub max_rest_secs: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub speed: f32,
    pub state: AnimalDataState,
}

#[derive(Debug)]
pub struct AnimalResource {
    pub storage: Vec<AnimalData>,
}

impl dataflow::Resource for AnimalResource {}

// system

pub struct AnimalSystem;

impl AnimalSystem {
    pub fn insert(dataflow: &mut dataflow::Dataflow) -> Result<(), dataflow::DataflowError> {
        let resource = AnimalResource {
            storage: Default::default(),
        };
        dataflow.insert_resources(resource)?;
        Ok(())
    }

    pub fn attach_entity(dataflow: &mut dataflow::Dataflow, entity_id: dataflow::EntityId) -> Result<(), dataflow::DataflowError> {
        let resource = dataflow.find_resources::<AnimalResource>()?;
        let mut resource = resource.borrow_mut().map_err(dataflow::DataflowError::from)?;

        resource.storage.push(AnimalData {
            entity_id,
            min_rest_secs: 1.0,
            max_rest_secs: 4.0,
            min_distance: 1.0,
            max_distance: 8.0,
            speed: 1.0,
            state: AnimalDataState::Init,
        });
        Ok(())
    }

    pub fn process(dataflow: &mut dataflow::Dataflow, delta_secs: f32) -> Result<(), dataflow::DataflowError> {
        let resource = dataflow.find_resources::<AnimalResource>()?;
        let mut resource = resource.borrow_mut().map_err(dataflow::DataflowError::from)?;

        let mut rng = rand::thread_rng();
        for data in resource.storage.iter_mut() {
            let mut entity = dataflow.get_entity(data.entity_id).unwrap().clone();

            match data.state {
                AnimalDataState::Init => {
                    data.state = AnimalDataState::WaitStart;
                }
                AnimalDataState::WaitStart => {
                    let tick = dataflow.get_tick() as u32;
                    let _ = dataflow.modify_entity(data.entity_id, |entity| {
                        entity.variant = IDLE_VARIANT;
                        entity.tick = tick;
                    });

                    let wait_secs = rand::Rng::gen_range(&mut rng, data.min_rest_secs..data.max_rest_secs);
                    data.state = AnimalDataState::Wait(wait_secs);
                }
                AnimalDataState::Wait(wait_secs) => {
                    if wait_secs <= 0.0 {
                        data.state = AnimalDataState::TripStart;
                    } else {
                        let new_wait_secs = wait_secs - delta_secs;
                        data.state = AnimalDataState::Wait(new_wait_secs);
                    }
                }
                AnimalDataState::TripStart => {
                    let tick = dataflow.get_tick() as u32;
                    let _ = dataflow.modify_entity(data.entity_id, |entity| {
                        entity.variant = WALK_VARIANT;
                        entity.tick = tick;
                    });

                    let angle = rand::Rng::gen_range(&mut rng, 0.0..std::f32::consts::PI * 2.0);
                    let distance = rand::Rng::gen_range(&mut rng, data.min_distance..data.max_distance);
                    let destination = entity.coord + Vec2::from_angle(angle) * distance;
                    data.state = AnimalDataState::Trip(destination);
                }
                AnimalDataState::Trip(destination) => {
                    if entity.coord != destination {
                        let difference = destination - entity.coord;
                        let distance = difference.length();
                        let direction = difference / distance;
                        let velocity = distance.min(data.speed * delta_secs);
                        let location = entity.coord + direction * velocity;

                        entity.coord = location;

                        dataflow.remove_entity(data.entity_id).unwrap();
                        data.entity_id = dataflow.insert_entity(entity).unwrap();
                    } else {
                        data.state = AnimalDataState::WaitStart;
                    }
                }
            }
        }

        Ok(())
    }
}
