use super::*;

type RectNode<T, U> = rstar::primitives::GeomWithData<rstar::primitives::Rectangle<T>, U>;

// tile field

pub type TileKey = (u32, u32);

#[derive(Debug, Clone)]
pub struct TileDescriptor {
    pub collision: bool,
}

#[derive(Debug, Clone)]
pub struct TileFieldDescriptor {
    pub tiles: Vec<TileDescriptor>,
}

#[derive(Debug, Clone)]
struct TileProperty {
    collision: bool,
}

impl TileProperty {
    #[rustfmt::skip]
    fn collision_rect(&self, location: IVec2) -> Option<[Vec2; 2]> {
        if !self.collision {
            return None;
        }

        Some([[
            location[0] as f32,
            location[1] as f32, ], [
            location[0] as f32 + 1.0,
            location[1] as f32 + 1.0,
        ]])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile<T> {
    pub id: u16,
    pub location: IVec2,
    pub variant: u8,
    pub tick: u64,
    pub data: Option<T>,
}

#[derive(Debug, Clone)]
pub struct TileChunk<T> {
    pub version: u64,
    pub tiles: slab::Slab<Tile<T>>,
}

#[derive(Debug, Clone)]
pub struct TileField<T> {
    props: Vec<TileProperty>,
    chunks: Vec<TileChunk<T>>,
    chunk_ref: ahash::AHashMap<IVec2, u32>,
    spatial_ref: ahash::AHashMap<IVec2, TileKey>,
    collision_ref: rstar::RTree<RectNode<Vec2, TileKey>>,
}

impl<T> TileField<T> {
    const CHUNK_SIZE: u32 = 32;

    pub fn new(desc: TileFieldDescriptor) -> Self {
        let mut props = vec![];
        for tile in desc.tiles {
            props.push(TileProperty {
                collision: tile.collision,
            });
        }

        Self {
            props,
            chunks: Default::default(),
            chunk_ref: Default::default(),
            spatial_ref: Default::default(),
            collision_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, tile: Tile<T>) -> Result<TileKey, FieldError> {
        let prop = self
            .props
            .get(tile.id as usize)
            .ok_or(FieldError::InvalidId)?;

        // check by spatial features
        if self.has_by_point(tile.location) {
            return Err(FieldError::Conflict);
        }

        let chunk_location = [
            tile.location[0].div_euclid(Self::CHUNK_SIZE as i32),
            tile.location[1].div_euclid(Self::CHUNK_SIZE as i32),
        ];

        // get or allocate chunk
        let chunk_key = if let Some(chunk_key) = self.chunk_ref.get(&chunk_location) {
            *chunk_key
        } else {
            if self.chunks.len() >= u32::MAX as usize {
                panic!("capacity overflow");
            }

            let chunk_key = self.chunks.len() as u32;
            self.chunks.push(TileChunk {
                version: 0,
                tiles: Default::default(),
            });
            self.chunk_ref.insert(chunk_location, chunk_key);
            chunk_key
        };

        let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();

        if chunk.tiles.vacant_key() >= u32::MAX as usize {
            panic!("capacity overflow");
        }

        let local_key = chunk.tiles.vacant_key() as u32;

        // spatial features
        self.spatial_ref
            .insert(tile.location, (chunk_key, local_key));

        // collision features
        if let Some(rect) = prop.collision_rect(tile.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, (chunk_key, local_key));
            self.collision_ref.insert(node);
        }

        // key is guaranteed to be less than u32::MAX.
        chunk.tiles.insert(tile);
        chunk.version += 1;

        Ok((chunk_key, local_key))
    }

    pub fn remove(&mut self, key: TileKey) -> Result<Tile<T>, FieldError> {
        let (chunk_key, local_key) = key;

        let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
        let tile = chunk
            .tiles
            .try_remove(local_key as usize)
            .ok_or(FieldError::NotFound)?;
        chunk.version += 1;

        let prop = self.props.get(tile.id as usize).unwrap();

        // spatial features
        self.spatial_ref.remove(&tile.location).unwrap();

        // collision features
        if let Some(rect) = prop.collision_rect(tile.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.remove(&node).unwrap();
        }

        Ok(tile)
    }

    pub fn modify(
        &mut self,
        key: TileKey,
        f: impl FnOnce(&mut Tile<T>),
    ) -> Result<TileKey, FieldError> {
        let (chunk_key, local_key) = key;

        let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
        let tile = chunk
            .tiles
            .get_mut(local_key as usize)
            .ok_or(FieldError::NotFound)?;

        // SAFETY: data in replaced old tile is not used after this.
        let mut new_tile = Tile {
            id: tile.id,
            location: tile.location,
            variant: tile.variant,
            tick: tile.tick,
            data: tile.data.take(),
        };
        f(&mut new_tile);

        if new_tile.id != tile.id {
            return Err(FieldError::InvalidId);
        }

        if new_tile.location != tile.location {
            // check by spatial features
            if self.has_by_point(new_tile.location) {
                return Err(FieldError::Conflict);
            }

            self.remove(key).unwrap();
            let key = self.insert(new_tile).unwrap();

            return Ok(key);
        }

        if new_tile.variant != tile.variant || new_tile.tick != tile.tick {
            let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
            *chunk.tiles.get_mut(local_key as usize).unwrap() = new_tile;
            chunk.version += 1;

            return Ok(key);
        }

        tile.data = new_tile.data;

        Ok(key)
    }

    pub fn get(&self, key: TileKey) -> Result<&Tile<T>, FieldError> {
        let (chunk_key, local_key) = key;

        let chunk = self.chunks.get(chunk_key as usize).unwrap();
        let tile = chunk
            .tiles
            .get(local_key as usize)
            .ok_or(FieldError::NotFound)?;
        Ok(tile)
    }

    #[inline]
    pub fn get_chunk_size(&self) -> u32 {
        Self::CHUNK_SIZE
    }

    #[inline]
    pub fn get_chunk(&self, chunk_key: u32) -> Result<&TileChunk<T>, FieldError> {
        self.chunks
            .get(chunk_key as usize)
            .ok_or(FieldError::NotFound)
    }

    // spatial features

    #[inline]
    pub fn has_by_point(&self, point: IVec2) -> bool {
        self.spatial_ref.contains_key(&point)
    }

    #[inline]
    pub fn get_by_point(&self, point: IVec2) -> Option<TileKey> {
        self.spatial_ref.get(&point).copied()
    }

    #[inline]
    pub fn get_by_chunk_location(&self, chunk_location: IVec2) -> Option<u32> {
        self.chunk_ref.get(&chunk_location).copied()
    }

    // collision features

    #[inline]
    pub fn get_collision_rect(&self, tile_key: TileKey) -> Result<[Vec2; 2], FieldError> {
        let tile = self.get(tile_key)?;
        let prop = self.props.get(tile.id as usize).unwrap();
        Ok(prop.collision_rect(tile.location).unwrap_or_default())
    }

    #[inline]
    pub fn has_by_collision_point(&self, point: Vec2) -> bool {
        self.collision_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = TileKey> + '_ {
        self.collision_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    #[inline]
    pub fn has_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    #[inline]
    pub fn get_by_collision_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = TileKey> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }
}

// block field

pub type BlockKey = (u32, u32);

#[derive(Debug, Clone)]
pub struct BlockDescriptor {
    pub size: IVec2,
    pub collision_size: Vec2,
    pub collision_offset: Vec2,
    pub hint_size: Vec2,
    pub hint_offset: Vec2,
}

#[derive(Debug, Clone)]
pub struct BlockFieldDescriptor {
    pub chunk_size: u32,
    pub blocks: Vec<BlockDescriptor>,
}

#[derive(Debug, Clone)]
struct BlockProperty {
    size: IVec2,
    collision_size: Vec2,
    collision_offset: Vec2,
    hint_size: Vec2,
    hint_offset: Vec2,
}

impl BlockProperty {
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
    fn collision_rect(&self, location: IVec2) -> Option<[Vec2; 2]> {
        if self.collision_size[0] * self.collision_size[1] == 0.0 {
            return None;
        }

        Some([[
            location[0] as f32 + self.collision_offset[0],
            location[1] as f32 + self.collision_offset[1], ], [
            location[0] as f32 + self.collision_offset[0] + self.collision_size[0],
            location[1] as f32 + self.collision_offset[1] + self.collision_size[1],
        ]])
    }

    #[rustfmt::skip]
    fn hint_rect(&self, location: IVec2) -> Option<[Vec2; 2]> {
        if self.hint_size[0] * self.hint_size[1] == 0.0 {
            return None;
        }

        Some([[
            location[0] as f32 + self.hint_offset[0],
            location[1] as f32 + self.hint_offset[1], ], [
            location[0] as f32 + self.hint_offset[0] + self.hint_size[0],
            location[1] as f32 + self.hint_offset[1] + self.hint_size[1],
        ]])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block<T> {
    pub id: u32,
    pub location: IVec2,
    pub variant: u8,
    pub data: T,
}

#[derive(Debug, Clone)]
pub struct BlockChunk<T> {
    pub version: u64,
    pub blocks: slab::Slab<Block<T>>,
}

#[derive(Debug, Clone)]
pub struct BlockField<T> {
    chunk_size: u32,
    props: Vec<BlockProperty>,
    chunks: Vec<BlockChunk<T>>,
    chunk_ref: ahash::AHashMap<IVec2, u32>,
    spatial_ref: rstar::RTree<RectNode<IVec2, BlockKey>>,
    collision_ref: rstar::RTree<RectNode<Vec2, BlockKey>>,
    hint_ref: rstar::RTree<RectNode<Vec2, BlockKey>>,
}

impl<T> BlockField<T> {
    pub fn new(desc: BlockFieldDescriptor) -> Self {
        let mut props = vec![];
        for block in desc.blocks {
            if block.size[0] <= 0 || block.size[1] <= 0 {
                panic!("size must be positive");
            }
            if block.collision_size[0] < 0.0 || block.collision_size[1] < 0.0 {
                panic!("collision size must be non-negative");
            }
            if block.hint_size[0] < 0.0 || block.hint_size[1] < 0.0 {
                panic!("hint size must be non-negative");
            }

            props.push(BlockProperty {
                size: block.size,
                collision_size: block.collision_size,
                collision_offset: block.collision_offset,
                hint_size: block.hint_size,
                hint_offset: block.hint_offset,
            });
        }

        Self {
            chunk_size: desc.chunk_size,
            props,
            chunks: Default::default(),
            chunk_ref: Default::default(),
            spatial_ref: Default::default(),
            collision_ref: Default::default(),
            hint_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, block: Block<T>) -> Result<BlockKey, FieldError> {
        let prop = self
            .props
            .get(block.id as usize)
            .ok_or(FieldError::InvalidId)?;

        // check by spatial features
        if self.has_by_rect(prop.rect(block.location)) {
            return Err(FieldError::Conflict);
        }

        let chunk_location = [
            block.location[0].div_euclid(self.chunk_size as i32),
            block.location[1].div_euclid(self.chunk_size as i32),
        ];

        // get or allocate chunk
        let chunk_key = if let Some(chunk_key) = self.chunk_ref.get(&chunk_location) {
            *chunk_key
        } else {
            if self.chunks.len() >= u32::MAX as usize {
                panic!("capacity overflow");
            }

            let chunk_key = self.chunks.len() as u32;
            self.chunks.push(BlockChunk {
                version: 0,
                blocks: Default::default(),
            });
            self.chunk_ref.insert(chunk_location, chunk_key);
            chunk_key
        };

        let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();

        if chunk.blocks.vacant_key() >= u32::MAX as usize {
            panic!("capacity overflow");
        }

        let local_key = chunk.blocks.vacant_key() as u32;

        // spatial features
        let rect = prop.rect(block.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = rstar::primitives::GeomWithData::new(rect, (chunk_key, local_key));
        self.spatial_ref.insert(node);

        // collision features
        if let Some(rect) = prop.collision_rect(block.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, (chunk_key, local_key));
            self.collision_ref.insert(node);
        }

        // hint features
        if let Some(rect) = prop.hint_rect(block.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, (chunk_key, local_key));
            self.hint_ref.insert(node);
        }

        // block_key is guaranteed to be less than u32::MAX.
        chunk.blocks.insert(block);
        chunk.version += 1;

        Ok((chunk_key, local_key))
    }

    pub fn remove(&mut self, key: BlockKey) -> Result<Block<T>, FieldError> {
        let (chunk_key, local_key) = key;

        let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
        let block = chunk
            .blocks
            .try_remove(local_key as usize)
            .ok_or(FieldError::NotFound)?;
        chunk.version += 1;

        let prop = self.props.get(block.id as usize).unwrap();

        // spatial features
        let rect = prop.rect(block.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = rstar::primitives::GeomWithData::new(rect, key);
        self.spatial_ref.remove(&node).unwrap();

        // collision features
        if let Some(rect) = prop.collision_rect(block.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.remove(&node).unwrap();
        }

        // hint features
        if let Some(rect) = prop.hint_rect(block.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.hint_ref.remove(&node).unwrap();
        }

        Ok(block)
    }

    pub fn modify(
        &mut self,
        key: BlockKey,
        f: impl FnOnce(&mut Block<T>),
    ) -> Result<BlockKey, FieldError> {
        let (chunk_key, local_key) = key;

        let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
        let block = chunk
            .blocks
            .get_mut(local_key as usize)
            .ok_or(FieldError::NotFound)?;

        let mut new_block = Block {
            id: block.id,
            location: block.location,
            variant: block.variant,
            data: std::mem::replace(&mut block.data, unsafe { std::mem::zeroed() }),
        };
        f(&mut new_block);

        if new_block.id != block.id {
            return Err(FieldError::InvalidId);
        }

        if new_block.location != block.location {
            let prop = self.props.get(block.id as usize).unwrap();

            // check by spatial features
            if self
                .get_by_rect(prop.rect(new_block.location))
                .any(|other_key| other_key != key)
            {
                return Err(FieldError::Conflict);
            }

            self.remove(key).unwrap();
            let key = self.insert(new_block).unwrap();

            return Ok(key);
        }

        if new_block.variant != block.variant {
            let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
            *chunk.blocks.get_mut(local_key as usize).unwrap() = new_block;
            chunk.version += 1;

            return Ok(key);
        }

        block.data = new_block.data;

        Ok(key)
    }

    pub fn get(&self, key: BlockKey) -> Result<&Block<T>, FieldError> {
        let (chunk_key, local_key) = key;

        let chunk = self.chunks.get(chunk_key as usize).unwrap();
        let block = chunk
            .blocks
            .get(local_key as usize)
            .ok_or(FieldError::NotFound)?;
        Ok(block)
    }

    #[inline]
    pub fn get_chunk_size(&self) -> u32 {
        self.chunk_size
    }

    #[inline]
    pub fn get_chunk(&self, chunk_key: u32) -> Result<&BlockChunk<T>, FieldError> {
        self.chunks
            .get(chunk_key as usize)
            .ok_or(FieldError::NotFound)
    }

    // spatial features

    #[inline]
    pub fn get_rect(&self, key: BlockKey) -> Result<[IVec2; 2], FieldError> {
        let block = self.get(key)?;
        let prop = self.props.get(block.id as usize).unwrap();
        Ok(prop.rect(block.location))
    }

    #[inline]
    pub fn has_by_point(&self, point: IVec2) -> bool {
        self.spatial_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_by_point(&self, point: IVec2) -> Option<BlockKey> {
        let node = self.spatial_ref.locate_at_point(&point)?;
        Some(node.data)
    }

    #[inline]
    pub fn has_by_rect(&self, rect: [IVec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.spatial_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    #[inline]
    pub fn get_by_rect(&self, rect: [IVec2; 2]) -> impl Iterator<Item = BlockKey> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.spatial_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }

    #[inline]
    pub fn get_by_chunk_location(&self, chunk_location: IVec2) -> Option<u32> {
        self.chunk_ref.get(&chunk_location).copied()
    }

    // collision features

    #[inline]
    pub fn get_collision_rect(&self, key: BlockKey) -> Result<[Vec2; 2], FieldError> {
        let block = self.get(key)?;
        let prop = self.props.get(block.id as usize).unwrap();
        Ok(prop.collision_rect(block.location).unwrap_or_default())
    }

    #[inline]
    pub fn has_by_collision_point(&self, point: Vec2) -> bool {
        self.collision_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = BlockKey> + '_ {
        self.collision_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    #[inline]
    pub fn has_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    #[inline]
    pub fn get_by_collision_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = BlockKey> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }

    // hint features

    #[inline]
    pub fn get_hint_rect(&self, key: BlockKey) -> Result<[Vec2; 2], FieldError> {
        let block = self.get(key)?;
        let prop = self.props.get(block.id as usize).unwrap();
        Ok(prop.hint_rect(block.location).unwrap_or_default())
    }

    #[inline]
    pub fn has_by_hint_point(&self, point: Vec2) -> bool {
        self.hint_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_by_hint_point(&self, point: Vec2) -> impl Iterator<Item = BlockKey> + '_ {
        self.hint_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    #[inline]
    pub fn has_by_hint_rect(&self, rect: [Vec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    #[inline]
    pub fn get_by_hint_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = BlockKey> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }
}

// entity field

pub type EntityKey = (u32, u32);

#[derive(Debug, Clone)]
pub struct EntityDescriptor {
    pub collision_size: Vec2,
    pub collision_offset: Vec2,
    pub hint_size: Vec2,
    pub hint_offset: Vec2,
}

#[derive(Debug, Clone)]
pub struct EntityFieldDescriptor {
    pub chunk_size: u32,
    pub entities: Vec<EntityDescriptor>,
}

#[derive(Debug, Clone)]
pub struct EntityProperty {
    collision_size: Vec2,
    collision_offset: Vec2,
    hint_size: Vec2,
    hint_offset: Vec2,
}

impl EntityProperty {
    #[rustfmt::skip]
    fn collision_rect(&self, location: Vec2) -> Option<[Vec2; 2]> {
        if self.collision_size[0] * self.collision_size[1] == 0.0 {
            return None;
        }
        Some([[
            location[0] + self.collision_offset[0],
            location[1] + self.collision_offset[1], ], [
            location[0] + self.collision_offset[0] + self.collision_size[0],
            location[1] + self.collision_offset[1] + self.collision_size[1],
        ]])
    }

    #[rustfmt::skip]
    fn hint_rect(&self, location: Vec2) -> Option<[Vec2; 2]> {
        if self.hint_size[0] * self.hint_size[1] == 0.0 {
            return None;
        }
        Some([[
            location[0] + self.hint_offset[0],
            location[1] + self.hint_offset[1], ], [
            location[0] + self.hint_offset[0] + self.hint_size[0],
            location[1] + self.hint_offset[1] + self.hint_size[1],
        ]])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entity<T> {
    pub id: u32,
    pub location: Vec2,
    pub variant: u8,
    pub data: T,
}

#[derive(Debug, Clone, Default)]
pub struct EntityChunk<T> {
    pub version: u64,
    pub entities: slab::Slab<Entity<T>>,
}

#[derive(Debug, Clone)]
pub struct EntityField<T> {
    chunk_size: u32,
    props: Vec<EntityProperty>,
    chunks: Vec<EntityChunk<T>>,
    chunk_ref: ahash::AHashMap<IVec2, u32>,
    collision_ref: rstar::RTree<RectNode<Vec2, EntityKey>>,
    hint_ref: rstar::RTree<RectNode<Vec2, EntityKey>>,
}

impl<T> EntityField<T> {
    pub fn new(desc: EntityFieldDescriptor) -> Self {
        let mut props = vec![];
        for entity in desc.entities {
            if entity.collision_size[0] < 0.0 || entity.collision_size[1] < 0.0 {
                panic!("collision size must be non-negative");
            }
            if entity.hint_size[0] < 0.0 || entity.hint_size[1] < 0.0 {
                panic!("hint size must be non-negative");
            }

            props.push(EntityProperty {
                collision_size: entity.collision_size,
                collision_offset: entity.collision_offset,
                hint_size: entity.hint_size,
                hint_offset: entity.hint_offset,
            });
        }

        Self {
            chunk_size: desc.chunk_size,
            props,
            chunks: Default::default(),
            chunk_ref: Default::default(),
            collision_ref: Default::default(),
            hint_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, entity: Entity<T>) -> Result<EntityKey, FieldError> {
        let prop = self
            .props
            .get(entity.id as usize)
            .ok_or(FieldError::InvalidId)?;

        let chunk_location = [
            entity.location[0].div_euclid(self.chunk_size as f32) as i32,
            entity.location[1].div_euclid(self.chunk_size as f32) as i32,
        ];

        // get or allocate chunk
        let chunk_key = if let Some(chunk_key) = self.chunk_ref.get(&chunk_location) {
            *chunk_key
        } else {
            if self.chunks.len() >= u32::MAX as usize {
                panic!("capacity overflow");
            }

            let chunk_key = self.chunks.len() as u32;
            self.chunks.push(EntityChunk {
                version: 0,
                entities: Default::default(),
            });
            self.chunk_ref.insert(chunk_location, chunk_key);
            chunk_key
        };

        let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();

        if chunk.entities.vacant_key() >= u32::MAX as usize {
            panic!("capacity overflow");
        }

        let local_key = chunk.entities.vacant_key() as u32;

        // collision features
        if let Some(rect) = prop.collision_rect(entity.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, (chunk_key, local_key));
            self.collision_ref.insert(node);
        }

        // hint features
        if let Some(rect) = prop.hint_rect(entity.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, (chunk_key, local_key));
            self.hint_ref.insert(node);
        }

        // entity_key is guaranteed to be less than u32::MAX.
        chunk.entities.insert(entity);
        chunk.version += 1;

        Ok((chunk_key, local_key))
    }

    pub fn remove(&mut self, key: EntityKey) -> Result<Entity<T>, FieldError> {
        let (chunk_key, local_key) = key;

        let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
        let entity = chunk
            .entities
            .try_remove(local_key as usize)
            .ok_or(FieldError::NotFound)?;
        chunk.version += 1;

        let prop = self.props.get(entity.id as usize).unwrap();

        // collision features
        if let Some(rect) = prop.collision_rect(entity.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = &rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.remove(node).unwrap();
        }

        // hint features
        if let Some(rect) = prop.hint_rect(entity.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = &rstar::primitives::GeomWithData::new(rect, key);
            self.hint_ref.remove(node).unwrap();
        }

        Ok(entity)
    }

    pub fn modify(
        &mut self,
        key: EntityKey,
        f: impl FnOnce(&mut Entity<T>),
    ) -> Result<EntityKey, FieldError> {
        let (chunk_key, local_key) = key;

        let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
        let entity = chunk
            .entities
            .get_mut(local_key as usize)
            .ok_or(FieldError::NotFound)?;

        let mut new_entity = Entity {
            id: entity.id,
            location: entity.location,
            variant: entity.variant,
            data: std::mem::replace(&mut entity.data, unsafe { std::mem::zeroed() }),
        };
        f(&mut new_entity);

        if new_entity.id != entity.id {
            return Err(FieldError::InvalidId);
        }

        if new_entity.location != entity.location {
            self.remove(key).unwrap();
            let key = self.insert(new_entity).unwrap();

            return Ok(key);
        }

        if new_entity.variant != entity.variant {
            let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
            *chunk.entities.get_mut(local_key as usize).unwrap() = new_entity;
            chunk.version += 1;

            return Ok(key);
        }

        entity.data = new_entity.data;

        Ok(key)
    }

    pub fn get(&self, key: EntityKey) -> Result<&Entity<T>, FieldError> {
        let (chunk_key, local_key) = key;

        let chunk = self.chunks.get(chunk_key as usize).unwrap();
        let entity = chunk
            .entities
            .get(local_key as usize)
            .ok_or(FieldError::NotFound)?;
        Ok(entity)
    }

    #[inline]
    pub fn get_chunk_size(&self) -> u32 {
        self.chunk_size
    }

    #[inline]
    pub fn get_chunk(&self, chunk_key: u32) -> Result<&EntityChunk<T>, FieldError> {
        self.chunks
            .get(chunk_key as usize)
            .ok_or(FieldError::NotFound)
    }

    // spatial features

    #[inline]
    pub fn get_by_chunk_location(&self, chunk_location: IVec2) -> Option<u32> {
        self.chunk_ref.get(&chunk_location).copied()
    }

    // collision features

    #[inline]
    pub fn get_collision_rect(&self, entity_key: EntityKey) -> Result<[Vec2; 2], FieldError> {
        let entity = self.get(entity_key)?;
        let prop = self.props.get(entity.id as usize).unwrap();
        Ok(prop.hint_rect(entity.location).unwrap_or_default())
    }

    #[inline]
    pub fn has_by_collision_point(&self, point: Vec2) -> bool {
        self.collision_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = EntityKey> + '_ {
        self.collision_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    #[inline]
    pub fn has_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    #[inline]
    pub fn get_by_collision_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = EntityKey> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }

    // hint features

    #[inline]
    pub fn get_hint_rect(&self, entity_key: EntityKey) -> Result<[Vec2; 2], FieldError> {
        let entity = self.get(entity_key)?;
        let prop = self.props.get(entity.id as usize).unwrap();
        Ok(prop.hint_rect(entity.location).unwrap_or_default())
    }

    #[inline]
    pub fn has_by_hint_point(&self, point: Vec2) -> bool {
        self.hint_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_by_hint_point(&self, point: Vec2) -> impl Iterator<Item = EntityKey> + '_ {
        self.hint_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    #[inline]
    pub fn has_by_hint_rect(&self, rect: [Vec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    #[inline]
    pub fn get_by_hint_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = EntityKey> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud_tile() {
        let mut field: TileField<()> = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor { collision: true },
                TileDescriptor { collision: true },
            ],
        });

        let key = field
            .insert(Tile {
                id: 1,
                location: [-1, 3],
                variant: Default::default(),
                tick: Default::default(),
                data: Default::default(),
            })
            .unwrap();

        assert_eq!(
            field.get(key),
            Ok(&Tile {
                id: 1,
                location: [-1, 3],
                variant: Default::default(),
                tick: Default::default(),
                data: Default::default(),
            })
        );
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key));
        assert_eq!(
            field.remove(key),
            Ok(Tile {
                id: 1,
                location: [-1, 3],
                variant: Default::default(),
                tick: Default::default(),
                data: Default::default(),
            })
        );

        assert_eq!(field.get(key), Err(FieldError::NotFound));
        assert!(!field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), None);
        assert_eq!(field.remove(key), Err(FieldError::NotFound));
    }

    #[test]
    fn insert_tile_with_invalid() {
        let mut field: TileField<()> = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor { collision: true },
                TileDescriptor { collision: true },
            ],
        });

        assert_eq!(
            field.insert(Tile {
                id: 2,
                location: [-1, 3],
                variant: Default::default(),
                tick: Default::default(),
                data: Default::default(),
            }),
            Err(FieldError::InvalidId)
        );
        assert!(!field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), None);

        let key = field
            .insert(Tile {
                id: 1,
                location: [-1, 3],
                variant: Default::default(),
                tick: Default::default(),
                data: Default::default(),
            })
            .unwrap();
        assert_eq!(
            field.insert(Tile {
                id: 0,
                location: [-1, 3],
                variant: Default::default(),
                tick: Default::default(),
                data: Default::default(),
            }),
            Err(FieldError::Conflict)
        );
        assert_eq!(
            field.get(key),
            Ok(&Tile {
                id: 1,
                location: [-1, 3],
                variant: Default::default(),
                tick: Default::default(),
                data: Default::default(),
            })
        );
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key));
    }

    #[test]
    fn modify_tile() {
        let mut field: TileField<()> = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor { collision: true },
                TileDescriptor { collision: true },
            ],
        });

        let key = field
            .insert(Tile {
                id: 1,
                location: [-1, 3],
                variant: 0,
                tick: Default::default(),
                data: Default::default(),
            })
            .unwrap();

        let key = field.modify(key, |tile| tile.location = [-1, 4]).unwrap();
        assert_eq!(
            field.get(key),
            Ok(&Tile {
                id: 1,
                location: [-1, 4],
                variant: 0,
                tick: Default::default(),
                data: Default::default(),
            })
        );
        assert!(!field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), None);
        assert!(field.has_by_point([-1, 4]));
        assert_eq!(field.get_by_point([-1, 4]), Some(key));

        let key = field.modify(key, |tile| tile.variant = 1).unwrap();
        assert_eq!(
            field.get(key),
            Ok(&Tile {
                id: 1,
                location: [-1, 4],
                variant: 1,
                tick: Default::default(),
                data: Default::default(),
            })
        );

        let key = field.modify(key, |_| {}).unwrap();
        assert_eq!(
            field.get(key),
            Ok(&Tile {
                id: 1,
                location: [-1, 4],
                variant: 1,
                tick: Default::default(),
                data: Default::default(),
            })
        );
    }

    #[test]
    fn modify_tile_with_data() {
        let mut field: TileField<Vec<u8>> = TileField::new(TileFieldDescriptor {
            tiles: vec![TileDescriptor { collision: true }],
        });

        let key = field
            .insert(Tile {
                id: 0,
                location: [0, 0],
                variant: 0,
                tick: Default::default(),
                data: Some(vec![0; 1024]),
            })
            .unwrap();

        let key = field.modify(key, |tile| tile.variant = 1).unwrap();
        assert_eq!(
            field.get(key),
            Ok(&Tile {
                id: 0,
                location: [0, 0],
                variant: 1,
                tick: Default::default(),
                data: Some(vec![0; 1024]),
            })
        );

        let key = field.modify(key, |_| {}).unwrap();
        assert_eq!(
            field.get(key),
            Ok(&Tile {
                id: 0,
                location: [0, 0],
                variant: 1,
                tick: Default::default(),
                data: Some(vec![0; 1024]),
            })
        );
    }

    #[test]
    fn modify_tile_with_invalid() {
        let mut field: TileField<()> = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor { collision: true },
                TileDescriptor { collision: true },
            ],
        });

        let key_0 = field
            .insert(Tile {
                id: 0,
                location: [-1, 3],
                variant: Default::default(),
                tick: Default::default(),
                data: Default::default(),
            })
            .unwrap();
        let key_1 = field
            .insert(Tile {
                id: 1,
                location: [-1, 4],
                variant: Default::default(),
                tick: Default::default(),
                data: Default::default(),
            })
            .unwrap();

        assert_eq!(
            field.modify(key_0, |tile| tile.id = 1),
            Err(FieldError::InvalidId)
        );

        assert_eq!(
            field.modify(key_0, |tile| tile.location = [-1, 4]),
            Err(FieldError::Conflict)
        );
        assert_eq!(
            field.get(key_0),
            Ok(&Tile {
                id: 0,
                location: [-1, 3],
                variant: Default::default(),
                tick: Default::default(),
                data: Default::default(),
            })
        );
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key_0));

        field.remove(key_1).unwrap();
        assert_eq!(field.modify(key_1, |_| {}), Err(FieldError::NotFound));
        assert_eq!(field.get(key_1), Err(FieldError::NotFound));
        assert!(!field.has_by_point([-1, 4]));
        assert_eq!(field.get_by_point([-1, 4]), None);
    }

    #[test]
    fn modify_tile_with_move() {
        let mut field: TileField<()> = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor { collision: true },
                TileDescriptor { collision: true },
            ],
        });

        let key = field
            .insert(Tile {
                id: 1,
                location: [-1, 3],
                variant: Default::default(),
                tick: Default::default(),
                data: Default::default(),
            })
            .unwrap();

        let key = field
            .modify(key, |tile| tile.location = [-1, 1000])
            .unwrap();
        assert_eq!(
            field.get(key),
            Ok(&Tile {
                id: 1,
                location: [-1, 1000],
                variant: Default::default(),
                tick: Default::default(),
                data: Default::default(),
            })
        );
        assert!(!field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), None);
        assert!(field.has_by_point([-1, 1000]));
        assert_eq!(field.get_by_point([-1, 1000]), Some(key));
    }

    #[test]
    fn collision_tile() {
        let mut field: TileField<()> = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor { collision: true },
                TileDescriptor { collision: true },
            ],
        });

        let key_0 = field
            .insert(Tile {
                id: 1,
                location: [-1, 3],
                variant: Default::default(),
                tick: Default::default(),
                data: Default::default(),
            })
            .unwrap();
        let key_1 = field
            .insert(Tile {
                id: 1,
                location: [-1, 4],
                variant: Default::default(),
                tick: Default::default(),
                data: Default::default(),
            })
            .unwrap();
        let _key_2 = field
            .insert(Tile {
                id: 1,
                location: [-1, 5],
                variant: Default::default(),
                tick: Default::default(),
                data: Default::default(),
            })
            .unwrap();

        assert_eq!(
            field.get_collision_rect(key_0),
            Ok([[-1.0, 3.0], [0.0, 4.0]])
        );

        let point = [-1.0, 4.0];
        assert!(field.has_by_collision_point(point));
        let vec = field.get_by_collision_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        let rect = [[-1.0, 3.0], [-1.0, 4.0]];
        assert!(field.has_by_collision_rect(rect));
        let vec = field.get_by_collision_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        field.remove(key_0).unwrap();
        assert_eq!(field.get_collision_rect(key_0), Err(FieldError::NotFound));
    }

    #[test]
    fn tile_chunk() {
        let mut field: TileField<()> = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor { collision: true },
                TileDescriptor { collision: true },
            ],
        });
        assert_eq!(field.get_chunk_size(), 32);

        let _key0 = field
            .insert(Tile {
                id: 1,
                location: [-1, 3],
                variant: Default::default(),
                tick: Default::default(),
                data: Default::default(),
            })
            .unwrap();
        let _key1 = field
            .insert(Tile {
                id: 1,
                location: [-1, 4],
                variant: Default::default(),
                tick: Default::default(),
                data: Default::default(),
            })
            .unwrap();
        let _key2 = field
            .insert(Tile {
                id: 1,
                location: [-1, 5],
                variant: Default::default(),
                tick: Default::default(),
                data: Default::default(),
            })
            .unwrap();

        assert!(field.get_by_chunk_location([0, 0]).is_none());

        let chunk_key = field.get_by_chunk_location([-1, 0]).unwrap();
        let chunk = field.get_chunk(chunk_key).unwrap();
        assert_eq!(chunk.tiles.len(), 3);
    }

    #[test]
    #[should_panic]
    fn block_field_with_invalid() {
        let _: BlockField<()> = BlockField::new(BlockFieldDescriptor {
            chunk_size: 16,
            blocks: vec![BlockDescriptor {
                size: [-1, -1],
                collision_size: [1.0, 1.0],
                collision_offset: [0.0, 0.0],
                hint_size: [1.0, 1.0],
                hint_offset: [0.0, 0.0],
            }],
        });
    }

    #[test]
    #[should_panic]
    fn block_field_with_invalid_collision() {
        let _: BlockField<()> = BlockField::new(BlockFieldDescriptor {
            chunk_size: 16,
            blocks: vec![BlockDescriptor {
                size: [1, 1],
                collision_size: [-1.0, -1.0],
                collision_offset: [0.0, 0.0],
                hint_size: [1.0, 1.0],
                hint_offset: [0.0, 0.0],
            }],
        });
    }

    #[test]
    #[should_panic]
    fn block_field_with_invalid_hint() {
        let _: BlockField<()> = BlockField::new(BlockFieldDescriptor {
            chunk_size: 16,
            blocks: vec![BlockDescriptor {
                size: [1, 1],
                collision_size: [1.0, 1.0],
                collision_offset: [0.0, 0.0],
                hint_size: [-1.0, -1.0],
                hint_offset: [0.0, 0.0],
            }],
        });
    }

    #[test]
    fn crud_block() {
        let mut field = BlockField::new(BlockFieldDescriptor {
            chunk_size: 16,
            blocks: vec![
                BlockDescriptor {
                    size: [1, 1],
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
                BlockDescriptor {
                    size: [1, 1],
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
            ],
        });

        let key = field
            .insert(Block {
                id: 1,
                location: [-1, 3],
                variant: 0,
                data: (),
            })
            .unwrap();
        assert_eq!(field.get_rect(key), Ok([[-1, 3], [-1, 3]]));

        assert_eq!(
            field.get(key),
            Ok(&Block {
                id: 1,
                location: [-1, 3],
                variant: 0,
                data: (),
            })
        );
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key));
        assert_eq!(
            field.remove(key),
            Ok(Block {
                id: 1,
                location: [-1, 3],
                variant: 0,
                data: (),
            })
        );

        assert_eq!(field.get(key), Err(FieldError::NotFound));
        assert!(!field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), None);
        assert_eq!(field.remove(key), Err(FieldError::NotFound));

        assert_eq!(field.get_rect(key), Err(FieldError::NotFound));
    }

    #[test]
    fn insert_block_with_invalid() {
        let mut field = BlockField::new(BlockFieldDescriptor {
            chunk_size: 16,
            blocks: vec![
                BlockDescriptor {
                    size: [1, 1],
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
                BlockDescriptor {
                    size: [1, 1],
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
            ],
        });

        assert_eq!(
            field.insert(Block {
                id: 2,
                location: [-1, 3],
                variant: 0,
                data: (),
            }),
            Err(FieldError::InvalidId)
        );
        assert!(!field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), None);

        let key = field
            .insert(Block {
                id: 1,
                location: [-1, 3],
                variant: 0,
                data: (),
            })
            .unwrap();
        assert_eq!(
            field.insert(Block {
                id: 0,
                location: [-1, 3],
                variant: 0,
                data: (),
            }),
            Err(FieldError::Conflict)
        );
        assert_eq!(
            field.get(key),
            Ok(&Block {
                id: 1,
                location: [-1, 3],
                variant: 0,
                data: (),
            })
        );
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key));
    }

    #[test]
    fn modify_block() {
        let mut field = BlockField::new(BlockFieldDescriptor {
            chunk_size: 16,
            blocks: vec![
                BlockDescriptor {
                    size: [1, 1],
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
                BlockDescriptor {
                    size: [1, 1],
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
            ],
        });

        let key = field
            .insert(Block {
                id: 1,
                location: [-1, 3],
                variant: 0,
                data: (),
            })
            .unwrap();

        let key = field.modify(key, |block| block.location = [-1, 4]).unwrap();
        assert_eq!(
            field.get(key),
            Ok(&Block {
                id: 1,
                location: [-1, 4],
                variant: 0,
                data: (),
            })
        );
        assert!(!field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), None);
        assert!(field.has_by_point([-1, 4]));
        assert_eq!(field.get_by_point([-1, 4]), Some(key));

        let key = field.modify(key, |block| block.variant = 1).unwrap();
        assert_eq!(
            field.get(key),
            Ok(&Block {
                id: 1,
                location: [-1, 4],
                variant: 1,
                data: (),
            })
        );

        let key = field.modify(key, |_| {}).unwrap();
        assert_eq!(
            field.get(key),
            Ok(&Block {
                id: 1,
                location: [-1, 4],
                variant: 1,
                data: (),
            })
        );
    }

    #[test]
    fn modify_block_with_invalid() {
        let mut field = BlockField::new(BlockFieldDescriptor {
            chunk_size: 16,
            blocks: vec![
                BlockDescriptor {
                    size: [1, 1],
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
                BlockDescriptor {
                    size: [1, 1],
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
            ],
        });

        let key_0 = field
            .insert(Block {
                id: 0,
                location: [-1, 3],
                variant: 0,
                data: (),
            })
            .unwrap();
        let key_1 = field
            .insert(Block {
                id: 1,
                location: [-1, 4],
                variant: 0,
                data: (),
            })
            .unwrap();

        assert_eq!(
            field.modify(key_0, |block| block.id = 1),
            Err(FieldError::InvalidId)
        );

        assert_eq!(
            field.modify(key_0, |block| block.location = [-1, 4]),
            Err(FieldError::Conflict)
        );
        assert_eq!(
            field.get(key_0),
            Ok(&Block {
                id: 0,
                location: [-1, 3],
                variant: 0,
                data: (),
            })
        );
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key_0));

        field.remove(key_1).unwrap();
        assert_eq!(field.modify(key_1, |_| {}), Err(FieldError::NotFound));
        assert_eq!(field.get(key_1), Err(FieldError::NotFound));
        assert!(!field.has_by_point([-1, 4]));
        assert_eq!(field.get_by_point([-1, 4]), None);
    }

    #[test]
    fn modify_block_with_move() {
        let mut field = BlockField::new(BlockFieldDescriptor {
            chunk_size: 16,
            blocks: vec![
                BlockDescriptor {
                    size: [1, 1],
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
                BlockDescriptor {
                    size: [1, 1],
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
            ],
        });

        let key = field
            .insert(Block {
                id: 1,
                location: [-1, 3],
                variant: 0,
                data: (),
            })
            .unwrap();

        let key = field
            .modify(key, |block| block.location = [-1, 1000])
            .unwrap();
        assert_eq!(
            field.get(key),
            Ok(&Block {
                id: 1,
                location: [-1, 1000],
                variant: 0,
                data: (),
            })
        );
        assert!(!field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), None);
        assert!(field.has_by_point([-1, 1000]));
        assert_eq!(field.get_by_point([-1, 1000]), Some(key));
    }

    #[test]
    fn collision_block() {
        let mut field = BlockField::new(BlockFieldDescriptor {
            chunk_size: 16,
            blocks: vec![
                BlockDescriptor {
                    size: [1, 1],
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
                BlockDescriptor {
                    size: [1, 1],
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
            ],
        });

        let key_0 = field
            .insert(Block {
                id: 1,
                location: [-1, 3],
                variant: 0,
                data: (),
            })
            .unwrap();
        let key_1 = field
            .insert(Block {
                id: 1,
                location: [-1, 4],
                variant: 0,
                data: (),
            })
            .unwrap();
        let _key_2 = field
            .insert(Block {
                id: 1,
                location: [-1, 5],
                variant: 0,
                data: (),
            })
            .unwrap();

        assert_eq!(
            field.get_collision_rect(key_0),
            Ok([[-1.0, 3.0], [0.0, 4.0]])
        );

        let point = [-1.0, 4.0];
        assert!(field.has_by_collision_point(point));
        let vec = field.get_by_collision_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        let rect = [[-1.0, 3.0], [-1.0, 4.0]];
        assert!(field.has_by_collision_rect(rect));
        let vec = field.get_by_collision_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        field.remove(key_0).unwrap();
        assert_eq!(field.get_collision_rect(key_0), Err(FieldError::NotFound));
    }

    #[test]
    fn hint_block() {
        let mut field = BlockField::new(BlockFieldDescriptor {
            chunk_size: 16,
            blocks: vec![
                BlockDescriptor {
                    size: [1, 1],
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
                BlockDescriptor {
                    size: [1, 1],
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
            ],
        });

        let key_0 = field
            .insert(Block {
                id: 1,
                location: [-1, 3],
                variant: 0,
                data: (),
            })
            .unwrap();
        let key_1 = field
            .insert(Block {
                id: 1,
                location: [-1, 4],
                variant: 0,
                data: (),
            })
            .unwrap();
        let _key_2 = field
            .insert(Block {
                id: 1,
                location: [-1, 5],
                variant: 0,
                data: (),
            })
            .unwrap();

        assert_eq!(field.get_hint_rect(key_0), Ok([[-1.0, 3.0], [0.0, 4.0]]));

        let point = [-1.0, 4.0];
        assert!(field.has_by_hint_point(point));
        let vec = field.get_by_hint_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        let rect = [[-1.0, 3.0], [-1.0, 4.0]];
        assert!(field.has_by_hint_rect(rect));
        let vec = field.get_by_hint_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        field.remove(key_0).unwrap();
        assert_eq!(field.get_hint_rect(key_0), Err(FieldError::NotFound));
    }

    #[test]
    fn block_chunk() {
        let mut field = BlockField::new(BlockFieldDescriptor {
            chunk_size: 16,
            blocks: vec![
                BlockDescriptor {
                    size: [1, 1],
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
                BlockDescriptor {
                    size: [1, 1],
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
            ],
        });
        assert_eq!(field.get_chunk_size(), 16);

        let _key0 = field
            .insert(Block {
                id: 1,
                location: [-1, 3],
                variant: 0,
                data: (),
            })
            .unwrap();
        let _key1 = field
            .insert(Block {
                id: 1,
                location: [-1, 4],
                variant: 0,
                data: (),
            })
            .unwrap();
        let _key2 = field
            .insert(Block {
                id: 1,
                location: [-1, 5],
                variant: 0,
                data: (),
            })
            .unwrap();

        assert!(field.get_by_chunk_location([0, 0]).is_none());

        let chunk_key = field.get_by_chunk_location([-1, 0]).unwrap();
        let chunk = field.get_chunk(chunk_key).unwrap();
        assert_eq!(chunk.blocks.len(), 3);
    }

    #[test]
    #[should_panic]
    fn entity_field_with_invalid_collision() {
        let _: EntityField<()> = EntityField::new(EntityFieldDescriptor {
            chunk_size: 16,
            entities: vec![EntityDescriptor {
                collision_size: [-1.0, -1.0],
                collision_offset: [0.0, 0.0],
                hint_size: [1.0, 1.0],
                hint_offset: [0.0, 0.0],
            }],
        });
    }

    #[test]
    #[should_panic]
    fn entity_field_with_invalid_hint() {
        let _: EntityField<()> = EntityField::new(EntityFieldDescriptor {
            chunk_size: 16,
            entities: vec![EntityDescriptor {
                collision_size: [1.0, 1.0],
                collision_offset: [0.0, 0.0],
                hint_size: [-1.0, -1.0],
                hint_offset: [0.0, 0.0],
            }],
        });
    }

    #[test]
    fn crud_entity() {
        let mut field = EntityField::new(EntityFieldDescriptor {
            chunk_size: 16,
            entities: vec![
                EntityDescriptor {
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
                EntityDescriptor {
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
            ],
        });

        let key = field
            .insert(Entity {
                id: 1,
                location: [-1.0, 3.0],
                variant: 0,
                data: (),
            })
            .unwrap();

        assert_eq!(
            field.get(key),
            Ok(&Entity {
                id: 1,
                location: [-1.0, 3.0],
                variant: 0,
                data: (),
            })
        );
        assert_eq!(
            field.remove(key),
            Ok(Entity {
                id: 1,
                location: [-1.0, 3.0],
                variant: 0,
                data: (),
            })
        );

        assert_eq!(field.get(key), Err(FieldError::NotFound));
        assert_eq!(field.remove(key), Err(FieldError::NotFound));
    }

    #[test]
    fn insert_entity_with_invalid() {
        let mut field = EntityField::new(EntityFieldDescriptor {
            chunk_size: 16,
            entities: vec![
                EntityDescriptor {
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
                EntityDescriptor {
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
            ],
        });

        assert_eq!(
            field.insert(Entity {
                id: 2,
                location: [-1.0, 3.0],
                variant: 0,
                data: (),
            }),
            Err(FieldError::InvalidId)
        );
    }

    #[test]
    fn modify_entity() {
        let mut field = EntityField::new(EntityFieldDescriptor {
            chunk_size: 16,
            entities: vec![
                EntityDescriptor {
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
                EntityDescriptor {
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
            ],
        });

        let key = field
            .insert(Entity {
                id: 0,
                location: [-1.0, 3.0],
                variant: 0,
                data: (),
            })
            .unwrap();

        let key = field
            .modify(key, |entity| entity.location = [-1.0, 4.0])
            .unwrap();
        assert_eq!(
            field.get(key),
            Ok(&Entity {
                id: 0,
                location: [-1.0, 4.0],
                variant: 0,
                data: (),
            })
        );

        let key = field.modify(key, |entity| entity.variant = 1).unwrap();
        assert_eq!(
            field.get(key),
            Ok(&Entity {
                id: 0,
                location: [-1.0, 4.0],
                variant: 1,
                data: (),
            })
        );

        let key = field.modify(key, |_| {}).unwrap();
        assert_eq!(
            field.get(key),
            Ok(&Entity {
                id: 0,
                location: [-1.0, 4.0],
                variant: 1,
                data: (),
            })
        );
    }

    #[test]
    fn modify_entity_with_invalid() {
        let mut field = EntityField::new(EntityFieldDescriptor {
            chunk_size: 16,
            entities: vec![
                EntityDescriptor {
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
                EntityDescriptor {
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
            ],
        });

        let key = field
            .insert(Entity {
                id: 0,
                location: [-1.0, 4.0],
                variant: 0,
                data: (),
            })
            .unwrap();

        assert_eq!(
            field.modify(key, |entity| entity.id = 1),
            Err(FieldError::InvalidId)
        );

        field.remove(key).unwrap();
        assert_eq!(field.modify(key, |_| {}), Err(FieldError::NotFound));
        assert_eq!(field.get(key), Err(FieldError::NotFound));
    }

    #[test]
    fn modify_entity_with_move() {
        let mut field = EntityField::new(EntityFieldDescriptor {
            chunk_size: 16,
            entities: vec![
                EntityDescriptor {
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
                EntityDescriptor {
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
            ],
        });

        let key = field
            .insert(Entity {
                id: 1,
                location: [-1.0, 3.0],
                variant: 0,
                data: (),
            })
            .unwrap();

        let key = field
            .modify(key, |tile| tile.location = [-1.0, 1000.0])
            .unwrap();
        assert_eq!(
            field.get(key),
            Ok(&Entity {
                id: 1,
                location: [-1.0, 1000.0],
                variant: 0,
                data: (),
            })
        );
    }

    #[test]
    fn collision_entity() {
        let mut field = EntityField::new(EntityFieldDescriptor {
            chunk_size: 16,
            entities: vec![
                EntityDescriptor {
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
                EntityDescriptor {
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
            ],
        });

        let key_0 = field
            .insert(Entity {
                id: 1,
                location: [-1.0, 3.0],
                variant: 0,
                data: (),
            })
            .unwrap();
        let key_1 = field
            .insert(Entity {
                id: 1,
                location: [-1.0, 4.0],
                variant: 0,
                data: (),
            })
            .unwrap();
        let _key_2 = field
            .insert(Entity {
                id: 1,
                location: [-1.0, 5.0],
                variant: 0,
                data: (),
            })
            .unwrap();

        assert_eq!(
            field.get_collision_rect(key_0),
            Ok([[-1.0, 3.0], [0.0, 4.0]])
        );

        let point = [-1.0, 4.0];
        assert!(field.has_by_collision_point(point));
        let vec = field.get_by_collision_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        let rect = [[-1.0, 3.0], [-1.0, 4.0]];
        assert!(field.has_by_collision_rect(rect));
        let vec = field.get_by_collision_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        field.remove(key_0).unwrap();
        assert_eq!(field.get_collision_rect(key_0), Err(FieldError::NotFound));
    }

    #[test]
    fn hint_entity() {
        let mut field = EntityField::new(EntityFieldDescriptor {
            chunk_size: 16,
            entities: vec![
                EntityDescriptor {
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
                EntityDescriptor {
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
            ],
        });

        let key_0 = field
            .insert(Entity {
                id: 1,
                location: [-1.0, 3.0],
                variant: 0,
                data: (),
            })
            .unwrap();
        let key_1 = field
            .insert(Entity {
                id: 1,
                location: [-1.0, 4.0],
                variant: 0,
                data: (),
            })
            .unwrap();
        let _key_2 = field
            .insert(Entity {
                id: 1,
                location: [-1.0, 5.0],
                variant: 0,
                data: (),
            })
            .unwrap();

        assert_eq!(field.get_hint_rect(key_0), Ok([[-1.0, 3.0], [0.0, 4.0]]));

        let point = [-1.0, 4.0];
        assert!(field.has_by_hint_point(point));
        let vec = field.get_by_hint_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        let rect = [[-1.0, 3.0], [-1.0, 4.0]];
        assert!(field.has_by_hint_rect(rect));
        let vec = field.get_by_hint_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        field.remove(key_0).unwrap();
        assert_eq!(field.get_hint_rect(key_0), Err(FieldError::NotFound));
    }

    #[test]
    fn entity_chunk() {
        let mut field = EntityField::new(EntityFieldDescriptor {
            chunk_size: 16,
            entities: vec![
                EntityDescriptor {
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
                EntityDescriptor {
                    collision_size: [1.0, 1.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [1.0, 1.0],
                    hint_offset: [0.0, 0.0],
                },
            ],
        });
        assert_eq!(field.get_chunk_size(), 16);

        let _key0 = field
            .insert(Entity {
                id: 1,
                location: [-1.0, 3.0],
                variant: 0,
                data: (),
            })
            .unwrap();
        let _key1 = field
            .insert(Entity {
                id: 1,
                location: [-1.0, 4.0],
                variant: 0,
                data: (),
            })
            .unwrap();
        let _key2 = field
            .insert(Entity {
                id: 1,
                location: [-1.0, 5.0],
                variant: 0,
                data: (),
            })
            .unwrap();

        assert!(field.get_by_chunk_location([0, 0]).is_none());

        let chunk_key = field.get_by_chunk_location([-1, 0]).unwrap();
        let chunk = field.get_chunk(chunk_key).unwrap();
        assert_eq!(chunk.entities.len(), 3);
    }
}
