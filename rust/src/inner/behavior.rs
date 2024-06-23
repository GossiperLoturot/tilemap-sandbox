use super::*;

impl TileBehavior for () {
    fn on_new(&self, _world: &mut World) {}
    fn on_drop(&self, _world: &mut World) {}
    fn on_place_tile(&self, _world: &mut World, _tile_key: u32) {}
    fn on_break_tile(&self, _world: &mut World, _tile_key: u32) {}
    fn on_update(&self, _world: &mut World) {}
}

impl BlockBehavior for () {
    fn on_new(&self, _world: &mut World) {}
    fn on_drop(&self, _world: &mut World) {}
    fn on_place_block(&self, _world: &mut World, _block_key: u32) {}
    fn on_break_block(&self, _world: &mut World, _block_key: u32) {}
    fn on_update(&self, _world: &mut World) {}
}

impl EntityBehavior for () {
    fn on_new(&self, _world: &mut World) {}
    fn on_drop(&self, _world: &mut World) {}
    fn on_place_entity(&self, _world: &mut World, _entity_key: u32) {}
    fn on_break_entity(&self, _world: &mut World, _entity_key: u32) {}
    fn on_update(&self, _world: &mut World) {}
}

#[derive(Debug, Clone)]
pub struct Time {
    pub uptime_secs: f32,
    pub delta_secs: f32,
    pub instance: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct TimeBehavior;

impl GlobalBehavior for TimeBehavior {
    fn on_new(&self, world: &mut World) {
        let inner = Time {
            uptime_secs: Default::default(),
            delta_secs: Default::default(),
            instance: std::time::Instant::now(),
        };
        let relation = NodeRelation::Global;
        world.node_store.insert(inner, relation);
    }

    fn on_drop(&self, world: &mut World) {
        let relation = NodeRelation::Global;
        world.node_store.remove_by_relation::<Time>(relation);
    }

    fn on_update(&self, world: &mut World) {
        let (_, inner) = world.node_store.one_mut::<Time>().check();
        let instance = std::mem::replace(&mut inner.instance, std::time::Instant::now());
        inner.delta_secs += instance.elapsed().as_secs_f32();
        inner.uptime_secs += inner.delta_secs;
    }
}

#[derive(Debug, Clone)]
pub enum RandomWalkState {
    Init,
    WaitStart,
    Wait(f32),
    TripStart,
    Trip(Vec2),
}

#[derive(Debug, Clone)]
pub struct RandomWalk {
    pub min_rest_secs: f32,
    pub max_rest_secs: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub speed: f32,
    pub state: RandomWalkState,
}

#[derive(Debug, Clone)]
pub struct RandomWalkBehavior {
    pub min_rest_secs: f32,
    pub max_rest_secs: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub speed: f32,
}

impl EntityBehavior for RandomWalkBehavior {
    fn on_new(&self, _world: &mut World) {}

    fn on_drop(&self, _world: &mut World) {}

    fn on_place_entity(&self, world: &mut World, entity_key: u32) {
        let inner = RandomWalk {
            min_rest_secs: self.min_rest_secs,
            max_rest_secs: self.max_rest_secs,
            min_distance: self.min_distance,
            max_distance: self.max_distance,
            speed: self.speed,
            state: RandomWalkState::Init,
        };
        let relation = NodeRelation::Entity(entity_key);
        world.node_store.insert::<RandomWalk>(inner, relation);
    }

    fn on_break_entity(&self, world: &mut World, entity_key: u32) {
        let relation = NodeRelation::Entity(entity_key);
        world.node_store.remove_by_relation::<RandomWalk>(relation);
    }

    fn on_update(&self, world: &mut World) {
        let Some((_, time)) = world.node_store.one::<Time>() else {
            panic!("time behavior not found");
        };
        let delta_secs = time.delta_secs;

        let Some(iter) = world.node_store.iter_mut::<RandomWalk>() else {
            return;
        };

        for (relation, inner) in iter {
            let NodeRelation::Entity(entity_key) = *relation else {
                unreachable!("invalid relation");
            };

            match inner.state {
                RandomWalkState::Init => {
                    inner.state = RandomWalkState::WaitStart;
                }
                RandomWalkState::WaitStart => {
                    let secs = rand::Rng::gen_range(
                        &mut rand::thread_rng(),
                        inner.min_rest_secs..inner.max_rest_secs,
                    );
                    inner.state = RandomWalkState::Wait(secs);
                }
                RandomWalkState::Wait(secs) => {
                    let new_secs = secs - delta_secs;
                    if new_secs <= 0.0 {
                        inner.state = RandomWalkState::TripStart;
                    } else {
                        inner.state = RandomWalkState::Wait(new_secs);
                    }
                }
                RandomWalkState::TripStart => {
                    let entity = world.entity_field.get(entity_key).check();
                    let distance = rand::Rng::gen_range(
                        &mut rand::thread_rng(),
                        inner.min_distance..inner.max_distance,
                    );
                    let direction = rand::Rng::gen_range(
                        &mut rand::thread_rng(),
                        0.0..std::f32::consts::PI * 2.0,
                    );
                    let destination = [
                        entity.location[0] + distance * direction.cos(),
                        entity.location[1] + distance * direction.sin(),
                    ];
                    inner.state = RandomWalkState::Trip(destination);
                }
                RandomWalkState::Trip(destination) => {
                    let entity = world.entity_field.get(entity_key).check();
                    if entity.location == destination {
                        inner.state = RandomWalkState::WaitStart;
                        return;
                    }
                    let diff = [
                        destination[0] - entity.location[0],
                        destination[1] - entity.location[1],
                    ];
                    let distance = (diff[0].powi(2) + diff[1].powi(2)).sqrt();
                    let direction = [diff[0] / distance, diff[1] / distance];
                    let delta_distance = distance.min(inner.speed * delta_secs);
                    let location = [
                        entity.location[0] + direction[0] * delta_distance,
                        entity.location[1] + direction[1] * delta_distance,
                    ];
                    if move_entity(world.block_field, world.entity_field, entity_key, location)
                        .is_ok()
                    {
                        inner.state = RandomWalkState::Trip(destination);
                    } else {
                        inner.state = RandomWalkState::WaitStart;
                    }
                }
            }
        }
    }
}
