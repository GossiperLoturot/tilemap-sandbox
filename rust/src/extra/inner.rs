use crate::inner::*;

// flow definition

pub trait IResource {
    fn before(&self, root: &mut Root);
    fn after(&self, root: &mut Root);
}

pub trait ITile {
    fn place_tile(&self, root: &mut Root, tile: Tile) -> Result<u32, FieldError>;
    fn break_tile(&self, root: &mut Root, tile_key: u32) -> Result<Tile, FieldError>;
    fn modify_tile(
        &self,
        root: &mut Root,
        tile_key: u32,
        new_tile: Tile,
    ) -> Result<Tile, FieldError>;
}

pub trait IBlock {
    fn place_block(&self, root: &mut Root, block: Block) -> Result<u32, FieldError>;
    fn break_block(&self, root: &mut Root, block_key: u32) -> Result<Block, FieldError>;
    fn modify_block(
        &self,
        root: &mut Root,
        block_key: u32,
        new_block: Block,
    ) -> Result<Block, FieldError>;
}

pub trait IEntity {
    fn place_entity(&self, root: &mut Root, entity: Entity) -> Result<u32, FieldError>;
    fn break_entity(&self, root: &mut Root, entity_key: u32) -> Result<Entity, FieldError>;
    fn modify_entity(
        &self,
        root: &mut Root,
        entity_key: u32,
        new_entity: Entity,
    ) -> Result<Entity, FieldError>;
}

pub trait IForward {
    fn forward(&self, root: &mut Root, delta_secs: f32);
}

pub trait IForwardLocal {
    fn forward_local(&self, root: &mut Root, delta_secs: f32, rect: [SpaceKey; 2]);
}

pub trait IGenerate {
    fn generate_chunk(&self, root: &mut Root, rect: [Vec2; 2]);
}

// action

pub fn before(root: &mut Root) {
    for flow in root
        .flow_iter::<std::rc::Rc<dyn IResource>>()
        .cloned()
        .collect::<Vec<_>>()
    {
        flow.before(root);
    }
}

pub fn after(root: &mut Root) {
    for flow in root
        .flow_iter::<std::rc::Rc<dyn IResource>>()
        .cloned()
        .collect::<Vec<_>>()
    {
        flow.after(root);
    }
}

pub fn place_tile(root: &mut Root, tile: Tile) -> Result<u32, FieldError> {
    let flow = root
        .flow_one_by_ref::<std::rc::Rc<dyn ITile>>(FlowRef::Tile(tile.id))
        .cloned()
        .unwrap();
    flow.place_tile(root, tile)
}

pub fn break_tile(root: &mut Root, tile_key: u32) -> Result<Tile, FieldError> {
    let tile = root.tile_get(tile_key)?;
    let flow = root
        .flow_one_by_ref::<std::rc::Rc<dyn ITile>>(FlowRef::Tile(tile.id))
        .cloned()
        .unwrap();
    flow.break_tile(root, tile_key)
}

pub fn modify_tile(root: &mut Root, tile_key: u32, new_tile: Tile) -> Result<Tile, FieldError> {
    let old_tile = root.tile_get(tile_key)?;

    if new_tile.id != old_tile.id {
        panic!("no support for changing tile id in modify_tile");
    }

    let flow = root
        .flow_one_by_ref::<std::rc::Rc<dyn ITile>>(FlowRef::Tile(new_tile.id))
        .cloned()
        .unwrap();
    flow.modify_tile(root, tile_key, new_tile)
}

pub fn place_block(root: &mut Root, block: Block) -> Result<u32, FieldError> {
    let flow = root
        .flow_one_by_ref::<std::rc::Rc<dyn IBlock>>(FlowRef::Block(block.id))
        .cloned()
        .unwrap();
    flow.place_block(root, block)
}

pub fn break_block(root: &mut Root, block_key: u32) -> Result<Block, FieldError> {
    let block = root.block_get(block_key)?;
    let flow = root
        .flow_one_by_ref::<std::rc::Rc<dyn IBlock>>(FlowRef::Block(block.id))
        .cloned()
        .unwrap();
    flow.break_block(root, block_key)
}

