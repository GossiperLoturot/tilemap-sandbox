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
    pub chunk_size: u32,
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
pub struct Tile {
    pub id: u32,
    pub location: IVec2,
    pub variant: u8,
}

impl Tile {
    #[inline]
    pub fn new(id: u32, location: IVec2, variant: u8) -> Self {
        Self {
            id,
            location,
            variant,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TileChunk {
    pub version: u64,
    pub tiles: slab::Slab<Tile>,
}

#[derive(Debug, Clone)]
pub struct TileField {
    chunk_size: u32,
    props: Vec<TileProperty>,
    chunks: Vec<TileChunk>,
    chunk_ref: ahash::AHashMap<IVec2, u32>,
    spatial_ref: ahash::AHashMap<IVec2, TileKey>,
    collision_ref: rstar::RTree<RectNode<Vec2, TileKey>>,
}

impl TileField {
    pub fn new(desc: TileFieldDescriptor) -> Self {
        let mut props = vec![];
        for tile in desc.tiles {
            props.push(TileProperty {
                collision: tile.collision,
            });
        }

        Self {
            chunk_size: desc.chunk_size,
            props,
            chunks: Default::default(),
            chunk_ref: Default::default(),
            spatial_ref: Default::default(),
            collision_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, tile: Tile) -> Result<TileKey, FieldError> {
        let prop = self
            .props
            .get(tile.id as usize)
            .ok_or(FieldError::InvalidId)?;

        // check by spatial features
        if self.has_by_point(tile.location) {
            return Err(FieldError::Conflict);
        }

        let chunk_location = [
            tile.location[0].div_euclid(self.chunk_size as i32),
            tile.location[1].div_euclid(self.chunk_size as i32),
        ];

        // get or allocate chunk
        let chunk_key = if let Some(chunk_key) = self.chunk_ref.get(&chunk_location) {
            *chunk_key
        } else {
            if self.chunks.len() >= u32::MAX as usize {
                panic!("capacity overflow");
            }

            let chunk_key = self.chunks.len() as u32;
            self.chunks.push(Default::default());
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

    pub fn remove(&mut self, key: TileKey) -> Result<Tile, FieldError> {
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

    pub fn modify(&mut self, _key: TileKey, _new_tile: Tile) -> Result<Tile, FieldError> {
        unimplemented!()
    }

    pub fn get(&self, key: TileKey) -> Result<&Tile, FieldError> {
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
        self.chunk_size
    }

    #[inline]
    pub fn get_chunk(&self, chunk_key: IVec2) -> Result<&TileChunk, FieldError> {
        let chunk_key = *self.chunk_ref.get(&chunk_key).ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get(chunk_key as usize).unwrap();
        Ok(chunk)
    }

    // spatial features

    #[inline]
    pub fn get_by_point(&self, point: IVec2) -> Option<TileKey> {
        self.spatial_ref.get(&point).copied()
    }

    #[inline]
    pub fn has_by_point(&self, point: IVec2) -> bool {
        self.spatial_ref.contains_key(&point)
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
pub struct Block {
    pub id: u32,
    pub location: IVec2,
    pub variant: u8,
}

impl Block {
    #[inline]
    pub fn new(id: u32, location: IVec2, variant: u8) -> Self {
        Self {
            id,
            location,
            variant,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BlockChunk {
    pub version: u64,
    pub blocks: slab::Slab<Block>,
}

#[derive(Debug, Clone)]
pub struct BlockField {
    chunk_size: u32,
    props: Vec<BlockProperty>,
    chunks: Vec<BlockChunk>,
    chunk_ref: ahash::AHashMap<IVec2, u32>,
    spatial_ref: rstar::RTree<RectNode<IVec2, BlockKey>>,
    collision_ref: rstar::RTree<RectNode<Vec2, BlockKey>>,
    hint_ref: rstar::RTree<RectNode<Vec2, BlockKey>>,
}

impl BlockField {
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

    pub fn insert(&mut self, block: Block) -> Result<BlockKey, FieldError> {
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
            self.chunks.push(Default::default());
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

    pub fn remove(&mut self, key: BlockKey) -> Result<Block, FieldError> {
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

    pub fn modify(&mut self, _key: BlockKey, _new_block: Block) -> Result<Block, FieldError> {
        unimplemented!()
    }

    pub fn get(&self, key: BlockKey) -> Result<&Block, FieldError> {
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
    pub fn get_chunk(&self, chunk_key: IVec2) -> Result<&BlockChunk, FieldError> {
        let chunk_key = *self.chunk_ref.get(&chunk_key).ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get(chunk_key as usize).unwrap();
        Ok(chunk)
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
pub struct Entity {
    pub id: u32,
    pub location: Vec2,
    pub variant: u8,
}

impl Entity {
    #[inline]
    pub fn new(id: u32, location: Vec2, variant: u8) -> Self {
        Self {
            id,
            location,
            variant,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EntityChunk {
    pub version: u64,
    pub entities: slab::Slab<Entity>,
}

#[derive(Debug, Clone)]
pub struct EntityField {
    chunk_size: u32,
    props: Vec<EntityProperty>,
    chunks: Vec<EntityChunk>,
    chunk_ref: ahash::AHashMap<IVec2, u32>,
    collision_ref: rstar::RTree<RectNode<Vec2, EntityKey>>,
    hint_ref: rstar::RTree<RectNode<Vec2, EntityKey>>,
}

impl EntityField {
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

    pub fn insert(&mut self, entity: Entity) -> Result<EntityKey, FieldError> {
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
            self.chunks.push(Default::default());
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

    pub fn remove(&mut self, key: EntityKey) -> Result<Entity, FieldError> {
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

    pub fn modify(&mut self, _key: EntityKey, _new_entity: Entity) -> Result<Entity, FieldError> {
        unreachable!()
    }

    pub fn get(&self, key: EntityKey) -> Result<&Entity, FieldError> {
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
    pub fn get_chunk(&self, chunk_key: IVec2) -> Result<&EntityChunk, FieldError> {
        let chunk_key = *self.chunk_ref.get(&chunk_key).ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get(chunk_key as usize).unwrap();
        Ok(chunk)
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
        let mut field = TileField::new(TileFieldDescriptor {
            chunk_size: 16,
            tiles: vec![
                TileDescriptor { collision: true },
                TileDescriptor { collision: true },
            ],
        });

        let key = field.insert(Tile::new(1, [-1, 3], 0)).unwrap();

        assert_eq!(field.get(key), Ok(&Tile::new(1, [-1, 3], 0)));
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key));
        assert_eq!(field.remove(key), Ok(Tile::new(1, [-1, 3], 0)));

        assert_eq!(field.get(key), Err(FieldError::NotFound));
        assert!(!field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), None);
        assert_eq!(field.remove(key), Err(FieldError::NotFound));
    }

    #[test]
    fn insert_tile_with_invalid() {
        let mut field = TileField::new(TileFieldDescriptor {
            chunk_size: 16,
            tiles: vec![
                TileDescriptor { collision: true },
                TileDescriptor { collision: true },
            ],
        });

        assert_eq!(
            field.insert(Tile::new(2, [-1, 3], 0)),
            Err(FieldError::InvalidId)
        );
        assert!(!field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), None);

        let key = field.insert(Tile::new(1, [-1, 3], 0)).unwrap();
        assert_eq!(
            field.insert(Tile::new(0, [-1, 3], 0)),
            Err(FieldError::Conflict)
        );
        assert_eq!(field.get(key), Ok(&Tile::new(1, [-1, 3], 0)));
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key));
    }

    #[test]
    fn modify_tile() {
        let mut field = TileField::new(TileFieldDescriptor {
            chunk_size: 16,
            tiles: vec![
                TileDescriptor { collision: true },
                TileDescriptor { collision: true },
            ],
        });

        let key = field.insert(Tile::new(1, [-1, 3], 0)).unwrap();

        assert_eq!(
            field.modify(key, Tile::new(0, [-1, 3], 0)),
            Ok(Tile::new(1, [-1, 3], 0))
        );
        assert_eq!(field.get(key), Ok(&Tile::new(0, [-1, 3], 0)));
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key));

        assert_eq!(
            field.modify(key, Tile::new(0, [-1, 4], 0)),
            Ok(Tile::new(0, [-1, 3], 0))
        );
        assert_eq!(field.get(key), Ok(&Tile::new(0, [-1, 4], 0)));
        assert!(!field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), None);
        assert!(field.has_by_point([-1, 4]));
        assert_eq!(field.get_by_point([-1, 4]), Some(key));
    }

    #[test]
    fn modify_tile_with_invalid() {
        let mut field = TileField::new(TileFieldDescriptor {
            chunk_size: 16,
            tiles: vec![
                TileDescriptor { collision: true },
                TileDescriptor { collision: true },
            ],
        });

        let key_0 = field.insert(Tile::new(1, [-1, 3], 0)).unwrap();
        let key_1 = field.insert(Tile::new(1, [-1, 4], 0)).unwrap();

        assert_eq!(
            field.modify(key_0, Tile::new(3, [-1, 3], 0)),
            Err(FieldError::InvalidId)
        );
        assert_eq!(field.get(key_0), Ok(&Tile::new(1, [-1, 3], 0)));
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key_0));

        assert_eq!(
            field.modify(key_0, Tile::new(1, [-1, 4], 0)),
            Err(FieldError::Conflict)
        );
        assert_eq!(field.get(key_0), Ok(&Tile::new(1, [-1, 3], 0)));
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key_0));

        field.remove(key_1).unwrap();
        assert_eq!(
            field.modify(key_1, Tile::new(1, [-1, 4], 0)),
            Err(FieldError::NotFound)
        );
        assert_eq!(field.get(key_1), Err(FieldError::NotFound));
        assert!(!field.has_by_point([-1, 4]));
        assert_eq!(field.get_by_point([-1, 4]), None);
    }

    #[test]
    fn modify_tile_with_different_collision() {
        let mut field = TileField::new(TileFieldDescriptor {
            chunk_size: 16,
            tiles: vec![
                TileDescriptor { collision: false },
                TileDescriptor { collision: true },
            ],
        });

        let key = field.insert(Tile::new(0, [-1, 3], 0)).unwrap();

        let point = [-1.0, 3.0];
        assert!(!field.has_by_collision_point(point));
        let vec = field.get_by_collision_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![]);

        assert_eq!(
            field.modify(key, Tile::new(1, [-1, 3], 0)),
            Ok(Tile::new(0, [-1, 3], 0))
        );
        assert_eq!(field.get(key), Ok(&Tile::new(1, [-1, 3], 0)));
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key));

        let point = [-1.0, 3.0];
        assert!(field.has_by_collision_point(point));
        let vec = field.get_by_collision_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![key]);

        assert_eq!(
            field.modify(key, Tile::new(0, [-1, 3], 0)),
            Ok(Tile::new(1, [-1, 3], 0))
        );
        assert_eq!(field.get(key), Ok(&Tile::new(0, [-1, 3], 0)));
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key));

        let point = [-1.0, 3.0];
        assert!(!field.has_by_collision_point(point));
        let vec = field.get_by_collision_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![]);
    }

    #[test]
    fn modify_tile_with_move() {
        let mut field = TileField::new(TileFieldDescriptor {
            chunk_size: 16,
            tiles: vec![
                TileDescriptor { collision: true },
                TileDescriptor { collision: true },
            ],
        });

        let key = field.insert(Tile::new(1, [-1, 3], 0)).unwrap();

        assert_eq!(
            field.modify(key, Tile::new(1, [-1, 1000], 0)),
            Ok(Tile::new(1, [-1, 3], 0))
        );
        assert_eq!(field.get(key), Ok(&Tile::new(1, [-1, 1000], 0)));
        assert!(!field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), None);
        assert!(field.has_by_point([-1, 1000]));
        assert_eq!(field.get_by_point([-1, 1000]), Some(key));
    }

    #[test]
    fn collision_tile() {
        let mut field = TileField::new(TileFieldDescriptor {
            chunk_size: 16,
            tiles: vec![
                TileDescriptor { collision: true },
                TileDescriptor { collision: true },
            ],
        });

        let key_0 = field.insert(Tile::new(1, [-1, 3], 0)).unwrap();
        let key_1 = field.insert(Tile::new(1, [-1, 4], 0)).unwrap();
        let _key_2 = field.insert(Tile::new(1, [-1, 5], 0)).unwrap();

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
        let mut field = TileField::new(TileFieldDescriptor {
            chunk_size: 16,
            tiles: vec![
                TileDescriptor { collision: true },
                TileDescriptor { collision: true },
            ],
        });
        assert_eq!(field.get_chunk_size(), 16);

        field.insert(Tile::new(1, [-1, 3], 0)).unwrap();
        field.insert(Tile::new(1, [-1, 4], 0)).unwrap();
        field.insert(Tile::new(1, [-1, 5], 0)).unwrap();

        assert!(field.get_chunk([0, 0]).is_err());

        let chunk_1 = field.get_chunk([-1, 0]).unwrap();
        assert_eq!(chunk_1.tiles.len(), 3);
    }

    #[test]
    #[should_panic]
    fn block_field_with_invalid() {
        BlockField::new(BlockFieldDescriptor {
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
        BlockField::new(BlockFieldDescriptor {
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
        BlockField::new(BlockFieldDescriptor {
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

        let key = field.insert(Block::new(1, [-1, 3], 0)).unwrap();
        assert_eq!(field.get_rect(key), Ok([[-1, 3], [-1, 3]]));

        assert_eq!(field.get(key), Ok(&Block::new(1, [-1, 3], 0)));
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key));
        assert_eq!(field.remove(key), Ok(Block::new(1, [-1, 3], 0)));

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
            field.insert(Block::new(2, [-1, 3], 0)),
            Err(FieldError::InvalidId)
        );
        assert!(!field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), None);

        let key = field.insert(Block::new(1, [-1, 3], 0)).unwrap();
        assert_eq!(
            field.insert(Block::new(0, [-1, 3], 0)),
            Err(FieldError::Conflict)
        );
        assert_eq!(field.get(key), Ok(&Block::new(1, [-1, 3], 0)));
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

        let key = field.insert(Block::new(1, [-1, 3], 0)).unwrap();

        assert_eq!(
            field.modify(key, Block::new(0, [-1, 3], 0)),
            Ok(Block::new(1, [-1, 3], 0))
        );
        assert_eq!(field.get(key), Ok(&Block::new(0, [-1, 3], 0)));
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key));

        assert_eq!(
            field.modify(key, Block::new(0, [-1, 4], 0)),
            Ok(Block::new(0, [-1, 3], 0))
        );
        assert_eq!(field.get(key), Ok(&Block::new(0, [-1, 4], 0)));
        assert!(!field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), None);
        assert!(field.has_by_point([-1, 4]));
        assert_eq!(field.get_by_point([-1, 4]), Some(key));
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

        let key_0 = field.insert(Block::new(1, [-1, 3], 0)).unwrap();
        let key_1 = field.insert(Block::new(1, [-1, 4], 0)).unwrap();

        assert_eq!(
            field.modify(key_0, Block::new(3, [-1, 3], 0)),
            Err(FieldError::InvalidId)
        );
        assert_eq!(field.get(key_0), Ok(&Block::new(1, [-1, 3], 0)));
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key_0));

