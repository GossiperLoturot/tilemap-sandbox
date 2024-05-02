use crate::{block, entity, inner};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Resource)]
struct AgentPluginDescEntryEmpty {}

#[derive(GodotClass)]
#[class(init, base=Resource)]
struct AgentPluginDescEntryHerbivore {
    #[export]
    min_rest_secs: f32,
    #[export]
    max_rest_secs: f32,
    #[export]
    min_distance: f32,
    #[export]
    max_distance: f32,
    #[export]
    speed: f32,
}

#[derive(GodotClass)]
#[class(init, base=Resource)]
struct AgentPluginDesc {
    #[export]
    entries: Array<Gd<godot::engine::Resource>>,
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
        desc: Gd<AgentPluginDesc>,
        block_field: Gd<block::BlockField>,
        entity_field: Gd<entity::EntityField>,
    ) -> Gd<Self> {
        let desc = desc.bind();

        let specs = desc
            .entries
            .iter_shared()
            .map(|entry| {
                if entry
                    .clone()
                    .try_cast::<AgentPluginDescEntryEmpty>()
                    .is_ok()
                {
                    inner::AgentSpec::Empty
                } else if let Ok(entry) = entry.try_cast::<AgentPluginDescEntryHerbivore>() {
                    let entry = entry.bind();
                    inner::AgentSpec::Herbivore {
                        min_rest_secs: entry.min_rest_secs,
                        max_rest_secs: entry.max_rest_secs,
                        min_distance: entry.min_distance,
                        max_distance: entry.max_distance,
                        speed: entry.speed,
                    }
                } else {
                    panic!("Invalid entry in AgentPluginDesc");
                }
            })
            .collect::<Vec<_>>();

        Gd::from_init_fn(|_| Self {
            inner: inner::AgentPlugin::new(specs),
            block_field,
            entity_field,
        })
    }

    #[func]
    fn insert(&mut self, entity_key: Gd<entity::EntityKey>, id: u32) -> bool {
        let entity_key = entity_key.bind().inner;
        self.inner.insert(entity_key, id).is_some()
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
