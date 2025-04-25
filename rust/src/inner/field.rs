use super::*;

type RectNode<T, U> = rstar::primitives::GeomWithData<rstar::primitives::Rectangle<T>, U>;

// tile field

pub type TileKey = (u32, u32);

#[derive(Debug, Clone)]
pub struct TileDescriptor {
    pub name_text: String,
    pub desc_text: String,
    pub collision: bool,
}

#[derive(Debug, Clone)]
pub struct TileFieldDescriptor {
    pub tiles: Vec<TileDescriptor>,
}

#[derive(Debug, Clone)]
struct TileProperty {
    name_text: String,
    desc_text: String,
    collision: bool,
}

impl TileProperty {
    #[rustfmt::skip]
    fn collision_rect(&self, location: IVec2) -> Option<[Vec2; 2]> {
        if !self.collision {
            return None;
        }

        Some([
            location.as_vec2(),
            location.as_vec2() + 1.0,
        ])
    }
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub id: u16,
    pub location: IVec2,
    pub data: Box<dyn TileData>,
    pub render_param: TileRenderParam,
}

#[derive(Debug, Clone)]
pub struct TileChunk {
    pub version: u64,
    pub tiles: slab::Slab<Tile>,
}

#[derive(Debug, Clone)]
pub struct TileField {
    props: Vec<TileProperty>,
    chunks: Vec<TileChunk>,
    chunk_ref: ahash::AHashMap<IVec2, u32>,
    spatial_ref: ahash::AHashMap<IVec2, TileKey>,
    collision_ref: rstar::RTree<RectNode<[f32; 2], TileKey>>,
}

impl TileField {
    const CHUNK_SIZE: u32 = 32;

    pub fn new(desc: TileFieldDescriptor) -> Self {
        let mut props = vec![];
        for tile in desc.tiles {
            props.push(TileProperty {
                name_text: tile.name_text,
                desc_text: tile.desc_text,
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

    pub fn insert(&mut self, tile: Tile) -> Result<TileKey, FieldError> {
        let prop = self
            .props
            .get(tile.id as usize)
            .ok_or(FieldError::InvalidId)?;

        // check by spatial features
        if self.has_by_point(tile.location) {
            return Err(FieldError::Conflict);
        }

        let chunk_size = IVec2::splat(Self::CHUNK_SIZE as i32);
        let chunk_location = tile.location.div_euclid(chunk_size);

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
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
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
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.remove(&node).unwrap();
        }

        Ok(tile)
    }

    pub fn modify(
        &mut self,
        key: TileKey,
        f: impl FnOnce(&mut Tile),
    ) -> Result<TileKey, FieldError> {
        let (chunk_key, local_key) = key;

        let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
        let tile = chunk
            .tiles
            .get_mut(local_key as usize)
            .ok_or(FieldError::NotFound)?;

        let mut new_tile = Tile {
            id: tile.id,
            location: tile.location,
            data: std::mem::take(&mut tile.data),
            render_param: tile.render_param.clone(),
        };
        f(&mut new_tile);

        if new_tile.id != tile.id {
            tile.data = new_tile.data;
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

        if new_tile.render_param != tile.render_param {
            let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
            *chunk.tiles.get_mut(local_key as usize).unwrap() = new_tile;
            chunk.version += 1;
            return Ok(key);
        }

        tile.data = new_tile.data;
        Ok(key)
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
        Self::CHUNK_SIZE
    }

    #[inline]
    pub fn get_chunk(&self, chunk_key: u32) -> Result<&TileChunk, FieldError> {
        self.chunks
            .get(chunk_key as usize)
            .ok_or(FieldError::NotFound)
    }

    #[inline]
    pub fn get_name_text(&self, key: TileKey) -> Result<&str, FieldError> {
        let tile = self.get(key)?;
        let prop = self.props.get(tile.id as usize).unwrap();
        Ok(&prop.name_text)
    }

    #[inline]
    pub fn get_desc_text(&self, key: TileKey) -> Result<&str, FieldError> {
        let tile = self.get(key)?;
        let prop = self.props.get(tile.id as usize).unwrap();
        Ok(&prop.desc_text)
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
        let point = [point.x, point.y];
        self.collision_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = TileKey> + '_ {
        let point = [point.x, point.y];
        self.collision_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    #[inline]
    pub fn has_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    #[inline]
    pub fn get_by_collision_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = TileKey> + '_ {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }
}

// block field

pub type BlockKey = (u32, u32);

#[derive(Debug, Clone)]
pub struct BlockDescriptor {
    pub name_text: String,
    pub desc_text: String,
    pub size: IVec2,
    pub collision_size: Vec2,
    pub collision_offset: Vec2,
    pub hint_size: Vec2,
    pub hint_offset: Vec2,
    pub z_along_y: bool,
}

#[derive(Debug, Clone)]
pub struct BlockFieldDescriptor {
    pub blocks: Vec<BlockDescriptor>,
}

#[derive(Debug, Clone)]
struct BlockProperty {
    name_text: String,
    desc_text: String,
    size: IVec2,
    collision_size: Vec2,
    collision_offset: Vec2,
    hint_size: Vec2,
    hint_offset: Vec2,
    z_along_y: bool,
}

impl BlockProperty {
    #[rustfmt::skip]
    fn rect(&self, location: IVec2) -> [IVec2; 2] {
        [
            location,
            location + self.size[1] - 1,
        ]
    }

    #[rustfmt::skip]
    fn collision_rect(&self, location: IVec2) -> Option<[Vec2; 2]> {
        if self.collision_size[0] * self.collision_size[1] == 0.0 {
            return None;
        }

        Some([
            location.as_vec2() + self.collision_offset,
            location.as_vec2() + self.collision_offset + self.collision_size,
        ])
    }

    #[rustfmt::skip]
    fn hint_rect(&self, location: IVec2) -> Option<[Vec2; 2]> {
        if self.hint_size[0] * self.hint_size[1] == 0.0 {
            return None;
        }

        Some([
            location.as_vec2() + self.hint_offset,
            location.as_vec2() + self.hint_offset + self.hint_size,
        ])
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub id: u16,
    pub location: IVec2,
    pub data: Box<dyn BlockData>,
    pub render_param: BlockRenderParam,
}

#[derive(Debug, Clone)]
pub struct BlockChunk {
    pub version: u64,
    pub blocks: slab::Slab<Block>,
}

#[derive(Debug, Clone)]
pub struct BlockField {
    props: Vec<BlockProperty>,
    chunks: Vec<BlockChunk>,
    chunk_ref: ahash::AHashMap<IVec2, u32>,
    spatial_ref: rstar::RTree<RectNode<[i32; 2], BlockKey>>,
    collision_ref: rstar::RTree<RectNode<[f32; 2], BlockKey>>,
    hint_ref: rstar::RTree<RectNode<[f32; 2], BlockKey>>,
}

impl BlockField {
    const CHUNK_SIZE: u32 = 32;

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
                name_text: block.name_text,
                desc_text: block.desc_text,
                size: block.size,
                collision_size: block.collision_size,
                collision_offset: block.collision_offset,
                hint_size: block.hint_size,
                hint_offset: block.hint_offset,
                z_along_y: block.z_along_y,
            });
        }

        Self {
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

        let chunk_size = IVec2::splat(Self::CHUNK_SIZE as i32);
        let chunk_location = block.location.div_euclid(chunk_size);

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
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
        let node = rstar::primitives::GeomWithData::new(rect, (chunk_key, local_key));
        self.spatial_ref.insert(node);

        // collision features
        if let Some(rect) = prop.collision_rect(block.location) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = rstar::primitives::GeomWithData::new(rect, (chunk_key, local_key));
            self.collision_ref.insert(node);
        }

        // hint features
        if let Some(rect) = prop.hint_rect(block.location) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
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
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
        let node = rstar::primitives::GeomWithData::new(rect, key);
        self.spatial_ref.remove(&node).unwrap();

        // collision features
        if let Some(rect) = prop.collision_rect(block.location) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.remove(&node).unwrap();
        }

        // hint features
        if let Some(rect) = prop.hint_rect(block.location) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.hint_ref.remove(&node).unwrap();
        }

        Ok(block)
    }