pub fn modify_block(
    root: &mut Root,
    block_key: u32,
    new_block: Block,
) -> Result<Block, FieldError> {
    let old_block = root.block_get(block_key)?;

    if new_block.id != old_block.id {
        panic!("no support for changing block id in modify_block");
    }

    let flow = root
        .flow_one_by_ref::<std::rc::Rc<dyn IBlock>>(FlowRef::Block(new_block.id))
        .cloned()
        .unwrap();
    flow.modify_block(root, block_key, new_block)
}

pub fn place_entity(root: &mut Root, entity: Entity) -> Result<u32, FieldError> {
    let flow = root
        .flow_one_by_ref::<std::rc::Rc<dyn IEntity>>(FlowRef::Entity(entity.id))
        .cloned()
        .unwrap();
    flow.place_entity(root, entity)
}

pub fn break_entity(root: &mut Root, entity_key: u32) -> Result<Entity, FieldError> {
    let entity = root.entity_get(entity_key)?;
    let flow = root
        .flow_one_by_ref::<std::rc::Rc<dyn IEntity>>(FlowRef::Entity(entity.id))
        .cloned()
        .unwrap();
    flow.break_entity(root, entity_key)
}

pub fn modify_entity(
    root: &mut Root,
    entity_key: u32,
    new_entity: Entity,
) -> Result<Entity, FieldError> {
    let old_entity = root.entity_get(entity_key)?;

    if new_entity.id != old_entity.id {
        panic!("no support for changing entity id in modify_entity");
    }

    let flow = root
        .flow_one_by_ref::<std::rc::Rc<dyn IEntity>>(FlowRef::Entity(new_entity.id))
        .cloned()
        .unwrap();
    flow.modify_entity(root, entity_key, new_entity)
}

pub fn forward(root: &mut Root, delta_secs: f32) {
    for flow in root
        .flow_iter::<std::rc::Rc<dyn IForward>>()
        .cloned()
        .collect::<Vec<_>>()
    {
        flow.forward(root, delta_secs);
    }
}

pub fn forward_local(root: &mut Root, delta_secs: f32, rect: [SpaceKey; 2]) {
    for flow in root
        .flow_iter::<std::rc::Rc<dyn IForwardLocal>>()
        .cloned()
        .collect::<Vec<_>>()
    {
        flow.forward_local(root, delta_secs, rect);
    }
}

pub fn generate_chunk(root: &mut Root, rect: [Vec2; 2]) {
    for flow in root
        .flow_iter::<std::rc::Rc<dyn IGenerate>>()
        .cloned()
        .collect::<Vec<_>>()
    {
        flow.generate_chunk(root, rect);
    }
}

// TODO: fix this function
pub fn move_entity(root: &mut Root, entity_key: u32, new_location: Vec2) -> Result<(), FieldError> {
    let entity = root.entity_get(entity_key)?;

    if let Ok(rect) = root.entity_get_collision_rect(entity_key) {
        let delta = [
            new_location[0] - entity.location[0],
            new_location[1] - entity.location[1],
        ];

        let rect = [
            [rect[0][0] + delta[0], rect[0][1] + delta[1]],
            [rect[1][0] + delta[0], rect[1][1] + delta[1]],
        ];

        if root.tile_has_by_collision_rect(rect) {
            return Err(FieldError::Conflict);
        }
        if root.block_has_by_collision_rect(rect) {
            return Err(FieldError::Conflict);
        }
        if root
            .entity_get_by_collision_rect(rect)
            .any(|other_entity_key| other_entity_key != entity_key)
        {
            return Err(FieldError::Conflict);
        }
    }

    let entity = root.entity_get(entity_key)?;
    let new_entity = Entity {
        location: new_location,
        ..entity.clone()
    };

    modify_entity(root, entity_key, new_entity)?;

    Ok(())
}

// base tile flow

#[derive(Debug, Clone)]
pub struct BaseTile {
    pub tile_id: u32,
}

impl ITile for BaseTile {
    #[inline]
    fn place_tile(&self, root: &mut Root, tile: Tile) -> Result<u32, FieldError> {
        root.tile_insert(tile)
    }

    #[inline]
    fn break_tile(&self, root: &mut Root, tile_key: u32) -> Result<Tile, FieldError> {
        root.tile_remove(tile_key)
    }

