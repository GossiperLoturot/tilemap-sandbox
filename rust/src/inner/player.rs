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

    pub fn insert_entity(&mut self, entity_key: EntityKey) -> Result<(), RootError> {
        if self.current.is_some() {
            return Err(PlayerError::AlreadyExist.into());
        }
        self.current = Some(entity_key);
        Ok(())
    }

    pub fn remove_entity(&mut self) -> Result<EntityKey, RootError> {
        self.current.take().ok_or(PlayerError::NotFound.into())
    }

    pub fn get_entity(&self) -> Result<EntityKey, RootError> {
        self.current.ok_or(PlayerError::NotFound.into())
    }

    pub fn push_input(&mut self, input: Vec2) -> Result<(), RootError> {
        if self.input.is_some() {
            return Err(PlayerError::AlreadyExist.into());
        }
        self.input = Some(input);
        Ok(())
    }

    pub fn pop_input(&mut self) -> Result<Vec2, RootError> {
        self.input.take().ok_or(PlayerError::NotFound.into())
    }

    pub fn get_input(&self) -> Result<Vec2, RootError> {
        self.input.ok_or(PlayerError::NotFound.into())
    }

    pub fn get_location(&mut self, root: &inner::Root) -> Result<Vec2, RootError> {
        let current = self.current.ok_or(PlayerError::NotFound)?;
        let location = root.entity_get(current)?.location;
        Ok(location)
    }

    pub fn get_inventory_key(&mut self, root: &inner::Root) -> Result<InventoryKey, RootError> {
        let current = self.current.ok_or(PlayerError::NotFound)?;
        let inventory_key = root.entity_get_inventory(current)?.unwrap();
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
        let inventory_key = root.item_insert_inventory(self.inventory_id).unwrap();

        root.entity_modify(key, |entity| {
            entity.data = Box::new(PlayerEntityData {
                state: PlayerEntityDataState::Wait,
                inventory_key,
            });
        })
        .unwrap();

        root.player_insert_entity(key).unwrap();
    }

    fn before_break(&self, root: &mut Root, key: EntityKey) {
        let entity = root.entity_get(key).unwrap();

        let data = entity.data.downcast_ref::<PlayerEntityData>().unwrap();

        let inventory_key = data.inventory_key;
        root.item_remove_inventory(inventory_key).unwrap();

        root.player_remove_entity().unwrap();
    }

    fn forward(&self, root: &mut Root, key: EntityKey, delta_secs: f32) {
        let mut entity = root.entity_get(key).unwrap().clone();

        let data = entity.data.downcast_mut::<PlayerEntityData>().unwrap();

        // consume input
        if let Ok(input) = root.player_pop_input() {
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
                        entity.render_param.tick = root.time_tick() as u32;
                        data.state = PlayerEntityDataState::Move;
                    }
                }
                PlayerEntityDataState::Move => {
                    if !is_move {
                        entity.render_param.variant = 0;
                        entity.render_param.tick = root.time_tick() as u32;
                        data.state = PlayerEntityDataState::Wait;
                    }
                }
            }
        }

        root.player_remove_entity().unwrap();
        let key = root.entity_modify(key, move |e| *e = entity).unwrap();
        root.player_insert_entity(key).unwrap();
    }

    fn has_inventory(&self, _root: &Root, _key: EntityKey) -> bool {
        true
    }

    fn get_inventory(&self, root: &Root, key: EntityKey) -> Option<InventoryKey> {
        let entity = root.entity_get(key).unwrap();

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
) -> Result<bool, FieldError> {
    let entity = root.entity_get(entity_key)?;
    let base_rect = root.entity_get_base_collision_rect(entity.id)?;

    #[rustfmt::skip]
    let rect = [
        new_location + base_rect[0],
        new_location + base_rect[1],
    ];

    if root.tile_has_by_collision_rect(rect) {
        return Ok(true);
    }

    if root.block_has_by_collision_rect(rect) {
        return Ok(true);
    }

    let intersect = root
        .entity_get_by_collision_rect(rect)
        .any(|other_key| other_key != entity_key);
    Ok(intersect)
}

// error handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerError {
    AlreadyExist,
    NotFound,
}

impl std::fmt::Display for PlayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyExist => write!(f, "already exist error"),
            Self::NotFound => write!(f, "not found error"),
        }
    }
}

impl std::error::Error for PlayerError {}
