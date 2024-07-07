use std::rc::Rc;

use crate::inner::*;

// delegate definition

pub trait OnNewDelegate {
    fn call(&self, world: &mut World);
}

pub trait OnDropDelegate {
    fn call(&self, world: &mut World);
}

pub trait OnUpdateDelegate {
    fn call(&self, world: &mut World, delta_secs: f32);
}

pub trait OnPlaceTileDelegate {
    fn call(&self, world: &mut World, tile_key: u32);
}

pub trait OnBreakTileDelegate {
    fn call(&self, world: &mut World, tile_key: u32);
}

pub trait OnPlaceBlockDelegate {
    fn call(&self, world: &mut World, block_key: u32);
}

pub trait OnBreakBlockDelegate {
    fn call(&self, world: &mut World, block_key: u32);
}

pub trait OnPlaceEntityDelegate {
    fn call(&self, world: &mut World, entity_key: u32);
}

pub trait OnBreakEntityDelegate {
    fn call(&self, world: &mut World, entity_key: u32);
}

// action

pub fn new(world: &mut World) {
    let delegates = world.delegate_store.iter_0::<Rc<dyn OnNewDelegate>>();
    delegates.for_each(|v| v.call(world));
}

pub fn drop(world: &mut World) {
    let delegates = world.delegate_store.iter_0::<Rc<dyn OnDropDelegate>>();
    delegates.for_each(|v| v.call(world));
}

pub fn update(world: &mut World, delta_secs: f32) {
    let delegates = world.delegate_store.iter_0::<Rc<dyn OnUpdateDelegate>>();
    delegates.for_each(|v| v.call(world, delta_secs));
}

pub fn place_tile(world: &mut World, tile: Tile) -> Result<u32, FieldError> {
    let delegates = world
        .delegate_store
        .iter_2::<Rc<dyn OnPlaceTileDelegate>>(DelegateStore::TILE_LAYER, tile.id);
    let tile_key = world.tile_field.insert(tile)?;
    delegates.for_each(|v| v.call(world, tile_key));
    Ok(tile_key)
}

pub fn break_tile(world: &mut World, tile_key: u32) -> Result<Tile, FieldError> {
    let tile = world.tile_field.get(tile_key)?;
    let delegates = world
        .delegate_store
        .iter_2::<Rc<dyn OnBreakTileDelegate>>(DelegateStore::TILE_LAYER, tile.id);
    delegates.for_each(|v| v.call(world, tile_key));
    let tile = world.tile_field.remove(tile_key)?;
    Ok(tile)
}

pub fn place_block(world: &mut World, block: Block) -> Result<u32, FieldError> {
    let delegates = world
        .delegate_store
        .iter_2::<Rc<dyn OnPlaceBlockDelegate>>(DelegateStore::BLOCK_LAYER, block.id);
    let block_key = world.block_field.insert(block)?;
    delegates.for_each(|v| v.call(world, block_key));
    Ok(block_key)
}

pub fn break_block(world: &mut World, block_key: u32) -> Result<Block, FieldError> {
    let block = world.block_field.get(block_key)?;
    let delegates = world
        .delegate_store
        .iter_2::<Rc<dyn OnBreakBlockDelegate>>(DelegateStore::BLOCK_LAYER, block.id);
    delegates.for_each(|v| v.call(world, block_key));
    let block = world.block_field.remove(block_key)?;
    Ok(block)
}

pub fn place_entity(world: &mut World, entity: Entity) -> Result<u32, FieldError> {
    let delegates = world
        .delegate_store
        .iter_2::<Rc<dyn OnPlaceEntityDelegate>>(DelegateStore::ENTITY_LAYER, entity.id);
    let entity_key = world.entity_field.insert(entity)?;
    delegates.for_each(|v| v.call(world, entity_key));
    Ok(entity_key)
}

pub fn break_entity(world: &mut World, entity_key: u32) -> Result<Entity, FieldError> {
    let entity = world.entity_field.get(entity_key)?;
    let delegates = world
        .delegate_store
        .iter_2::<Rc<dyn OnBreakEntityDelegate>>(DelegateStore::ENTITY_LAYER, entity.id);
    delegates.for_each(|v| v.call(world, entity_key));
    let entity = world.entity_field.remove(entity_key)?;
    Ok(entity)
}

pub fn generate_chunk(world: &mut World, chunk_key: IVec2) {
    let delegate = world.delegate_store.iter_0::<Rc<GeneratorDelegate>>();
    delegate.for_each(|v| v.generate_chunk(world, chunk_key));
}

// generator delegate

