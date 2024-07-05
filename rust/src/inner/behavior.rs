use super::*;

// unit behaviors

impl GlobalBehavior for () {}

impl TileBehavior for () {}

impl BlockBehavior for () {}

impl EntityBehavior for () {}

// time behavior

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
        let node = Time {
            uptime_secs: Default::default(),
            delta_secs: Default::default(),
            instance: std::time::Instant::now(),
        };
        let relation = NodeRelation::Global;
        world.node_store.insert(node, relation);
    }

    fn on_drop(&self, world: &mut World) {
        let relation = NodeRelation::Global;
        world.node_store.remove_by_relation::<Time>(relation);
    }

    fn on_update(&self, world: &mut World) {
        let (_, node) = world.node_store.one_mut::<Time>().check();
        let instance = std::mem::replace(&mut node.instance, std::time::Instant::now());
        node.delta_secs = instance.elapsed().as_secs_f32();
        node.uptime_secs += node.delta_secs;
    }
}

// generator behavior

#[derive(Debug, Clone)]
pub struct Generator {
    pub chunk_size: u32,
    pub size: u32,
    pub visited_chunk: ahash::AHashSet<IVec2>,
}

#[derive(Debug, Clone)]
pub struct GeneratorBehavior {
    pub chunk_size: u32,
    pub size: u32,
}

impl GlobalBehavior for GeneratorBehavior {
    fn on_new(&self, world: &mut World) {
        let node = Generator {
            chunk_size: self.chunk_size,
            size: self.size,
            visited_chunk: Default::default(),
        };
        let relation = NodeRelation::Global;
        world.node_store.insert(node, relation);
    }

    fn on_drop(&self, world: &mut World) {
        let relation = NodeRelation::Global;
        world.node_store.remove_by_relation::<Generator>(relation);
    }

    fn on_update(&self, world: &mut World) {
        let relations = world
            .node_store
            .iter::<GeneratorAnchor>()
            .map(|(relation, _)| *relation)
            .collect::<Vec<_>>();

        let (_, node) = world.node_store.one_mut::<Generator>().check();

        for relation in relations {
            let NodeRelation::Entity(entity_key) = relation else {
                unreachable!();
            };

            let entity = world.entity_field.get(entity_key).check();
            let location = entity.location;

            let chunk_key = [
                location[0].div_euclid(node.chunk_size as f32) as i32,
                location[1].div_euclid(node.chunk_size as f32) as i32,
            ];

            for y in -(node.size as i32)..node.size as i32 {
                for x in -(node.size as i32)..node.size as i32 {
                    let chunk_key = [chunk_key[0] + x as i32, chunk_key[1] + y as i32];

                    if node.visited_chunk.contains(&chunk_key) {
                        continue;
                    }

                    for v in 0..node.chunk_size {
                        for u in 0..node.chunk_size {
                            let id = rand::Rng::gen_range(&mut rand::thread_rng(), 0..=1);
                            let location = [
                                chunk_key[0] * node.chunk_size as i32 + u as i32,
                                chunk_key[1] * node.chunk_size as i32 + v as i32,
                            ];
                            let _ = world.tile_field.insert(Tile::new(id, location));
                        }
                    }

                    node.visited_chunk.insert(chunk_key);
                }
            }
        }
    }
}

// generator anchor behavior

#[derive(Debug, Clone)]
pub struct GeneratorAnchor;

#[derive(Debug, Clone)]
pub struct GeneratorAnchorBehavior;

impl EntityBehavior for GeneratorAnchorBehavior {
    fn on_place_entity(&self, world: &mut World, entity_key: u32) {
        let node = GeneratorAnchor;
        let relation = NodeRelation::Entity(entity_key);
        world.node_store.insert::<GeneratorAnchor>(node, relation);
    }

    fn on_break_entity(&self, world: &mut World, entity_key: u32) {
        let relation = NodeRelation::Entity(entity_key);
        world
            .node_store
            .remove_by_relation::<GeneratorAnchor>(relation);
    }
}

// random walk behavior

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
    fn on_place_entity(&self, world: &mut World, entity_key: u32) {
        let node = RandomWalk {
            min_rest_secs: self.min_rest_secs,
            max_rest_secs: self.max_rest_secs,
            min_distance: self.min_distance,
            max_distance: self.max_distance,
            speed: self.speed,
            state: RandomWalkState::Init,
        };
        let relation = NodeRelation::Entity(entity_key);
        world.node_store.insert::<RandomWalk>(node, relation);
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

        for (relation, node) in world.node_store.iter_mut::<RandomWalk>() {
            let NodeRelation::Entity(entity_key) = *relation else {
                unreachable!();
            };

            match node.state {
                RandomWalkState::Init => {
                    node.state = RandomWalkState::WaitStart;
                }
                RandomWalkState::WaitStart => {
                    let secs = rand::Rng::gen_range(
                        &mut rand::thread_rng(),
                        node.min_rest_secs..node.max_rest_secs,
                    );
                    node.state = RandomWalkState::Wait(secs);
                }
                RandomWalkState::Wait(secs) => {
                    let new_secs = secs - delta_secs;
                    if new_secs <= 0.0 {
                        node.state = RandomWalkState::TripStart;
                    } else {
                        node.state = RandomWalkState::Wait(new_secs);
                    }
                }
                RandomWalkState::TripStart => {
                    let entity = world.entity_field.get(entity_key).check();
                    let distance = rand::Rng::gen_range(
                        &mut rand::thread_rng(),
                        node.min_distance..node.max_distance,
                    );
                    let direction = rand::Rng::gen_range(
                        &mut rand::thread_rng(),
                        0.0..std::f32::consts::PI * 2.0,
                    );
                    let destination = [
                        entity.location[0] + distance * direction.cos(),
                        entity.location[1] + distance * direction.sin(),
                    ];
                    node.state = RandomWalkState::Trip(destination);
                }
                RandomWalkState::Trip(destination) => {
                    let entity = world.entity_field.get(entity_key).check();
                    if entity.location == destination {
                        node.state = RandomWalkState::WaitStart;
                        return;
                    }
                    let diff = [
                        destination[0] - entity.location[0],
                        destination[1] - entity.location[1],
                    ];
                    let distance = (diff[0].powi(2) + diff[1].powi(2)).sqrt();
                    let direction = [diff[0] / distance, diff[1] / distance];
                    let delta_distance = distance.min(node.speed * delta_secs);
                    let location = [
                        entity.location[0] + direction[0] * delta_distance,
                        entity.location[1] + direction[1] * delta_distance,
                    ];
                    if move_entity(
                        world.tile_field,
                        world.block_field,
                        world.entity_field,
                        entity_key,
                        location,
                    )
                    .is_ok()
                    {
                        node.state = RandomWalkState::Trip(destination);
                    } else {
                        node.state = RandomWalkState::WaitStart;
                    }
                }
            }
        }
    }
}
