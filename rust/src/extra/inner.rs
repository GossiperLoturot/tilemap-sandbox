use crate::inner::*;

// callback definition

pub struct BeforeCallback(pub Box<dyn Fn(&mut RootMut)>);
pub struct AfterCallback(pub Box<dyn Fn(&mut RootMut)>);
pub struct ForwardCallback(pub Box<dyn Fn(&mut RootMut, f32)>);
pub struct ForwardLocalCallback(pub Box<dyn Fn(&mut RootMut, f32, [SpcKey; 2])>);

pub struct PlaceTileCallback(pub Box<dyn Fn(&mut RootMut, Tile) -> Result<u32, FieldError>>);
pub struct BreakTileCallback(pub Box<dyn Fn(&mut RootMut, u32) -> Result<Tile, FieldError>>);
pub struct ModifyTileCallback(pub Box<dyn Fn(&mut RootMut, u32, Tile) -> Result<Tile, FieldError>>);

pub struct PlaceBlockCallback(pub Box<dyn Fn(&mut RootMut, Block) -> Result<u32, FieldError>>);
pub struct BreakBlockCallback(pub Box<dyn Fn(&mut RootMut, u32) -> Result<Block, FieldError>>);
pub struct ModifyBlockCallback(
    pub Box<dyn Fn(&mut RootMut, u32, Block) -> Result<Block, FieldError>>,
);

pub struct PlaceEntityCallback(pub Box<dyn Fn(&mut RootMut, Entity) -> Result<u32, FieldError>>);
pub struct BreakEntityCallback(pub Box<dyn Fn(&mut RootMut, u32) -> Result<Entity, FieldError>>);
pub struct ModifyEntityCallback(
    pub Box<dyn Fn(&mut RootMut, u32, Entity) -> Result<Entity, FieldError>>,
);

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

pub fn forward_local(root: &mut RootMut, delta_secs: f32, rect: [SpcKey; 2]) {
    let callbacks = root.callback_store.iter::<ForwardLocalCallback>();
    callbacks.for_each(|f| f.0(root, delta_secs, rect));
}

pub fn place_tile(root: &mut RootMut, tile: Tile) -> Result<u32, FieldError> {
    let callback = root
        .callback_store
        .one_by_ref::<PlaceTileCallback>(CallbackRef::Tile(tile.id))
        .unwrap();
    callback.0(root, tile)
}

pub fn break_tile(root: &mut RootMut, tile_key: u32) -> Result<Tile, FieldError> {
    let tile = root.tile_field.get(tile_key)?;
    let callback = root
        .callback_store
        .one_by_ref::<BreakTileCallback>(CallbackRef::Tile(tile.id))
        .unwrap();
    callback.0(root, tile_key)
}

pub fn modify_tile(root: &mut RootMut, tile_key: u32, new_tile: Tile) -> Result<Tile, FieldError> {
    let old_tile = root.tile_field.get(tile_key)?;

    if new_tile.id != old_tile.id {
        panic!("no support for changing tile id in modify_tile");
    }

    let callback = root
        .callback_store
        .one_by_ref::<ModifyTileCallback>(CallbackRef::Tile(new_tile.id))
        .unwrap();
    callback.0(root, tile_key, new_tile)
}

pub fn place_block(root: &mut RootMut, block: Block) -> Result<u32, FieldError> {
    let callback = root
        .callback_store
        .one_by_ref::<PlaceBlockCallback>(CallbackRef::Block(block.id))
        .unwrap();
    callback.0(root, block)
}

pub fn break_block(root: &mut RootMut, block_key: u32) -> Result<Block, FieldError> {
    let block = root.block_field.get(block_key)?;
    let callback = root
        .callback_store
        .one_by_ref::<BreakBlockCallback>(CallbackRef::Block(block.id))
        .unwrap();
    callback.0(root, block_key)
}

pub fn modify_block(
    root: &mut RootMut,
    block_key: u32,
    new_block: Block,
) -> Result<Block, FieldError> {
    let old_block = root.block_field.get(block_key)?;

    if new_block.id != old_block.id {
        panic!("no support for changing block id in modify_block");
    }

    let callback = root
        .callback_store
        .one_by_ref::<ModifyBlockCallback>(CallbackRef::Block(new_block.id))
        .unwrap();
    callback.0(root, block_key, new_block)
}

