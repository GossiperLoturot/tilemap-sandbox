use crate::{block, entity, inner, tile};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct FieldKey {
    inner: inner::FieldKey,
}

#[godot_api]
impl FieldKey {
    #[func]
    fn new_tile(tile_key: Gd<tile::TileKey>) -> Gd<Self> {
        Gd::from_init_fn(|_| Self {
            inner: inner::FieldKey::Tile(tile_key.bind().inner),
        })
    }

    #[func]
    fn new_block(block_key: Gd<block::BlockKey>) -> Gd<Self> {
        Gd::from_init_fn(|_| Self {
            inner: inner::FieldKey::Block(block_key.bind().inner),
        })
    }

    #[func]
    fn new_entity(entity_key: Gd<entity::EntityKey>) -> Gd<Self> {
        Gd::from_init_fn(|_| Self {
            inner: inner::FieldKey::Entity(entity_key.bind().inner),
        })
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct AgentData {
    inner: inner::AgentData,
}

#[godot_api]
impl AgentData {
    #[func]
    fn new_empty() -> Gd<Self> {
        Gd::from_init_fn(|_| Self {
            inner: inner::AgentData::Empty,
        })
    }

    #[func]
    fn new_random_walk(
        min_rest_secs: f32,
        max_rest_secs: f32,
        min_distance: f32,
        max_distance: f32,
        speed: f32,
    ) -> Gd<Self> {
        Gd::from_init_fn(|_| Self {
            inner: inner::AgentData::RandomWalk(inner::RandomWalk::new(
                min_rest_secs,
                max_rest_secs,
                min_distance,
                max_distance,
                speed,
            )),
        })
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct Agent {
    inner: inner::Agent,
}

#[godot_api]
impl Agent {
    #[func]
    fn new_from(field_key: Gd<FieldKey>, data: Gd<AgentData>, update: bool) -> Gd<Self> {
        Gd::from_init_fn(|_| Self {
            inner: inner::Agent {
                field_ref: field_key.bind().inner,
                data: data.bind().inner.clone(),
                update,
            },
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
    fn get_by_field(&self, field_key: Gd<FieldKey>) -> Array<Gd<AgentKey>> {
        let field_key = field_key.bind().inner;
        let keys = self
            .inner
            .get_by_field(field_key)
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
