use crate::{block, entity, inner, tile};
use godot::prelude::*;

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
            inner: inner::AgentData::RandomWalk(inner::AgentDataRandomWalk::new(
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
    fn insert_tile(&mut self, tile_key: Gd<tile::TileKey>, data: Gd<AgentData>) -> bool {
        let tile_field = &self.tile_field.bind().inner;
        let block_field = &self.block_field.bind().inner;
        let entity_field = &self.entity_field.bind().inner;
        let attach_key = inner::AgentKey::Tile(tile_key.bind().inner);
        let data = data.bind().inner.clone();
        self.inner
            .insert(tile_field, block_field, entity_field, attach_key, data)
            .is_ok()
    }

    #[func]
    fn remove_tile(&mut self, tile_key: Gd<tile::TileKey>) -> Option<Gd<AgentData>> {
        let attach_key = inner::AgentKey::Tile(tile_key.bind().inner);
        let data = self.inner.remove(attach_key).ok()?;
        Some(Gd::from_init_fn(|_| AgentData { inner: data }))
    }

    #[func]
    fn insert_block(&mut self, block_key: Gd<block::BlockKey>, data: Gd<AgentData>) -> bool {
        let tile_field = &self.tile_field.bind().inner;
        let block_field = &self.block_field.bind().inner;
        let entity_field = &self.entity_field.bind().inner;
        let attach_key = inner::AgentKey::Block(block_key.bind().inner);
        let data = data.bind().inner.clone();
        self.inner
            .insert(tile_field, block_field, entity_field, attach_key, data)
            .is_ok()
    }

    #[func]
    fn remove_block(&mut self, block_key: Gd<block::BlockKey>) -> Option<Gd<AgentData>> {
        let attach_key = inner::AgentKey::Block(block_key.bind().inner);
        let data = self.inner.remove(attach_key).ok()?;
        Some(Gd::from_init_fn(|_| AgentData { inner: data }))
    }

    #[func]
    fn insert_entity(&mut self, entity_key: Gd<entity::EntityKey>, data: Gd<AgentData>) -> bool {
        let tile_field = &self.tile_field.bind().inner;
        let block_field = &self.block_field.bind().inner;
        let entity_field = &self.entity_field.bind().inner;
        let attach_key = inner::AgentKey::Entity(entity_key.bind().inner);
        let data = data.bind().inner.clone();
        self.inner
            .insert(tile_field, block_field, entity_field, attach_key, data)
            .is_ok()
    }

    #[func]
    fn remove_entity(&mut self, entity_key: Gd<entity::EntityKey>) -> Option<Gd<AgentData>> {
        let attach_key = inner::AgentKey::Entity(entity_key.bind().inner);
        let data = self.inner.remove(attach_key).ok()?;
        Some(Gd::from_init_fn(|_| AgentData { inner: data }))
    }

    #[func]
    fn update(&mut self, delta_secs: f32) {
        let block_field = &self.block_field.bind().inner;
        let entity_field = &mut self.entity_field.bind_mut().inner;
        self.inner.update(block_field, entity_field, delta_secs);
    }
}
