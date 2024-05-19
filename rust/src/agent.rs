use crate::{block, entity, inner, tile};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct AgentKey {
    inner: (std::any::TypeId, u32),
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct AgentPlugin {
    inner: inner::AgentPlugin,
    tile_field: Gd<tile::TileField>,
    block_field: Gd<block::BlockField>,
    entity_field: Gd<entity::EntityField>,
}

#[godot_api]
impl AgentPlugin {
    #[func]
    fn new_from(
        tile_field: Gd<tile::TileField>,
        block_field: Gd<block::BlockField>,
        entity_field: Gd<entity::EntityField>,
    ) -> Gd<Self> {
        Gd::from_init_fn(|_| Self {
            inner: inner::AgentPlugin::new(),
            tile_field,
            block_field,
            entity_field,
        })
    }

    #[func]
    fn insert_random_walk(
        &mut self,
        entity_key: Gd<entity::EntityKey>,
        min_rest_secs: f32,
        max_rest_secs: f32,
        min_distance: f32,
        max_distance: f32,
        speed: f32,
    ) -> Option<Gd<AgentKey>> {
        let entity_key = entity_key.bind().inner;
        let agent = inner::RandomWalk::new(
            entity_key,
            min_rest_secs,
            max_rest_secs,
            min_distance,
            max_distance,
            speed,
        );
        let key = self.inner.insert(agent).ok()?;
        Some(Gd::from_init_fn(|_| AgentKey { inner: key }))
    }

    #[func]
    fn update(&mut self, delta_secs: f32) {
        let tile_field = &mut self.tile_field.bind_mut().inner;
        let block_field = &mut self.block_field.bind_mut().inner;
        let entity_field = &mut self.entity_field.bind_mut().inner;
        self.inner
            .update(tile_field, block_field, entity_field, delta_secs);
    }
}