pub fn place_entity(root: &mut RootMut, entity: Entity) -> Result<u32, FieldError> {
    let callback = root
        .callback_store
        .one_by_ref::<PlaceEntityCallback>(CallbackRef::Entity(entity.id))
        .unwrap();
    callback.0(root, entity)
}

pub fn break_entity(root: &mut RootMut, entity_key: u32) -> Result<Entity, FieldError> {
    let entity = root.entity_field.get(entity_key)?;
    let callback = root
        .callback_store
        .one_by_ref::<BreakEntityCallback>(CallbackRef::Entity(entity.id))
        .unwrap();
    callback.0(root, entity_key)
}

pub fn modify_entity(
    root: &mut RootMut,
    entity_key: u32,
    new_entity: Entity,
) -> Result<Entity, FieldError> {
    let old_entity = root.entity_field.get(entity_key)?;

    if new_entity.id != old_entity.id {
        panic!("no support for changing entity id in modify_entity");
    }

    let callback = root
        .callback_store
        .one_by_ref::<ModifyEntityCallback>(CallbackRef::Entity(new_entity.id))
        .unwrap();
    callback.0(root, entity_key, new_entity)
}

pub fn generate_chunk(root: &mut RootMut, rect: [Vec2; 2]) {
    let callback = root.callback_store.iter::<GenerateCallback>();
    callback.for_each(|f| f.0(root, rect));
}

// TODO: fix this function
pub fn move_entity(
    root: &mut RootMut,
    entity_key: u32,
    new_location: Vec2,
) -> Result<(), FieldError> {
    let entity = root.entity_field.get(entity_key)?;

    if let Ok(rect) = root.entity_field.get_collision_rect(entity_key) {
        let delta = [
            new_location[0] - entity.location[0],
            new_location[1] - entity.location[1],
        ];

        let rect = [
            [rect[0][0] + delta[0], rect[0][1] + delta[1]],
            [rect[1][0] + delta[0], rect[1][1] + delta[1]],
        ];

        if root.tile_field.has_collision_by_rect(rect) {
            return Err(FieldError::Conflict);
        }
        if root.block_field.has_collision_by_rect(rect) {
            return Err(FieldError::Conflict);
        }
        if root
            .entity_field
            .get_collision_by_rect(rect)
            .any(|other_entity_key| other_entity_key != entity_key)
        {
            return Err(FieldError::Conflict);
        }
    }

    let entity = root.entity_field.get(entity_key)?;
    let new_entity = Entity {
        location: new_location,
        ..entity.clone()
    };

    modify_entity(root, entity_key, new_entity)?;

    Ok(())
}

// base tile callback

#[derive(Debug, Clone)]
pub struct BaseTile {
    pub tile_id: u32,
}

impl BaseTile {
    #[inline]
    fn place_tile(&self, root: &mut RootMut, tile: Tile) -> Result<u32, FieldError> {
        root.tile_field.insert(tile)
    }

    #[inline]
    fn break_tile(&self, root: &mut RootMut, tile_key: u32) -> Result<Tile, FieldError> {
        root.tile_field.remove(tile_key)
    }

    #[inline]
    fn modify_tile(
        &self,
        root: &mut RootMut,
        tile_key: u32,
        new_tile: Tile,
    ) -> Result<Tile, FieldError> {
        root.tile_field.modify(tile_key, new_tile)
    }
}

impl CallbackBundle for BaseTile {
    fn insert(&self, builder: &mut CallbackStoreBuilder) {
        let slf = std::rc::Rc::new(self.clone());
        builder.insert(
            CallbackRef::Tile(self.tile_id),
            PlaceTileCallback(Box::new({
                let slf = slf.clone();
                move |root, tile| slf.place_tile(root, tile)
            })),
        );
        builder.insert(
            CallbackRef::Tile(self.tile_id),
            BreakTileCallback(Box::new({
                let slf = slf.clone();
                move |root, tile_key| slf.break_tile(root, tile_key)
            })),
        );
        builder.insert(
            CallbackRef::Tile(self.tile_id),
            ModifyTileCallback(Box::new({
                let slf = slf.clone();
                move |root, tile_key, new_tile| slf.modify_tile(root, tile_key, new_tile)
            })),
        );
    }
}

