pub type Vec2 = [f32; 2];
pub type IVec2 = [i32; 2];

type RectNode<T, U> = rstar::primitives::GeomWithData<rstar::primitives::Rectangle<T>, U>;

// Tile Field

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
    pub serial: u64,
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

    pub fn insert(&mut self, tile: Tile) -> Result<u32, FieldError> {
        let location = tile.location;

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
        let ukey = UnstableTileKey {
            chunk_key,
            tile_key,
        };

        let key = self.stable_ref.insert(ukey) as u32;

        // spatial features
        self.spatial_ref.insert(location, key);

        Ok(key)
    }

    pub fn remove(&mut self, key: u32) -> Result<Tile, FieldError> {
        let ukey = self
            .stable_ref
            .try_remove(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get_mut(&ukey.chunk_key).check();
        let tile = chunk.tiles.try_remove(ukey.tile_key as usize).check();
        chunk.serial += 1;

        // spatial features
        self.spatial_ref.remove(&tile.location).check();

        Ok(tile)
    }

    pub fn modify(&mut self, key: u32, new_tile: Tile) -> Result<Tile, FieldError> {
        // validate modification

        if !self
            .spatial_ref
            .get(&new_tile.location)
            .map_or(true, |other_key| *other_key == key)
        {
            return Err(FieldError::Conflict);
        }

        // remove old tile

        let ukey = self
            .stable_ref
            .get_mut(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get_mut(&ukey.chunk_key).check();
        let tile = chunk.tiles.try_remove(ukey.tile_key as usize).check();
        chunk.serial += 1;

        // spatial features
        self.spatial_ref.remove(&tile.location).check();

        // insert new tile

        let location = new_tile.location;

        let chunk_key = [
            location[0].div_euclid(self.chunk_size as i32),
            location[1].div_euclid(self.chunk_size as i32),
        ];
        let chunk = self.chunks.entry(chunk_key).or_default();
        let tile_key = chunk.tiles.insert(new_tile) as u32;
        chunk.serial += 1;
        *ukey = UnstableTileKey {
            chunk_key,
            tile_key,
        };

        // spatial features
        self.spatial_ref.insert(location, key);

        Ok(tile)
    }

    pub fn get(&self, key: u32) -> Result<&Tile, FieldError> {
        let ukey = self
            .stable_ref
            .get(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get(&ukey.chunk_key).check();
        let tile = chunk.tiles.get(ukey.tile_key as usize).check();
        Ok(tile)
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

// Block Field

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
        if size[0] * size[1] <= 0 {
            panic!("size must be positive");
        }
        if collision_size[0] * collision_size[1] < 0.0 {
            panic!("collision size must be non-negative");
        }
        if hint_size[0] * hint_size[1] < 0.0 {
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
    pub serial: u64,
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
        let ukey = UnstableBlockKey {
            chunk_key,
            block_key,
        };

        let key = self.stable_ref.insert(ukey) as u32;

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
        let ukey = self
            .stable_ref
            .try_remove(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get_mut(&ukey.chunk_key).check();
        let block = chunk.blocks.try_remove(ukey.block_key as usize).check();
        chunk.serial += 1;

        let spec = &self.specs.get(block.id as usize).check();

        // spatial features
        let rect = spec.rect(block.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = rstar::primitives::GeomWithData::new(rect, key);
        self.spatial_ref.remove(&node).check();

        // collision features
        if let Some(rect) = spec.collision_rect(block.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.remove(&node).check();
        }

        // hint features
        if let Some(rect) = spec.hint_rect(block.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.hint_ref.remove(&node).check();
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

        let ukey = self
            .stable_ref
            .get_mut(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get_mut(&ukey.chunk_key).check();
        let block = chunk.blocks.try_remove(ukey.block_key as usize).check();
        chunk.serial += 1;

        let spec = self.specs.get(block.id as usize).check();

        // spatial features
        let rect = spec.rect(block.location);
        let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
        let node = &rstar::primitives::GeomWithData::new(rect, key);
        self.spatial_ref.remove(node).check();

        // collision features
        if let Some(rect) = spec.collision_rect(block.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.remove(&node).check();
        }

        // hint features
        if let Some(rect) = spec.hint_rect(block.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.hint_ref.remove(&node).check();
        }

        // insert new block

        let location = new_block.location;

        let chunk_key = [
            location[0].div_euclid(self.chunk_size as i32),
            location[1].div_euclid(self.chunk_size as i32),
        ];
        let chunk = self.chunks.entry(chunk_key).or_default();
        let block_key = chunk.blocks.insert(new_block) as u32;
        chunk.serial += 1;
        *ukey = UnstableBlockKey {
            chunk_key,
            block_key,
        };

        // spatial features
        let rect = new_spec.rect(location);
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

        Ok(block)
    }

    pub fn get(&self, key: u32) -> Result<&Block, FieldError> {
        let ukey = self
            .stable_ref
            .get(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get(&ukey.chunk_key).check();
        let block = chunk.blocks.get(ukey.block_key as usize).check();
        Ok(block)
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

// Entity Field

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
        if collision_size[0] * collision_size[1] < 0.0 {
            panic!("collision size must be non-negative");
        }
        if hint_size[0] * hint_size[1] < 0.0 {
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
    pub serial: u64,
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
        let ukey = UnstableEntityKey {
            chunk_key,
            entity_key,
        };

        let key = self.stable_ref.insert(ukey) as u32;

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
        let ukey = self
            .stable_ref
            .try_remove(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get_mut(&ukey.chunk_key).check();
        let entity = chunk.entities.try_remove(ukey.entity_key as usize).check();
        chunk.serial += 1;

        let spec = &self.specs.get(entity.id as usize).check();

        // collision features
        if let Some(rect) = spec.collision_rect(entity.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = &rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.remove(node).check();
        }

        // hint features
        if let Some(rect) = spec.hint_rect(entity.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = &rstar::primitives::GeomWithData::new(rect, key);
            self.hint_ref.remove(node).check();
        }

        Ok(entity)
    }

    pub fn modify(&mut self, key: u32, new_entity: Entity) -> Result<Entity, FieldError> {
        let new_spec = &self
            .specs
            .get(new_entity.id as usize)
            .ok_or(FieldError::InvalidId)?;

        // remove old entity

        let ukey = self
            .stable_ref
            .get_mut(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get_mut(&ukey.chunk_key).check();
        let entity = chunk.entities.try_remove(ukey.entity_key as usize).check();
        chunk.serial += 1;

        let spec = &self.specs.get(entity.id as usize).check();

        // collision features
        if let Some(rect) = spec.collision_rect(entity.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = &rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.remove(node).check();
        }

        // hint features
        if let Some(rect) = spec.hint_rect(entity.location) {
            let rect = rstar::primitives::Rectangle::from_corners(rect[0], rect[1]);
            let node = &rstar::primitives::GeomWithData::new(rect, key);
            self.hint_ref.remove(node).check();
        }

        // insert new entity

        let location = new_entity.location;

        let chunk_key = [
            location[0].div_euclid(self.chunk_size as f32) as i32,
            location[1].div_euclid(self.chunk_size as f32) as i32,
        ];
        let chunk = self.chunks.entry(chunk_key).or_default();
        let entity_key = chunk.entities.insert(new_entity) as u32;
        chunk.serial += 1;
        *ukey = UnstableEntityKey {
            chunk_key,
            entity_key,
        };

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
        let ukey = self
            .stable_ref
            .get(key as usize)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.chunks.get(&ukey.chunk_key).check();
        let entity = chunk.entities.get(ukey.entity_key as usize).check();
        Ok(entity)
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

// Field Extra

fn move_entity(
    block_field: &BlockField,
    entity_field: &mut EntityField,
    entity_key: u32,
    new_location: Vec2,
) -> Result<(), FieldError> {
    let entity = entity_field.get(entity_key)?;
    let spec = &entity_field.specs.get(entity.id as usize).check();

    if let Some(rect) = spec.collision_rect(new_location) {
        if block_field.has_collision_by_rect(rect) {
            return Err(FieldError::Conflict);
        }
    }

    let mut new_entity = entity.clone();
    new_entity.location = new_location;
    entity_field.modify(entity_key, new_entity)?;

    Ok(())
}

// Agent Plugin

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FieldKey {
    Global,
    Tile(u32),
    Block(u32),
    Entity(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum UpdateKey {
    None,
    RealTime,
    RoundRobin,
}

#[enum_dispatch::enum_dispatch]
trait IAgent {
    fn field_key(&self) -> FieldKey;

    fn update_key(&self) -> UpdateKey;

    fn update(
        &mut self,
        _tile_field: &mut TileField,
        _block_field: &mut BlockField,
        _entity_field: &mut EntityField,
        _delta_secs: f32,
    ) {
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
#[enum_dispatch::enum_dispatch(IAgent)]
pub enum Agent {
    Empty(Empty),
    RandomWalk(RandomWalk),
    Timestamp(Timestamp),
}

#[derive(Debug, Clone)]
struct AgentMeta {
    inner: Agent,
    inv_field_key: u32,
    inv_update_key: u32,
}

#[derive(Debug, Clone)]
pub struct AgentPlugin {
    metas: slab::Slab<AgentMeta>,
    field_ref: ahash::AHashMap<FieldKey, slab::Slab<u32>>,
    rt_ref: slab::Slab<u32>,
    rr_ref: slab::Slab<u32>,
    rr_stack: Vec<u32>,
}

impl AgentPlugin {
    pub fn new() -> Self {
        Self {
            metas: Default::default(),
            field_ref: Default::default(),
            rt_ref: Default::default(),
            rr_ref: Default::default(),
            rr_stack: Default::default(),
        }
    }

    pub fn insert(
        &mut self,
        tile_field: &TileField,
        block_field: &BlockField,
        entity_field: &EntityField,
        agent: Agent,
    ) -> Result<u32, FieldError> {
        match agent.field_key() {
            FieldKey::Tile(key) if tile_field.get(key).is_err() => {
                return Err(FieldError::NotFound);
            }
            FieldKey::Block(key) if block_field.get(key).is_err() => {
                return Err(FieldError::NotFound);
            }
            FieldKey::Entity(key) if entity_field.get(key).is_err() => {
                return Err(FieldError::NotFound);
            }
            _ => {}
        }

        let key = self.metas.vacant_key() as u32;

        let field_keys = self.field_ref.entry(agent.field_key()).or_default();
        let inv_field_key = field_keys.insert(key) as u32;

        let inv_update_key = match agent.update_key() {
            UpdateKey::None => Default::default(),
            UpdateKey::RealTime => self.rt_ref.insert(key) as u32,
            UpdateKey::RoundRobin => self.rr_ref.insert(key) as u32,
        };

        let meta = AgentMeta {
            inner: agent,
            inv_field_key,
            inv_update_key,
        };
        self.metas.insert(meta);
        Ok(key)
    }

    pub fn remove(&mut self, key: u32) -> Result<Agent, FieldError> {
        let meta = self
            .metas
            .try_remove(key as usize)
            .ok_or(FieldError::NotFound)?;

        let field_keys = self.field_ref.get_mut(&meta.inner.field_key()).check();
        field_keys.try_remove(meta.inv_field_key as usize).check();

        match meta.inner.update_key() {
            UpdateKey::None => {
                // NOOP
            }
            UpdateKey::RealTime => {
                self.rt_ref.try_remove(meta.inv_update_key as usize).check();
            }
            UpdateKey::RoundRobin => {
                self.rr_ref.try_remove(meta.inv_update_key as usize).check();
            }
        }

        Ok(meta.inner)
    }

    pub fn get(&self, key: u32) -> Result<&Agent, FieldError> {
        let meta = self.metas.get(key as usize).ok_or(FieldError::NotFound)?;
        Ok(&meta.inner)
    }

    pub fn get_by_global(&self) -> impl Iterator<Item = u32> + '_ {
        self.field_ref
            .get(&FieldKey::Global)
            .map(|field_keys| field_keys.iter().map(|(_, key)| *key))
            .into_iter()
            .flatten()
    }

    pub fn get_by_tile(&self, tile_key: u32) -> impl Iterator<Item = u32> + '_ {
        self.field_ref
            .get(&FieldKey::Tile(tile_key))
            .map(|field_keys| field_keys.iter().map(|(_, key)| *key))
            .into_iter()
            .flatten()
    }

    pub fn get_by_block(&self, block_key: u32) -> impl Iterator<Item = u32> + '_ {
        self.field_ref
            .get(&FieldKey::Block(block_key))
            .map(|field_keys| field_keys.iter().map(|(_, key)| *key))
            .into_iter()
            .flatten()
    }

    pub fn get_by_entity(&self, entity_key: u32) -> impl Iterator<Item = u32> + '_ {
        self.field_ref
            .get(&FieldKey::Entity(entity_key))
            .map(|field_keys| field_keys.iter().map(|(_, key)| *key))
            .into_iter()
            .flatten()
    }

    pub fn update(
        &mut self,
        tile_field: &mut TileField,
        block_field: &mut BlockField,
        entity_field: &mut EntityField,
        delta_secs: f32,
    ) {
        // real time
        for (_, key) in self.rt_ref.iter() {
            let meta = self.metas.get_mut(*key as usize).check();
            meta.inner
                .update(tile_field, block_field, entity_field, delta_secs);
        }

        // round robin
        if !self.rr_ref.is_empty() {
            if self.rr_stack.is_empty() {
                self.rr_ref
                    .iter()
                    .map(|(key, _)| key as u32)
                    .for_each(|key| self.rr_stack.push(key));
            }

            let rr_key = self.rr_stack.pop().check();
            if let Some(key) = self.rr_ref.get(rr_key as usize) {
                let meta = self.metas.get_mut(*key as usize).check();
                let agent = &mut meta.inner;
                agent.update(tile_field, block_field, entity_field, delta_secs);
            }
        }
    }
}

// Agent Plugin Extra

#[derive(Debug, Clone)]
pub struct Empty;

impl Empty {
    pub fn new() -> Self {
        Self
    }
}

impl IAgent for Empty {
    fn field_key(&self) -> FieldKey {
        FieldKey::Global
    }

    fn update_key(&self) -> UpdateKey {
        UpdateKey::None
    }
}

#[derive(Debug, Clone)]
enum RandomWalkState {
    Init,
    WaitStart,
    Wait(f32),
    TripStart,
    Trip(Vec2),
}

#[derive(Debug, Clone)]
pub struct RandomWalk {
    entity_key: u32,
    min_rest_secs: f32,
    max_rest_secs: f32,
    min_distance: f32,
    max_distance: f32,
    speed: f32,
    state: RandomWalkState,
}

impl RandomWalk {
    pub fn new(
        entity_key: u32,
        min_rest_secs: f32,
        max_rest_secs: f32,
        min_distance: f32,
        max_distance: f32,
        speed: f32,
    ) -> Self {
        Self {
            entity_key,
            min_rest_secs,
            max_rest_secs,
            min_distance,
            max_distance,
            speed,
            state: RandomWalkState::Init,
        }
    }
}

impl IAgent for RandomWalk {
    fn field_key(&self) -> FieldKey {
        FieldKey::Entity(self.entity_key)
    }

    fn update_key(&self) -> UpdateKey {
        UpdateKey::RealTime
    }

    fn update(
        &mut self,
        _tile_field: &mut TileField,
        block_field: &mut BlockField,
        entity_field: &mut EntityField,
        delta_secs: f32,
    ) {
        use rand::Rng;

        match self.state {
            RandomWalkState::Init => {
                self.state = RandomWalkState::WaitStart;
            }
            RandomWalkState::WaitStart => {
                let secs = rand::thread_rng().gen_range(self.min_rest_secs..self.max_rest_secs);
                self.state = RandomWalkState::Wait(secs);
            }
            RandomWalkState::Wait(secs) => {
                let new_secs = secs - delta_secs;
                if new_secs <= 0.0 {
                    self.state = RandomWalkState::TripStart;
                } else {
                    self.state = RandomWalkState::Wait(new_secs);
                }
            }
            RandomWalkState::TripStart => {
                let entity = entity_field.get(self.entity_key).check();

                let distance = rand::thread_rng().gen_range(self.min_distance..self.max_distance);
                let direction = rand::thread_rng().gen_range(0.0..std::f32::consts::PI * 2.0);
                let destination = [
                    entity.location[0] + distance * direction.cos(),
                    entity.location[1] + distance * direction.sin(),
                ];

                self.state = RandomWalkState::Trip(destination);
            }
            RandomWalkState::Trip(destination) => {
                let entity = entity_field.get(self.entity_key).check();

                if entity.location == destination {
                    self.state = RandomWalkState::WaitStart;
                    return;
                }

                let diff = [
                    destination[0] - entity.location[0],
                    destination[1] - entity.location[1],
                ];
                let distance = (diff[0].powi(2) + diff[1].powi(2)).sqrt();
                let direction = [diff[0] / distance, diff[1] / distance];
                let delta_distance = distance.min(self.speed * delta_secs);
                let location = [
                    entity.location[0] + direction[0] * delta_distance,
                    entity.location[1] + direction[1] * delta_distance,
                ];

                if move_entity(block_field, entity_field, self.entity_key, location).is_ok() {
                    self.state = RandomWalkState::Trip(destination);
                } else {
                    self.state = RandomWalkState::WaitStart;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Timestamp;

impl Timestamp {
    pub fn new() -> Self {
        Self
    }
}

impl IAgent for Timestamp {
    fn field_key(&self) -> FieldKey {
        FieldKey::Global
    }

    fn update_key(&self) -> UpdateKey {
        UpdateKey::RoundRobin
    }

    fn update(
        &mut self,
        _tile_field: &mut TileField,
        _block_field: &mut BlockField,
        _entity_field: &mut EntityField,
        _delta_secs: f32,
    ) {
        godot::log::godot_print!("{:?}", std::time::SystemTime::now())
    }
}

// Error Handling

trait IntegrityCheck<T> {
    fn check(self) -> T;
}

impl<T> IntegrityCheck<T> for Option<T> {
    fn check(self) -> T {
        self.expect("integrity error")
    }
}

impl<T, U: std::error::Error> IntegrityCheck<T> for Result<T, U> {
    fn check(self) -> T {
        self.expect("integrity error")
    }
}

#[derive(Debug, Clone)]
pub enum FieldError {
    NotFound,
    Conflict,
    InvalidId,
}

impl std::fmt::Display for FieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldError::NotFound => write!(f, "not found error"),
            FieldError::Conflict => write!(f, "conflict error"),
            FieldError::InvalidId => write!(f, "invalid id error"),
        }
    }
}

impl std::error::Error for FieldError {}