    pub fn modify(
        &mut self,
        key: BlockKey,
        f: impl FnOnce(&mut Block),
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
            data: std::mem::take(&mut block.data),
            render_param: block.render_param.clone(),
        };
        f(&mut new_block);

        if new_block.id != block.id {
            block.data = new_block.data;
            return Err(FieldError::InvalidId);
        }

        if new_block.location != block.location {
            // check by spatial features
            let prop = self.props.get(block.id as usize).unwrap();
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

        if new_block.render_param != block.render_param {
            let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
            *chunk.blocks.get_mut(local_key as usize).unwrap() = new_block;
            chunk.version += 1;
            return Ok(key);
        }

        block.data = new_block.data;
        Ok(key)
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
        Self::CHUNK_SIZE
    }

    #[inline]
    pub fn get_chunk(&self, chunk_key: u32) -> Result<&BlockChunk, FieldError> {
        self.chunks
            .get(chunk_key as usize)
            .ok_or(FieldError::NotFound)
    }

    #[inline]
    pub fn get_name_text(&self, key: BlockKey) -> Result<&str, FieldError> {
        let block = self.get(key)?;
        let prop = self.props.get(block.id as usize).unwrap();
        Ok(&prop.name_text)
    }

    #[inline]
    pub fn get_desc_text(&self, key: BlockKey) -> Result<&str, FieldError> {
        let block = self.get(key)?;
        let prop = self.props.get(block.id as usize).unwrap();
        Ok(&prop.desc_text)
    }

    // spatial features

    #[inline]
    pub fn get_base_rect(&self, id: u16) -> Result<[IVec2; 2], FieldError> {
        let prop = self.props.get(id as usize).ok_or(FieldError::InvalidId)?;
        Ok(prop.rect(Default::default()))
    }

    #[inline]
    pub fn get_rect(&self, key: BlockKey) -> Result<[IVec2; 2], FieldError> {
        let block = self.get(key)?;
        let prop = self.props.get(block.id as usize).unwrap();
        Ok(prop.rect(block.location))
    }

