use crate::inner;

use super::*;

// resource

#[derive(Debug, Clone)]
pub struct PlayerResource {
    current: Option<EntityKey>,
    input: Option<Vec2>,
}

impl PlayerResource {
    pub fn init(root: &mut inner::Root) -> Result<(), PlayerError> {
        let resource = Self {
            current: Default::default(),
            input: Default::default(),
        };
        root.resource_insert(resource)?;
        Ok(())
    }

    pub fn input(root: &mut inner::Root, input: Vec2) -> Result<(), PlayerError> {
        let resource = root.resource_get_mut::<Self>()?;
        resource.input = Some(input);
        Ok(())
    }

    pub fn location(root: &inner::Root) -> Result<Vec2, PlayerError> {
        let resource = root.resource_get::<Self>()?;
        let current = resource.current.ok_or(PlayerError::NotFoundPlayer)?;
        let location = root.entity_get(current).unwrap().location;
        Ok(location)
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

#[derive(Debug, Clone)]
pub struct PlayerEntityFeature;

impl PlayerEntityFeature {
    const MOVE_SPEED: f32 = 3.0;
    const INVENTORY_SIZE: u32 = 16;
}

impl EntityFeatureTrait for PlayerEntityFeature {
    fn after_place(&self, root: &mut Root, key: EntityKey) {
        let inventory_key = root
            .inventory_insert(Inventory::new(Self::INVENTORY_SIZE))
            .unwrap();

        root.entity_modify(key, |entity| {
            entity.data = Some(EntityData::Player(PlayerEntityData {
                state: PlayerEntityDataState::Wait,
                inventory_key,
            }))
        })
        .unwrap();

        let resource = root.resource_get_mut::<PlayerResource>().unwrap();
        resource.current = Some(key);
    }

    fn before_break(&self, root: &mut Root, key: EntityKey) {
        let entity = root.entity_get(key).unwrap();

        let Some(EntityData::Player(data)) = &entity.data else {
            unreachable!();
        };

        let inventory_key = data.inventory_key;
        root.inventory_remove(inventory_key).unwrap();

        let resource = root.resource_get_mut::<PlayerResource>().unwrap();
        resource.current = None;
    }

    fn forward(&self, root: &mut Root, key: EntityKey, delta_secs: f32) {
        let mut entity = root.entity_get(key).cloned().unwrap();

        let Some(EntityData::Player(data)) = &mut entity.data else {
            return;
        };

        let resource = root.resource_get_mut::<PlayerResource>().unwrap();

        // consume input
        if let Some(input) = resource.input.take() {
            let is_move = input[0].powi(2) + input[1].powi(2) > f32::EPSILON;

            if is_move {
                let location = [
                    entity.location[0] + Self::MOVE_SPEED * input[0] * delta_secs,
                    entity.location[1] + Self::MOVE_SPEED * input[1] * delta_secs,
                ];

                if !intersection_guard(root, key, location) {
                    entity.location = location;
                }
            }

            match data.state {
                PlayerEntityDataState::Wait => {
                    if is_move {
                        entity.render_param.variant = Some(1);
                        entity.render_param.tick = Some(root.time_tick() as u32);
                        data.state = PlayerEntityDataState::Move;
                    }
                }
                PlayerEntityDataState::Move => {
                    if !is_move {
                        entity.render_param.variant = Some(0);
                        entity.render_param.tick = Some(root.time_tick() as u32);
                        data.state = PlayerEntityDataState::Wait;
                    }
                }
            }
        }

        let key = root.entity_modify(key, move |e| *e = entity).unwrap();

        let resource = root.resource_get_mut::<PlayerResource>().unwrap();
        resource.current = Some(key);
    }
}

fn intersection_guard(root: &mut Root, entity_key: EntityKey, new_location: Vec2) -> bool {
    let entity = root.entity_get(entity_key).unwrap();
    let base_rect = root.entity_get_base_collision_rect(entity.id).unwrap();

    #[rustfmt::skip]
    let rect = [[
        new_location[0] + base_rect[0][0],
        new_location[1] + base_rect[0][1], ], [
        new_location[0] + base_rect[1][0],
        new_location[1] + base_rect[1][1],
    ]];

    if root.tile_has_by_collision_rect(rect) {
        return true;
    }

    if root.block_has_by_collision_rect(rect) {
        return true;
    }

    root.entity_get_by_collision_rect(rect)
        .any(|other_key| other_key != entity_key)
}

// error handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerError {
    Resource(ResourceError),
    NotFoundPlayer,
}

impl std::fmt::Display for PlayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resource(e) => e.fmt(f),
            Self::NotFoundPlayer => write!(f, "not found player error"),
        }
    }
}

impl std::error::Error for PlayerError {}

impl From<ResourceError> for PlayerError {
    fn from(value: ResourceError) -> Self {
        Self::Resource(value)
    }
}
