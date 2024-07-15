use crate::inner::*;

// callback definition

pub struct BeforeCallback(pub Box<dyn Fn(&mut RootMut)>);
pub struct AfterCallback(pub Box<dyn Fn(&mut RootMut)>);
pub struct ForwardCallback(pub Box<dyn Fn(&mut RootMut, f32)>);
pub struct PlaceTileCallback(pub Box<dyn Fn(&mut RootMut, u32)>);
pub struct BreakTileCallback(pub Box<dyn Fn(&mut RootMut, u32)>);
pub struct PlaceBlockCallback(pub Box<dyn Fn(&mut RootMut, u32)>);
pub struct BreakBlockCallback(pub Box<dyn Fn(&mut RootMut, u32)>);
pub struct PlaceEntityCallback(pub Box<dyn Fn(&mut RootMut, u32)>);
pub struct BreakEntityCallback(pub Box<dyn Fn(&mut RootMut, u32)>);
pub struct GenerateCallback(pub Box<dyn Fn(&mut RootMut, [Vec2; 2])>);

// action

pub fn before(root: &mut RootMut) {
    let callbacks = root.callback_store.iter::<BeforeCallback>();
    callbacks.for_each(|f| f.0(root));
}

pub fn after(root: &mut RootMut) {
    let callbacks = root.callback_store.iter::<AfterCallback>();
    callbacks.for_each(|f| f.0(root));
}

pub fn forward(root: &mut RootMut, delta_secs: f32) {
    let callbacks = root.callback_store.iter::<ForwardCallback>();
    callbacks.for_each(|f| f.0(root, delta_secs));
}

pub fn place_tile(root: &mut RootMut, tile: Tile) -> Result<u32, FieldError> {
    let callbacks = root
        .callback_store
        .iter_by_ref::<PlaceTileCallback>(CallbackRef::Tile(tile.id));
    let tile_key = root.tile_field.insert(tile)?;
    callbacks.for_each(|f| f.0(root, tile_key));
    Ok(tile_key)
}

pub fn break_tile(root: &mut RootMut, tile_key: u32) -> Result<Tile, FieldError> {
    let tile = root.tile_field.get(tile_key)?;
    let callbacks = root
        .callback_store
        .iter_by_ref::<BreakTileCallback>(CallbackRef::Tile(tile.id));
    callbacks.for_each(|f| f.0(root, tile_key));
    let tile = root.tile_field.remove(tile_key)?;
    Ok(tile)
}

pub fn place_block(root: &mut RootMut, block: Block) -> Result<u32, FieldError> {
    let callbacks = root
        .callback_store
        .iter_by_ref::<PlaceBlockCallback>(CallbackRef::Block(block.id));
    let block_key = root.block_field.insert(block)?;
    callbacks.for_each(|f| f.0(root, block_key));
    Ok(block_key)
}

pub fn break_block(root: &mut RootMut, block_key: u32) -> Result<Block, FieldError> {
    let block = root.block_field.get(block_key)?;
    let callbacks = root
        .callback_store
        .iter_by_ref::<BreakBlockCallback>(CallbackRef::Block(block.id));
    callbacks.for_each(|f| f.0(root, block_key));
    let block = root.block_field.remove(block_key)?;
    Ok(block)
}

pub fn place_entity(root: &mut RootMut, entity: Entity) -> Result<u32, FieldError> {
    let callbacks = root
        .callback_store
        .iter_by_ref::<PlaceEntityCallback>(CallbackRef::Entity(entity.id));
    let entity_key = root.entity_field.insert(entity)?;
    callbacks.for_each(|f| f.0(root, entity_key));
    Ok(entity_key)
}

pub fn break_entity(root: &mut RootMut, entity_key: u32) -> Result<Entity, FieldError> {
    let entity = root.entity_field.get(entity_key)?;
    let callbacks = root
        .callback_store
        .iter_by_ref::<BreakEntityCallback>(CallbackRef::Entity(entity.id));
    callbacks.for_each(|f| f.0(root, entity_key));
    let entity = root.entity_field.remove(entity_key)?;
    Ok(entity)
}

pub fn generate_chunk(root: &mut RootMut, rect: [Vec2; 2]) {
    let callback = root.callback_store.iter::<GenerateCallback>();
    callback.for_each(|f| f.0(root, rect));
}

pub fn move_entity_ex(root: &mut RootMut, entity_key: u32, new_location: Vec2) {
    let _ = move_entity(
        root.tile_field,
        root.block_field,
        root.entity_field,
        entity_key,
        new_location,
    );
}

// generator callback

#[derive(Debug, Clone)]
pub struct GeneratorNode {
    pub chunk_size: u32,
    pub prev_rect: Option<[IVec2; 2]>,
    pub visited_chunk: ahash::AHashSet<IVec2>,
}

#[derive(Debug, Clone)]
pub struct Generator {
    pub chunk_size: u32,
}

impl Generator {
    fn before(&self, root: &mut RootMut) {
        let node = GeneratorNode {
            chunk_size: self.chunk_size,
            prev_rect: None,
            visited_chunk: Default::default(),
        };
        root.node_store.insert(node, NodeRef::Global);
    }

    fn after(&self, root: &mut RootMut) {
        root.node_store
            .remove_by_ref::<GeneratorNode>(NodeRef::Global);
    }

