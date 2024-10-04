use crate::inner;

use super::*;

#[derive(Debug, Clone)]
pub struct PlayerResource {
    pub location: Option<Vec2>,
    pub input_move: Vec2,
}

impl PlayerResource {
    pub fn init(root: &mut inner::Root) {
        let resource = Self {
            location: Default::default(),
            input_move: Default::default(),
        };
        root.resource_insert(resource);
    }

    pub fn set_input_move(root: &mut inner::Root, input_move: Vec2) {
        let resource = root.resource_get_mut::<Self>().unwrap();
        resource.input_move = input_move;
    }

    pub fn get_location(root: &inner::Root) -> Option<Vec2> {
        let resource = root.resource_get::<Self>().unwrap();
        resource.location
    }
}

#[derive(Debug, Clone)]
pub enum EntityDataPlayerState {
    Wait,
    Move,
}

#[derive(Debug, Clone)]
pub struct EntityDataPlayer {
    pub state: EntityDataPlayerState,
}

#[derive(Debug, Clone)]
pub struct EntityFeaturePlayer;

impl EntityFeatureTrait for EntityFeaturePlayer {
    fn after_place(&self, root: &mut Root, key: TileKey) {
        root.entity_modify(key, |entity| {
            entity.data = Some(EntityData::Player(EntityDataPlayer {
                state: EntityDataPlayerState::Wait,
            }))
        })
        .unwrap();
    }

    fn before_break(&self, _root: &mut Root, _key: TileKey) {}

    fn forward(&self, root: &mut Root, key: TileKey, delta_secs: f32) {
        let mut entity = root.entity_get(key).unwrap().clone();

        let Some(EntityData::Player(data)) = &mut entity.data else {
            return;
        };

        let resource = root.resource_get_mut::<PlayerResource>().unwrap();
        resource.location = Some(entity.location);

        let is_move =
            resource.input_move[0].powi(2) + resource.input_move[1].powi(2) > f32::EPSILON;
        if is_move {
            let location = [
                entity.location[0] + resource.input_move[0] * delta_secs,
                entity.location[1] + resource.input_move[1] * delta_secs,
            ];

            if !intersection_guard(root, key, location) {
                entity.location = location;
            }
        }

        match data.state {
            EntityDataPlayerState::Wait => {
                if is_move {
                    entity.render_param.variant = Some(1);
                    entity.render_param.tick = Some(root.tick_get() as u32);
                    data.state = EntityDataPlayerState::Move;
                }
            }
            EntityDataPlayerState::Move => {
                if !is_move {
                    entity.render_param.variant = Some(0);
                    entity.render_param.tick = Some(root.tick_get() as u32);
                    data.state = EntityDataPlayerState::Wait;
                }
            }
        }

        root.entity_modify(key, move |e| *e = entity).unwrap();
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
