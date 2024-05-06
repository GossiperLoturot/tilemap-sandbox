use crate::{block, entity, inner, tile};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct Agent {
    inner: inner::Agent,
}

#[godot_api]
impl Agent {
    #[func]
    fn new_empty() -> Gd<Self> {
        Gd::from_init_fn(|_| Self {
            inner: inner::Agent::Empty(inner::Empty::new()),
        })
    }

    #[func]
    fn new_random_walk(
        entity_key: Gd<entity::EntityKey>,
        min_rest_secs: f32,
        max_rest_secs: f32,
        min_distance: f32,
        max_distance: f32,
        speed: f32,
    ) -> Gd<Self> {
        let entity_key = entity_key.bind().inner;
        Gd::from_init_fn(|_| Self {
            inner: inner::Agent::RandomWalk(inner::RandomWalk::new(
                entity_key,
                min_rest_secs,
                max_rest_secs,
                min_distance,
                max_distance,
                speed,
            )),
        })
    }

    #[func]
    fn new_timestamp() -> Gd<Self> {
        Gd::from_init_fn(|_| Self {
            inner: inner::Agent::Timestamp(inner::Timestamp::new()),
        })
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct AgentKey {
    inner: u32,
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
    fn insert(&mut self, agent: Gd<Agent>) -> Option<Gd<AgentKey>> {
        let tile_field = &self.tile_field.bind().inner;
        let block_field = &self.block_field.bind().inner;
        let entity_field = &self.entity_field.bind().inner;
        let agent = agent.bind().inner.clone();
        let key = self
            .inner
            .insert(tile_field, block_field, entity_field, agent)
            .ok()?;
        Some(Gd::from_init_fn(|_| AgentKey { inner: key }))
    }

    #[func]
    fn remove(&mut self, key: Gd<AgentKey>) -> Option<Gd<Agent>> {
        let key = key.bind().inner;
        let agent = self.inner.remove(key).ok()?;
        Some(Gd::from_init_fn(|_| Agent { inner: agent }))
    }

    #[func]
    fn get(&self, key: Gd<AgentKey>) -> Option<Gd<Agent>> {
        let key = key.bind().inner;
        let agent = self.inner.get(key).ok()?.clone();
        Some(Gd::from_init_fn(|_| Agent { inner: agent }))
    }

    #[func]
    fn get_by_global(&self) -> Array<Gd<AgentKey>> {
        let keys = self
            .inner
            .get_by_global()
            .map(|key| Gd::from_init_fn(|_| AgentKey { inner: key }));
        Array::from_iter(keys)
    }

    #[func]
    fn get_by_tile(&self, tile_key: Gd<tile::TileKey>) -> Array<Gd<AgentKey>> {
        let tile_key = tile_key.bind().inner;
        let keys = self
            .inner
            .get_by_tile(tile_key)
            .map(|key| Gd::from_init_fn(|_| AgentKey { inner: key }));
        Array::from_iter(keys)
    }

    #[func]
    fn get_by_block(&self, block_key: Gd<block::BlockKey>) -> Array<Gd<AgentKey>> {
        let block_key = block_key.bind().inner;
        let keys = self
            .inner
            .get_by_block(block_key)
            .map(|key| Gd::from_init_fn(|_| AgentKey { inner: key }));
        Array::from_iter(keys)
    }

    #[func]
    fn get_by_entity(&self, entity_key: Gd<entity::EntityKey>) -> Array<Gd<AgentKey>> {
        let entity_key = entity_key.bind().inner;
        let keys = self
            .inner
            .get_by_entity(entity_key)
            .map(|key| Gd::from_init_fn(|_| AgentKey { inner: key }));
        Array::from_iter(keys)
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
