use glam::*;
use native_core::dataflow::*;

// resource

#[derive(Debug, Default)]
pub struct PlayerResource {
    current: Option<EntityKey>,
    input: Option<Vec2>,
}

impl PlayerResource {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }
}

impl Resource for PlayerResource {}

// system

pub struct PlayerSystem;

impl PlayerSystem {
    pub fn insert_entity(
        dataflow: &mut Dataflow,
        entity_key: EntityKey,
    ) -> Result<(), PlayerError> {
        let resource = dataflow.find_resources::<PlayerResource>()?;
        let mut resource = resource.borrow_mut().map_err(DataflowError::from)?;

        if resource.current.is_some() {
            return Err(PlayerError::AlreadyExist);
        }
        resource.current = Some(entity_key);
        Ok(())
    }

    pub fn remove_entity(dataflow: &mut Dataflow) -> Result<EntityKey, PlayerError> {
        let resource = dataflow.find_resources::<PlayerResource>()?;
        let mut resource = resource.borrow_mut().map_err(DataflowError::from)?;

        resource.current.take().ok_or(PlayerError::NotFound)
    }

    pub fn get_entity(dataflow: &Dataflow) -> Result<EntityKey, PlayerError> {
        let resource = dataflow.find_resources::<PlayerResource>()?;
        let resource = resource.borrow().map_err(DataflowError::from)?;

        resource.current.ok_or(PlayerError::NotFound)
    }

    pub fn push_input(dataflow: &mut Dataflow, input: Vec2) -> Result<(), PlayerError> {
        let resource = dataflow.find_resources::<PlayerResource>()?;
        let mut resource = resource.borrow_mut().map_err(DataflowError::from)?;

        if resource.input.is_some() {
            return Err(PlayerError::AlreadyExist);
        }
        resource.input = Some(input);
        Ok(())
    }

    pub fn pop_input(dataflow: &mut Dataflow) -> Result<Vec2, PlayerError> {
        let resource = dataflow.find_resources::<PlayerResource>()?;
        let mut resource = resource.borrow_mut().map_err(DataflowError::from)?;

        resource.input.take().ok_or(PlayerError::NotFound)
    }

    pub fn get_input(dataflow: &Dataflow) -> Result<Vec2, PlayerError> {
        let resource = dataflow.find_resources::<PlayerResource>()?;
        let resource = resource.borrow().map_err(DataflowError::from)?;

        resource.input.ok_or(PlayerError::NotFound)
    }

    pub fn get_location(dataflow: &Dataflow) -> Result<Vec2, PlayerError> {
        let resource = dataflow.find_resources::<PlayerResource>()?;
        let resource = resource.borrow().map_err(DataflowError::from)?;

        let current = resource.current.ok_or(PlayerError::NotFound)?;
        let location = dataflow.get_entity(current)?.location;
        Ok(location)
    }

    pub fn get_inventory_key(dataflow: &Dataflow) -> Result<InventoryKey, PlayerError> {
        let resource = dataflow.find_resources::<PlayerResource>()?;
        let resource = resource.borrow().map_err(DataflowError::from)?;

        let current = resource.current.ok_or(PlayerError::NotFound)?;
        let inventory_key = dataflow.get_inventory_by_entity(current)?.unwrap();
        Ok(inventory_key)
    }
}

// feature

#[derive(Debug, Clone)]
pub enum PlayerEntityDataState {
    Wait,
    Move,
}

#[derive(Debug, Clone)]
pub struct PlayerEntityData {
    pub state: PlayerEntityDataState,
    pub inventory_key: InventoryKey,
}

impl EntityData for PlayerEntityData {}

#[derive(Debug, Clone)]
pub struct PlayerEntityFeature {
    pub move_speed: f32,
    pub inventory_id: u16,
}

impl EntityFeature for PlayerEntityFeature {
    fn after_place(&self, dataflow: &mut Dataflow, key: EntityKey) {
        let inventory_key = dataflow.insert_inventory(self.inventory_id).unwrap();

        dataflow
            .modify_entity(key, |entity| {
                entity.data = Box::new(PlayerEntityData {
                    state: PlayerEntityDataState::Wait,
                    inventory_key,
                });
            })
            .unwrap();

        PlayerSystem::insert_entity(dataflow, key).unwrap();
    }

    fn before_break(&self, dataflow: &mut Dataflow, key: EntityKey) {
        let entity = dataflow.get_entity(key).unwrap();

        let data = entity.data.downcast_ref::<PlayerEntityData>().unwrap();

        let inventory_key = data.inventory_key;
        dataflow.remove_inventory(inventory_key).unwrap();

        PlayerSystem::remove_entity(dataflow).unwrap();
    }

    fn forward(&self, dataflow: &mut Dataflow, key: EntityKey, delta_secs: f32) {
        let mut entity = dataflow.get_entity(key).unwrap().clone();

        let data = entity.data.downcast_mut::<PlayerEntityData>().unwrap();

        // consume input
        if let Ok(input) = PlayerSystem::pop_input(dataflow) {
            let is_move = input.length_squared() > f32::EPSILON;

            if is_move {
                let location = entity.location + self.move_speed * input * delta_secs;

                if !intersection_guard(dataflow, key, location).unwrap() {
                    entity.location = location;
                }
            }

            match data.state {
                PlayerEntityDataState::Wait => {
                    if is_move {
                        entity.render_param.variant = 1;
                        entity.render_param.tick = dataflow.get_tick() as u32;
                        data.state = PlayerEntityDataState::Move;
                    }
                }
                PlayerEntityDataState::Move => {
                    if !is_move {
                        entity.render_param.variant = 0;
                        entity.render_param.tick = dataflow.get_tick() as u32;
                        data.state = PlayerEntityDataState::Wait;
                    }
                }
            }
        }

        PlayerSystem::remove_entity(dataflow).unwrap();
        let key = dataflow.modify_entity(key, move |e| *e = entity).unwrap();
        PlayerSystem::insert_entity(dataflow, key).unwrap();
    }

    fn has_inventory(&self, _dataflow: &Dataflow, _key: EntityKey) -> bool {
        true
    }

    fn get_inventory(&self, dataflow: &Dataflow, key: EntityKey) -> Option<InventoryKey> {
        let entity = dataflow.get_entity(key).unwrap();

        let data = entity.data.downcast_ref::<PlayerEntityData>().unwrap();

        Some(data.inventory_key)
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

// error handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerError {
    DataflowError(DataflowError),
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

impl From<DataflowError> for PlayerError {
    fn from(e: DataflowError) -> Self {
        Self::DataflowError(e)
    }
}