        assert_eq!(
            field.modify(key_0, Block::new(1, [-1, 4], 0)),
            Err(FieldError::Conflict)
        );
        assert_eq!(field.get(key_0), Ok(&Block::new(1, [-1, 3], 0)));
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key_0));

        field.remove(key_1).unwrap();
        assert_eq!(
            field.modify(key_1, Block::new(1, [-1, 4], 0)),
            Err(FieldError::NotFound)
        );
        assert_eq!(field.get(key_1), Err(FieldError::NotFound));
        assert!(!field.has_by_point([-1, 4]));
        assert_eq!(field.get_by_point([-1, 4]), None);
    }

    #[test]
    fn modify_block_with_different_attr() {
        let mut field = BlockField::new(BlockFieldDescriptor {
            chunk_size: 16,
            blocks: vec![
                BlockDescriptor {
                    size: [1, 1],
                    collision_size: [0.0, 0.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [0.0, 0.0],
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

        let key = field.insert(Block::new(0, [-1, 3], 0)).unwrap();

        let point = [-1.0, 3.0];
        assert!(!field.has_by_collision_point(point));
        assert!(!field.has_by_hint_point(point));
        let vec = field.get_by_collision_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![]);
        let vec = field.get_by_hint_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![]);

        assert_eq!(
            field.modify(key, Block::new(1, [-1, 3], 0)),
            Ok(Block::new(0, [-1, 3], 0))
        );
        assert_eq!(field.get(key), Ok(&Block::new(1, [-1, 3], 0)));
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key));

        let point = [-1.0, 3.0];
        assert!(field.has_by_collision_point(point));
        assert!(field.has_by_hint_point(point));
        let vec = field.get_by_collision_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![key]);
        let vec = field.get_by_hint_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![key]);

        assert_eq!(
            field.modify(key, Block::new(0, [-1, 3], 0)),
            Ok(Block::new(1, [-1, 3], 0))
        );
        assert_eq!(field.get(key), Ok(&Block::new(0, [-1, 3], 0)));
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key));

        let point = [-1.0, 3.0];
        assert!(!field.has_by_collision_point(point));
        assert!(!field.has_by_hint_point(point));
        let vec = field.get_by_collision_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![]);
        let vec = field.get_by_hint_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![]);
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

        let key = field.insert(Block::new(1, [-1, 3], 0)).unwrap();

        assert_eq!(
            field.modify(key, Block::new(1, [-1, 1000], 0)),
            Ok(Block::new(1, [-1, 3], 0))
        );
        assert_eq!(field.get(key), Ok(&Block::new(1, [-1, 1000], 0)));
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

        let key_0 = field.insert(Block::new(1, [-1, 3], 0)).unwrap();
        let key_1 = field.insert(Block::new(1, [-1, 4], 0)).unwrap();
        let _key_2 = field.insert(Block::new(1, [-1, 5], 0)).unwrap();

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

        let key_0 = field.insert(Block::new(1, [-1, 3], 0)).unwrap();
        let key_1 = field.insert(Block::new(1, [-1, 4], 0)).unwrap();
        let _key_2 = field.insert(Block::new(1, [-1, 5], 0)).unwrap();

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

        field.insert(Block::new(1, [-1, 3], 0)).unwrap();
        field.insert(Block::new(1, [-1, 4], 0)).unwrap();
        field.insert(Block::new(1, [-1, 5], 0)).unwrap();

        assert!(field.get_chunk([0, 0]).is_err());

        let chunk_1 = field.get_chunk([-1, 0]).unwrap();
        assert_eq!(chunk_1.blocks.len(), 3);
    }

    #[test]
    #[should_panic]
    fn entity_field_with_invalid_collision() {
        EntityField::new(EntityFieldDescriptor {
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
        EntityField::new(EntityFieldDescriptor {
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

        let key = field.insert(Entity::new(1, [-1.0, 3.0], 0)).unwrap();

        assert_eq!(field.get(key), Ok(&Entity::new(1, [-1.0, 3.0], 0)));
        assert_eq!(field.remove(key), Ok(Entity::new(1, [-1.0, 3.0], 0)));

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
            field.insert(Entity::new(2, [-1.0, 3.0], 0)),
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

        let key = field.insert(Entity::new(1, [-1.0, 3.0], 0)).unwrap();

        assert_eq!(
            field.modify(key, Entity::new(0, [-1.0, 3.0], 0)),
            Ok(Entity::new(1, [-1.0, 3.0], 0))
        );
        assert_eq!(field.get(key), Ok(&Entity::new(0, [-1.0, 3.0], 0)));

        assert_eq!(
            field.modify(key, Entity::new(0, [-1.0, 4.0], 0)),
            Ok(Entity::new(0, [-1.0, 3.0], 0))
        );
        assert_eq!(field.get(key), Ok(&Entity::new(0, [-1.0, 4.0], 0)));
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

        let key_0 = field.insert(Entity::new(1, [-1.0, 3.0], 0)).unwrap();
        let key_1 = field.insert(Entity::new(1, [-1.0, 4.0], 0)).unwrap();

        assert_eq!(
            field.modify(key_0, Entity::new(3, [-1.0, 3.0], 0)),
            Err(FieldError::InvalidId)
        );
        assert_eq!(field.get(key_0), Ok(&Entity::new(1, [-1.0, 3.0], 0)));

        field.remove(key_1).unwrap();
        assert_eq!(
            field.modify(key_1, Entity::new(1, [-1.0, 4.0], 0)),
            Err(FieldError::NotFound)
        );
        assert_eq!(field.get(key_1), Err(FieldError::NotFound));
    }

    #[test]
    fn modify_entity_with_different_attr() {
        let mut field = EntityField::new(EntityFieldDescriptor {
            chunk_size: 16,
            entities: vec![
                EntityDescriptor {
                    collision_size: [0.0, 0.0],
                    collision_offset: [0.0, 0.0],
                    hint_size: [0.0, 0.0],
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
        let key = field.insert(Entity::new(0, [-1.0, 3.0], 0)).unwrap();

        let point = [-1.0, 3.0];
        assert!(!field.has_by_collision_point(point));
        assert!(!field.has_by_hint_point(point));
        let vec = field.get_by_collision_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![]);
        let vec = field.get_by_hint_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![]);

        assert_eq!(
            field.modify(key, Entity::new(1, [-1.0, 3.0], 0)),
            Ok(Entity::new(0, [-1.0, 3.0], 0))
        );
        assert_eq!(field.get(key), Ok(&Entity::new(1, [-1.0, 3.0], 0)));

        let point = [-1.0, 3.0];
        assert!(field.has_by_collision_point(point));
        assert!(field.has_by_hint_point(point));
        let vec = field.get_by_collision_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![key]);
        let vec = field.get_by_hint_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![key]);

        assert_eq!(
            field.modify(key, Entity::new(0, [-1.0, 3.0], 0)),
            Ok(Entity::new(1, [-1.0, 3.0], 0))
        );
        assert_eq!(field.get(key), Ok(&Entity::new(0, [-1.0, 3.0], 0)));

        let point = [-1.0, 3.0];
        assert!(!field.has_by_collision_point(point));
        assert!(!field.has_by_hint_point(point));
        let vec = field.get_by_collision_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![]);
        let vec = field.get_by_hint_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![]);
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

        let key = field.insert(Entity::new(1, [-1.0, 3.0], 0)).unwrap();

        assert_eq!(
            field.modify(key, Entity::new(1, [-1.0, 1000.0], 0)),
            Ok(Entity::new(1, [-1.0, 3.0], 0))
        );
        assert_eq!(field.get(key), Ok(&Entity::new(1, [-1.0, 1000.0], 0)));
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

        let key_0 = field.insert(Entity::new(1, [-1.0, 3.0], 0)).unwrap();
        let key_1 = field.insert(Entity::new(1, [-1.0, 4.0], 0)).unwrap();
        let _key_2 = field.insert(Entity::new(1, [-1.0, 5.0], 0)).unwrap();

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

        let key_0 = field.insert(Entity::new(1, [-1.0, 3.0], 0)).unwrap();
        let key_1 = field.insert(Entity::new(1, [-1.0, 4.0], 0)).unwrap();
        let _key_2 = field.insert(Entity::new(1, [-1.0, 5.0], 0)).unwrap();

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

        field.insert(Entity::new(1, [-1.0, 3.0], 0)).unwrap();
        field.insert(Entity::new(1, [-1.0, 4.0], 0)).unwrap();
        field.insert(Entity::new(1, [-1.0, 5.0], 0)).unwrap();

        assert!(field.get_chunk([0, 0]).is_err());

        let chunk_1 = field.get_chunk([-1, 0]).unwrap();
        assert_eq!(chunk_1.entities.len(), 3);
    }
}