// base block callback

#[derive(Debug, Clone)]
pub struct BaseBlock {
    pub block_id: u32,
}

impl BaseBlock {
    #[inline]
    fn place_block(&self, root: &mut RootMut, block: Block) -> Result<u32, FieldError> {
        root.block_field.insert(block)
    }

    #[inline]
    fn break_block(&self, root: &mut RootMut, block_key: u32) -> Result<Block, FieldError> {
        root.block_field.remove(block_key)
    }

    #[inline]
    fn modify_block(
        &self,
        root: &mut RootMut,
        block_key: u32,
        new_block: Block,
    ) -> Result<Block, FieldError> {
        root.block_field.modify(block_key, new_block)
    }
}

impl CallbackBundle for BaseBlock {
    fn insert(&self, builder: &mut CallbackStoreBuilder) {
        let slf = std::rc::Rc::new(self.clone());
        builder.insert(
            CallbackRef::Block(self.block_id),
            PlaceBlockCallback(Box::new({
                let slf = slf.clone();
                move |root, block| slf.place_block(root, block)
            })),
        );
        builder.insert(
            CallbackRef::Block(self.block_id),
            BreakBlockCallback(Box::new({
                let slf = slf.clone();
                move |root, block_key| slf.break_block(root, block_key)
            })),
        );
        builder.insert(
            CallbackRef::Block(self.block_id),
            ModifyBlockCallback(Box::new({
                let slf = slf.clone();
                move |root, block_key, new_block| slf.modify_block(root, block_key, new_block)
            })),
        );
    }
}

// base entity callback

#[derive(Debug, Clone)]
pub struct BaseEntity {
    pub entity_id: u32,
}

impl BaseEntity {
    #[inline]
    fn place_entity(&self, root: &mut RootMut, entity: Entity) -> Result<u32, FieldError> {
        root.entity_field.insert(entity)
    }

    #[inline]
    fn break_entity(&self, root: &mut RootMut, entity_key: u32) -> Result<Entity, FieldError> {
        root.entity_field.remove(entity_key)
    }

    #[inline]
    fn modify_entity(
        &self,
        root: &mut RootMut,
        entity_key: u32,
        new_entity: Entity,
    ) -> Result<Entity, FieldError> {
        root.entity_field.modify(entity_key, new_entity)
    }
}

impl CallbackBundle for BaseEntity {
    fn insert(&self, builder: &mut CallbackStoreBuilder) {
        let slf = std::rc::Rc::new(self.clone());
        builder.insert(
            CallbackRef::Entity(self.entity_id),
            PlaceEntityCallback(Box::new({
                let slf = slf.clone();
                move |root, entity| slf.place_entity(root, entity)
            })),
        );
        builder.insert(
            CallbackRef::Entity(self.entity_id),
            BreakEntityCallback(Box::new({
                let slf = slf.clone();
                move |root, entity_key| slf.break_entity(root, entity_key)
            })),
        );
        builder.insert(
            CallbackRef::Entity(self.entity_id),
            ModifyEntityCallback(Box::new({
                let slf = slf.clone();
                move |root, entity_key, new_entity| slf.modify_entity(root, entity_key, new_entity)
            })),
        );
    }
}

// animal entity callback

#[derive(Debug, Clone)]
pub struct AnimalEntity {
    pub entity_id: u32,
    pub min_rest_secs: f32,
    pub max_rest_secs: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub speed: f32,
}

impl AnimalEntity {
    #[inline]
    fn place_entity(&self, root: &mut RootMut, entity: Entity) -> Result<u32, FieldError> {
        let location = entity.location;

        let entity_key = root.entity_field.insert(entity)?;

        let node = RandomWalkNode {
            min_rest_secs: self.min_rest_secs,
            max_rest_secs: self.max_rest_secs,
            min_distance: self.min_distance,
            max_distance: self.max_distance,
            speed: self.speed,
            state: RandomWalkState::Init,
        };
        root.node_store
            .insert(RefKey::Entity(entity_key), SpcKey::from(location), node)
            .unwrap();

        Ok(entity_key)
    }

