use crate::{block, entity, inner, tile};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct AgentKey {
    inner: inner::AgentKey,
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct AgentRelation {
    inner: inner::AgentRelation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentTypeEnum {
    RandomWalk,
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct AgentType {
    inner: AgentTypeEnum,
}

#[godot_api]
impl AgentType {
    #[func]
    fn new_random_walk() -> Gd<Self> {
        let agent = AgentTypeEnum::RandomWalk;
        Gd::from_init_fn(|_| Self { inner: agent })
    }
}

#[derive(Debug, Clone)]
enum AgentEnum {
    RandomWalk(inner::RandomWalk),
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct Agent {
    inner: AgentEnum,
}

#[godot_api]
impl Agent {
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
        let agent = AgentEnum::RandomWalk(inner::RandomWalk::new(
            entity_key,
            min_rest_secs,
            max_rest_secs,
            min_distance,
            max_distance,
            speed,
        ));
        Gd::from_init_fn(|_| Self { inner: agent })
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
    fn insert(&mut self, agent: Gd<Agent>) -> Option<Gd<AgentKey>> {
        let agent = agent.bind().inner.clone();

        let key = match agent {
            AgentEnum::RandomWalk(agent) => self.inner.insert(agent).ok()?,
        };

        Some(Gd::from_init_fn(|_| AgentKey { inner: key }))
    }

    #[func]
    fn remove(&mut self, key: Gd<AgentKey>) -> bool {
        let key = key.bind().inner;

        if key.0 == std::any::TypeId::of::<inner::RandomWalk>() {
            self.inner.remove::<inner::RandomWalk>(key).is_ok()
        } else {
            unreachable!()
        }
    }

    #[func]
    fn get(&self, key: Gd<AgentKey>) -> Option<Gd<Agent>> {
        let key = key.bind().inner;

        let agent = if key.0 == std::any::TypeId::of::<inner::RandomWalk>() {
            let agent = self.inner.get::<inner::RandomWalk>(key).ok()?;
            AgentEnum::RandomWalk(agent.clone())
        } else {
            unreachable!()
        };

        Some(Gd::from_init_fn(|_| Agent { inner: agent }))
    }

    #[func]
    fn get_by_type(&self, r#type: Gd<AgentType>) -> Array<Gd<Agent>> {
        let r#type = r#type.bind().inner;

        let iter = match r#type {
            AgentTypeEnum::RandomWalk => {
                let agents = self.inner.iter::<inner::RandomWalk>();
                agents.into_iter().flatten().map(|agent| {
                    Gd::from_init_fn(|_| Agent {
                        inner: AgentEnum::RandomWalk(agent.clone()),
                    })
                })
            }
        };

        Array::from_iter(iter)
    }

    #[func]
    fn get_by_type_and_relation(
        &self,
        r#type: Gd<AgentType>,
        relation: Gd<AgentRelation>,
    ) -> Array<Gd<Agent>> {
        let r#type = r#type.bind().inner;
        let relation = relation.bind().inner;

        let iter = match r#type {
            AgentTypeEnum::RandomWalk => {
                let agents = self.inner.iter_by_relation::<inner::RandomWalk>(relation);
                agents.into_iter().flatten().map(|agent| {
                    Gd::from_init_fn(|_| Agent {
                        inner: AgentEnum::RandomWalk(agent.clone()),
                    })
                })
            }
        };

        Array::from_iter(iter)
    }

    #[func]
    fn remove_by_relation(&mut self, relation: Gd<AgentRelation>) -> bool {
        let relation = relation.bind().inner;
        self.inner.remove_by_relation(relation).is_ok()
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
