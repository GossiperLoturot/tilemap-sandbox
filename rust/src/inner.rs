pub type Vec2 = [f32; 2];
pub type IVec2 = [i32; 2];

type RectNode<T, U> = rstar::primitives::GeomWithData<rstar::primitives::Rectangle<T>, U>;

#[derive(Debug, Clone)]
struct UnstableTileKey {
    chunk_key: IVec2,
    tile_key: u32,
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub id: u32,
    pub location: IVec2,
}

#[derive(Debug, Clone, Default)]
pub struct TileChunk {
    pub serial: u32,
    pub tiles: slab::Slab<Tile>,
}

#[derive(Debug, Clone)]
pub struct TileField {
    chunk_size: u32,
    chunks: ahash::AHashMap<IVec2, TileChunk>,
    stable_ref: slab::Slab<UnstableTileKey>,
    spatial_ref: ahash::AHashMap<IVec2, u32>,
}

impl TileField {
    pub fn new(chunk_size: u32) -> Self {
        Self {
            chunk_size,
            stable_ref: Default::default(),
            chunks: Default::default(),
            spatial_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, tile: Tile) -> Option<u32> {
        // check by spatial features
        if self.spatial_ref.contains_key(&tile.location) {
            return None;
        }

        let chunk_key = [
            tile.location[0].div_euclid(self.chunk_size as i32),
            tile.location[1].div_euclid(self.chunk_size as i32),
        ];
        let chunk = self.chunks.entry(chunk_key).or_default();
        let tile_key = chunk.tiles.insert(tile.clone()) as u32;
        chunk.serial += 1;
        let ukey = UnstableTileKey {
            chunk_key,
            tile_key,
        };

        let key = self.stable_ref.insert(ukey) as u32;

        // spatial features
        self.spatial_ref.insert(tile.location, key);

        Some(key)
    }

    pub fn remove(&mut self, key: u32) -> Option<Tile> {
        let ukey = self.stable_ref.try_remove(key as usize)?;
        let chunk = self.chunks.get_mut(&ukey.chunk_key).unwrap();
        let tile = chunk.tiles.try_remove(ukey.tile_key as usize).unwrap();
        chunk.serial += 1;

        // spatial features
        self.spatial_ref.remove(&tile.location).unwrap();

        Some(tile)
    }

    pub fn modify(&mut self, key: u32, new_tile: Tile) -> Option<Tile> {
        // validate modification

        if !self.stable_ref.contains(key as usize) {
            return None;
        }

        if !self
            .spatial_ref
            .get(&new_tile.location)
            .map_or(true, |other_key| *other_key == key)
        {
            return None;
        }

        // remove old tile

        let ukey = self.stable_ref.get(key as usize).unwrap();
        let chunk = self.chunks.get_mut(&ukey.chunk_key).unwrap();
        let tile = chunk.tiles.try_remove(ukey.tile_key as usize).unwrap();
        chunk.serial += 1;

        // spatial features
        self.spatial_ref.remove(&tile.location).unwrap();

        // insert new tile

        let chunk_key = [
            new_tile.location[0].div_euclid(self.chunk_size as i32),
            new_tile.location[1].div_euclid(self.chunk_size as i32),
        ];
        let chunk = self.chunks.entry(chunk_key).or_default();
        let tile_key = chunk.tiles.insert(new_tile.clone()) as u32;
        chunk.serial += 1;
        let ukey = UnstableTileKey {
            chunk_key,
            tile_key,
        };

        *self.stable_ref.get_mut(key as usize).unwrap() = ukey;

        // spatial features
        self.spatial_ref.insert(new_tile.location, key);

        Some(tile)
    }

    pub fn get(&self, key: u32) -> Option<&Tile> {
        let ukey = self.stable_ref.get(key as usize)?;
        let chunk = self.chunks.get(&ukey.chunk_key).unwrap();
        let tile = chunk.tiles.get(ukey.tile_key as usize).unwrap();
        Some(tile)
    }

    pub fn get_chunk(&self, chunk_key: IVec2) -> Option<&TileChunk> {
        self.chunks.get(&chunk_key)
    }

    // spatial features

    pub fn has_by_point(&self, point: IVec2) -> bool {
        self.spatial_ref.contains_key(&point)
    }

    pub fn get_by_point(&self, point: IVec2) -> Option<u32> {
        self.spatial_ref.get(&point).copied()
    }
}

#[derive(Debug, Clone)]
pub struct BlockSpec {
    pub size: IVec2,
    pub collision_size: Vec2,
    pub collision_offset: Vec2,
    pub hint_size: Vec2,
    pub hint_offset: Vec2,
}

impl BlockSpec {
    #[rustfmt::skip]
    fn rect(&self, location: IVec2) -> [IVec2; 2] {
        [[
            location[0],
            location[1], ], [
            location[0] + self.size[0] - 1,
            location[1] + self.size[1] - 1,
        ]]
    }

