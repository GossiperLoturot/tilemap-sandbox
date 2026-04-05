use glam::*;
use native_core::*;

const IDLE_VARIANT: u16 = 0;
const WALK_VARIANT: u16 = 1;

// resource

pub enum AnimalDataState {
    Init,
    WaitStart,
    Wait(f32),
    TripStart,
    Trip(Vec2),
}

pub struct AnimalData {
    pub entity_id: dataflow::EntityId,
    pub min_rest_secs: f32,
    pub max_rest_secs: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub speed: f32,
    pub state: AnimalDataState,
}

pub struct AnimalResource {
    storage: Vec<AnimalData>,
}

impl AnimalResource {
    pub fn new() -> Self {
        Self { storage: Default::default() }
    }
}

impl dataflow::Resource for AnimalResource {}

// event handler

pub struct AnimalEventHandler;

impl dataflow::EventHandler<dataflow::EntityId> for AnimalEventHandler {
    fn on_insert(&self, dataflow: &mut dataflow::Dataflow, id: dataflow::EntityId) {
        let resource = dataflow.find_resources::<AnimalResource>().unwrap();
        let mut resource = resource.borrow_mut().map_err(dataflow::DataflowError::from).unwrap();

        resource.storage.push(AnimalData {
            entity_id: id,
            min_rest_secs: 1.0,
            max_rest_secs: 4.0,
            min_distance: 1.0,
            max_distance: 8.0,
            speed: 1.0,
            state: AnimalDataState::Init,
        });
    }

    fn on_remove(&self, dataflow: &mut dataflow::Dataflow, id: dataflow::EntityId) {
        let resource = dataflow.find_resources::<AnimalResource>().unwrap();
        let mut resource = resource.borrow_mut().map_err(dataflow::DataflowError::from).unwrap();

        let index = resource.storage.iter().position(|data| data.entity_id == id).unwrap();
        resource.storage.swap_remove(index);
    }

    fn on_use(&self, _: &mut dataflow::Dataflow, _: dataflow::EntityId) { }
}

// system

pub struct AnimalSystem;

impl AnimalSystem {
    pub fn process(dataflow: &mut dataflow::Dataflow, delta_secs: f32) -> Result<(), dataflow::DataflowError> {
        let resource = dataflow.find_resources::<AnimalResource>()?;
        let mut resource = resource.borrow_mut().map_err(dataflow::DataflowError::from)?;

        let mut rng = rand::thread_rng();
        for data in resource.storage.iter_mut() {
            let entity = dataflow.get_entity(data.entity_id).unwrap().clone();

            match data.state {
                AnimalDataState::Init => {
                    data.state = AnimalDataState::WaitStart;
                }
                AnimalDataState::WaitStart => {
                    dataflow.modify_entity_variant(data.entity_id, IDLE_VARIANT).unwrap();
                    dataflow.modify_entity_tick(data.entity_id, dataflow.get_tick() as u32).unwrap();
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
                    dataflow.modify_entity_variant(data.entity_id, WALK_VARIANT).unwrap();
                    dataflow.modify_entity_tick(data.entity_id, dataflow.get_tick() as u32).unwrap();
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
                        let new_coord = entity.coord + direction * velocity;
                        dataflow.move_entity(data.entity_id, new_coord).unwrap();
                    } else {
                        data.state = AnimalDataState::WaitStart;
                    }
                }
            }
        }

        Ok(())
    }
}
