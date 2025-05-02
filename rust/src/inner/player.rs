use crate::inner;

use super::*;

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
    pub fn insert_entity(root: &mut Root, entity_key: EntityKey) -> Result<(), PlayerError> {
        let resource = root.find_resources::<PlayerResource>()?;
        let mut resource = resource.borrow_mut().map_err(RootError::from)?;

        if resource.current.is_some() {
            return Err(PlayerError::AlreadyExist);
        }
        resource.current = Some(entity_key);
        Ok(())
    }

    pub fn remove_entity(root: &mut Root) -> Result<EntityKey, PlayerError> {
        let resource = root.find_resources::<PlayerResource>()?;
        let mut resource = resource.borrow_mut().map_err(RootError::from)?;

        resource.current.take().ok_or(PlayerError::NotFound)
    }

    pub fn get_entity(root: &Root) -> Result<EntityKey, PlayerError> {
        let resource = root.find_resources::<PlayerResource>()?;
        let resource = resource.borrow().map_err(RootError::from)?;

        resource.current.ok_or(PlayerError::NotFound)
    }

    pub fn push_input(root: &mut Root, input: Vec2) -> Result<(), PlayerError> {
        let resource = root.find_resources::<PlayerResource>()?;
        let mut resource = resource.borrow_mut().map_err(RootError::from)?;

        if resource.input.is_some() {
            return Err(PlayerError::AlreadyExist);
        }
        resource.input = Some(input);
        Ok(())
    }

    pub fn pop_input(root: &mut Root) -> Result<Vec2, PlayerError> {
        let resource = root.find_resources::<PlayerResource>()?;
        let mut resource = resource.borrow_mut().map_err(RootError::from)?;

        resource.input.take().ok_or(PlayerError::NotFound)
    }

    pub fn get_input(root: &Root) -> Result<Vec2, PlayerError> {
        let resource = root.find_resources::<PlayerResource>()?;
        let resource = resource.borrow().map_err(RootError::from)?;

        resource.input.ok_or(PlayerError::NotFound)
    }

    pub fn get_location(root: &inner::Root) -> Result<Vec2, PlayerError> {
        let resource = root.find_resources::<PlayerResource>()?;
        let resource = resource.borrow().map_err(RootError::from)?;

        let current = resource.current.ok_or(PlayerError::NotFound)?;
        let location = root.get_entity(current)?.location;
        Ok(location)
    }

    pub fn get_inventory_key(root: &inner::Root) -> Result<InventoryKey, PlayerError> {
        let resource = root.find_resources::<PlayerResource>()?;
        let resource = resource.borrow().map_err(RootError::from)?;

        let current = resource.current.ok_or(PlayerError::NotFound)?;
        let inventory_key = root.get_inventory_by_entity(current)?.unwrap();
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
    fn after_place(&self, root: &mut Root, key: EntityKey) {
        let inventory_key = root.insert_inventory(self.inventory_id).unwrap();

        root.modify_entity(key, |entity| {
            entity.data = Box::new(PlayerEntityData {
                state: PlayerEntityDataState::Wait,
                inventory_key,
            });
        })
        .unwrap();

        PlayerSystem::insert_entity(root, key).unwrap();
    }

    fn before_break(&self, root: &mut Root, key: EntityKey) {
        let entity = root.get_entity(key).unwrap();

        let data = entity.data.downcast_ref::<PlayerEntityData>().unwrap();

        let inventory_key = data.inventory_key;
        root.remove_inventory(inventory_key).unwrap();

        PlayerSystem::remove_entity(root).unwrap();
    }

    fn forward(&self, root: &mut Root, key: EntityKey, delta_secs: f32) {
        let mut entity = root.get_entity(key).unwrap().clone();

        let data = entity.data.downcast_mut::<PlayerEntityData>().unwrap();

        // consume input
        if let Ok(input) = PlayerSystem::pop_input(root) {
            let is_move = input.length_squared() > f32::EPSILON;

            if is_move {
                let location = entity.location + self.move_speed * input * delta_secs;

                if !intersection_guard(root, key, location).unwrap() {
                    entity.location = location;
                }
            }

            match data.state {
                PlayerEntityDataState::Wait => {
                    if is_move {
                        entity.render_param.variant = 1;
                        entity.render_param.tick = root.get_tick() as u32;
                        data.state = PlayerEntityDataState::Move;
                    }
                }
                PlayerEntityDataState::Move => {
                    if !is_move {
                        entity.render_param.variant = 0;
                        entity.render_param.tick = root.get_tick() as u32;
                        data.state = PlayerEntityDataState::Wait;
                    }
                }
            }
        }

        PlayerSystem::remove_entity(root).unwrap();
        let key = root.modify_entity(key, move |e| *e = entity).unwrap();
        PlayerSystem::insert_entity(root, key).unwrap();
    }

    fn has_inventory(&self, _root: &Root, _key: EntityKey) -> bool {
        true
    }

    fn get_inventory(&self, root: &Root, key: EntityKey) -> Option<InventoryKey> {
        let entity = root.get_entity(key).unwrap();

        let data = entity.data.downcast_ref::<PlayerEntityData>().unwrap();

        Some(data.inventory_key)
    }
}

// intersection guard
// DUPLICATE: src/inner/player.rs
fn intersection_guard(
    root: &mut Root,
    entity_key: EntityKey,
    new_location: Vec2,
) -> Result<bool, RootError> {
    let entity = root.get_entity(entity_key)?;
    let base_rect = root.get_entity_base_collision_rect(entity.id)?;

    #[rustfmt::skip]
    let rect = [
        new_location + base_rect[0],
        new_location + base_rect[1],
    ];

    if root.has_tile_by_collision_rect(rect) {
        return Ok(true);
    }

    if root.has_block_by_collision_rect(rect) {
        return Ok(true);
    }

    let intersect = root
        .get_entity_by_collision_rect(rect)
        .any(|other_key| other_key != entity_key);
    Ok(intersect)
}

// error handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerError {
    RootError(RootError),
    AlreadyExist,
    NotFound,
}

impl std::fmt::Display for PlayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RootError(e) => e.fmt(f),
            Self::AlreadyExist => write!(f, "already exist error"),
            Self::NotFound => write!(f, "not found error"),
        }
    }
}

impl std::error::Error for PlayerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RootError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<RootError> for PlayerError {
    fn from(e: RootError) -> Self {
        Self::RootError(e)
    }
}