    #[inline]
    fn modify_tile(
        &self,
        root: &mut Root,
        tile_key: u32,
        new_tile: Tile,
    ) -> Result<Tile, FieldError> {
        root.tile_modify(tile_key, new_tile)
    }
}

impl FlowBundle for BaseTile {
    fn insert(&self, buf: &mut FlowBuffer) {
        let slf = std::rc::Rc::new(self.clone());
        buf.register::<std::rc::Rc<dyn ITile>>(FlowRef::Tile(self.tile_id), slf);
    }
}

// base block flow

#[derive(Debug, Clone)]
pub struct BaseBlock {
    pub block_id: u32,
}

impl IBlock for BaseBlock {
    #[inline]
    fn place_block(&self, root: &mut Root, block: Block) -> Result<u32, FieldError> {
        root.block_insert(block)
    }

    #[inline]
    fn break_block(&self, root: &mut Root, block_key: u32) -> Result<Block, FieldError> {
        root.block_remove(block_key)
    }

    #[inline]
    fn modify_block(
        &self,
        root: &mut Root,
        block_key: u32,
        new_block: Block,
    ) -> Result<Block, FieldError> {
        root.block_modify(block_key, new_block)
    }
}

impl FlowBundle for BaseBlock {
    fn insert(&self, buf: &mut FlowBuffer) {
        let slf = std::rc::Rc::new(self.clone());
        buf.register::<std::rc::Rc<dyn IBlock>>(FlowRef::Block(self.block_id), slf);
    }
}

// base entity flow

#[derive(Debug, Clone)]
pub struct BaseEntity {
    pub entity_id: u32,
}

impl IEntity for BaseEntity {
    #[inline]
    fn place_entity(&self, root: &mut Root, entity: Entity) -> Result<u32, FieldError> {
        root.entity_insert(entity)
    }

    #[inline]
    fn break_entity(&self, root: &mut Root, entity_key: u32) -> Result<Entity, FieldError> {
        root.entity_remove(entity_key)
    }

    #[inline]
    fn modify_entity(
        &self,
        root: &mut Root,
        entity_key: u32,
        new_entity: Entity,
    ) -> Result<Entity, FieldError> {
        root.entity_modify(entity_key, new_entity)
    }
}

impl FlowBundle for BaseEntity {
    fn insert(&self, buf: &mut FlowBuffer) {
        let slf = std::rc::Rc::new(self.clone());
        buf.register::<std::rc::Rc<dyn IEntity>>(FlowRef::Entity(self.entity_id), slf);
    }
}

// animal entity flow

#[derive(Debug, Clone)]
pub struct AnimalEntity {
    pub entity_id: u32,
    pub min_rest_secs: f32,
    pub max_rest_secs: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub speed: f32,
}

impl IEntity for AnimalEntity {
    #[inline]
    fn place_entity(&self, root: &mut Root, entity: Entity) -> Result<u32, FieldError> {
        let location = entity.location;

        let entity_key = root.entity_insert(entity)?;

        let tag = RandomWalkTag {
            min_rest_secs: self.min_rest_secs,
            max_rest_secs: self.max_rest_secs,
            min_distance: self.min_distance,
            max_distance: self.max_distance,
            speed: self.speed,
            state: RandomWalkState::Init,
        };
        root.tag_insert(RefKey::Entity(entity_key), SpaceKey::from(location), tag)
            .unwrap();

        Ok(entity_key)
    }

    #[inline]
    fn break_entity(&self, root: &mut Root, entity_key: u32) -> Result<Entity, FieldError> {
        let tag_key = *root
            .tag_one_by_ref::<RandomWalkTag>(RefKey::Entity(entity_key))
            .unwrap();

        root.tag_remove::<RandomWalkTag>(tag_key).unwrap();

        root.entity_remove(entity_key)
    }

    #[inline]
    fn modify_entity(
        &self,
        root: &mut Root,
        entity_key: u32,
        new_entity: Entity,
    ) -> Result<Entity, FieldError> {
        let location = new_entity.location;

        let old_entity = root.entity_modify(entity_key, new_entity)?;

        let tag_key = *root
            .tag_one_by_ref::<RandomWalkTag>(RefKey::Entity(entity_key))
            .unwrap();

        root.tag_modify::<RandomWalkTag>(tag_key, |_, spc, _| {
            *spc = SpaceKey::from(location);
        });

        Ok(old_entity)
    }
}