    #[inline]
    pub fn has_by_point(&self, point: IVec2) -> bool {
        let point = [point.x, point.y];
        self.spatial_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_by_point(&self, point: IVec2) -> Option<BlockKey> {
        let point = [point.x, point.y];
        let node = self.spatial_ref.locate_at_point(&point)?;
        Some(node.data)
    }

    #[inline]
    pub fn has_by_rect(&self, rect: [IVec2; 2]) -> bool {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.spatial_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    #[inline]
    pub fn get_by_rect(&self, rect: [IVec2; 2]) -> impl Iterator<Item = BlockKey> + '_ {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
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
    pub fn get_base_collision_rect(&self, id: u16) -> Result<[Vec2; 2], FieldError> {
        let prop = self.props.get(id as usize).ok_or(FieldError::InvalidId)?;
        Ok(prop.collision_rect(Default::default()).unwrap_or_default())
    }

    #[inline]
    pub fn get_collision_rect(&self, key: BlockKey) -> Result<[Vec2; 2], FieldError> {
        let block = self.get(key)?;
        let prop = self.props.get(block.id as usize).unwrap();
        Ok(prop.collision_rect(block.location).unwrap_or_default())
    }

    #[inline]
    pub fn has_by_collision_point(&self, point: Vec2) -> bool {
        let point = [point.x, point.y];
        self.collision_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = BlockKey> + '_ {
        let point = [point.x, point.y];
        self.collision_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    #[inline]
    pub fn has_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    #[inline]
    pub fn get_by_collision_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = BlockKey> + '_ {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }

    // hint features

    #[inline]
    pub fn get_base_z_along_y(&self, id: u16) -> Result<bool, FieldError> {
        let prop = self.props.get(id as usize).ok_or(FieldError::InvalidId)?;
        Ok(prop.z_along_y)
    }

    #[inline]
    pub fn get_base_hint_rect(&self, id: u16) -> Result<[Vec2; 2], FieldError> {
        let prop = self.props.get(id as usize).ok_or(FieldError::InvalidId)?;
        Ok(prop.hint_rect(Default::default()).unwrap_or_default())
    }

    #[inline]
    pub fn get_hint_rect(&self, key: BlockKey) -> Result<[Vec2; 2], FieldError> {
        let block = self.get(key)?;
        let prop = self.props.get(block.id as usize).unwrap();
        Ok(prop.hint_rect(block.location).unwrap_or_default())
    }

    #[inline]
    pub fn has_by_hint_point(&self, point: Vec2) -> bool {
        let point = [point.x, point.y];
        self.hint_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_by_hint_point(&self, point: Vec2) -> impl Iterator<Item = BlockKey> + '_ {
        let point = [point.x, point.y];
        self.hint_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    #[inline]
    pub fn has_by_hint_rect(&self, rect: [Vec2; 2]) -> bool {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    #[inline]
    pub fn get_by_hint_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = BlockKey> + '_ {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }
}

// entity field

pub type EntityKey = (u32, u32);

#[derive(Debug, Clone)]
pub struct EntityDescriptor {
    pub name_text: String,
    pub desc_text: String,
    pub collision_size: Vec2,
    pub collision_offset: Vec2,
    pub hint_size: Vec2,
    pub hint_offset: Vec2,
    pub z_along_y: bool,
}

#[derive(Debug, Clone)]
pub struct EntityFieldDescriptor {
    pub entities: Vec<EntityDescriptor>,
}

#[derive(Debug, Clone)]
pub struct EntityProperty {
    name_text: String,
    desc_text: String,
    collision_size: Vec2,
    collision_offset: Vec2,
    hint_size: Vec2,
    hint_offset: Vec2,
    z_along_y: bool,
}

impl EntityProperty {
    #[rustfmt::skip]
    fn collision_rect(&self, location: Vec2) -> Option<[Vec2; 2]> {
        if self.collision_size[0] * self.collision_size[1] == 0.0 {
            return None;
        }

        Some([
            location + self.collision_offset,
            location + self.collision_offset + self.collision_size,
        ])
    }

    #[rustfmt::skip]
    fn hint_rect(&self, location: Vec2) -> Option<[Vec2; 2]> {
        if self.hint_size[0] * self.hint_size[1] == 0.0 {
            return None;
        }

        Some([
            location + self.hint_offset,
            location + self.hint_offset + self.hint_size,
        ])
    }
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub id: u16,
    pub location: Vec2,
    pub data: Box<dyn EntityData>,
    pub render_param: EntityRenderParam,
}

#[derive(Debug, Clone, Default)]
pub struct EntityChunk {
    pub version: u64,
    pub entities: slab::Slab<Entity>,
}

#[derive(Debug, Clone)]
pub struct EntityField {
    props: Vec<EntityProperty>,
    chunks: Vec<EntityChunk>,
    chunk_ref: ahash::AHashMap<IVec2, u32>,
    collision_ref: rstar::RTree<RectNode<[f32; 2], EntityKey>>,
    hint_ref: rstar::RTree<RectNode<[f32; 2], EntityKey>>,
}

impl EntityField {
    const CHUNK_SIZE: u32 = 32;

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
                name_text: entity.name_text,
                desc_text: entity.desc_text,
                collision_size: entity.collision_size,
                collision_offset: entity.collision_offset,
                hint_size: entity.hint_size,
                hint_offset: entity.hint_offset,
                z_along_y: entity.z_along_y,
            });
        }

        Self {
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

        let chunk_size = Vec2::splat(Self::CHUNK_SIZE as f32);
        let chunk_location = entity.location.div_euclid(chunk_size).as_ivec2();

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
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = rstar::primitives::GeomWithData::new(rect, (chunk_key, local_key));
            self.collision_ref.insert(node);
        }

        // hint features
        if let Some(rect) = prop.hint_rect(entity.location) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
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
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = &rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.remove(node).unwrap();
        }

        // hint features
        if let Some(rect) = prop.hint_rect(entity.location) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = &rstar::primitives::GeomWithData::new(rect, key);
            self.hint_ref.remove(node).unwrap();
        }