    pub fn generate_chunk(&self, root: &mut RootMut, rect: [Vec2; 2]) {
        let (_, node) = root.node_store.one_mut::<GeneratorNode>().unwrap();

        let chunk_size = node.chunk_size;

        #[rustfmt::skip]
        let rect = [[
            rect[0][0].div_euclid(chunk_size as f32) as i32,
            rect[0][1].div_euclid(chunk_size as f32) as i32, ], [
            rect[1][0].div_euclid(chunk_size as f32) as i32,
            rect[1][1].div_euclid(chunk_size as f32) as i32,
        ]];

        let mut chunk_keys = vec![];
        if Some(rect) != node.prev_rect {
            for y in rect[0][1]..=rect[1][1] {
                for x in rect[0][0]..=rect[1][0] {
                    let chunk_key = [x, y];

                    if node.visited_chunk.contains(&chunk_key) {
                        continue;
                    }

                    chunk_keys.push(chunk_key);

                    node.visited_chunk.insert(chunk_key);
                }
            }

            node.prev_rect = Some(rect);
        }

        for [x, y] in chunk_keys {
            for v in 0..chunk_size {
                for u in 0..chunk_size {
                    let id = rand::Rng::gen_range(&mut rand::thread_rng(), 0..=1);
                    let location = [
                        x * chunk_size as i32 + u as i32,
                        y * chunk_size as i32 + v as i32,
                    ];
                    let _ = place_tile(root, Tile::new(id, location, 0));
                }
            }

            for _ in 0..64 {
                let id = rand::Rng::gen_range(&mut rand::thread_rng(), 0..=3);
                let u = rand::Rng::gen_range(&mut rand::thread_rng(), 0..chunk_size);
                let v = rand::Rng::gen_range(&mut rand::thread_rng(), 0..chunk_size);
                let location = [
                    x * chunk_size as i32 + u as i32,
                    y * chunk_size as i32 + v as i32,
                ];
                let _ = place_block(root, Block::new(id, location, 0));
            }

            for _ in 0..4 {
                let id = rand::Rng::gen_range(&mut rand::thread_rng(), 1..=5);
                let u = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..chunk_size as f32);
                let v = rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..chunk_size as f32);
                let location = [
                    x as f32 * chunk_size as f32 + u,
                    y as f32 * chunk_size as f32 + v,
                ];
                let _ = place_entity(root, Entity::new(id, location, 0));
            }
        }
    }
}

impl CallbackBundle for Generator {
    fn insert(&self, builder: &mut CallbackStoreBuilder) {
        let slf = std::rc::Rc::new(self.clone());
        builder.insert(
            CallbackRef::Global,
            BeforeCallback(Box::new({
                let slf = slf.clone();
                move |root| slf.before(root)
            })),
        );
        builder.insert(
            CallbackRef::Global,
            AfterCallback(Box::new({
                let slf = slf.clone();
                move |root| slf.after(root)
            })),
        );
        builder.insert(
            CallbackRef::Global,
            GenerateCallback(Box::new({
                let slf = slf.clone();
                move |root, chunk_key| slf.generate_chunk(root, chunk_key)
            })),
        );
    }
}

// random walk callback

#[derive(Debug, Clone)]
pub enum RandomWalkState {
    Init,
    WaitStart,
    Wait(f32),
    TripStart,
    Trip(Vec2),
}

#[derive(Debug, Clone)]
pub struct RandomWalkNode {
    pub min_rest_secs: f32,
    pub max_rest_secs: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub speed: f32,
    pub state: RandomWalkState,
}

#[derive(Debug, Clone)]
pub struct RandomWalk {
    pub entity_id: u32,
    pub min_rest_secs: f32,
    pub max_rest_secs: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub speed: f32,
}

impl RandomWalk {
    fn place_entity(&self, root: &mut RootMut, entity_key: u32) {
        let node = RandomWalkNode {
            min_rest_secs: self.min_rest_secs,
            max_rest_secs: self.max_rest_secs,
            min_distance: self.min_distance,
            max_distance: self.max_distance,
            speed: self.speed,
            state: RandomWalkState::Init,
        };
        root.node_store.insert(node, NodeRef::Entity(entity_key));
    }

    fn break_entity(&self, root: &mut RootMut, entity_key: u32) {
        root.node_store
            .remove_by_ref::<RandomWalkNode>(NodeRef::Entity(entity_key));
    }
}

impl CallbackBundle for RandomWalk {
    fn insert(&self, builder: &mut CallbackStoreBuilder) {
        let slf = std::rc::Rc::new(self.clone());
        builder.insert(
            CallbackRef::Entity(self.entity_id),
            PlaceEntityCallback(Box::new({
                let slf = slf.clone();
                move |root, entity_key| slf.place_entity(root, entity_key)
            })),
        );
        builder.insert(
            CallbackRef::Entity(self.entity_id),
            BreakEntityCallback(Box::new({
                let slf = slf.clone();
                move |root, entity_key| slf.break_entity(root, entity_key)
            })),
        );
    }
}

#[derive(Debug, Clone)]
pub struct RandomWalkForward;

impl RandomWalkForward {
    fn forward(root: &mut RootMut, delta_secs: f32) {
        for (r#ref, node) in root.node_store.iter_mut::<RandomWalkNode>() {
            let NodeRef::Entity(entity_key) = *r#ref else {
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
                    let entity = root.entity_field.get(entity_key).unwrap();
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
                    let entity = root.entity_field.get(entity_key).unwrap();
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
                        root.tile_field,
                        root.block_field,
                        root.entity_field,
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

impl CallbackBundle for RandomWalkForward {
    fn insert(&self, builder: &mut CallbackStoreBuilder) {
        builder.insert(
            CallbackRef::Global,
            ForwardCallback(Box::new(Self::forward)),
        );
    }
}
