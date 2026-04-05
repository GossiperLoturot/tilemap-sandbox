use glam::*;
use native_core::*;

use crate::addon::CallbackResource;

// resource

pub enum PlayerState {
    Wait,
    Move,
}

pub struct PlayerResource {
    current: Option<dataflow::EntityId>,
    inventory_id: Option<dataflow::InventoryId>,
    input: Option<Vec2>,
    state: PlayerState,
    move_speed: f32,
    reverse: bool,
}

impl PlayerResource {
    pub fn new() -> Self {
        Self {
            current: Default::default(),
            inventory_id: Default::default(),
            input: Default::default(),
            state: PlayerState::Wait,
            move_speed: 2.0,
            reverse: false,
        }
    }
}

impl dataflow::Resource for PlayerResource {}

// event handler

pub struct PlayerEventHandler;

impl dataflow::EventHandler<dataflow::EntityId> for PlayerEventHandler {
    fn on_insert(&self, dataflow: &mut dataflow::Dataflow, id: dataflow::EntityId) {
        let resource = dataflow.find_resources::<PlayerResource>().unwrap();
        let mut resource = resource.borrow_mut().map_err(dataflow::DataflowError::from).unwrap();

        if resource.current.is_some() {
            panic!("player is already exist.");
        }
        resource.current = Some(id);

        let inventory_id = dataflow.insert_inventory(dataflow::Inventory { max_variety: 32, max_stack: 64 }).unwrap();
        resource.inventory_id = Some(inventory_id);
    }

    fn on_remove(&self, dataflow: &mut dataflow::Dataflow, _: dataflow::EntityId) {
        let resource = dataflow.find_resources::<PlayerResource>().unwrap();
        let mut resource = resource.borrow_mut().map_err(dataflow::DataflowError::from).unwrap();

        if resource.current.is_none() {
            panic!("player is already no exist.");
        }
        resource.current = None;

        let inventory_id = resource.inventory_id.unwrap();
        dataflow.remove_inventory(inventory_id).unwrap();
    }

    fn on_use(&self, dataflow: &mut dataflow::Dataflow, id: dataflow::EntityId) {
        let resource = dataflow.find_resources::<PlayerResource>().unwrap();
        let resource = resource.borrow().map_err(dataflow::DataflowError::from).unwrap();

        let callback = dataflow.find_resources::<CallbackResource>().unwrap();
        let callback = callback.borrow().map_err(dataflow::DataflowError::from).unwrap();
        callback.callback.call(&[]);
    }
}

// system

pub struct PlayerSystem;

impl PlayerSystem {
    pub fn process(dataflow: &mut dataflow::Dataflow, delta_secs: f32) -> Result<(), PlayerError> {
        let resource = dataflow.find_resources::<PlayerResource>()?;
        let mut resource = resource.borrow_mut().map_err(dataflow::DataflowError::from)?;

        let entity_id = resource.current.ok_or(PlayerError::NotFound)?;
        let mut entity = dataflow.get_entity(entity_id).unwrap().clone();

        if let Some(input) = resource.input.take() {
            let is_move = input.length_squared() > f32::EPSILON;

            if is_move {
                let new_coord = entity.coord + resource.move_speed * input * delta_secs;

                entity.coord = new_coord;

                if input.x < 0.0 {
                    resource.reverse = true;
                } else if input.x > 0.0 {
                    resource.reverse = false;
                }
            }

            match resource.state {
                PlayerState::Wait if is_move => {
                    entity.variant = 0b0010;
                    entity.tick = dataflow.get_tick() as u32;
                    resource.state = PlayerState::Move;
                }
                PlayerState::Move if !is_move => {
                    entity.variant = 0b0000;
                    entity.tick = dataflow.get_tick() as u32;
                    resource.state = PlayerState::Wait;
                }
                _ => {}
            }

            entity.variant = (entity.variant & 0b1111_1110) | if resource.reverse { 0b0000_0001 } else { 0b0000_0000 };
        }

        dataflow.move_entity(entity_id, entity.coord).unwrap();
        dataflow.modify_entity_variant(entity_id, entity.variant).unwrap();
        dataflow.modify_entity_tick(entity_id, entity.tick).unwrap();

        Ok(())
    }

    pub fn queue_input(dataflow: &mut dataflow::Dataflow, input: Vec2) -> Result<(), PlayerError> {
        let resource = dataflow.find_resources::<PlayerResource>()?;
        let mut resource = resource.borrow_mut().map_err(dataflow::DataflowError::from)?;

        if resource.input.is_some() {
            return Err(PlayerError::AlreadyExist);
        }
        resource.input = Some(input);
        Ok(())
    }
}

// spawn mod

pub struct PlayerSpawnResource {
    pub archetype_id: u16
}

impl dataflow::Resource for PlayerSpawnResource {}

pub struct PlayerSpawnSystem;

impl PlayerSpawnSystem {
    pub fn spawn(dataflow: &mut dataflow::Dataflow) -> Result<(), PlayerError> {
        let resource = dataflow.find_resources::<PlayerSpawnResource>()?;
        let resource = resource.borrow().map_err(dataflow::DataflowError::from)?;

        dataflow.insert_entity(dataflow::Entity {
            archetype_id: resource.archetype_id,
            ..Default::default()
        })?;

        Ok(())
    }
}

// error handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerError {
    DataflowError(dataflow::DataflowError),
    AlreadyExist,
    NotFound,
}

impl std::fmt::Display for PlayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DataflowError(e) => e.fmt(f),
            Self::AlreadyExist => write!(f, "already exist error"),
            Self::NotFound => write!(f, "not found error"),
        }
    }
}

impl std::error::Error for PlayerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DataflowError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<dataflow::DataflowError> for PlayerError {
    fn from(e: dataflow::DataflowError) -> Self {
        Self::DataflowError(e)
    }
}
