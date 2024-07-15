use super::*;

type RectNode<T, U> = rstar::primitives::GeomWithData<rstar::primitives::Rectangle<T>, U>;

// tile field

#[derive(Debug, Clone)]
pub struct TileSpec {
    collision: bool,
}

impl TileSpec {
    pub fn new(collision: bool) -> Self {
        Self { collision }
    }

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
    pub serial: u64,
    pub tiles: slab::Slab<Tile>,
}

#[derive(Debug, Clone)]
pub struct TileField {
    chunk_size: u32,
    specs: Vec<TileSpec>,
    chunks: ahash::AHashMap<IVec2, TileChunk>,
    stable_ref: slab::Slab<(IVec2, u32)>,
    spatial_ref: ahash::AHashMap<IVec2, u32>,
    collision_ref: rstar::RTree<RectNode<Vec2, u32>>,
}

impl TileField {
    pub fn new(chunk_size: u32, specs: Vec<TileSpec>) -> Self {
        Self {
            chunk_size,
            specs,
            stable_ref: Default::default(),
            chunks: Default::default(),
            spatial_ref: Default::default(),
            collision_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, tile: Tile) -> Result<u32, FieldError> {
        let location = tile.location;

        let spec = self
            .specs
            .get(tile.id as usize)
            .ok_or(FieldError::InvalidId)?;

        // check by spatial features
        if self.spatial_ref.contains_key(&location) {
            return Err(FieldError::Conflict);
        }

        let chunk_key = [
            location[0].div_euclid(self.chunk_size as i32),
            location[1].div_euclid(self.chunk_size as i32),
        ];
        let chunk = self.chunks.entry(chunk_key).or_default();
        let tile_key = chunk.tiles.insert(tile) as u32;
        chunk.serial += 1;

        let key = self.stable_ref.insert((chunk_key, tile_key)) as u32;

        // spatial features
        self.spatial_ref.insert(location, key);

        // collision features
        if let Some(rect) = spec.collision_rect(location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.insert(node);
        }

        Ok(key)
    }

    pub fn remove(&mut self, key: u32) -> Result<Tile, FieldError> {
        let (chunk_key, tile_key) = self
            .stable_ref
            .try_remove(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get_mut(&chunk_key).unwrap();
        let tile = chunk.tiles.try_remove(tile_key as usize).unwrap();
        chunk.serial += 1;

        let spec = &self.specs.get(tile.id as usize).unwrap();

        // spatial features
        self.spatial_ref.remove(&tile.location).unwrap();

        // collision features
        if let Some(rect) = spec.collision_rect(tile.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.remove(&node).unwrap();
        }

        Ok(tile)
    }

    pub fn modify(&mut self, key: u32, new_tile: Tile) -> Result<Tile, FieldError> {
        let new_spec = self
            .specs
            .get(new_tile.id as usize)
            .ok_or(FieldError::InvalidId)?;

        // validate modification
        if !self
            .spatial_ref
            .get(&new_tile.location)
            .map_or(true, |other_key| *other_key == key)
        {
            return Err(FieldError::Conflict);
        }

        // remove old tile
        let (chunk_key, tile_key) = self
            .stable_ref
            .get_mut(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get_mut(chunk_key).unwrap();
        let tile = chunk.tiles.try_remove(*tile_key as usize).unwrap();
        chunk.serial += 1;

        let spec = self.specs.get(tile.id as usize).unwrap();

        // spatial features
        self.spatial_ref.remove(&tile.location).unwrap();

        // collision features
        if let Some(rect) = spec.collision_rect(tile.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.remove(&node).unwrap();
        }

        let location = new_tile.location;

        // insert new tile
        *chunk_key = [
            location[0].div_euclid(self.chunk_size as i32),
            location[1].div_euclid(self.chunk_size as i32),
        ];
        let chunk = self.chunks.entry(*chunk_key).or_default();
        *tile_key = chunk.tiles.insert(new_tile) as u32;
        chunk.serial += 1;

        // spatial features
        self.spatial_ref.insert(location, key);

        // collision features
        if let Some(rect) = new_spec.collision_rect(location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.insert(node);
        }

        Ok(tile)
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
    pub fn has_by_point(&self, point: IVec2) -> bool {
        self.spatial_ref.contains_key(&point)
    }

    #[inline]
    pub fn get_by_point(&self, point: IVec2) -> Option<u32> {
        self.spatial_ref.get(&point).copied()
    }

    // collision features

    #[inline]
    pub fn get_collision_rect(&self, entity_key: u32) -> Result<[Vec2; 2], FieldError> {
        let entity = self.get(entity_key)?;
        let spec = self.specs.get(entity.id as usize).unwrap();
        Ok(spec.collision_rect(entity.location).unwrap_or_default())
    }

    #[inline]
    pub fn has_collision_by_point(&self, point: Vec2) -> bool {
        self.collision_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_collision_by_point(&self, point: Vec2) -> impl Iterator<Item = u32> + '_ {
        self.collision_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    #[inline]
    pub fn has_collision_by_rect(&self, rect: [Vec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    #[inline]
    pub fn get_collision_by_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = u32> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }
}

// block field

#[derive(Debug, Clone)]
pub struct BlockSpec {
    size: IVec2,
    collision_size: Vec2,
    collision_offset: Vec2,
    hint_size: Vec2,
    hint_offset: Vec2,
}

impl BlockSpec {
    pub fn new(
        size: IVec2,
        collision_size: Vec2,
        collision_offset: Vec2,
        hint_size: Vec2,
        hint_offset: Vec2,
    ) -> Self {
        if size[0] <= 0 || size[1] <= 0 {
            panic!("size must be positive");
        }
        if collision_size[0] < 0.0 || collision_size[1] < 0.0 {
            panic!("collision size must be non-negative");
        }
        if hint_size[0] < 0.0 || hint_size[1] < 0.0 {
            panic!("hint size must be non-negative");
        }

        Self {
            size,
            collision_size,
            collision_offset,
            hint_size,
            hint_offset,
        }
    }

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
    pub serial: u64,
    pub blocks: slab::Slab<Block>,
}

#[derive(Debug, Clone)]
pub struct BlockField {
    chunk_size: u32,
    specs: Vec<BlockSpec>,
    chunks: ahash::AHashMap<IVec2, BlockChunk>,
    stable_ref: slab::Slab<(IVec2, u32)>,
    spatial_ref: rstar::RTree<RectNode<IVec2, u32>>,
    collision_ref: rstar::RTree<RectNode<Vec2, u32>>,
    hint_ref: rstar::RTree<RectNode<Vec2, u32>>,
}

impl BlockField {
    pub fn new(chunk_size: u32, specs: Vec<BlockSpec>) -> Self {
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

    pub fn insert(&mut self, block: Block) -> Result<u32, FieldError> {
        let location = block.location;

        let spec = &self
            .specs
            .get(block.id as usize)
            .ok_or(FieldError::InvalidId)?;

        // check by spatial features
        if self.has_by_rect(spec.rect(location)) {
            return Err(FieldError::Conflict);
        }

        let chunk_key = [
            location[0].div_euclid(self.chunk_size as i32),
            location[1].div_euclid(self.chunk_size as i32),
        ];
        let chunk = self.chunks.entry(chunk_key).or_default();
        let block_key = chunk.blocks.insert(block) as u32;
        chunk.serial += 1;

        let key = self.stable_ref.insert((chunk_key, block_key)) as u32;

        // spatial features
        let rect = spec.rect(location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = rstar::primitives::GeomWithData::new(rect, key);
        self.spatial_ref.insert(node);

        // collision features
        if let Some(rect) = spec.collision_rect(location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.insert(node);
        }

        // hint features
        if let Some(rect) = spec.hint_rect(location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.hint_ref.insert(node);
        }

        Ok(key)
    }

    pub fn remove(&mut self, key: u32) -> Result<Block, FieldError> {
        let (chunk_key, block_key) = self
            .stable_ref
            .try_remove(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get_mut(&chunk_key).unwrap();
        let block = chunk.blocks.try_remove(block_key as usize).unwrap();
        chunk.serial += 1;

        let spec = &self.specs.get(block.id as usize).unwrap();

        // spatial features
        let rect = spec.rect(block.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = rstar::primitives::GeomWithData::new(rect, key);
        self.spatial_ref.remove(&node).unwrap();

        // collision features
        if let Some(rect) = spec.collision_rect(block.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.remove(&node).unwrap();
        }

        // hint features
        if let Some(rect) = spec.hint_rect(block.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.hint_ref.remove(&node).unwrap();
        }

        Ok(block)
    }

    pub fn modify(&mut self, key: u32, new_block: Block) -> Result<Block, FieldError> {
        // check by spatial features
        let new_spec = &self
            .specs
            .get(new_block.id as usize)
            .ok_or(FieldError::InvalidId)?;

        let rect = new_spec.rect(new_block.location);
        if !self.get_by_rect(rect).all(|other_key| other_key == key) {
            return Err(FieldError::Conflict);
        }

        // remove old block
        let (chunk_key, block_key) = self
            .stable_ref
            .get_mut(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get_mut(chunk_key).unwrap();
        let block = chunk.blocks.try_remove(*block_key as usize).unwrap();
        chunk.serial += 1;

        let spec = self.specs.get(block.id as usize).unwrap();

        // spatial features
        let rect = spec.rect(block.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = &rstar::primitives::GeomWithData::new(rect, key);
        self.spatial_ref.remove(node).unwrap();

        // collision features
        if let Some(rect) = spec.collision_rect(block.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.remove(&node).unwrap();
        }

        // hint features
        if let Some(rect) = spec.hint_rect(block.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.hint_ref.remove(&node).unwrap();
        }

        let location = new_block.location;

        // insert new block
        *chunk_key = [
            location[0].div_euclid(self.chunk_size as i32),
            location[1].div_euclid(self.chunk_size as i32),
        ];
        let chunk = self.chunks.entry(*chunk_key).or_default();
        *block_key = chunk.blocks.insert(new_block) as u32;
        chunk.serial += 1;

        // spatial features
        let rect = new_spec.rect(location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = rstar::primitives::GeomWithData::new(rect, key);
        self.spatial_ref.insert(node);

        // collision features
        if let Some(rect) = new_spec.collision_rect(location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.insert(node);
        }

        // hint features
        if let Some(rect) = new_spec.hint_rect(location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.hint_ref.insert(node);
        }

        Ok(block)
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
        let spec = self.specs.get(block.id as usize).unwrap();
        Ok(spec.rect(block.location))
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
        let spec = self.specs.get(block.id as usize).unwrap();
        Ok(spec.collision_rect(block.location).unwrap_or_default())
    }

    #[inline]
    pub fn has_collision_by_point(&self, point: Vec2) -> bool {
        self.collision_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_collision_by_point(&self, point: Vec2) -> impl Iterator<Item = u32> + '_ {
        self.collision_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    #[inline]
    pub fn has_collision_by_rect(&self, rect: [Vec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    #[inline]
    pub fn get_collision_by_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = u32> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }

    // hint features

    #[inline]
    pub fn get_hint_rect(&self, block_key: u32) -> Result<[Vec2; 2], FieldError> {
        let block = self.get(block_key)?;
        let spec = self.specs.get(block.id as usize).unwrap();
        Ok(spec.hint_rect(block.location).unwrap_or_default())
    }

    #[inline]
    pub fn has_hint_by_point(&self, point: Vec2) -> bool {
        self.hint_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_hint_by_point(&self, point: Vec2) -> impl Iterator<Item = u32> + '_ {
        self.hint_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    #[inline]
    pub fn has_hint_by_rect(&self, rect: [Vec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    #[inline]
    pub fn get_hint_by_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = u32> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }
}

// entity field

#[derive(Debug, Clone)]
pub struct EntitySpec {
    collision_size: Vec2,
    collision_offset: Vec2,
    hint_size: Vec2,
    hint_offset: Vec2,
}

impl EntitySpec {
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
    pub serial: u64,
    pub entities: slab::Slab<Entity>,
}

#[derive(Debug, Clone)]
pub struct EntityField {
    chunk_size: u32,
    specs: Vec<EntitySpec>,
    chunks: ahash::AHashMap<IVec2, EntityChunk>,
    stable_ref: slab::Slab<(IVec2, u32)>,
    collision_ref: rstar::RTree<RectNode<Vec2, u32>>,
    hint_ref: rstar::RTree<RectNode<Vec2, u32>>,
}

impl EntityField {
    pub fn new(chunk_size: u32, specs: Vec<EntitySpec>) -> Self {
        Self {
            chunk_size,
            specs,
            chunks: Default::default(),
            stable_ref: Default::default(),
            collision_ref: Default::default(),
            hint_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, entity: Entity) -> Result<u32, FieldError> {
        let location = entity.location;

        let spec = &self
            .specs
            .get(entity.id as usize)
            .ok_or(FieldError::InvalidId)?;

        let chunk_key = [
            location[0].div_euclid(self.chunk_size as f32) as i32,
            location[1].div_euclid(self.chunk_size as f32) as i32,
        ];
        let chunk = self.chunks.entry(chunk_key).or_default();
        let entity_key = chunk.entities.insert(entity) as u32;
        chunk.serial += 1;

        let key = self.stable_ref.insert((chunk_key, entity_key)) as u32;

        // collision features
        if let Some(rect) = spec.collision_rect(location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.insert(node);
        }

        // hint features
        if let Some(rect) = spec.hint_rect(location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.hint_ref.insert(node);
        }

        Ok(key)
    }

    pub fn remove(&mut self, key: u32) -> Result<Entity, FieldError> {
        let (chunk_key, entity_key) = self
            .stable_ref
            .try_remove(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get_mut(&chunk_key).unwrap();
        let entity = chunk.entities.try_remove(entity_key as usize).unwrap();
        chunk.serial += 1;

        let spec = &self.specs.get(entity.id as usize).unwrap();

        // collision features
        if let Some(rect) = spec.collision_rect(entity.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = &rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.remove(node).unwrap();
        }

        // hint features
        if let Some(rect) = spec.hint_rect(entity.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = &rstar::primitives::GeomWithData::new(rect, key);
            self.hint_ref.remove(node).unwrap();
        }

        Ok(entity)
    }

    pub fn modify(&mut self, key: u32, new_entity: Entity) -> Result<Entity, FieldError> {
        let new_spec = &self
            .specs
            .get(new_entity.id as usize)
            .ok_or(FieldError::InvalidId)?;

        // remove old entity
        let (chunk_key, entity_key) = self
            .stable_ref
            .get_mut(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get_mut(chunk_key).unwrap();
        let entity = chunk.entities.try_remove(*entity_key as usize).unwrap();
        chunk.serial += 1;

        let spec = &self.specs.get(entity.id as usize).unwrap();

        // collision features
        if let Some(rect) = spec.collision_rect(entity.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = &rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.remove(node).unwrap();
        }

        // hint features
        if let Some(rect) = spec.hint_rect(entity.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = &rstar::primitives::GeomWithData::new(rect, key);
            self.hint_ref.remove(node).unwrap();
        }

        let location = new_entity.location;

        // insert new entity
        *chunk_key = [
            location[0].div_euclid(self.chunk_size as f32) as i32,
            location[1].div_euclid(self.chunk_size as f32) as i32,
        ];
        let chunk = self.chunks.entry(*chunk_key).or_default();
        *entity_key = chunk.entities.insert(new_entity) as u32;
        chunk.serial += 1;

        // collision features
        if let Some(rect) = new_spec.collision_rect(location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.insert(node);
        }

        // hint features
        if let Some(rect) = new_spec.hint_rect(location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.hint_ref.insert(node);
        }

        Ok(entity)
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
        let spec = self.specs.get(entity.id as usize).unwrap();
        Ok(spec.hint_rect(entity.location).unwrap_or_default())
    }

    #[inline]
    pub fn has_collision_by_point(&self, point: Vec2) -> bool {
        self.collision_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_collision_by_point(&self, point: Vec2) -> impl Iterator<Item = u32> + '_ {
        self.collision_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    #[inline]
    pub fn has_collision_by_rect(&self, rect: [Vec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    #[inline]
    pub fn get_collision_by_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = u32> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }

    // hint features

    #[inline]
    pub fn get_hint_rect(&self, entity_key: u32) -> Result<[Vec2; 2], FieldError> {
        let entity = self.get(entity_key)?;
        let spec = self.specs.get(entity.id as usize).unwrap();
        Ok(spec.hint_rect(entity.location).unwrap_or_default())
    }

    #[inline]
    pub fn has_hint_by_point(&self, point: Vec2) -> bool {
        self.hint_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_hint_by_point(&self, point: Vec2) -> impl Iterator<Item = u32> + '_ {
        self.hint_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    #[inline]
    pub fn has_hint_by_rect(&self, rect: [Vec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    #[inline]
    pub fn get_hint_by_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = u32> + '_ {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }
}

// utility functions

pub fn move_entity(
    tile_field: &TileField,
    block_field: &BlockField,
    entity_field: &mut EntityField,
    entity_key: u32,
    new_location: Vec2,
) -> Result<(), FieldError> {
    let entity = entity_field.get(entity_key)?;

    let spec = &entity_field.specs.get(entity.id as usize).unwrap();

    if let Some(rect) = spec.collision_rect(new_location) {
        if tile_field.has_collision_by_rect(rect) {
            return Err(FieldError::Conflict);
        }
        if block_field.has_collision_by_rect(rect) {
            return Err(FieldError::Conflict);
        }
    }

    let mut new_entity = entity.clone();
    new_entity.location = new_location;
    entity_field.modify(entity_key, new_entity)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud_tile() {
        let mut field = TileField::new(16, vec![TileSpec::new(true), TileSpec::new(true)]);
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
        let mut field = TileField::new(16, vec![TileSpec::new(true), TileSpec::new(true)]);

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
        let mut field = TileField::new(16, vec![TileSpec::new(true), TileSpec::new(true)]);
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
        let mut field = TileField::new(16, vec![TileSpec::new(true), TileSpec::new(true)]);
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
        let mut field = TileField::new(16, vec![TileSpec::new(false), TileSpec::new(true)]);
        let key = field.insert(Tile::new(0, [-1, 3], 0)).unwrap();

        let point = [-1.0, 3.0];
        assert!(!field.has_collision_by_point(point));
        let vec = field.get_collision_by_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![]);

        assert_eq!(
            field.modify(key, Tile::new(1, [-1, 3], 0)),
            Ok(Tile::new(0, [-1, 3], 0))
        );
        assert_eq!(field.get(key), Ok(&Tile::new(1, [-1, 3], 0)));
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key));

        let point = [-1.0, 3.0];
        assert!(field.has_collision_by_point(point));
        let vec = field.get_collision_by_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![key]);

        assert_eq!(
            field.modify(key, Tile::new(0, [-1, 3], 0)),
            Ok(Tile::new(1, [-1, 3], 0))
        );
        assert_eq!(field.get(key), Ok(&Tile::new(0, [-1, 3], 0)));
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key));

        let point = [-1.0, 3.0];
        assert!(!field.has_collision_by_point(point));
        let vec = field.get_collision_by_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![]);
    }

    #[test]
    fn collision_tile() {
        let mut field = TileField::new(16, vec![TileSpec::new(true), TileSpec::new(true)]);
        let key_0 = field.insert(Tile::new(1, [-1, 3], 0)).unwrap();
        let key_1 = field.insert(Tile::new(1, [-1, 4], 0)).unwrap();
        let _key_2 = field.insert(Tile::new(1, [-1, 5], 0)).unwrap();

        let point = [-1.0, 4.0];
        assert!(field.has_collision_by_point(point));
        let vec = field.get_collision_by_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        let rect = [[-1.0, 3.0], [-1.0, 4.0]];
        assert!(field.has_collision_by_rect(rect));
        let vec = field.get_collision_by_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));
    }

    #[test]
    fn crud_block() {
        let specs = vec![
            BlockSpec::new([1, 1], [1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
            BlockSpec::new([1, 1], [1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
        ];
        let mut field = BlockField::new(16, specs);
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
        let specs = vec![
            BlockSpec::new([1, 1], [1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
            BlockSpec::new([1, 1], [1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
        ];
        let mut field = BlockField::new(16, specs);

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
        let specs = vec![
            BlockSpec::new([1, 1], [1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
            BlockSpec::new([1, 1], [1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
        ];
        let mut field = BlockField::new(16, specs);
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
        let specs = vec![
            BlockSpec::new([1, 1], [1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
            BlockSpec::new([1, 1], [1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
        ];
        let mut field = BlockField::new(16, specs);
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
        let specs = vec![
            BlockSpec::new([1, 1], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]),
            BlockSpec::new([1, 1], [1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
        ];
        let mut field = BlockField::new(16, specs);
        let key = field.insert(Block::new(0, [-1, 3], 0)).unwrap();

        let point = [-1.0, 3.0];
        assert!(!field.has_collision_by_point(point));
        assert!(!field.has_hint_by_point(point));
        let vec = field.get_collision_by_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![]);
        let vec = field.get_hint_by_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![]);

        assert_eq!(
            field.modify(key, Block::new(1, [-1, 3], 0)),
            Ok(Block::new(0, [-1, 3], 0))
        );
        assert_eq!(field.get(key), Ok(&Block::new(1, [-1, 3], 0)));
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key));

        let point = [-1.0, 3.0];
        assert!(field.has_collision_by_point(point));
        assert!(field.has_hint_by_point(point));
        let vec = field.get_collision_by_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![key]);
        let vec = field.get_hint_by_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![key]);

        assert_eq!(
            field.modify(key, Block::new(0, [-1, 3], 0)),
            Ok(Block::new(1, [-1, 3], 0))
        );
        assert_eq!(field.get(key), Ok(&Block::new(0, [-1, 3], 0)));
        assert!(field.has_by_point([-1, 3]));
        assert_eq!(field.get_by_point([-1, 3]), Some(key));

        let point = [-1.0, 3.0];
        assert!(!field.has_collision_by_point(point));
        assert!(!field.has_hint_by_point(point));
        let vec = field.get_collision_by_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![]);
        let vec = field.get_hint_by_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![]);
    }

    #[test]
    fn collision_block() {
        let specs = vec![
            BlockSpec::new([1, 1], [1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
            BlockSpec::new([1, 1], [1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
        ];
        let mut field = BlockField::new(16, specs);
        let key_0 = field.insert(Block::new(1, [-1, 3], 0)).unwrap();
        let key_1 = field.insert(Block::new(1, [-1, 4], 0)).unwrap();
        let _key_2 = field.insert(Block::new(1, [-1, 5], 0)).unwrap();

        let point = [-1.0, 4.0];
        assert!(field.has_collision_by_point(point));
        let vec = field.get_collision_by_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        let rect = [[-1.0, 3.0], [-1.0, 4.0]];
        assert!(field.has_collision_by_rect(rect));
        let vec = field.get_collision_by_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));
    }

    #[test]
    fn hint_block() {
        let specs = vec![
            BlockSpec::new([1, 1], [1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
            BlockSpec::new([1, 1], [1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
        ];
        let mut field = BlockField::new(16, specs);
        let key_0 = field.insert(Block::new(1, [-1, 3], 0)).unwrap();
        let key_1 = field.insert(Block::new(1, [-1, 4], 0)).unwrap();
        let _key_2 = field.insert(Block::new(1, [-1, 5], 0)).unwrap();

        let point = [-1.0, 4.0];
        assert!(field.has_hint_by_point(point));
        let vec = field.get_hint_by_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        let rect = [[-1.0, 3.0], [-1.0, 4.0]];
        assert!(field.has_hint_by_rect(rect));
        let vec = field.get_hint_by_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));
    }

    #[test]
    fn crud_entity() {
        let specs = vec![
            EntitySpec::new([1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
            EntitySpec::new([1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
        ];
        let mut field = EntityField::new(16, specs);
        let key = field.insert(Entity::new(1, [-1.0, 3.0], 0)).unwrap();

        assert_eq!(field.get(key), Ok(&Entity::new(1, [-1.0, 3.0], 0)));
        assert_eq!(field.remove(key), Ok(Entity::new(1, [-1.0, 3.0], 0)));

        assert_eq!(field.get(key), Err(FieldError::NotFound));
        assert_eq!(field.remove(key), Err(FieldError::NotFound));
    }

    #[test]
    fn insert_entity_with_invalid() {
        let specs = vec![
            EntitySpec::new([1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
            EntitySpec::new([1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
        ];
        let mut field = EntityField::new(16, specs);

        assert_eq!(
            field.insert(Entity::new(2, [-1.0, 3.0], 0)),
            Err(FieldError::InvalidId)
        );
    }

    #[test]
    fn modify_entity() {
        let specs = vec![
            EntitySpec::new([1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
            EntitySpec::new([1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
        ];
        let mut field = EntityField::new(16, specs);
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
        let specs = vec![
            EntitySpec::new([1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
            EntitySpec::new([1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
        ];
        let mut field = EntityField::new(16, specs);
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
        let specs = vec![
            EntitySpec::new([0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]),
            EntitySpec::new([1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
        ];
        let mut field = EntityField::new(16, specs);
        let key = field.insert(Entity::new(0, [-1.0, 3.0], 0)).unwrap();

        let point = [-1.0, 3.0];
        assert!(!field.has_collision_by_point(point));
        assert!(!field.has_hint_by_point(point));
        let vec = field.get_collision_by_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![]);
        let vec = field.get_hint_by_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![]);

        assert_eq!(
            field.modify(key, Entity::new(1, [-1.0, 3.0], 0)),
            Ok(Entity::new(0, [-1.0, 3.0], 0))
        );
        assert_eq!(field.get(key), Ok(&Entity::new(1, [-1.0, 3.0], 0)));

        let point = [-1.0, 3.0];
        assert!(field.has_collision_by_point(point));
        assert!(field.has_hint_by_point(point));
        let vec = field.get_collision_by_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![key]);
        let vec = field.get_hint_by_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![key]);

        assert_eq!(
            field.modify(key, Entity::new(0, [-1.0, 3.0], 0)),
            Ok(Entity::new(1, [-1.0, 3.0], 0))
        );
        assert_eq!(field.get(key), Ok(&Entity::new(0, [-1.0, 3.0], 0)));

        let point = [-1.0, 3.0];
        assert!(!field.has_collision_by_point(point));
        assert!(!field.has_hint_by_point(point));
        let vec = field.get_collision_by_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![]);
        let vec = field.get_hint_by_point(point).collect::<Vec<_>>();
        assert_eq!(vec, vec![]);
    }

    #[test]
    fn collision_entity() {
        let specs = vec![
            EntitySpec::new([1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
            EntitySpec::new([1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
        ];
        let mut field = EntityField::new(16, specs);
        let key_0 = field.insert(Entity::new(1, [-1.0, 3.0], 0)).unwrap();
        let key_1 = field.insert(Entity::new(1, [-1.0, 4.0], 0)).unwrap();
        let _key_2 = field.insert(Entity::new(1, [-1.0, 5.0], 0)).unwrap();

        let point = [-1.0, 4.0];
        assert!(field.has_collision_by_point(point));
        let vec = field.get_collision_by_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        let rect = [[-1.0, 3.0], [-1.0, 4.0]];
        assert!(field.has_collision_by_rect(rect));
        let vec = field.get_collision_by_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));
    }

    #[test]
    fn hint_entity() {
        let specs = vec![
            EntitySpec::new([1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
            EntitySpec::new([1.0, 1.0], [0.0, 0.0], [1.0, 1.0], [0.0, 0.0]),
        ];
        let mut field = EntityField::new(16, specs);
        let key_0 = field.insert(Entity::new(1, [-1.0, 3.0], 0)).unwrap();
        let key_1 = field.insert(Entity::new(1, [-1.0, 4.0], 0)).unwrap();
        let _key_2 = field.insert(Entity::new(1, [-1.0, 5.0], 0)).unwrap();

        let point = [-1.0, 4.0];
        assert!(field.has_hint_by_point(point));
        let vec = field.get_hint_by_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        let rect = [[-1.0, 3.0], [-1.0, 4.0]];
        assert!(field.has_hint_by_rect(rect));
        let vec = field.get_hint_by_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));
    }
}
