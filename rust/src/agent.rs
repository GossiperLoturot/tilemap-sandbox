use crate::{entity, inner};
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
        let data = inner::AgentData::Empty;
        Gd::from_init_fn(|_| Self { inner: data })
    }

    #[func]
    fn new_herbivore(scan_secs: f32, scan_distance: f32, speed: f32) -> Gd<Self> {
        let data = inner::AgentData::Herbivore {
            scan_secs,
            scan_distance,
            speed,
            next_scan: Default::default(),
            next_location: Default::default(),
            elapsed: Default::default(),
        };
        Gd::from_init_fn(|_| Self { inner: data })
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct AgentSystem {
    inner: inner::AgentSystem,
    entity_field: Gd<entity::EntityField>,
}

#[godot_api]
impl AgentSystem {
    #[func]
    fn new_from(entity_field: Gd<entity::EntityField>) -> Gd<Self> {
        Gd::from_init_fn(|_| Self {
            inner: inner::AgentSystem::new(),
            entity_field,
        })
    }

    #[func]
    fn insert(&mut self, entity_key: Gd<entity::EntityKey>, data: Gd<AgentData>) -> bool {
        let entity_key = entity_key.bind().inner.clone();
        let data = data.bind().inner.clone();
        self.inner.insert(entity_key, data).is_some()
    }

    #[func]
    fn remove(&mut self, entity_key: Gd<entity::EntityKey>) -> Option<Gd<AgentData>> {
        let entity_key = entity_key.bind().inner.clone();
        let agent_data = self.inner.remove(entity_key)?;
        Some(Gd::from_init_fn(|_| AgentData { inner: agent_data }))
    }

    #[func]
    fn update(&mut self, delta_secs: f32) {
        let entity_field = &mut self.entity_field.bind_mut().inner;
        self.inner.update(entity_field, delta_secs);
    }
}
