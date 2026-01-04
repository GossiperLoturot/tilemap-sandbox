use super::*;

type RectNode<T, U> = rstar::primitives::GeomWithData<rstar::primitives::Rectangle<T>, U>;

pub type BlockKey = (u32, u32);

#[derive(Debug, Clone)]
pub struct BlockDescriptor {
    pub display_name: String,
    pub description: String,
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
    display_name: String,
    description: String,
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
        if self.collision_size.x * self.collision_size.y == 0.0 {
            return None;
        }

        Some([
            location.as_vec2() + self.collision_offset,
            location.as_vec2() + self.collision_offset + self.collision_size,
        ])
    }

    #[rustfmt::skip]
    fn hint_rect(&self, location: IVec2) -> Option<[Vec2; 2]> {
        if self.hint_size.x * self.hint_size.y == 0.0 {
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
struct BlockChunk {
    version: u64,
    blocks: slab::Slab<Block>,
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
            if block.size.x <= 0 || block.size.y <= 0 {
                panic!("size must be positive");
            }
            if block.collision_size.x < 0.0 || block.collision_size.y < 0.0 {
                panic!("collision size must be non-negative");
            }
            if block.hint_size.x < 0.0 || block.hint_size.y < 0.0 {
                panic!("hint size must be non-negative");
            }

            props.push(BlockProperty {
                display_name: block.display_name,
                description: block.description,
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

    pub fn insert(&mut self, block: Block) -> Result<BlockKey, BlockError> {
        let prop = self
            .props
            .get(block.id as usize)
            .ok_or(BlockError::InvalidId)?;

        // check by spatial features
        if self.has_by_rect(prop.rect(block.location)) {
            return Err(BlockError::Conflict);
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

    pub fn remove(&mut self, key: BlockKey) -> Result<Block, BlockError> {
        let (chunk_key, local_key) = key;

        let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
        let block = chunk
            .blocks
            .try_remove(local_key as usize)
            .ok_or(BlockError::NotFound)?;
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
    ) -> Result<BlockKey, BlockError> {
        let (chunk_key, local_key) = key;

        let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
        let block = chunk
            .blocks
            .get_mut(local_key as usize)
            .ok_or(BlockError::NotFound)?;

        let mut new_block = Block {
            id: block.id,
            location: block.location,
            data: std::mem::take(&mut block.data),
            render_param: block.render_param.clone(),
        };
        f(&mut new_block);

        if new_block.id != block.id {
            block.data = new_block.data;
            return Err(BlockError::InvalidId);
        }

        if new_block.location != block.location {
            // check by spatial features
            let prop = self.props.get(block.id as usize).unwrap();
            if self
                .get_keys_by_rect(prop.rect(new_block.location))
                .any(|other_key| other_key != key)
            {
                return Err(BlockError::Conflict);
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

    pub fn get(&self, key: BlockKey) -> Result<&Block, BlockError> {
        let (chunk_key, local_key) = key;

        let chunk = self.chunks.get(chunk_key as usize).unwrap();
        let block = chunk
            .blocks
            .get(local_key as usize)
            .ok_or(BlockError::NotFound)?;
        Ok(block)
    }

    // transfer chunk data

    pub fn get_version_by_chunk_location(&self, chunk_location: IVec2) -> Result<u64, BlockError> {
        let chunk_key = self
            .chunk_ref
            .get(&chunk_location)
            .ok_or(BlockError::NotFound)?;
        let chunk = self.chunks.get(*chunk_key as usize).unwrap();

        Ok(chunk.version)
    }

    pub fn get_keys_by_chunk_location(
        &self,
        chunk_location: IVec2,
    ) -> Result<impl Iterator<Item = BlockKey>, BlockError> {
        let chunk_key = self
            .chunk_ref
            .get(&chunk_location)
            .ok_or(BlockError::NotFound)?;
        let chunk = self.chunks.get(*chunk_key as usize).unwrap();

        let keys = chunk
            .blocks
            .iter()
            .map(move |(local_key, _)| (*chunk_key, local_key as u32));
        Ok(keys)
    }

    // property

    pub fn get_display_name(&self, key: BlockKey) -> Result<&str, BlockError> {
        let block = self.get(key)?;
        let prop = self.props.get(block.id as usize).unwrap();
        Ok(&prop.display_name)
    }

    pub fn get_description(&self, key: BlockKey) -> Result<&str, BlockError> {
        let block = self.get(key)?;
        let prop = self.props.get(block.id as usize).unwrap();
        Ok(&prop.description)
    }

    // spatial features

    pub fn get_base_rect(&self, id: u16) -> Result<[IVec2; 2], BlockError> {
        let prop = self.props.get(id as usize).ok_or(BlockError::InvalidId)?;
        Ok(prop.rect(Default::default()))
    }

    pub fn get_rect(&self, key: BlockKey) -> Result<[IVec2; 2], BlockError> {
        let block = self.get(key)?;
        let prop = self.props.get(block.id as usize).unwrap();
        Ok(prop.rect(block.location))
    }

    pub fn has_by_point(&self, point: IVec2) -> bool {
        let point = [point.x, point.y];
        self.spatial_ref.locate_at_point(&point).is_some()
    }

    pub fn get_key_by_point(&self, point: IVec2) -> Option<BlockKey> {
        let point = [point.x, point.y];
        let node = self.spatial_ref.locate_at_point(&point)?;
        Some(node.data)
    }

    pub fn has_by_rect(&self, rect: [IVec2; 2]) -> bool {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.spatial_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    pub fn get_keys_by_rect(&self, rect: [IVec2; 2]) -> impl Iterator<Item = BlockKey> + '_ {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.spatial_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }

    pub fn get_chunk_location(&self, point: Vec2) -> IVec2 {
        let chunk_size = Vec2::splat(Self::CHUNK_SIZE as f32);
        point.div_euclid(chunk_size).as_ivec2()
    }

    // collision features

    pub fn get_base_collision_rect(&self, id: u16) -> Result<[Vec2; 2], BlockError> {
        let prop = self.props.get(id as usize).ok_or(BlockError::InvalidId)?;
        Ok(prop.collision_rect(Default::default()).unwrap_or_default())
    }

    pub fn get_collision_rect(&self, key: BlockKey) -> Result<[Vec2; 2], BlockError> {
        let block = self.get(key)?;
        let prop = self.props.get(block.id as usize).unwrap();
        Ok(prop.collision_rect(block.location).unwrap_or_default())
    }

    pub fn has_by_collision_point(&self, point: Vec2) -> bool {
        let point = [point.x, point.y];
        self.collision_ref.locate_at_point(&point).is_some()
    }

    pub fn get_keys_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = BlockKey> + '_ {
        let point = [point.x, point.y];
        self.collision_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    pub fn has_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    pub fn get_keys_by_collision_rect(
        &self,
        rect: [Vec2; 2],
    ) -> impl Iterator<Item = BlockKey> + '_ {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }

    // hint features

    pub fn get_base_z_along_y(&self, id: u16) -> Result<bool, BlockError> {
        let prop = self.props.get(id as usize).ok_or(BlockError::InvalidId)?;
        Ok(prop.z_along_y)
    }

    pub fn get_base_hint_rect(&self, id: u16) -> Result<[Vec2; 2], BlockError> {
        let prop = self.props.get(id as usize).ok_or(BlockError::InvalidId)?;
        Ok(prop.hint_rect(Default::default()).unwrap_or_default())
    }

    pub fn get_hint_rect(&self, key: BlockKey) -> Result<[Vec2; 2], BlockError> {
        let block = self.get(key)?;
        let prop = self.props.get(block.id as usize).unwrap();
        Ok(prop.hint_rect(block.location).unwrap_or_default())
    }

    pub fn has_by_hint_point(&self, point: Vec2) -> bool {
        let point = [point.x, point.y];
        self.hint_ref.locate_at_point(&point).is_some()
    }

    pub fn get_keys_by_hint_point(&self, point: Vec2) -> impl Iterator<Item = BlockKey> + '_ {
        let point = [point.x, point.y];
        self.hint_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    pub fn has_by_hint_rect(&self, rect: [Vec2; 2]) -> bool {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    pub fn get_keys_by_hint_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = BlockKey> + '_ {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockError {
    NotFound,
    Conflict,
    InvalidId,
}

impl std::fmt::Display for BlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "not found error"),
            Self::Conflict => write!(f, "conflict error"),
            Self::InvalidId => write!(f, "invalid id error"),
        }
    }
}

impl std::error::Error for BlockError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn block_field_with_invalid() {
        let _: BlockField = BlockField::new(BlockFieldDescriptor {
            blocks: vec![BlockDescriptor {
                display_name: "block_0".into(),
                description: "block_0_desc".into(),
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
                display_name: "block_0".into(),
                description: "block_0_desc".into(),
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
                display_name: "block_0".into(),
                description: "block_0_desc".into(),
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
                    display_name: "block_0".into(),
                    description: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockDescriptor {
                    display_name: "block_1".into(),
                    description: "block_1_desc".into(),
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
        assert_eq!(field.get_key_by_point(IVec2::new(-1, 3)), Some(key));

        let block = field.remove(key).unwrap();
        assert_eq!(block.id, 1);
        assert_eq!(block.location, IVec2::new(-1, 3));

        assert_eq!(field.get(key).unwrap_err(), BlockError::NotFound);
        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_key_by_point(IVec2::new(-1, 3)), None);
        assert_eq!(field.remove(key).unwrap_err(), BlockError::NotFound);

        assert_eq!(field.get_rect(key).unwrap_err(), BlockError::NotFound);
    }

    #[test]
    fn insert_block_with_invalid() {
        let mut field: BlockField = BlockField::new(BlockFieldDescriptor {
            blocks: vec![
                BlockDescriptor {
                    display_name: "block_0".into(),
                    description: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockDescriptor {
                    display_name: "block_1".into(),
                    description: "block_1_desc".into(),
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
            Err(BlockError::InvalidId)
        );
        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_key_by_point(IVec2::new(-1, 3)), None);

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
            Err(BlockError::Conflict)
        );

        let block = field.get(key).unwrap();
        assert_eq!(block.id, 1);
        assert_eq!(block.location, IVec2::new(-1, 3));

        assert!(field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_key_by_point(IVec2::new(-1, 3)), Some(key));
    }

    #[test]
    fn modify_block() {
        let mut field: BlockField = BlockField::new(BlockFieldDescriptor {
            blocks: vec![
                BlockDescriptor {
                    display_name: "block_0".into(),
                    description: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockDescriptor {
                    display_name: "block_1".into(),
                    description: "block_1_desc".into(),
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
        assert_eq!(field.get_key_by_point(IVec2::new(-1, 3)), None);
        assert!(field.has_by_point(IVec2::new(-1, 4)));
        assert_eq!(field.get_key_by_point(IVec2::new(-1, 4)), Some(key));

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
                    display_name: "block_0".into(),
                    description: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockDescriptor {
                    display_name: "block_1".into(),
                    description: "block_1_desc".into(),
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
            Err(BlockError::InvalidId)
        );

        assert_eq!(
            field.modify(key_0, |block| block.location = IVec2::new(-1, 4)),
            Err(BlockError::Conflict)
        );

        let block = field.get(key_0).unwrap();
        assert_eq!(block.id, 0);
        assert_eq!(block.location, IVec2::new(-1, 3));

        assert!(field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_key_by_point(IVec2::new(-1, 3)), Some(key_0));

        field.remove(key_1).unwrap();

        assert_eq!(field.modify(key_1, |_| {}), Err(BlockError::NotFound));

        assert_eq!(field.get(key_1).unwrap_err(), BlockError::NotFound);
        assert!(!field.has_by_point(IVec2::new(-1, 4)));
        assert_eq!(field.get_key_by_point(IVec2::new(-1, 4)), None);
    }

    #[test]
    fn modify_block_with_move() {
        let mut field: BlockField = BlockField::new(BlockFieldDescriptor {
            blocks: vec![
                BlockDescriptor {
                    display_name: "block_0".into(),
                    description: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockDescriptor {
                    display_name: "block_1".into(),
                    description: "block_1_desc".into(),
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
        assert_eq!(field.get_key_by_point(IVec2::new(-1, 3)), None);
        assert!(field.has_by_point(IVec2::new(-1, 1000)));
        assert_eq!(field.get_key_by_point(IVec2::new(-1, 1000)), Some(key));
    }

    #[test]
    fn collision_block() {
        let mut field: BlockField = BlockField::new(BlockFieldDescriptor {
            blocks: vec![
                BlockDescriptor {
                    display_name: "block_0".into(),
                    description: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockDescriptor {
                    display_name: "block_1".into(),
                    description: "block_1_desc".into(),
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
        let vec = field.get_keys_by_collision_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        let rect = [Vec2::new(-1.0, 3.0), Vec2::new(-1.0, 4.0)];
        assert!(field.has_by_collision_rect(rect));
        let vec = field.get_keys_by_collision_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        field.remove(key_0).unwrap();
        assert_eq!(field.get_collision_rect(key_0), Err(BlockError::NotFound));
    }

    #[test]
    fn hint_block() {
        let mut field: BlockField = BlockField::new(BlockFieldDescriptor {
            blocks: vec![
                BlockDescriptor {
                    display_name: "block_0".into(),
                    description: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockDescriptor {
                    display_name: "block_1".into(),
                    description: "block_1_desc".into(),
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
        let vec = field.get_keys_by_hint_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        let rect = [Vec2::new(-1.0, 3.0), Vec2::new(-1.0, 4.0)];
        assert!(field.has_by_hint_rect(rect));
        let vec = field.get_keys_by_hint_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        field.remove(key_0).unwrap();
        assert_eq!(field.get_hint_rect(key_0), Err(BlockError::NotFound));
    }

    #[test]
    fn block_chunk() {
        let mut field: BlockField = BlockField::new(BlockFieldDescriptor {
            blocks: vec![
                BlockDescriptor {
                    display_name: "block_0".into(),
                    description: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockDescriptor {
                    display_name: "block_1".into(),
                    description: "block_1_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

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

        assert!(field.get_keys_by_chunk_location(IVec2::new(0, 0)).is_err());

        let keys = field.get_keys_by_chunk_location(IVec2::new(-1, 0)).unwrap();
        assert_eq!(keys.count(), 3);
    }
}