        Ok(entity)
    }

    pub fn modify(
        &mut self,
        key: EntityKey,
        f: impl FnOnce(&mut Entity),
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
            data: std::mem::take(&mut entity.data),
            render_param: entity.render_param.clone(),
        };
        f(&mut new_entity);

        if new_entity.id != entity.id {
            entity.data = new_entity.data;
            return Err(FieldError::InvalidId);
        }

        if new_entity.location != entity.location {
            self.remove(key).unwrap();
            let key = self.insert(new_entity).unwrap();
            return Ok(key);
        }

        if new_entity.render_param != entity.render_param {
            let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
            *chunk.entities.get_mut(local_key as usize).unwrap() = new_entity;
            chunk.version += 1;
            return Ok(key);
        }

        entity.data = new_entity.data;
        Ok(key)
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
        Self::CHUNK_SIZE
    }

    #[inline]
    pub fn get_chunk(&self, chunk_key: u32) -> Result<&EntityChunk, FieldError> {
        self.chunks
            .get(chunk_key as usize)
            .ok_or(FieldError::NotFound)
    }

    #[inline]
    pub fn get_name_text(&self, key: EntityKey) -> Result<&str, FieldError> {
        let entity = self.get(key)?;
        let prop = self.props.get(entity.id as usize).unwrap();
        Ok(&prop.name_text)
    }

    #[inline]
    pub fn get_desc_text(&self, key: EntityKey) -> Result<&str, FieldError> {
        let entity = self.get(key)?;
        let prop = self.props.get(entity.id as usize).unwrap();
        Ok(&prop.desc_text)
    }

    // spatial features

    #[inline]
    pub fn get_by_chunk_location(&self, chunk_location: IVec2) -> Option<u32> {
        self.chunk_ref.get(&chunk_location).copied()
    }

    // collision features

    #[inline]
    pub fn get_base_collision_rect(&self, id: u16) -> Result<[Vec2; 2], FieldError> {
        let prop = self.props.get(id as usize).ok_or(FieldError::InvalidId)?;
        Ok(prop.collision_rect(Default::default()).unwrap_or_default())
    }

    #[inline]
    pub fn get_collision_rect(&self, entity_key: EntityKey) -> Result<[Vec2; 2], FieldError> {
        let entity = self.get(entity_key)?;
        let prop = self.props.get(entity.id as usize).unwrap();
        Ok(prop.collision_rect(entity.location).unwrap_or_default())
    }

    #[inline]
    pub fn has_by_collision_point(&self, point: Vec2) -> bool {
        let point = [point.x, point.y];
        self.collision_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = EntityKey> + '_ {
        let point = [point.x, point.y];
        self.collision_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    #[inline]
    pub fn has_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    #[inline]
    pub fn get_by_collision_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = EntityKey> + '_ {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }

    // hint features

    #[inline]
    pub fn get_base_z_along_y(&self, id: u16) -> Result<bool, FieldError> {
        let prop = self.props.get(id as usize).ok_or(FieldError::InvalidId)?;
        Ok(prop.z_along_y)
    }

    #[inline]
    pub fn get_base_hint_rect(&self, id: u16) -> Result<[Vec2; 2], FieldError> {
        let prop = self.props.get(id as usize).ok_or(FieldError::InvalidId)?;
        Ok(prop.hint_rect(Default::default()).unwrap_or_default())
    }

    #[inline]
    pub fn get_hint_rect(&self, entity_key: EntityKey) -> Result<[Vec2; 2], FieldError> {
        let entity = self.get(entity_key)?;
        let prop = self.props.get(entity.id as usize).unwrap();
        Ok(prop.hint_rect(entity.location).unwrap_or_default())
    }

    #[inline]
    pub fn has_by_hint_point(&self, point: Vec2) -> bool {
        let point = [point.x, point.y];
        self.hint_ref.locate_at_point(&point).is_some()
    }

    #[inline]
    pub fn get_by_hint_point(&self, point: Vec2) -> impl Iterator<Item = EntityKey> + '_ {
        let point = [point.x, point.y];
        self.hint_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    #[inline]
    pub fn has_by_hint_rect(&self, rect: [Vec2; 2]) -> bool {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    #[inline]
    pub fn get_by_hint_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = EntityKey> + '_ {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }
}

// error handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldError {
    NotFound,
    Conflict,
    InvalidId,
}

impl std::fmt::Display for FieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "not found error"),
            Self::Conflict => write!(f, "conflict error"),
            Self::InvalidId => write!(f, "invalid id error"),
        }
    }
}

impl std::error::Error for FieldError {}