impl FlowBundle for AnimalEntity {
    fn insert(&self, buf: &mut FlowBuffer) {
        let slf = std::rc::Rc::new(self.clone());
        buf.register::<std::rc::Rc<dyn IEntity>>(FlowRef::Entity(self.entity_id), slf);
    }
}

// generator flow

#[derive(Debug, Clone)]
pub struct GeneratorResource {
    pub prev_rect: Option<[IVec2; 2]>,
    pub visited_chunk: ahash::AHashSet<IVec2>,
}

#[derive(Debug, Clone)]
pub struct Generator {}

impl IResource for Generator {
    fn before(&self, root: &mut Root) {
        let resource = GeneratorResource {
            prev_rect: None,
            visited_chunk: Default::default(),
        };
        root.resource_insert(resource).unwrap();
    }

    fn after(&self, root: &mut Root) {
        root.resource_remove::<GeneratorResource>();
    }
}

impl IGenerate for Generator {
    fn generate_chunk(&self, root: &mut Root, rect: [Vec2; 2]) {
        const CHUNK_SIZE: u32 = 32;

        #[rustfmt::skip]
        let rect = [[
            rect[0][0].div_euclid(CHUNK_SIZE as f32) as i32,
            rect[0][1].div_euclid(CHUNK_SIZE as f32) as i32, ], [
            rect[1][0].div_euclid(CHUNK_SIZE as f32) as i32,
            rect[1][1].div_euclid(CHUNK_SIZE as f32) as i32,
        ]];

        let tag = root.resource_get::<GeneratorResource>().unwrap();
        if Some(rect) != tag.prev_rect {
            for y in rect[0][1]..=rect[1][1] {
                for x in rect[0][0]..=rect[1][0] {
                    let chunk_key = [x, y];

                    let tag = root.resource_get::<GeneratorResource>().unwrap();
                    if tag.visited_chunk.contains(&chunk_key) {
                        continue;
                    }

                    let tag = root.resource_get_mut::<GeneratorResource>().unwrap();
                    tag.visited_chunk.insert(chunk_key);

                    for v in 0..CHUNK_SIZE {
                        for u in 0..CHUNK_SIZE {
                            let id = rand::Rng::gen_range(&mut rand::thread_rng(), 0..=1);
                            let location = [
                                x * CHUNK_SIZE as i32 + u as i32,
                                y * CHUNK_SIZE as i32 + v as i32,
                            ];
                            let _ = place_tile(root, Tile::new(id, location, 0));
                        }
                    }

                    for _ in 0..64 {
                        let id = rand::Rng::gen_range(&mut rand::thread_rng(), 0..=3);
                        let u = rand::Rng::gen_range(&mut rand::thread_rng(), 0..CHUNK_SIZE);
                        let v = rand::Rng::gen_range(&mut rand::thread_rng(), 0..CHUNK_SIZE);
                        let location = [
                            x * CHUNK_SIZE as i32 + u as i32,
                            y * CHUNK_SIZE as i32 + v as i32,
                        ];
                        let _ = place_block(root, Block::new(id, location, 0));
                    }

                    for _ in 0..64 {
                        let id = rand::Rng::gen_range(&mut rand::thread_rng(), 1..=5);
                        let u =
                            rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..CHUNK_SIZE as f32);
                        let v =
                            rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..CHUNK_SIZE as f32);
                        let location = [
                            x as f32 * CHUNK_SIZE as f32 + u,
                            y as f32 * CHUNK_SIZE as f32 + v,
                        ];
                        let _ = place_entity(root, Entity::new(id, location, 0));
                    }
                }
            }

            let tag = root.resource_get_mut::<GeneratorResource>().unwrap();
            tag.prev_rect = Some(rect);
        }
    }
}

impl FlowBundle for Generator {
    fn insert(&self, buf: &mut FlowBuffer) {
        let slf = std::rc::Rc::new(self.clone());
        buf.register::<std::rc::Rc<dyn IResource>>(FlowRef::Global, slf.clone());
        buf.register::<std::rc::Rc<dyn IGenerate>>(FlowRef::Global, slf);
    }
}

// random walk flow

