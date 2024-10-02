use crate::inner;

use super::*;

#[derive(Debug, Clone)]
pub struct PlayerResource {
    pub input_move: Vec2,
}

// TODO: get player location.
impl PlayerResource {
    pub fn init(root: &mut inner::Root) {
        let resource = Self {
            input_move: Default::default(),
        };
        root.resource_insert(resource);
    }

    pub fn set_input_move(root: &mut inner::Root, input_move: Vec2) {
        let resource = root.resource_get_mut::<Self>().unwrap();
        resource.input_move = input_move;
    }
}

// TODO: prepare render param variant sets.
#[derive(Debug, Clone)]
pub struct EntityFeaturePlayer;

impl EntityFeatureTrait for EntityFeaturePlayer {
    fn after_place(&self, _root: &mut Root, _key: TileKey) {}

    fn before_break(&self, _root: &mut Root, _key: TileKey) {}

    // TODO: set render param tick on appropriate events.
    fn forward(&self, root: &mut Root, key: TileKey, delta_secs: f32) {
        let resource = root.resource_get::<PlayerResource>().unwrap();

        let mut entity = root.entity_get(key).unwrap().clone();

        entity.render_param.variant = Some(0);

        if resource.input_move[0].powi(2) + resource.input_move[1].powi(2) > f32::EPSILON {
            let location = [
                entity.location[0] + resource.input_move[0] * delta_secs,
                entity.location[1] + resource.input_move[1] * delta_secs,
            ];

            if !intersection_guard(root, key, location) {
                entity.location = location;

                entity.render_param.variant = Some(1);
            }
        }

        root.entity_modify(key, move |e| *e = entity).unwrap();
    }
}

// TODO: fix intersection guard.
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