    #[inline]
    fn break_entity(&self, root: &mut RootMut, entity_key: u32) -> Result<Entity, FieldError> {
        let node_key = *root
            .node_store
            .one_by_ref::<RandomWalkNode>(RefKey::Entity(entity_key))
            .unwrap();
        root.node_store.remove::<RandomWalkNode>(node_key).unwrap();

        root.entity_field.remove(entity_key)
    }

    #[inline]
    fn modify_entity(
        &self,
        root: &mut RootMut,
        entity_key: u32,
        new_entity: Entity,
    ) -> Result<Entity, FieldError> {
        let location = new_entity.location;

        let old_entity = root.entity_field.modify(entity_key, new_entity)?;

        let node_key = *root
            .node_store
            .one_by_ref::<RandomWalkNode>(RefKey::Entity(entity_key))
            .unwrap();
        root.node_store
            .modify::<RandomWalkNode, _>(node_key, |_, spc, _| *spc = SpcKey::from(location));

        Ok(old_entity)
    }
}

impl CallbackBundle for AnimalEntity {
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
        builder.insert(
            CallbackRef::Entity(self.entity_id),
            ModifyEntityCallback(Box::new({
                let slf = slf.clone();
                move |root, entity_key, new_entity| slf.modify_entity(root, entity_key, new_entity)
            })),
        );
    }
}

// generator callback

#[derive(Debug, Clone)]
pub struct GeneratorNode {
    pub prev_rect: Option<[IVec2; 2]>,
    pub visited_chunk: ahash::AHashSet<IVec2>,
}

#[derive(Debug, Clone)]
pub struct Generator {}

impl Generator {
    const CHUNK_SIZE: u32 = 32;

    fn before(&self, root: &mut RootMut) {
        let node = GeneratorNode {
            prev_rect: None,
            visited_chunk: Default::default(),
        };
        root.node_store.insert(RefKey::Global, SpcKey::GLOBAL, node);
    }

    fn after(&self, root: &mut RootMut) {
        root.node_store
            .remove_by_ref::<GeneratorNode>(RefKey::Global);
    }

