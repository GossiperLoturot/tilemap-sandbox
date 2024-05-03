use crate::{
    block, entity,
    inner::{self, AgentStateRandomWalk},
};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct AgentState {
    inner: inner::AgentState,
}

#[godot_api]
impl AgentState {
    #[func]
    fn new_empty() -> Gd<Self> {
        Gd::from_init_fn(|_| Self {
            inner: inner::AgentState::Empty,
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
            inner: inner::AgentState::RandomWalk(AgentStateRandomWalk {
                min_rest_secs,
                max_rest_secs,
                min_distance,
                max_distance,
                speed,
                local: Default::default(),
            }),
        })
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct AgentPlugin {
    inner: inner::AgentPlugin,
    block_field: Gd<block::BlockField>,
    entity_field: Gd<entity::EntityField>,
}

#[godot_api]
impl AgentPlugin {
    #[func]
    fn new_from(
        block_field: Gd<block::BlockField>,
        entity_field: Gd<entity::EntityField>,
    ) -> Gd<Self> {
        Gd::from_init_fn(|_| Self {
            inner: inner::AgentPlugin::new(),
            block_field,
            entity_field,
        })
    }

    #[func]
    fn insert(&mut self, entity_key: Gd<entity::EntityKey>, state: Gd<AgentState>) -> bool {
        let entity_key = entity_key.bind().inner;
        let state = state.bind().inner.clone();
        self.inner.insert(entity_key, state).is_some()
    }

    #[func]
    fn remove(&mut self, entity_key: Gd<entity::EntityKey>) -> bool {
        let entity_key = entity_key.bind().inner;
        self.inner.remove(entity_key).is_some()
    }

    #[func]
    fn update(&mut self, delta_secs: f32) {
        let block_field = &self.block_field.bind().inner;
        let entity_field = &mut self.entity_field.bind_mut().inner;
        self.inner.update(block_field, entity_field, delta_secs);
    }
}