    #[rustfmt::skip]
    fn collision_rect(&self, location: IVec2) -> [Vec2; 2] {
        [[
            location[0] as f32 + self.collision_offset[0],
            location[1] as f32 + self.collision_offset[1], ], [
            location[0] as f32 + self.collision_offset[0] + self.collision_size[0],
            location[1] as f32 + self.collision_offset[1] + self.collision_size[1],
        ]]
    }

    #[rustfmt::skip]
    fn hint_rect(&self, location: IVec2) -> [Vec2; 2] {
        [[
            location[0] as f32 + self.hint_offset[0],
            location[1] as f32 + self.hint_offset[1], ], [
            location[0] as f32 + self.hint_offset[0] + self.hint_size[0],
            location[1] as f32 + self.hint_offset[1] + self.hint_size[1],
        ]]
    }
}

#[derive(Debug, Clone)]
struct UnstableBlockKey {
    chunk_key: IVec2,
    block_key: u32,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub id: u32,
    pub location: IVec2,
}

#[derive(Debug, Clone, Default)]
pub struct BlockChunk {
    pub serial: u32,
    pub blocks: slab::Slab<Block>,
}

#[derive(Debug, Clone)]
pub struct BlockField {
    chunk_size: u32,
    specs: Vec<BlockSpec>,
    chunks: ahash::AHashMap<IVec2, BlockChunk>,
    stable_ref: slab::Slab<UnstableBlockKey>,
    spatial_ref: rstar::RTree<RectNode<IVec2, u32>>,
    collision_ref: rstar::RTree<RectNode<Vec2, u32>>,
    hint_ref: rstar::RTree<RectNode<Vec2, u32>>,
}

impl BlockField {
    pub fn new(chunk_size: u32, specs: Vec<BlockSpec>) -> Self {
        // validate specs
        specs.iter().for_each(|spec| {
            assert!(spec.size[0] > 0);
            assert!(spec.size[1] > 0);
            assert!(spec.collision_size[0] >= 0.0);
            assert!(spec.collision_size[1] >= 0.0);
            assert!(spec.hint_size[0] >= 0.0);
            assert!(spec.hint_size[1] >= 0.0);
        });

        Self {
            chunk_size,
            specs,
            chunks: Default::default(),
            stable_ref: Default::default(),
            spatial_ref: Default::default(),
            collision_ref: Default::default(),
            hint_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, block: Block) -> Option<u32> {
        let spec = &self.specs[block.id as usize];

        // check by spatial features
        if self.has_by_rect(spec.rect(block.location)) {
            return None;
        }

        let chunk_key = [
            block.location[0].div_euclid(self.chunk_size as i32),
            block.location[1].div_euclid(self.chunk_size as i32),
        ];
        let chunk = self.chunks.entry(chunk_key).or_default();
        let block_key = chunk.blocks.insert(block.clone()) as u32;
        chunk.serial += 1;
        let ukey = UnstableBlockKey {
            chunk_key,
            block_key,
        };

        let key = self.stable_ref.insert(ukey) as u32;

        // spatial features
        let rect = spec.rect(block.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = rstar::primitives::GeomWithData::new(rect, key);
        self.spatial_ref.insert(node);

        // collision features
        let rect = spec.collision_rect(block.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = rstar::primitives::GeomWithData::new(rect, key);
        self.collision_ref.insert(node);

        // hint features
        let rect = spec.hint_rect(block.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = rstar::primitives::GeomWithData::new(rect, key);
        self.hint_ref.insert(node);

        Some(key)
    }

    pub fn remove(&mut self, key: u32) -> Option<Block> {
        let ukey = self.stable_ref.try_remove(key as usize)?;
        let chunk = self.chunks.get_mut(&ukey.chunk_key).unwrap();
        let block = chunk.blocks.try_remove(ukey.block_key as usize).unwrap();
        chunk.serial += 1;

        let spec = &self.specs[block.id as usize];

        // spatial features
        let rect = spec.rect(block.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = &rstar::primitives::GeomWithData::new(rect, key);
        self.spatial_ref.remove(node);

        // collision features
        let rect = spec.collision_rect(block.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = &rstar::primitives::GeomWithData::new(rect, key);
        self.collision_ref.remove(node);

        // hint features
        let rect = spec.hint_rect(block.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = &rstar::primitives::GeomWithData::new(rect, key);
        self.hint_ref.remove(node);

        Some(block)
    }

    pub fn modify(&mut self, key: u32, new_block: Block) -> Option<Block> {
        // validate modification

        if !self.stable_ref.contains(key as usize) {
            return None;
        }

        // check by spatial features
        let new_spec = &self.specs[new_block.id as usize];
        let rect = new_spec.rect(new_block.location);
        if !self.get_by_rect(rect).all(|other_key| other_key == key) {
            return None;
        }

        // remove old block

        let ukey = self.stable_ref.get(key as usize).unwrap();
        let chunk = self.chunks.get_mut(&ukey.chunk_key).unwrap();
        let block = chunk.blocks.try_remove(ukey.block_key as usize).unwrap();
        chunk.serial += 1;

        let spec = &self.specs[block.id as usize];

        // spatial features
        let rect = spec.rect(block.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = &rstar::primitives::GeomWithData::new(rect, key);
        self.spatial_ref.remove(node);

        // collision features
        let rect = spec.collision_rect(block.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = &rstar::primitives::GeomWithData::new(rect, key);
        self.collision_ref.remove(node);

        // hint features
        let rect = spec.hint_rect(block.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = &rstar::primitives::GeomWithData::new(rect, key);
        self.hint_ref.remove(node);

        // insert new block

        let chunk_key = [
            new_block.location[0].div_euclid(self.chunk_size as i32),
            new_block.location[1].div_euclid(self.chunk_size as i32),
        ];
        let chunk = self.chunks.entry(chunk_key).or_default();
        let block_key = chunk.blocks.insert(new_block.clone()) as u32;
        chunk.serial += 1;
        let ukey = UnstableBlockKey {
            chunk_key,
            block_key,
        };

        *self.stable_ref.get_mut(key as usize).unwrap() = ukey;

        // spatial features
        let rect = new_spec.rect(new_block.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = rstar::primitives::GeomWithData::new(rect, key);
        self.spatial_ref.insert(node);

        // collision features
        let rect = new_spec.collision_rect(new_block.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = rstar::primitives::GeomWithData::new(rect, key);
        self.collision_ref.insert(node);

        // hint features
        let rect = new_spec.hint_rect(new_block.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = rstar::primitives::GeomWithData::new(rect, key);
        self.hint_ref.insert(node);

        Some(block)
    }

    pub fn get(&self, key: u32) -> Option<&Block> {
        let ukey = self.stable_ref.get(key as usize)?;
        let chunk = self.chunks.get(&ukey.chunk_key).unwrap();
        let block = chunk.blocks.get(ukey.block_key as usize).unwrap();
        Some(block)
    }

    pub fn get_chunk(&self, chunk_key: IVec2) -> Option<&BlockChunk> {
        self.chunks.get(&chunk_key)
    }

    // spatial features

    pub fn has_by_point(&self, point: IVec2) -> bool {
        self.spatial_ref.locate_at_point(&point).is_some()
    }

    pub fn get_by_point(&self, point: IVec2) -> Option<u32> {
        let node = self.spatial_ref.locate_at_point(&point)?;
        Some(node.data)
    }

    pub fn has_by_rect(&self, rect: [IVec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.spatial_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    pub fn get_by_rect(&self, rect: [IVec2; 2]) -> impl Iterator<Item = u32> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.spatial_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }

    // collision features

    pub fn has_collision_by_point(&self, point: Vec2) -> bool {
        self.collision_ref.locate_at_point(&point).is_some()
    }

    pub fn get_collision_by_point(&self, point: Vec2) -> impl Iterator<Item = u32> + '_ {
        self.collision_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    pub fn has_collision_by_rect(&self, rect: [Vec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    pub fn get_collision_by_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = u32> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }

    // hint features

    pub fn has_hint_by_point(&self, point: Vec2) -> bool {
        self.hint_ref.locate_at_point(&point).is_some()
    }

    pub fn get_hint_by_point(&self, point: Vec2) -> impl Iterator<Item = u32> + '_ {
        self.hint_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    pub fn has_hint_by_rect(&self, rect: [Vec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    pub fn get_hint_by_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = u32> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }
}

#[derive(Debug, Clone)]
pub struct EntitySpec {
    pub collision_size: Vec2,
    pub collision_offset: Vec2,
    pub hint_size: Vec2,
    pub hint_offset: Vec2,
}

impl EntitySpec {
    #[rustfmt::skip]
    fn collision_rect(&self, location: Vec2) -> [Vec2; 2] {
        [[
            location[0] + self.collision_offset[0],
            location[1] + self.collision_offset[1], ], [
            location[0] + self.collision_offset[0] + self.collision_size[0],
            location[1] + self.collision_offset[1] + self.collision_size[1],
        ]]
    }

    #[rustfmt::skip]
    fn hint_rect(&self, location: Vec2) -> [Vec2; 2] {
        [[
            location[0] + self.hint_offset[0],
            location[1] + self.hint_offset[1], ], [
            location[0] + self.hint_offset[0] + self.hint_size[0],
            location[1] + self.hint_offset[1] + self.hint_size[1],
        ]]
    }
}

#[derive(Debug, Clone)]
struct UnstableEntityKey {
    chunk_key: IVec2,
    entity_key: u32,
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub id: u32,
    pub location: Vec2,
}

#[derive(Debug, Clone, Default)]
pub struct EntityChunk {
    pub serial: u32,
    pub entities: slab::Slab<Entity>,
}

#[derive(Debug, Clone)]
pub struct EntityField {
    chunk_size: u32,
    specs: Vec<EntitySpec>,
    chunks: ahash::AHashMap<IVec2, EntityChunk>,
    stable_ref: slab::Slab<UnstableEntityKey>,
    collision_ref: rstar::RTree<RectNode<Vec2, u32>>,
    hint_ref: rstar::RTree<RectNode<Vec2, u32>>,
}

impl EntityField {
    pub fn new(chunk_size: u32, specs: Vec<EntitySpec>) -> Self {
        // validate specs
        specs.iter().for_each(|spec| {
            assert!(spec.collision_size[0] >= 0.0);
            assert!(spec.collision_size[1] >= 0.0);
            assert!(spec.hint_size[0] >= 0.0);
            assert!(spec.hint_size[1] >= 0.0);
        });

        Self {
            chunk_size,
            specs,
            chunks: Default::default(),
            stable_ref: Default::default(),
            collision_ref: Default::default(),
            hint_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, entity: Entity) -> u32 {
        let spec = &self.specs[entity.id as usize];

        let chunk_key = [
            entity.location[0].div_euclid(self.chunk_size as f32) as i32,
            entity.location[1].div_euclid(self.chunk_size as f32) as i32,
        ];
        let chunk = self.chunks.entry(chunk_key).or_default();
        let entity_key = chunk.entities.insert(entity.clone()) as u32;
        chunk.serial += 1;
        let ukey = UnstableEntityKey {
            chunk_key,
            entity_key,
        };

        let key = self.stable_ref.insert(ukey) as u32;

        // collision features
        let rect = spec.collision_rect(entity.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = rstar::primitives::GeomWithData::new(rect, key);
        self.collision_ref.insert(node);

        // hint features
        let rect = spec.hint_rect(entity.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = rstar::primitives::GeomWithData::new(rect, key);
        self.hint_ref.insert(node);

        key
    }

    pub fn remove(&mut self, key: u32) -> Option<Entity> {
        let ukey = self.stable_ref.try_remove(key as usize)?;
        let chunk = self.chunks.get_mut(&ukey.chunk_key).unwrap();
        let entity = chunk.entities.try_remove(ukey.entity_key as usize).unwrap();
        chunk.serial += 1;

        let spec = &self.specs[entity.id as usize];

        // collision features
        let rect = spec.collision_rect(entity.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = &rstar::primitives::GeomWithData::new(rect, key);
        self.collision_ref.remove(node);

        // hint features
        let rect = spec.hint_rect(entity.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = &rstar::primitives::GeomWithData::new(rect, key);
        self.hint_ref.remove(node);

        Some(entity)
    }

    pub fn modify(&mut self, key: u32, new_entity: Entity) -> Option<Entity> {
        // validate modification

        if !self.stable_ref.contains(key as usize) {
            return None;
        }

        let new_spec = &self.specs[new_entity.id as usize];

        // remove old entity

        let ukey = self.stable_ref.get(key as usize).unwrap();
        let chunk = self.chunks.get_mut(&ukey.chunk_key).unwrap();
        let entity = chunk.entities.try_remove(ukey.entity_key as usize).unwrap();
        chunk.serial += 1;

        let spec = &self.specs[entity.id as usize];

        // collision features
        let rect = spec.collision_rect(entity.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = &rstar::primitives::GeomWithData::new(rect, key);
        self.collision_ref.remove(node);

        // hint features
        let rect = spec.hint_rect(entity.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = &rstar::primitives::GeomWithData::new(rect, key);
        self.hint_ref.remove(node);

        // insert new entity

        let chunk_key = [
            new_entity.location[0].div_euclid(self.chunk_size as f32) as i32,
            new_entity.location[1].div_euclid(self.chunk_size as f32) as i32,
        ];
        let chunk = self.chunks.entry(chunk_key).or_default();
        let entity_key = chunk.entities.insert(new_entity.clone()) as u32;
        chunk.serial += 1;
        let ukey = UnstableEntityKey {
            chunk_key,
            entity_key,
        };

        *self.stable_ref.get_mut(key as usize).unwrap() = ukey;

        // collision features
        let rect = new_spec.collision_rect(new_entity.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = rstar::primitives::GeomWithData::new(rect, key);
        self.collision_ref.insert(node);

        // hint features
        let rect = new_spec.hint_rect(new_entity.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = rstar::primitives::GeomWithData::new(rect, key);
        self.hint_ref.insert(node);

        Some(entity)
    }

    pub fn get(&self, key: u32) -> Option<&Entity> {
        let ukey = self.stable_ref.get(key as usize)?;
        let chunk = self.chunks.get(&ukey.chunk_key).unwrap();
        let entity = chunk.entities.get(ukey.entity_key as usize).unwrap();
        Some(entity)
    }

    pub fn get_chunk(&self, chunk_key: IVec2) -> Option<&EntityChunk> {
        self.chunks.get(&chunk_key)
    }

    // collision features

    pub fn has_collision_by_point(&self, point: Vec2) -> bool {
        self.collision_ref.locate_at_point(&point).is_some()
    }

    pub fn get_collision_by_point(&self, point: Vec2) -> impl Iterator<Item = u32> + '_ {
        self.collision_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    pub fn has_collision_by_rect(&self, rect: [Vec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    pub fn get_collision_by_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = u32> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }

    // hint features

    pub fn has_hint_by_point(&self, point: Vec2) -> bool {
        self.hint_ref.locate_at_point(&point).is_some()
    }

    pub fn get_hint_by_point(&self, point: Vec2) -> impl Iterator<Item = u32> + '_ {
        self.hint_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    pub fn has_hint_by_rect(&self, rect: [Vec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    pub fn get_hint_by_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = u32> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }
}

#[derive(Debug, Clone)]
pub enum AgentState {
    Empty,
    RandomWalk(AgentStateRandomWalk),
}

#[derive(Debug, Clone)]
struct Agent {
    entity_key: u32,
    state: AgentState,
}

#[derive(Debug, Clone)]
pub struct AgentPlugin {
    agents: slab::Slab<Agent>,
    inverse_ref: ahash::AHashMap<u32, u32>,
}

impl AgentPlugin {
    pub fn new() -> Self {
        Self {
            agents: Default::default(),
            inverse_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, entity_key: u32, state: AgentState) -> Option<()> {
        if self.inverse_ref.contains_key(&entity_key) {
            return None;
        }

        let agent = Agent { entity_key, state };
        let key = self.agents.insert(agent) as u32;
        self.inverse_ref.insert(entity_key, key);
        Some(())
    }

    pub fn remove(&mut self, entity_key: u32) -> Option<AgentState> {
        let key = self.inverse_ref.remove(&entity_key)?;
        let agent = self.agents.try_remove(key as usize).unwrap();
        Some(agent.state)
    }

    pub fn update(
        &mut self,
        block_field: &BlockField,
        entity_field: &mut EntityField,
        delta_secs: f32,
    ) {
        for (_, agent) in self.agents.iter_mut() {
            match &mut agent.state {
                AgentState::Empty => {}
                AgentState::RandomWalk(state) => {
                    state.update(block_field, entity_field, delta_secs, agent.entity_key)
                }
            }
        }
    }
}

fn move_entity(
    block_field: &BlockField,
    entity_field: &mut EntityField,
    entity_key: u32,
    new_location: Vec2,
) -> Option<()> {
    let entity = entity_field.get(entity_key).expect("entity key not found");
    let spec = &entity_field.specs[entity.id as usize];

    let rect = spec.collision_rect(new_location);
    if block_field.has_collision_by_rect(rect) {
        return None;
    }

    let mut new_entity = entity.clone();
    new_entity.location = new_location;
    entity_field.modify(entity_key, new_entity);

    Some(())
}

#[derive(Debug, Clone, Default)]
pub enum AgentStateRandomWalkLocal {
    #[default]
    Init,
    WaitStart,
    Wait(f32),
    TripStart,
    Trip(Vec2),
}

#[derive(Debug, Clone)]
pub struct AgentStateRandomWalk {
    pub min_rest_secs: f32,
    pub max_rest_secs: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub speed: f32,
    pub local: AgentStateRandomWalkLocal,
}

impl AgentStateRandomWalk {
    fn update(
        &mut self,
        block_field: &BlockField,
        entity_field: &mut EntityField,
        delta_secs: f32,
        entity_key: u32,
    ) {
        use rand::Rng;

        type StateLocal = AgentStateRandomWalkLocal;

        match self.local {
            StateLocal::Init => {
                self.local = StateLocal::WaitStart;
            }
            StateLocal::WaitStart => {
                let secs = rand::thread_rng().gen_range(self.min_rest_secs..self.max_rest_secs);
                self.local = StateLocal::Wait(secs);
            }
            StateLocal::Wait(secs) => {
                let new_secs = secs - delta_secs;
                if new_secs <= 0.0 {
                    self.local = StateLocal::TripStart;
                } else {
                    self.local = StateLocal::Wait(new_secs);
                }
            }
            StateLocal::TripStart => {
                let entity = entity_field.get(entity_key).unwrap();

                let distance = rand::thread_rng().gen_range(self.min_distance..self.max_distance);
                let direction = rand::thread_rng().gen_range(0.0..std::f32::consts::PI * 2.0);
                let destination = [
                    entity.location[0] + distance * direction.cos(),
                    entity.location[1] + distance * direction.sin(),
                ];

                self.local = StateLocal::Trip(destination);
            }
            StateLocal::Trip(destination) => {
                let entity = entity_field.get(entity_key).unwrap();

                if entity.location == destination {
                    self.local = StateLocal::WaitStart;
                    return;
                }

                let diff = [
                    destination[0] - entity.location[0],
                    destination[1] - entity.location[1],
                ];
                let distance = (diff[0].powi(2) + diff[1].powi(2)).sqrt();
                let direction = [diff[0] / distance, diff[1] / distance];
                let delta_distance = distance.min(self.speed * delta_secs);
                let new_location = [
                    entity.location[0] + direction[0] * delta_distance,
                    entity.location[1] + direction[1] * delta_distance,
                ];

                if move_entity(block_field, entity_field, entity_key, new_location).is_some() {
                    self.local = StateLocal::Trip(destination);
                } else {
                    self.local = StateLocal::WaitStart;
                }
            }
        }
    }
}