    pub fn generate_chunk(&self, root: &mut RootMut, rect: [Vec2; 2]) {
        let node_key = *root.node_store.one::<GeneratorNode>().unwrap();
        let (_, _, node) = root.node_store.get::<GeneratorNode>(node_key).unwrap();

        #[rustfmt::skip]
        let rect = [[
            rect[0][0].div_euclid(Self::CHUNK_SIZE as f32) as i32,
            rect[0][1].div_euclid(Self::CHUNK_SIZE as f32) as i32, ], [
            rect[1][0].div_euclid(Self::CHUNK_SIZE as f32) as i32,
            rect[1][1].div_euclid(Self::CHUNK_SIZE as f32) as i32,
        ]];

        if Some(rect) != node.prev_rect {
            for y in rect[0][1]..=rect[1][1] {
                for x in rect[0][0]..=rect[1][0] {
                    let chunk_key = [x, y];

                    let (_, _, node) = root.node_store.get::<GeneratorNode>(node_key).unwrap();
                    if node.visited_chunk.contains(&chunk_key) {
                        continue;
                    }

                    root.node_store
                        .modify::<GeneratorNode, _>(node_key, |_, _, node| {
                            node.visited_chunk.insert(chunk_key);
                        });

                    for v in 0..Self::CHUNK_SIZE {
                        for u in 0..Self::CHUNK_SIZE {
                            let id = rand::Rng::gen_range(&mut rand::thread_rng(), 0..=1);
                            let location = [
                                x * Self::CHUNK_SIZE as i32 + u as i32,
                                y * Self::CHUNK_SIZE as i32 + v as i32,
                            ];
                            let _ = place_tile(root, Tile::new(id, location, 0));
                        }
                    }

                    for _ in 0..64 {
                        let id = rand::Rng::gen_range(&mut rand::thread_rng(), 0..=3);
                        let u = rand::Rng::gen_range(&mut rand::thread_rng(), 0..Self::CHUNK_SIZE);
                        let v = rand::Rng::gen_range(&mut rand::thread_rng(), 0..Self::CHUNK_SIZE);
                        let location = [
                            x * Self::CHUNK_SIZE as i32 + u as i32,
                            y * Self::CHUNK_SIZE as i32 + v as i32,
                        ];
                        let _ = place_block(root, Block::new(id, location, 0));
                    }

                    for _ in 0..64 {
                        let id = rand::Rng::gen_range(&mut rand::thread_rng(), 1..=5);
                        let u = rand::Rng::gen_range(
                            &mut rand::thread_rng(),
                            0.0..Self::CHUNK_SIZE as f32,
                        );
                        let v = rand::Rng::gen_range(
                            &mut rand::thread_rng(),
                            0.0..Self::CHUNK_SIZE as f32,
                        );
                        let location = [
                            x as f32 * Self::CHUNK_SIZE as f32 + u,
                            y as f32 * Self::CHUNK_SIZE as f32 + v,
                        ];
                        let _ = place_entity(root, Entity::new(id, location, 0));
                    }
                }
            }

            root.node_store
                .modify::<GeneratorNode, _>(node_key, |_, _, node| {
                    node.prev_rect = Some(rect);
                });
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
pub struct RandomWalkForwardLocal {}

impl RandomWalkForwardLocal {
    fn forward_local(root: &mut RootMut, delta_secs: f32, rect: [SpcKey; 2]) {
        for node_key in root.node_store.detach_iter_by_rect::<RandomWalkNode>(rect) {
            let (r#ref, _, node) = root.node_store.get::<RandomWalkNode>(node_key).unwrap();

            let RefKey::Entity(entity_key) = *r#ref else {
                unreachable!();
            };

            match node.state {
                RandomWalkState::Init => {
                    root.node_store
                        .modify::<RandomWalkNode, _>(node_key, |_, _, node| {
                            node.state = RandomWalkState::WaitStart;
                        });
                }
                RandomWalkState::WaitStart => {
                    let secs = rand::Rng::gen_range(
                        &mut rand::thread_rng(),
                        node.min_rest_secs..node.max_rest_secs,
                    );
                    root.node_store
                        .modify::<RandomWalkNode, _>(node_key, |_, _, node| {
                            node.state = RandomWalkState::Wait(secs);
                        });
                }
                RandomWalkState::Wait(secs) => {
                    let new_secs = secs - delta_secs;
                    if new_secs <= 0.0 {
                        root.node_store
                            .modify::<RandomWalkNode, _>(node_key, |_, _, node| {
                                node.state = RandomWalkState::TripStart;
                            });
                    } else {
                        root.node_store
                            .modify::<RandomWalkNode, _>(node_key, |_, _, node| {
                                node.state = RandomWalkState::Wait(new_secs);
                            });
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
                    root.node_store
                        .modify::<RandomWalkNode, _>(node_key, |_, _, node| {
                            node.state = RandomWalkState::Trip(destination);
                        });
                }
                RandomWalkState::Trip(destination) => {
                    let entity = root.entity_field.get(entity_key).unwrap();
                    if entity.location != destination {
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
                        if move_entity(root, entity_key, location).is_ok() {
                            root.node_store
                                .modify::<RandomWalkNode, _>(node_key, |_, _, node| {
                                    node.state = RandomWalkState::Trip(destination);
                                });
                        } else {
                            root.node_store
                                .modify::<RandomWalkNode, _>(node_key, |_, _, node| {
                                    node.state = RandomWalkState::WaitStart;
                                });
                        }
                    } else {
                        root.node_store
                            .modify::<RandomWalkNode, _>(node_key, |_, _, node| {
                                node.state = RandomWalkState::WaitStart;
                            });
                    }
                }
            }
        }
    }
}

impl CallbackBundle for RandomWalkForwardLocal {
    fn insert(&self, builder: &mut CallbackStoreBuilder) {
        builder.insert(
            CallbackRef::Global,
            ForwardLocalCallback(Box::new(Self::forward_local)),
        );
    }
}