#[derive(Debug, Clone)]
pub struct Generator {
    pub chunk_size: u32,
    pub size: u32,
    pub visited_chunk: ahash::AHashSet<IVec2>,
}

#[derive(Debug, Clone)]
pub struct GeneratorDelegate {
    pub chunk_size: u32,
    pub size: u32,
}

impl GeneratorDelegate {
    pub fn inserted(self, delegate_store: &mut DelegateStore, id: u32) {
        let layer = DelegateStore::GLOBAL_LAYER;
        let delegate = Rc::new(self);
        delegate_store.insert::<Rc<dyn OnNewDelegate>>(layer, id, delegate.clone());
        delegate_store.insert::<Rc<dyn OnDropDelegate>>(layer, id, delegate.clone());
        delegate_store.insert(layer, id, delegate);
    }

    pub fn generate_chunk(&self, world: &mut World, chunk_key: IVec2) {
        let (_, node) = world.node_store.one_mut::<Generator>().unwrap();

        let chunk_size = node.chunk_size;
        let mut chunks = vec![];

        for y in -(node.size as i32)..node.size as i32 {
            for x in -(node.size as i32)..node.size as i32 {
                let chunk_key = [chunk_key[0] + x, chunk_key[1] + y];

                if node.visited_chunk.contains(&chunk_key) {
                    continue;
                }

                chunks.push(chunk_key);

                node.visited_chunk.insert(chunk_key);
            }
        }

        for [x, y] in chunks {
            for v in 0..self.chunk_size {
                for u in 0..self.chunk_size {
                    let id = rand::Rng::gen_range(&mut rand::thread_rng(), 0..=1);
                    let location = [
                        x * chunk_size as i32 + u as i32,
                        y * chunk_size as i32 + v as i32,
                    ];
                    let _ = place_tile(world, Tile::new(id, location));
                }
            }

            for _ in 0..64 {
                let id = rand::Rng::gen_range(&mut rand::thread_rng(), 0..=7);
                let u = rand::Rng::gen_range(&mut rand::thread_rng(), 0..chunk_size);
                let v = rand::Rng::gen_range(&mut rand::thread_rng(), 0..chunk_size);
                let location = [
                    x * chunk_size as i32 + u as i32,
                    y * chunk_size as i32 + v as i32,
                ];
                let _ = place_block(world, Block::new(id, location));
            }

            for _ in 0..8 {
                let u = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..chunk_size as f32);
                let v = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..chunk_size as f32);
                let location = [
                    x as f32 * chunk_size as f32 + u,
                    y as f32 * chunk_size as f32 + v,
                ];
                let _ = place_entity(world, Entity::new(1, location));
            }
        }
    }
}

impl OnNewDelegate for GeneratorDelegate {
    fn call(&self, world: &mut World) {
        let node = Generator {
            chunk_size: self.chunk_size,
            size: self.size,
            visited_chunk: Default::default(),
        };
        let relation = NodeRelation::Global;
        world.node_store.insert(node, relation);
    }
}

impl OnDropDelegate for GeneratorDelegate {
    fn call(&self, world: &mut World) {
        let relation = NodeRelation::Global;
        world.node_store.remove_by_relation::<Generator>(relation);
    }
}

// random walk delegate

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
pub struct RandomWalkDelegate {
    pub min_rest_secs: f32,
    pub max_rest_secs: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub speed: f32,
}

impl RandomWalkDelegate {
    pub fn inserted(self, delegate_store: &mut DelegateStore, id: u32) {
        let layer = DelegateStore::ENTITY_LAYER;
        let delegate = Rc::new(self);
        delegate_store.insert::<Rc<dyn OnPlaceEntityDelegate>>(layer, id, delegate.clone());
        delegate_store.insert::<Rc<dyn OnBreakEntityDelegate>>(layer, id, delegate.clone());
        delegate_store.insert::<Rc<dyn OnUpdateDelegate>>(layer, id, delegate);
    }
}

impl OnPlaceEntityDelegate for RandomWalkDelegate {
    fn call(&self, world: &mut World, entity_key: u32) {
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
}

impl OnBreakEntityDelegate for RandomWalkDelegate {
    fn call(&self, world: &mut World, entity_key: u32) {
        let relation = NodeRelation::Entity(entity_key);
        world.node_store.remove_by_relation::<RandomWalk>(relation);
    }
}

impl OnUpdateDelegate for RandomWalkDelegate {
    fn call(&self, world: &mut World, delta_secs: f32) {
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
                    let entity = world.entity_field.get(entity_key).unwrap();
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
                    let entity = world.entity_field.get(entity_key).unwrap();
                    if entity.location == destination {
                        node.state = RandomWalkState::WaitStart;
                        continue;
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
