use super::*;

type RectNode<T, U> = rstar::primitives::GeomWithData<rstar::primitives::Rectangle<T>, U>;

// tile field

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
    chunks: ahash::AHashMap<IVec2, TileChunk>,
    stable_ref: slab::Slab<(IVec2, u32)>,
    spatial_ref: ahash::AHashMap<IVec2, u32>,
    collision_ref: rstar::RTree<RectNode<Vec2, u32>>,
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
            stable_ref: Default::default(),
            chunks: Default::default(),
            spatial_ref: Default::default(),
            collision_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, tile: Tile) -> Result<u32, FieldError> {
        let prop = self
            .props
            .get(tile.id as usize)
            .ok_or(FieldError::InvalidId)?;

        // check by spatial features
        if self.has_by_point(tile.location) {
            return Err(FieldError::Conflict);
        }

        let chunk_key = [
            tile.location[0].div_euclid(self.chunk_size as i32),
            tile.location[1].div_euclid(self.chunk_size as i32),
        ];
        let chunk = self.chunks.entry(chunk_key).or_default();

        if chunk.tiles.vacant_key() >= u32::MAX as usize {
            panic!("capacity overflow");
        }

        // key is guaranteed to be less than u32::MAX.
        let key = self.stable_ref.vacant_key() as u32;

        // spatial features
        self.spatial_ref.insert(tile.location, key);

        // collision features
        if let Some(rect) = prop.collision_rect(tile.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.insert(node);
        }

        // key is guaranteed to be less than u32::MAX.
        let tile_key = chunk.tiles.insert(tile) as u32;
        chunk.version += 1;

        self.stable_ref.insert((chunk_key, tile_key));

        Ok(key)
    }

    pub fn remove(&mut self, key: u32) -> Result<Tile, FieldError> {
        let (chunk_key, tile_key) = self
            .stable_ref
            .try_remove(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get_mut(&chunk_key).unwrap();
        let tile = chunk.tiles.try_remove(tile_key as usize).unwrap();
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

    pub fn modify(&mut self, key: u32, new_tile: Tile) -> Result<Tile, FieldError> {
        let (chunk_key, tile_key) = *self
            .stable_ref
            .get(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get(&chunk_key).unwrap();
        let tile = chunk.tiles.get(tile_key as usize).unwrap();

        if tile.id != new_tile.id || tile.location != new_tile.location {
            let prop = self.props.get(tile.id as usize).unwrap();

            let new_prop = self
                .props
                .get(new_tile.id as usize)
                .ok_or(FieldError::InvalidId)?;

            // check by spatial features
            if self
                .get_by_point(new_tile.location)
                .is_some_and(|other_key| other_key != key)
            {
                return Err(FieldError::Conflict);
            }

            // spatial features

            self.spatial_ref.remove(&tile.location).unwrap();

            self.spatial_ref.insert(new_tile.location, key);

            // collision features

            if let Some(rect) = prop.collision_rect(tile.location) {
                let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
                let node = rstar::primitives::GeomWithData::new(rect, key);
                self.collision_ref.remove(&node).unwrap();
            }

            if let Some(rect) = new_prop.collision_rect(new_tile.location) {
                let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
                let node = rstar::primitives::GeomWithData::new(rect, key);
                self.collision_ref.insert(node);
            }
        }

        let new_chunk_key = [
            new_tile.location[0].div_euclid(self.chunk_size as i32),
            new_tile.location[1].div_euclid(self.chunk_size as i32),
        ];

        // wheather to move tile to another chunk or not
        if chunk_key != new_chunk_key {
            let chunk = self.chunks.get_mut(&chunk_key).unwrap();
            let old_tile = chunk.tiles.try_remove(tile_key as usize).unwrap();
            chunk.version += 1;

            let new_chunk = self.chunks.entry(new_chunk_key).or_default();
            let new_tile_key = new_chunk.tiles.insert(new_tile) as u32;
            new_chunk.version += 1;

            *self.stable_ref.get_mut(key as usize).unwrap() = (new_chunk_key, new_tile_key);

            Ok(old_tile)
        } else {
            let chunk = self.chunks.get_mut(&chunk_key).unwrap();
            let tile = chunk.tiles.get_mut(tile_key as usize).unwrap();
            let old_tile = std::mem::replace(tile, new_tile);
            chunk.version += 1;

            Ok(old_tile)
        }
    }

    pub fn get(&self, key: u32) -> Result<&Tile, FieldError> {
        let (chunk_key, tile_key) = self
            .stable_ref
            .get(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get(chunk_key).unwrap();
        let tile = chunk.tiles.get(*tile_key as usize).unwrap();
        Ok(tile)
    }

    #[inline]
    pub fn get_chunk_size(&self) -> u32 {
        self.chunk_size
    }

    #[inline]
    pub fn get_chunk(&self, chunk_key: IVec2) -> Option<&TileChunk> {
        self.chunks.get(&chunk_key)
    }

    // spatial features

    #[inline]
    pub fn get_by_point(&self, point: IVec2) -> Option<u32> {
        self.spatial_ref.get(&point).copied()
    }

    #[inline]
    pub fn has_by_point(&self, point: IVec2) -> bool {
        self.spatial_ref.contains_key(&point)
    }

    // collision features

    #[inline]
    pub fn get_collision_rect(&self, tile_key: u32) -> Result<[Vec2; 2], FieldError> {
        let tile = self.get(tile_key)?;
        let prop = self.props.get(tile.id as usize).unwrap();
        Ok(prop.collision_rect(tile.location).unwrap_or_default())
    }

    #[inline]
    pub fn has_by_collision_point(&self, point: Vec2) -> bool {
        self.collision_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = u32> + '_ {
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
    pub fn get_by_collision_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = u32> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }
}

// block field

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
    chunks: ahash::AHashMap<IVec2, BlockChunk>,
    stable_ref: slab::Slab<(IVec2, u32)>,
    spatial_ref: rstar::RTree<RectNode<IVec2, u32>>,
    collision_ref: rstar::RTree<RectNode<Vec2, u32>>,
    hint_ref: rstar::RTree<RectNode<Vec2, u32>>,
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
            stable_ref: Default::default(),
            spatial_ref: Default::default(),
            collision_ref: Default::default(),
            hint_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, block: Block) -> Result<u32, FieldError> {
        let prop = self
            .props
            .get(block.id as usize)
            .ok_or(FieldError::InvalidId)?;

        // check by spatial features
        if self.has_by_rect(prop.rect(block.location)) {
            return Err(FieldError::Conflict);
        }

        let chunk_key = [
            block.location[0].div_euclid(self.chunk_size as i32),
            block.location[1].div_euclid(self.chunk_size as i32),
        ];
        let chunk = self.chunks.entry(chunk_key).or_default();

        if chunk.blocks.vacant_key() >= u32::MAX as usize {
            panic!("capacity overflow");
        }

        // key is guaranteed to be less than u32::MAX.
        let key = self.stable_ref.vacant_key() as u32;

        // spatial features
        let rect = prop.rect(block.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = rstar::primitives::GeomWithData::new(rect, key);
        self.spatial_ref.insert(node);

        // collision features
        if let Some(rect) = prop.collision_rect(block.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.insert(node);
        }

        // hint features
        if let Some(rect) = prop.hint_rect(block.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.hint_ref.insert(node);
        }

        // block_key is guaranteed to be less than u32::MAX.
        let block_key = chunk.blocks.insert(block) as u32;
        chunk.version += 1;

        self.stable_ref.insert((chunk_key, block_key));

        Ok(key)
    }

    pub fn remove(&mut self, key: u32) -> Result<Block, FieldError> {
        let (chunk_key, block_key) = self
            .stable_ref
            .try_remove(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get_mut(&chunk_key).unwrap();
        let block = chunk.blocks.try_remove(block_key as usize).unwrap();
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

    pub fn modify(&mut self, key: u32, new_block: Block) -> Result<Block, FieldError> {
        let (chunk_key, block_key) = *self
            .stable_ref
            .get(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get(&chunk_key).unwrap();
        let block = chunk.blocks.get(block_key as usize).unwrap();

        if block.id != new_block.id || block.location != new_block.location {
            let prop = self.props.get(block.id as usize).unwrap();

            let new_prop = self
                .props
                .get(new_block.id as usize)
                .ok_or(FieldError::InvalidId)?;

            // check by spatial features
            if self
                .get_by_rect(new_prop.rect(new_block.location))
                .any(|other_key| other_key != key)
            {
                return Err(FieldError::Conflict);
            }

            // spatial features

            let rect = prop.rect(block.location);
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = &rstar::primitives::GeomWithData::new(rect, key);
            self.spatial_ref.remove(node).unwrap();

            let rect = new_prop.rect(new_block.location);
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.spatial_ref.insert(node);

            // collision features

            if let Some(rect) = prop.collision_rect(block.location) {
                let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
                let node = rstar::primitives::GeomWithData::new(rect, key);
                self.collision_ref.remove(&node).unwrap();
            }

            if let Some(rect) = new_prop.collision_rect(new_block.location) {
                let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
                let node = rstar::primitives::GeomWithData::new(rect, key);
                self.collision_ref.insert(node);
            }

            // hint features

            if let Some(rect) = prop.hint_rect(block.location) {
                let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
                let node = rstar::primitives::GeomWithData::new(rect, key);
                self.hint_ref.remove(&node).unwrap();
            }

            if let Some(rect) = new_prop.hint_rect(new_block.location) {
                let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
                let node = rstar::primitives::GeomWithData::new(rect, key);
                self.hint_ref.insert(node);
            }
        }

        let new_chunk_key = [
            new_block.location[0].div_euclid(self.chunk_size as i32),
            new_block.location[1].div_euclid(self.chunk_size as i32),
        ];

        // wheather to move block to another chunk or not
        if chunk_key != new_chunk_key {
            let chunk = self.chunks.get_mut(&chunk_key).unwrap();
            let old_block = chunk.blocks.try_remove(block_key as usize).unwrap();
            chunk.version += 1;

            let new_chunk = self.chunks.entry(new_chunk_key).or_default();
            let new_block_key = new_chunk.blocks.insert(new_block) as u32;
            new_chunk.version += 1;

            *self.stable_ref.get_mut(key as usize).unwrap() = (new_chunk_key, new_block_key);

            Ok(old_block)
        } else {
            let chunk = self.chunks.get_mut(&chunk_key).unwrap();
            let block = chunk.blocks.get_mut(block_key as usize).unwrap();
            let old_block = std::mem::replace(block, new_block);
            chunk.version += 1;

            Ok(old_block)
        }
    }

    pub fn get(&self, key: u32) -> Result<&Block, FieldError> {
        let (chunk_key, block_key) = self
            .stable_ref
            .get(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get(chunk_key).unwrap();
        let block = chunk.blocks.get(*block_key as usize).unwrap();
        Ok(block)
    }

    #[inline]
    pub fn get_chunk_size(&self) -> u32 {
        self.chunk_size
    }

    #[inline]
    pub fn get_chunk(&self, chunk_key: IVec2) -> Option<&BlockChunk> {
        self.chunks.get(&chunk_key)
    }

    // spatial features

    #[inline]
    pub fn get_rect(&self, block_key: u32) -> Result<[IVec2; 2], FieldError> {
        let block = self.get(block_key)?;
        let prop = self.props.get(block.id as usize).unwrap();
        Ok(prop.rect(block.location))
    }

    #[inline]
    pub fn has_by_point(&self, point: IVec2) -> bool {
        self.spatial_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_by_point(&self, point: IVec2) -> Option<u32> {
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
    pub fn get_by_rect(&self, rect: [IVec2; 2]) -> impl Iterator<Item = u32> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.spatial_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }

    // collision features

    #[inline]
    pub fn get_collision_rect(&self, block_key: u32) -> Result<[Vec2; 2], FieldError> {
        let block = self.get(block_key)?;
        let prop = self.props.get(block.id as usize).unwrap();
        Ok(prop.collision_rect(block.location).unwrap_or_default())
    }

    #[inline]
    pub fn has_by_collision_point(&self, point: Vec2) -> bool {
        self.collision_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = u32> + '_ {
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
    pub fn get_by_collision_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = u32> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }

    // hint features

    #[inline]
    pub fn get_hint_rect(&self, block_key: u32) -> Result<[Vec2; 2], FieldError> {
        let block = self.get(block_key)?;
        let prop = self.props.get(block.id as usize).unwrap();
        Ok(prop.hint_rect(block.location).unwrap_or_default())
    }

    #[inline]
    pub fn has_by_hint_point(&self, point: Vec2) -> bool {
        self.hint_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_by_hint_point(&self, point: Vec2) -> impl Iterator<Item = u32> + '_ {
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
    pub fn get_by_hint_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = u32> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }
}

// entity field

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
    pub fn new(
        collision_size: Vec2,
        collision_offset: Vec2,
        hint_size: Vec2,
        hint_offset: Vec2,
    ) -> Self {
        if collision_size[0] < 0.0 || collision_size[1] < 0.0 {
            panic!("collision size must be non-negative");
        }
        if hint_size[0] < 0.0 || hint_size[1] < 0.0 {
            panic!("hint size must be non-negative");
        }

        Self {
            collision_size,
            collision_offset,
            hint_size,
            hint_offset,
        }
    }

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
    chunks: ahash::AHashMap<IVec2, EntityChunk>,
    stable_ref: slab::Slab<(IVec2, u32)>,
    collision_ref: rstar::RTree<RectNode<Vec2, u32>>,
    hint_ref: rstar::RTree<RectNode<Vec2, u32>>,
}

impl EntityField {
    pub fn new(desc: EntityFieldDescriptor) -> Self {
        let mut props = vec![];
        for entity in desc.entities {
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
            stable_ref: Default::default(),
            collision_ref: Default::default(),
            hint_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, entity: Entity) -> Result<u32, FieldError> {
        let prop = self
            .props
            .get(entity.id as usize)
            .ok_or(FieldError::InvalidId)?;

        let chunk_key = [
            entity.location[0].div_euclid(self.chunk_size as f32) as i32,
            entity.location[1].div_euclid(self.chunk_size as f32) as i32,
        ];
        let chunk = self.chunks.entry(chunk_key).or_default();

        if chunk.entities.vacant_key() >= u32::MAX as usize {
            panic!("capacity overflow");
        }

        // key is guaranteed to be less than u32::MAX.
        let key = self.stable_ref.vacant_key() as u32;

        // collision features
        if let Some(rect) = prop.collision_rect(entity.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.insert(node);
        }

        // hint features
        if let Some(rect) = prop.hint_rect(entity.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.hint_ref.insert(node);
        }

        // entity_key is guaranteed to be less than u32::MAX.
        let entity_key = chunk.entities.insert(entity) as u32;
        chunk.version += 1;

        self.stable_ref.insert((chunk_key, entity_key));

        Ok(key)
    }

    pub fn remove(&mut self, key: u32) -> Result<Entity, FieldError> {
        let (chunk_key, entity_key) = self
            .stable_ref
            .try_remove(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get_mut(&chunk_key).unwrap();
        let entity = chunk.entities.try_remove(entity_key as usize).unwrap();
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

    pub fn modify(&mut self, key: u32, new_entity: Entity) -> Result<Entity, FieldError> {
        let (chunk_key, entity_key) = *self
            .stable_ref
            .get(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get(&chunk_key).unwrap();
        let entity = chunk.entities.get(entity_key as usize).unwrap();

        if entity.id != new_entity.id || entity.location != new_entity.location {
            let prop = self.props.get(entity.id as usize).unwrap();

            let new_prop = self
                .props
                .get(new_entity.id as usize)
                .ok_or(FieldError::InvalidId)?;

            // collision features

            if let Some(rect) = prop.collision_rect(entity.location) {
                let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
                let node = rstar::primitives::GeomWithData::new(rect, key);
                self.collision_ref.remove(&node).unwrap();
            }

            if let Some(rect) = new_prop.collision_rect(new_entity.location) {
                let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
                let node = rstar::primitives::GeomWithData::new(rect, key);
                self.collision_ref.insert(node);
            }

            // hint features

            if let Some(rect) = prop.hint_rect(entity.location) {
                let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
                let node = rstar::primitives::GeomWithData::new(rect, key);
                self.hint_ref.remove(&node).unwrap();
            }

            if let Some(rect) = new_prop.hint_rect(new_entity.location) {
                let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
                let node = rstar::primitives::GeomWithData::new(rect, key);
                self.hint_ref.insert(node);
            }
        }

        let new_chunk_key = [
            new_entity.location[0].div_euclid(self.chunk_size as f32) as i32,
            new_entity.location[1].div_euclid(self.chunk_size as f32) as i32,
        ];

        // wheather to move entity to another chunk or not
        if chunk_key != new_chunk_key {
            let chunk = self.chunks.get_mut(&chunk_key).unwrap();
            let old_entity = chunk.entities.try_remove(entity_key as usize).unwrap();
            chunk.version += 1;

            let new_chunk = self.chunks.entry(new_chunk_key).or_default();
            let new_entity_key = new_chunk.entities.insert(new_entity) as u32;
            new_chunk.version += 1;

            *self.stable_ref.get_mut(key as usize).unwrap() = (new_chunk_key, new_entity_key);

            Ok(old_entity)
        } else {
            let chunk = self.chunks.get_mut(&chunk_key).unwrap();
            let entity = chunk.entities.get_mut(entity_key as usize).unwrap();
            let old_entity = std::mem::replace(entity, new_entity);
            chunk.version += 1;

            Ok(old_entity)
        }
    }

    pub fn get(&self, key: u32) -> Result<&Entity, FieldError> {
        let (chunk_key, entity_key) = self
            .stable_ref
            .get(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get(chunk_key).unwrap();
        let entity = chunk.entities.get(*entity_key as usize).unwrap();
        Ok(entity)
    }

    #[inline]
    pub fn get_chunk_size(&self) -> u32 {
        self.chunk_size
    }

    #[inline]
    pub fn get_chunk(&self, chunk_key: IVec2) -> Option<&EntityChunk> {
        self.chunks.get(&chunk_key)
    }

    // collision features

    #[inline]
    pub fn get_collision_rect(&self, entity_key: u32) -> Result<[Vec2; 2], FieldError> {
        let entity = self.get(entity_key)?;
        let prop = self.props.get(entity.id as usize).unwrap();
        Ok(prop.hint_rect(entity.location).unwrap_or_default())
    }

    #[inline]
    pub fn has_by_collision_point(&self, point: Vec2) -> bool {
        self.collision_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = u32> + '_ {
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
    pub fn get_by_collision_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = u32> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }

    // hint features

    #[inline]
    pub fn get_hint_rect(&self, entity_key: u32) -> Result<[Vec2; 2], FieldError> {
        let entity = self.get(entity_key)?;
        let prop = self.props.get(entity.id as usize).unwrap();
        Ok(prop.hint_rect(entity.location).unwrap_or_default())
    }

    #[inline]
    pub fn has_by_hint_point(&self, point: Vec2) -> bool {
        self.hint_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_by_hint_point(&self, point: Vec2) -> impl Iterator<Item = u32> + '_ {
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
    pub fn get_by_hint_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = u32> + '_ {
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

        assert_eq!(field.get(key), Ok(&Block::new(1, [-1, 3], 0)));
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key));
        assert_eq!(field.remove(key), Ok(Block::new(1, [-1, 3], 0)));

        assert_eq!(field.get(key), Err(FieldError::NotFound));
        assert!(!field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), None);
        assert_eq!(field.remove(key), Err(FieldError::NotFound));
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
    }
}