#[derive(Debug, Clone)]
pub enum RandomWalkState {
    Init,
    WaitStart,
    Wait(f32),
    TripStart,
    Trip(Vec2),
}

#[derive(Debug, Clone)]
pub struct RandomWalkTag {
    pub min_rest_secs: f32,
    pub max_rest_secs: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub speed: f32,
    pub state: RandomWalkState,
}

#[derive(Debug, Clone)]
pub struct RandomWalkForwardLocal {}

impl IForwardLocal for RandomWalkForwardLocal {
    fn forward_local(&self, root: &mut Root, delta_secs: f32, rect: [SpaceKey; 2]) {
        for tag_key in root.tag_detach_iter_by_rect::<RandomWalkTag>(rect) {
            let (r#ref, _, tag) = root.tag_get::<RandomWalkTag>(tag_key).unwrap();

            let entity_key = match *r#ref {
                RefKey::Entity(entity_key) => entity_key,
                _ => continue,
            };

            match tag.state {
                RandomWalkState::Init => {
                    root.tag_modify::<RandomWalkTag>(tag_key, |_, _, tag| {
                        tag.state = RandomWalkState::WaitStart;
                    });
                }
                RandomWalkState::WaitStart => {
                    let secs = rand::Rng::gen_range(
                        &mut rand::thread_rng(),
                        tag.min_rest_secs..tag.max_rest_secs,
                    );
                    root.tag_modify::<RandomWalkTag>(tag_key, |_, _, tag| {
                        tag.state = RandomWalkState::Wait(secs);
                    });
                }
                RandomWalkState::Wait(secs) => {
                    let new_secs = secs - delta_secs;
                    if new_secs <= 0.0 {
                        root.tag_modify::<RandomWalkTag>(tag_key, |_, _, tag| {
                            tag.state = RandomWalkState::TripStart;
                        });
                    } else {
                        root.tag_modify::<RandomWalkTag>(tag_key, |_, _, tag| {
                            tag.state = RandomWalkState::Wait(new_secs);
                        });
                    }
                }
                RandomWalkState::TripStart => {
                    let entity = root.entity_get(entity_key).unwrap();
                    let distance = rand::Rng::gen_range(
                        &mut rand::thread_rng(),
                        tag.min_distance..tag.max_distance,
                    );
                    let direction = rand::Rng::gen_range(
                        &mut rand::thread_rng(),
                        0.0..std::f32::consts::PI * 2.0,
                    );
                    let destination = [
                        entity.location[0] + distance * direction.cos(),
                        entity.location[1] + distance * direction.sin(),
                    ];
                    root.tag_modify::<RandomWalkTag>(tag_key, |_, _, tag| {
                        tag.state = RandomWalkState::Trip(destination);
                    });
                }
                RandomWalkState::Trip(destination) => {
                    let entity = root.entity_get(entity_key).unwrap();
                    if entity.location != destination {
                        let diff = [
                            destination[0] - entity.location[0],
                            destination[1] - entity.location[1],
                        ];
                        let distance = (diff[0].powi(2) + diff[1].powi(2)).sqrt();
                        let direction = [diff[0] / distance, diff[1] / distance];
                        let delta_distance = distance.min(tag.speed * delta_secs);
                        let location = [
                            entity.location[0] + direction[0] * delta_distance,
                            entity.location[1] + direction[1] * delta_distance,
                        ];
                        if move_entity(root, entity_key, location).is_ok() {
                            root.tag_modify::<RandomWalkTag>(tag_key, |_, _, tag| {
                                tag.state = RandomWalkState::Trip(destination);
                            });
                        } else {
                            root.tag_modify::<RandomWalkTag>(tag_key, |_, _, tag| {
                                tag.state = RandomWalkState::WaitStart;
                            });
                        }
                    } else {
                        root.tag_modify::<RandomWalkTag>(tag_key, |_, _, tag| {
                            tag.state = RandomWalkState::WaitStart;
                        });
                    }
                }
            }
        }
    }
}

impl FlowBundle for RandomWalkForwardLocal {
    fn insert(&self, buf: &mut FlowBuffer) {
        let slf = std::rc::Rc::new(self.clone());
        buf.register::<std::rc::Rc<dyn IForwardLocal>>(FlowRef::Global, slf);
    }
}