// tests
// TODO: minimize test code using by fn, macro, etc.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud_tile() {
        let mut field: TileField = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor {
                    name_text: "tile_0".into(),
                    desc_text: "tile_0_desc".into(),
                    collision: true,
                },
                TileDescriptor {
                    name_text: "tile_0".into(),
                    desc_text: "tile_0_desc".into(),
                    collision: true,
                },
            ],
        });

        let key = field
            .insert(Tile {
                id: 1,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        let tile = field.get(key).unwrap();
        assert_eq!(tile.id, 1);
        assert_eq!(tile.location, IVec2::new(-1, 3));

        assert!(field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_by_point(IVec2::new(-1, 3)), Some(key));

        let tile = field.remove(key).unwrap();
        assert_eq!(tile.id, 1);
        assert_eq!(tile.location, IVec2::new(-1, 3));

        assert_eq!(field.get(key).unwrap_err(), FieldError::NotFound);
        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_by_point(IVec2::new(-1, 3)), None);
        assert_eq!(field.remove(key).unwrap_err(), FieldError::NotFound);
    }

    #[test]
    fn insert_tile_with_invalid() {
        let mut field: TileField = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor {
                    name_text: "tile_0".into(),
                    desc_text: "tile_0_desc".into(),
                    collision: true,
                },
                TileDescriptor {
                    name_text: "tile_0".into(),
                    desc_text: "tile_0_desc".into(),
                    collision: true,
                },
            ],
        });

        assert_eq!(
            field.insert(Tile {
                id: 2,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            }),
            Err(FieldError::InvalidId)
        );
        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_by_point(IVec2::new(-1, 3)), None);

        let key = field
            .insert(Tile {
                id: 1,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        assert_eq!(
            field.insert(Tile {
                id: 0,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            }),
            Err(FieldError::Conflict)
        );

        let tile = field.get(key).unwrap();
        assert_eq!(tile.id, 1);
        assert_eq!(tile.location, IVec2::new(-1, 3));

        assert!(field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_by_point(IVec2::new(-1, 3)), Some(key));
    }

    #[test]
    fn modify_tile() {
        let mut field: TileField = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor {
                    name_text: "tile_0".into(),
                    desc_text: "tile_0_desc".into(),
                    collision: true,
                },
                TileDescriptor {
                    name_text: "tile_0".into(),
                    desc_text: "tile_0_desc".into(),
                    collision: true,
                },
            ],
        });

        let key = field
            .insert(Tile {
                id: 1,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        let key = field
            .modify(key, |tile| tile.location = IVec2::new(-1, 4))
            .unwrap();

        let tile = field.get(key).unwrap();
        assert_eq!(tile.id, 1);
        assert_eq!(tile.location, IVec2::new(-1, 4));

        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_by_point(IVec2::new(-1, 3)), None);
        assert!(field.has_by_point(IVec2::new(-1, 4)));
        assert_eq!(field.get_by_point(IVec2::new(-1, 4)), Some(key));

        let key = field
            .modify(key, |tile| tile.render_param.variant = 1)
            .unwrap();

        let tile = field.get(key).unwrap();
        assert_eq!(tile.id, 1);
        assert_eq!(tile.location, IVec2::new(-1, 4));
        assert_eq!(tile.render_param.variant, 1);

        let key = field.modify(key, |_| {}).unwrap();

        let tile = field.get(key).unwrap();
        assert_eq!(tile.id, 1);
        assert_eq!(tile.location, IVec2::new(-1, 4));
        assert_eq!(tile.render_param.variant, 1);
    }

    #[test]
    fn modify_tile_with_invalid() {
        let mut field: TileField = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor {
                    name_text: "tile_0".into(),
                    desc_text: "tile_0_desc".into(),
                    collision: true,
                },
                TileDescriptor {
                    name_text: "tile_0".into(),
                    desc_text: "tile_0_desc".into(),
                    collision: true,
                },
            ],
        });

        let key_0 = field
            .insert(Tile {
                id: 0,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let key_1 = field
            .insert(Tile {
                id: 1,
                location: IVec2::new(-1, 4),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        assert_eq!(
            field.modify(key_0, |tile| tile.id = 1),
            Err(FieldError::InvalidId)
        );

        assert_eq!(
            field.modify(key_0, |tile| tile.location = IVec2::new(-1, 4)),
            Err(FieldError::Conflict)
        );

        let tile = field.get(key_0).unwrap();
        assert_eq!(tile.id, 0);
        assert_eq!(tile.location, IVec2::new(-1, 3));

        assert!(field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_by_point(IVec2::new(-1, 3)), Some(key_0));

        field.remove(key_1).unwrap();
        assert_eq!(field.modify(key_1, |_| {}), Err(FieldError::NotFound));
        assert_eq!(field.get(key_1).unwrap_err(), FieldError::NotFound);
        assert!(!field.has_by_point(IVec2::new(-1, 4)));
        assert_eq!(field.get_by_point(IVec2::new(-1, 4)), None);
    }

    #[test]
    fn modify_tile_with_move() {
        let mut field: TileField = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor {
                    name_text: "tile_0".into(),
                    desc_text: "tile_0_desc".into(),
                    collision: true,
                },
                TileDescriptor {
                    name_text: "tile_0".into(),
                    desc_text: "tile_0_desc".into(),
                    collision: true,
                },
            ],
        });

        let key = field
            .insert(Tile {
                id: 1,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        let key = field
            .modify(key, |tile| tile.location = IVec2::new(-1, 1000))
            .unwrap();

        let tile = field.get(key).unwrap();
        assert_eq!(tile.id, 1);
        assert_eq!(tile.location, IVec2::new(-1, 1000));

        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_by_point(IVec2::new(-1, 3)), None);
        assert!(field.has_by_point(IVec2::new(-1, 1000)));
        assert_eq!(field.get_by_point(IVec2::new(-1, 1000)), Some(key));
    }

    #[test]
    fn collision_tile() {
        let mut field: TileField = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor {
                    name_text: "tile_0".into(),
                    desc_text: "tile_0_desc".into(),
                    collision: true,
                },
                TileDescriptor {
                    name_text: "tile_0".into(),
                    desc_text: "tile_0_desc".into(),
                    collision: true,
                },
            ],
        });

        let key_0 = field
            .insert(Tile {
                id: 1,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let key_1 = field
            .insert(Tile {
                id: 1,
                location: IVec2::new(-1, 4),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let _key_2 = field
            .insert(Tile {
                id: 1,
                location: IVec2::new(-1, 5),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        assert_eq!(
            field.get_collision_rect(key_0),
            Ok([Vec2::new(-1.0, 3.0), Vec2::new(0.0, 4.0)])
        );

        let point = Vec2::new(-1.0, 4.0);
        assert!(field.has_by_collision_point(point));
        let vec = field.get_by_collision_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        let rect = [Vec2::new(-1.0, 3.0), Vec2::new(-1.0, 4.0)];
        assert!(field.has_by_collision_rect(rect));
        let vec = field.get_by_collision_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        field.remove(key_0).unwrap();
        assert_eq!(field.get_collision_rect(key_0), Err(FieldError::NotFound));
    }

    #[test]
    fn tile_chunk() {
        let mut field: TileField = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor {
                    name_text: "tile_0".into(),
                    desc_text: "tile_0_desc".into(),
                    collision: true,
                },
                TileDescriptor {
                    name_text: "tile_0".into(),
                    desc_text: "tile_0_desc".into(),
                    collision: true,
                },
            ],
        });
        assert_eq!(field.get_chunk_size(), 32);

        let _key0 = field
            .insert(Tile {
                id: 1,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let _key1 = field
            .insert(Tile {
                id: 1,
                location: IVec2::new(-1, 4),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let _key2 = field
            .insert(Tile {
                id: 1,
                location: IVec2::new(-1, 5),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        assert!(field.get_by_chunk_location(IVec2::new(0, 0)).is_none());

        let chunk_key = field.get_by_chunk_location(IVec2::new(-1, 0)).unwrap();
        let chunk = field.get_chunk(chunk_key).unwrap();
        assert_eq!(chunk.tiles.len(), 3);
    }

    #[test]
    #[should_panic]
    fn block_field_with_invalid() {
        let _: BlockField = BlockField::new(BlockFieldDescriptor {
            blocks: vec![BlockDescriptor {
                name_text: "block_0".into(),
                desc_text: "block_0_desc".into(),
                size: IVec2::new(-1, -1),
                collision_size: Vec2::new(1.0, 1.0),
                collision_offset: Vec2::new(0.0, 0.0),
                hint_size: Vec2::new(1.0, 1.0),
                hint_offset: Vec2::new(0.0, 0.0),
                z_along_y: false,
            }],
        });
    }

    #[test]
    #[should_panic]
    fn block_field_with_invalid_collision() {
        let _: BlockField = BlockField::new(BlockFieldDescriptor {
            blocks: vec![BlockDescriptor {
                name_text: "block_0".into(),
                desc_text: "block_0_desc".into(),
                size: IVec2::new(1, 1),
                collision_size: Vec2::new(-1.0, -1.0),
                collision_offset: Vec2::new(0.0, 0.0),
                hint_size: Vec2::new(1.0, 1.0),
                hint_offset: Vec2::new(0.0, 0.0),
                z_along_y: false,
            }],
        });
    }

    #[test]
    #[should_panic]
    fn block_field_with_invalid_hint() {
        let _: BlockField = BlockField::new(BlockFieldDescriptor {
            blocks: vec![BlockDescriptor {
                name_text: "block_0".into(),
                desc_text: "block_0_desc".into(),
                size: IVec2::new(1, 1),
                collision_size: Vec2::new(1.0, 1.0),
                collision_offset: Vec2::new(0.0, 0.0),
                hint_size: Vec2::new(-1.0, -1.0),
                hint_offset: Vec2::new(0.0, 0.0),
                z_along_y: false,
            }],
        });
    }

    #[test]
    fn crud_block() {
        let mut field: BlockField = BlockField::new(BlockFieldDescriptor {
            blocks: vec![
                BlockDescriptor {
                    name_text: "block_0".into(),
                    desc_text: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockDescriptor {
                    name_text: "block_1".into(),
                    desc_text: "block_1_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        let key = field
            .insert(Block {
                id: 1,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        assert_eq!(
            field.get_rect(key),
            Ok([IVec2::new(-1, 3), IVec2::new(-1, 3)])
        );

        let block = field.get(key).unwrap();
        assert_eq!(block.id, 1);
        assert_eq!(block.location, IVec2::new(-1, 3));

        assert!(field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_by_point(IVec2::new(-1, 3)), Some(key));

        let block = field.remove(key).unwrap();
        assert_eq!(block.id, 1);
        assert_eq!(block.location, IVec2::new(-1, 3));

        assert_eq!(field.get(key).unwrap_err(), FieldError::NotFound);
        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_by_point(IVec2::new(-1, 3)), None);
        assert_eq!(field.remove(key).unwrap_err(), FieldError::NotFound);

        assert_eq!(field.get_rect(key).unwrap_err(), FieldError::NotFound);
    }

    #[test]
    fn insert_block_with_invalid() {
        let mut field: BlockField = BlockField::new(BlockFieldDescriptor {
            blocks: vec![
                BlockDescriptor {
                    name_text: "block_0".into(),
                    desc_text: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockDescriptor {
                    name_text: "block_1".into(),
                    desc_text: "block_1_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        assert_eq!(
            field.insert(Block {
                id: 2,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            }),
            Err(FieldError::InvalidId)
        );
        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_by_point(IVec2::new(-1, 3)), None);

        let key = field
            .insert(Block {
                id: 1,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        assert_eq!(
            field.insert(Block {
                id: 0,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            }),
            Err(FieldError::Conflict)
        );

        let block = field.get(key).unwrap();
        assert_eq!(block.id, 1);
        assert_eq!(block.location, IVec2::new(-1, 3));

        assert!(field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_by_point(IVec2::new(-1, 3)), Some(key));
    }

    #[test]
    fn modify_block() {
        let mut field: BlockField = BlockField::new(BlockFieldDescriptor {
            blocks: vec![
                BlockDescriptor {
                    name_text: "block_0".into(),
                    desc_text: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockDescriptor {
                    name_text: "block_1".into(),
                    desc_text: "block_1_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        let key = field
            .insert(Block {
                id: 1,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        let key = field
            .modify(key, |block| block.location = IVec2::new(-1, 4))
            .unwrap();

        let block = field.get(key).unwrap();
        assert_eq!(block.id, 1);
        assert_eq!(block.location, IVec2::new(-1, 4));

        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_by_point(IVec2::new(-1, 3)), None);
        assert!(field.has_by_point(IVec2::new(-1, 4)));
        assert_eq!(field.get_by_point(IVec2::new(-1, 4)), Some(key));

        let key = field
            .modify(key, |block| block.render_param.variant = 1)
            .unwrap();

        let block = field.get(key).unwrap();
        assert_eq!(block.id, 1);
        assert_eq!(block.location, IVec2::new(-1, 4));
        assert_eq!(block.render_param.variant, 1);

        let key = field.modify(key, |_| {}).unwrap();

        let block = field.get(key).unwrap();
        assert_eq!(block.id, 1);
        assert_eq!(block.location, IVec2::new(-1, 4));
        assert_eq!(block.render_param.variant, 1);
    }

    #[test]
    fn modify_block_with_invalid() {
        let mut field: BlockField = BlockField::new(BlockFieldDescriptor {
            blocks: vec![
                BlockDescriptor {
                    name_text: "block_0".into(),
                    desc_text: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockDescriptor {
                    name_text: "block_1".into(),
                    desc_text: "block_1_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        let key_0 = field
            .insert(Block {
                id: 0,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let key_1 = field
            .insert(Block {
                id: 1,
                location: IVec2::new(-1, 4),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        assert_eq!(
            field.modify(key_0, |block| block.id = 1),
            Err(FieldError::InvalidId)
        );

        assert_eq!(
            field.modify(key_0, |block| block.location = IVec2::new(-1, 4)),
            Err(FieldError::Conflict)
        );

        let block = field.get(key_0).unwrap();
        assert_eq!(block.id, 0);
        assert_eq!(block.location, IVec2::new(-1, 3));

        assert!(field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_by_point(IVec2::new(-1, 3)), Some(key_0));

        field.remove(key_1).unwrap();

        assert_eq!(field.modify(key_1, |_| {}), Err(FieldError::NotFound));

        assert_eq!(field.get(key_1).unwrap_err(), FieldError::NotFound);
        assert!(!field.has_by_point(IVec2::new(-1, 4)));
        assert_eq!(field.get_by_point(IVec2::new(-1, 4)), None);
    }

    #[test]
    fn modify_block_with_move() {
        let mut field: BlockField = BlockField::new(BlockFieldDescriptor {
            blocks: vec![
                BlockDescriptor {
                    name_text: "block_0".into(),
                    desc_text: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockDescriptor {
                    name_text: "block_1".into(),
                    desc_text: "block_1_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        let key = field
            .insert(Block {
                id: 1,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        let key = field
            .modify(key, |block| block.location = IVec2::new(-1, 1000))
            .unwrap();

        let block = field.get(key).unwrap();
        assert_eq!(block.id, 1);
        assert_eq!(block.location, IVec2::new(-1, 1000));

        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_by_point(IVec2::new(-1, 3)), None);
        assert!(field.has_by_point(IVec2::new(-1, 1000)));
        assert_eq!(field.get_by_point(IVec2::new(-1, 1000)), Some(key));
    }

    #[test]
    fn collision_block() {
        let mut field: BlockField = BlockField::new(BlockFieldDescriptor {
            blocks: vec![
                BlockDescriptor {
                    name_text: "block_0".into(),
                    desc_text: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockDescriptor {
                    name_text: "block_1".into(),
                    desc_text: "block_1_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        let key_0 = field
            .insert(Block {
                id: 1,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let key_1 = field
            .insert(Block {
                id: 1,
                location: IVec2::new(-1, 4),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let _key_2 = field
            .insert(Block {
                id: 1,
                location: IVec2::new(-1, 5),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        assert_eq!(
            field.get_collision_rect(key_0),
            Ok([Vec2::new(-1.0, 3.0), Vec2::new(0.0, 4.0)])
        );

        let point = Vec2::new(-1.0, 4.0);
        assert!(field.has_by_collision_point(point));
        let vec = field.get_by_collision_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        let rect = [Vec2::new(-1.0, 3.0), Vec2::new(-1.0, 4.0)];
        assert!(field.has_by_collision_rect(rect));
        let vec = field.get_by_collision_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        field.remove(key_0).unwrap();
        assert_eq!(field.get_collision_rect(key_0), Err(FieldError::NotFound));
    }

    #[test]
    fn hint_block() {
        let mut field: BlockField = BlockField::new(BlockFieldDescriptor {
            blocks: vec![
                BlockDescriptor {
                    name_text: "block_0".into(),
                    desc_text: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockDescriptor {
                    name_text: "block_1".into(),
                    desc_text: "block_1_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        let key_0 = field
            .insert(Block {
                id: 1,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let key_1 = field
            .insert(Block {
                id: 1,
                location: IVec2::new(-1, 4),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let _key_2 = field
            .insert(Block {
                id: 1,
                location: IVec2::new(-1, 5),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        assert_eq!(
            field.get_hint_rect(key_0),
            Ok([Vec2::new(-1.0, 3.0), Vec2::new(0.0, 4.0)])
        );

        let point = Vec2::new(-1.0, 4.0);
        assert!(field.has_by_hint_point(point));
        let vec = field.get_by_hint_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        let rect = [Vec2::new(-1.0, 3.0), Vec2::new(-1.0, 4.0)];
        assert!(field.has_by_hint_rect(rect));
        let vec = field.get_by_hint_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        field.remove(key_0).unwrap();
        assert_eq!(field.get_hint_rect(key_0), Err(FieldError::NotFound));
    }

    #[test]
    fn block_chunk() {
        let mut field: BlockField = BlockField::new(BlockFieldDescriptor {
            blocks: vec![
                BlockDescriptor {
                    name_text: "block_0".into(),
                    desc_text: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockDescriptor {
                    name_text: "block_1".into(),
                    desc_text: "block_1_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });
        assert_eq!(field.get_chunk_size(), 32);

        let _key0 = field
            .insert(Block {
                id: 1,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let _key1 = field
            .insert(Block {
                id: 1,
                location: IVec2::new(-1, 4),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let _key2 = field
            .insert(Block {
                id: 1,
                location: IVec2::new(-1, 5),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        assert!(field.get_by_chunk_location(IVec2::new(0, 0)).is_none());

        let chunk_key = field.get_by_chunk_location(IVec2::new(-1, 0)).unwrap();
        let chunk = field.get_chunk(chunk_key).unwrap();
        assert_eq!(chunk.blocks.len(), 3);
    }

    #[test]
    #[should_panic]
    fn entity_field_with_invalid_collision() {
        let _: EntityField = EntityField::new(EntityFieldDescriptor {
            entities: vec![EntityDescriptor {
                name_text: "entity_0".into(),
                desc_text: "entity_0_desc".into(),
                collision_size: Vec2::new(-1.0, -1.0),
                collision_offset: Vec2::new(0.0, 0.0),
                hint_size: Vec2::new(1.0, 1.0),
                hint_offset: Vec2::new(0.0, 0.0),
                z_along_y: false,
            }],
        });
    }

    #[test]
    #[should_panic]
    fn entity_field_with_invalid_hint() {
        let _: EntityField = EntityField::new(EntityFieldDescriptor {
            entities: vec![EntityDescriptor {
                name_text: "entity_0".into(),
                desc_text: "entity_0_desc".into(),
                collision_size: Vec2::new(1.0, 1.0),
                collision_offset: Vec2::new(0.0, 0.0),
                hint_size: Vec2::new(-1.0, -1.0),
                hint_offset: Vec2::new(0.0, 0.0),
                z_along_y: false,
            }],
        });
    }

    #[test]
    fn crud_entity() {
        let mut field: EntityField = EntityField::new(EntityFieldDescriptor {
            entities: vec![
                EntityDescriptor {
                    name_text: "entity_0".into(),
                    desc_text: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                EntityDescriptor {
                    name_text: "entity_1".into(),
                    desc_text: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        let key = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        let entity = field.get(key).unwrap();
        assert_eq!(entity.id, 1);
        assert_eq!(entity.location, Vec2::new(-1.0, 3.0));

        let entity = field.remove(key).unwrap();
        assert_eq!(entity.id, 1);
        assert_eq!(entity.location, Vec2::new(-1.0, 3.0));

        assert_eq!(field.get(key).unwrap_err(), FieldError::NotFound);
        assert_eq!(field.remove(key).unwrap_err(), FieldError::NotFound);
    }

    #[test]
    fn insert_entity_with_invalid() {
        let mut field: EntityField = EntityField::new(EntityFieldDescriptor {
            entities: vec![
                EntityDescriptor {
                    name_text: "entity_0".into(),
                    desc_text: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                EntityDescriptor {
                    name_text: "entity_1".into(),
                    desc_text: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        assert_eq!(
            field.insert(Entity {
                id: 2,
                location: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_param: Default::default(),
            }),
            Err(FieldError::InvalidId)
        );
    }

    #[test]
    fn modify_entity() {
        let mut field: EntityField = EntityField::new(EntityFieldDescriptor {
            entities: vec![
                EntityDescriptor {
                    name_text: "entity_0".into(),
                    desc_text: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                EntityDescriptor {
                    name_text: "entity_1".into(),
                    desc_text: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        let key = field
            .insert(Entity {
                id: 0,
                location: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        let key = field
            .modify(key, |entity| entity.location = Vec2::new(-1.0, 4.0))
            .unwrap();

        let entity = field.get(key).unwrap();
        assert_eq!(entity.id, 0);
        assert_eq!(entity.location, Vec2::new(-1.0, 4.0));

        let key = field
            .modify(key, |entity| entity.render_param.variant = 1)
            .unwrap();

        let entity = field.get(key).unwrap();
        assert_eq!(entity.id, 0);
        assert_eq!(entity.location, Vec2::new(-1.0, 4.0));
        assert_eq!(entity.render_param.variant, 1);

        let key = field.modify(key, |_| {}).unwrap();

        let entity = field.get(key).unwrap();
        assert_eq!(entity.id, 0);
        assert_eq!(entity.location, Vec2::new(-1.0, 4.0));
        assert_eq!(entity.render_param.variant, 1);
    }

    #[test]
    fn modify_entity_with_invalid() {
        let mut field: EntityField = EntityField::new(EntityFieldDescriptor {
            entities: vec![
                EntityDescriptor {
                    name_text: "entity_0".into(),
                    desc_text: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                EntityDescriptor {
                    name_text: "entity_1".into(),
                    desc_text: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        let key = field
            .insert(Entity {
                id: 0,
                location: Vec2::new(-1.0, 4.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        assert_eq!(
            field.modify(key, |entity| entity.id = 1),
            Err(FieldError::InvalidId)
        );

        field.remove(key).unwrap();
        assert_eq!(field.modify(key, |_| {}), Err(FieldError::NotFound));
        assert_eq!(field.get(key).unwrap_err(), FieldError::NotFound);
    }

    #[test]
    fn modify_entity_with_move() {
        let mut field: EntityField = EntityField::new(EntityFieldDescriptor {
            entities: vec![
                EntityDescriptor {
                    name_text: "entity_0".into(),
                    desc_text: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                EntityDescriptor {
                    name_text: "entity_1".into(),
                    desc_text: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        let key = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        let key = field
            .modify(key, |tile| tile.location = Vec2::new(-1.0, 1000.0))
            .unwrap();

        let entity = field.get(key).unwrap();
        assert_eq!(entity.id, 1);
        assert_eq!(entity.location, Vec2::new(-1.0, 1000.0));
    }

    #[test]
    fn collision_entity() {
        let mut field: EntityField = EntityField::new(EntityFieldDescriptor {
            entities: vec![
                EntityDescriptor {
                    name_text: "entity_0".into(),
                    desc_text: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                EntityDescriptor {
                    name_text: "entity_1".into(),
                    desc_text: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        let key_0 = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let key_1 = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 4.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let _key_2 = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 5.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        assert_eq!(
            field.get_collision_rect(key_0),
            Ok([Vec2::new(-1.0, 3.0), Vec2::new(0.0, 4.0)])
        );

        let point = Vec2::new(-1.0, 4.0);
        assert!(field.has_by_collision_point(point));
        let vec = field.get_by_collision_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        let rect = [Vec2::new(-1.0, 3.0), Vec2::new(-1.0, 4.0)];
        assert!(field.has_by_collision_rect(rect));
        let vec = field.get_by_collision_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        field.remove(key_0).unwrap();
        assert_eq!(field.get_collision_rect(key_0), Err(FieldError::NotFound));
    }

    #[test]
    fn hint_entity() {
        let mut field: EntityField = EntityField::new(EntityFieldDescriptor {
            entities: vec![
                EntityDescriptor {
                    name_text: "entity_0".into(),
                    desc_text: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                EntityDescriptor {
                    name_text: "entity_1".into(),
                    desc_text: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        let key_0 = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let key_1 = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 4.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let _key_2 = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 5.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        assert_eq!(
            field.get_hint_rect(key_0),
            Ok([Vec2::new(-1.0, 3.0), Vec2::new(0.0, 4.0)])
        );

        let point = Vec2::new(-1.0, 4.0);
        assert!(field.has_by_hint_point(point));
        let vec = field.get_by_hint_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        let rect = [Vec2::new(-1.0, 3.0), Vec2::new(-1.0, 4.0)];
        assert!(field.has_by_hint_rect(rect));
        let vec = field.get_by_hint_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        field.remove(key_0).unwrap();
        assert_eq!(field.get_hint_rect(key_0), Err(FieldError::NotFound));
    }

    #[test]
    fn entity_chunk() {
        let mut field: EntityField = EntityField::new(EntityFieldDescriptor {
            entities: vec![
                EntityDescriptor {
                    name_text: "entity_0".into(),
                    desc_text: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                EntityDescriptor {
                    name_text: "entity_1".into(),
                    desc_text: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });
        assert_eq!(field.get_chunk_size(), 32);

        let _key0 = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let _key1 = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 4.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let _key2 = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 5.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        assert!(field.get_by_chunk_location(IVec2::new(0, 0)).is_none());

        let chunk_key = field.get_by_chunk_location(IVec2::new(-1, 0)).unwrap();
        let chunk = field.get_chunk(chunk_key).unwrap();
        assert_eq!(chunk.entities.len(), 3);
    }

    #[test]
    fn memory_size() {
        println!("Tile: {}B", std::mem::size_of::<Tile>());
        println!("Block: {}B", std::mem::size_of::<Block>());
        println!("Entity: {}B", std::mem::size_of::<Entity>());
    }
}
