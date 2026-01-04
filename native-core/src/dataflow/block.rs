use super::*;

type RectNode<T, U> = rstar::primitives::GeomWithData<rstar::primitives::Rectangle<T>, U>;

pub type BlockId = (u32, u32);

#[derive(Debug, Clone)]
pub struct BlockInfo {
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
pub struct BlockFieldInfo {
    pub blocks: Vec<BlockInfo>,
}

#[derive(Debug, Clone)]
struct BlockArchetype {
    display_name: String,
    description: String,
    size: IVec2,
    collision_size: Vec2,
    collision_offset: Vec2,
    hint_size: Vec2,
    hint_offset: Vec2,
    y_sorting: bool,
}

impl BlockArchetype {
    #[rustfmt::skip]
    fn rect(&self, coord: IVec2) -> [IVec2; 2] {
        [coord, coord + self.size[1] - 1]
    }

    #[rustfmt::skip]
    fn collision_rect(&self, coord: IVec2) -> Option<[Vec2; 2]> {
        if self.collision_size.x * self.collision_size.y == 0.0 {
            return None;
        }

        Some([coord.as_vec2() + self.collision_offset, coord.as_vec2() + self.collision_offset + self.collision_size])
    }

    #[rustfmt::skip]
    fn hint_rect(&self, coord: IVec2) -> Option<[Vec2; 2]> {
        if self.hint_size.x * self.hint_size.y == 0.0 {
            return None;
        }

        Some([coord.as_vec2() + self.hint_offset, coord.as_vec2() + self.hint_offset + self.hint_size])
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlockRenderState {
    pub variant: u8,
    pub tick: u32,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub archetype_id: u16,
    pub coord: IVec2,
    pub data: Box<dyn BlockData>,
    pub render_state: BlockRenderState,
}

#[derive(Debug, Clone)]
struct BlockChunk {
    version: u64,
    blocks: slab::Slab<Block>,
}

#[derive(Debug, Clone)]
pub struct BlockField {
    archetypes: Vec<BlockArchetype>,
    chunks: Vec<BlockChunk>,
    chunk_index: ahash::AHashMap<IVec2, u32>,
    spatial_index: rstar::RTree<RectNode<[i32; 2], BlockId>>,
    collision_index: rstar::RTree<RectNode<[f32; 2], BlockId>>,
    hint_index: rstar::RTree<RectNode<[f32; 2], BlockId>>,
}

impl BlockField {
    const CHUNK_SIZE: u32 = 32;

    pub fn new(desc: BlockFieldInfo) -> Self {
        let mut archetypes = vec![];

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

            archetypes.push(BlockArchetype {
                display_name: block.display_name,
                description: block.description,
                size: block.size,
                collision_size: block.collision_size,
                collision_offset: block.collision_offset,
                hint_size: block.hint_size,
                hint_offset: block.hint_offset,
                y_sorting: block.z_along_y,
            });
        }

        Self {
            archetypes,
            chunks: Default::default(),
            chunk_index: Default::default(),
            spatial_index: Default::default(),
            collision_index: Default::default(),
            hint_index: Default::default(),
        }
    }

    pub fn insert(&mut self, block: Block) -> Result<BlockId, BlockError> {
        let archetype = self
            .archetypes
            .get(block.archetype_id as usize)
            .ok_or(BlockError::InvalidId)?;

        // check by spatial features
        if self.has_by_rect(archetype.rect(block.coord)) {
            return Err(BlockError::Conflict);
        }

        let chunk_size = IVec2::splat(Self::CHUNK_SIZE as i32);
        let chunk_coord = block.coord.div_euclid(chunk_size);

        // get or allocate chunk
        let chunk_id = if let Some(chunk_id) = self.chunk_index.get(&chunk_coord) {
            *chunk_id
        } else {
            if self.chunks.len() >= u32::MAX as usize {
                panic!("capacity overflow");
            }

            let chunk_id = self.chunks.len() as u32;
            self.chunks.push(BlockChunk {
                version: 0,
                blocks: Default::default(),
            });
            self.chunk_index.insert(chunk_coord, chunk_id);
            chunk_id
        };

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();

        if chunk.blocks.vacant_key() >= u32::MAX as usize {
            panic!("capacity overflow");
        }

        let local_id = chunk.blocks.vacant_key() as u32;

        // spatial features
        let rect = archetype.rect(block.coord);
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
        let node = rstar::primitives::GeomWithData::new(rect, (chunk_id, local_id));
        self.spatial_index.insert(node);

        // collision features
        if let Some(rect) = archetype.collision_rect(block.coord) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = rstar::primitives::GeomWithData::new(rect, (chunk_id, local_id));
            self.collision_index.insert(node);
        }

        // hint features
        if let Some(rect) = archetype.hint_rect(block.coord) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = rstar::primitives::GeomWithData::new(rect, (chunk_id, local_id));
            self.hint_index.insert(node);
        }

        // block_key is guaranteed to be less than u32::MAX.
        chunk.blocks.insert(block);
        chunk.version += 1;

        Ok((chunk_id, local_id))
    }

    pub fn remove(&mut self, id: BlockId) -> Result<Block, BlockError> {
        let (chunk_id, local_id) = id;

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let block = chunk
            .blocks
            .try_remove(local_id as usize)
            .ok_or(BlockError::NotFound)?;
        chunk.version += 1;

        let archetype = self.archetypes.get(block.archetype_id as usize).unwrap();

        // spatial features
        let rect = archetype.rect(block.coord);
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
        let node = rstar::primitives::GeomWithData::new(rect, id);
        self.spatial_index.remove(&node).unwrap();

        // collision features
        if let Some(rect) = archetype.collision_rect(block.coord) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = rstar::primitives::GeomWithData::new(rect, id);
            self.collision_index.remove(&node).unwrap();
        }

        // hint features
        if let Some(rect) = archetype.hint_rect(block.coord) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = rstar::primitives::GeomWithData::new(rect, id);
            self.hint_index.remove(&node).unwrap();
        }

        Ok(block)
    }

    pub fn modify(&mut self, id: BlockId, f: impl FnOnce(&mut Block)) -> Result<BlockId, BlockError> {
        let (chunk_id, local_id) = id;

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let block = chunk
            .blocks
            .get_mut(local_id as usize)
            .ok_or(BlockError::NotFound)?;

        let mut new_block = Block {
            archetype_id: block.archetype_id,
            coord: block.coord,
            data: std::mem::take(&mut block.data),
            render_state: block.render_state.clone(),
        };
        f(&mut new_block);

        if new_block.archetype_id != block.archetype_id {
            block.data = new_block.data;
            return Err(BlockError::InvalidId);
        }

        if new_block.coord != block.coord {
            // check by spatial features
            let archetype = self.archetypes.get(block.archetype_id as usize).unwrap();
            if self
                .get_ids_by_rect(archetype.rect(new_block.coord))
                .any(|other_id| other_id != id)
            {
                return Err(BlockError::Conflict);
            }

            self.remove(id).unwrap();
            let new_id = self.insert(new_block).unwrap();
            return Ok(new_id);
        }

        if new_block.render_state != block.render_state {
            let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
            *chunk.blocks.get_mut(local_id as usize).unwrap() = new_block;
            chunk.version += 1;
            return Ok(id);
        }

        block.data = new_block.data;
        Ok(id)
    }

    pub fn get(&self, id: BlockId) -> Result<&Block, BlockError> {
        let (chunk_id, local_id) = id;

        let chunk = self.chunks.get(chunk_id as usize).unwrap();
        let block = chunk
            .blocks
            .get(local_id as usize)
            .ok_or(BlockError::NotFound)?;
        Ok(block)
    }

    // transfer chunk data

    pub fn get_version_by_chunk_coord(&self, chunk_coord: IVec2) -> Result<u64, BlockError> {
        let chunk_id = self
            .chunk_index
            .get(&chunk_coord)
            .ok_or(BlockError::NotFound)?;
        let chunk = self.chunks.get(*chunk_id as usize).unwrap();

        Ok(chunk.version)
    }

    pub fn get_ids_by_chunk_coord(&self, chunk_coord: IVec2) -> Result<impl Iterator<Item = BlockId>, BlockError> {
        let chunk_id = self
            .chunk_index
            .get(&chunk_coord)
            .ok_or(BlockError::NotFound)?;
        let chunk = self.chunks.get(*chunk_id as usize).unwrap();

        let ids = chunk
            .blocks
            .iter()
            .map(move |(local_id, _)| (*chunk_id, local_id as u32));
        Ok(ids)
    }

    // property

    pub fn get_display_name(&self, id: BlockId) -> Result<&str, BlockError> {
        let block = self.get(id)?;
        let archetype = self.archetypes.get(block.archetype_id as usize).unwrap();
        Ok(&archetype.display_name)
    }

    pub fn get_description(&self, id: BlockId) -> Result<&str, BlockError> {
        let block = self.get(id)?;
        let archetype = self.archetypes.get(block.archetype_id as usize).unwrap();
        Ok(&archetype.description)
    }

    // spatial features

    pub fn get_base_rect(&self, archetype_id: u16) -> Result<[IVec2; 2], BlockError> {
        let archetype = self.archetypes.get(archetype_id as usize).ok_or(BlockError::InvalidId)?;
        Ok(archetype.rect(Default::default()))
    }

    pub fn get_rect(&self, id: BlockId) -> Result<[IVec2; 2], BlockError> {
        let block = self.get(id)?;
        let archetype = self.archetypes.get(block.archetype_id as usize).unwrap();
        Ok(archetype.rect(block.coord))
    }

    pub fn has_by_point(&self, point: IVec2) -> bool {
        let point = [point.x, point.y];
        self.spatial_index.locate_at_point(&point).is_some()
    }

    pub fn get_id_by_point(&self, point: IVec2) -> Option<BlockId> {
        let point = [point.x, point.y];
        let node = self.spatial_index.locate_at_point(&point)?;
        Some(node.data)
    }

    pub fn has_by_rect(&self, rect: [IVec2; 2]) -> bool {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.spatial_index.locate_in_envelope_intersecting(&rect).next().is_some()
    }

    pub fn get_ids_by_rect(&self, rect: [IVec2; 2]) -> impl Iterator<Item = BlockId> + '_ {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.spatial_index.locate_in_envelope_intersecting(&rect).map(|node| node.data)
    }

    pub fn get_chunk_coord(&self, point: Vec2) -> IVec2 {
        let chunk_size = Vec2::splat(Self::CHUNK_SIZE as f32);
        point.div_euclid(chunk_size).as_ivec2()
    }

    // collision features

    pub fn get_base_collision_rect(&self, archetype_id: u16) -> Result<[Vec2; 2], BlockError> {
        let archetype = self.archetypes.get(archetype_id as usize).ok_or(BlockError::InvalidId)?;
        Ok(archetype.collision_rect(Default::default()).unwrap_or_default())
    }

    pub fn get_collision_rect(&self, id: BlockId) -> Result<[Vec2; 2], BlockError> {
        let block = self.get(id)?;
        let archetype = self.archetypes.get(block.archetype_id as usize).unwrap();
        Ok(archetype.collision_rect(block.coord).unwrap_or_default())
    }

    pub fn has_by_collision_point(&self, point: Vec2) -> bool {
        let point = [point.x, point.y];
        self.collision_index.locate_at_point(&point).is_some()
    }

    pub fn get_ids_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = BlockId> + '_ {
        let point = [point.x, point.y];
        self.collision_index.locate_all_at_point(&point).map(|node| node.data)
    }

    pub fn has_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.collision_index.locate_in_envelope_intersecting(&rect).next().is_some()
    }

    pub fn get_ids_by_collision_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = BlockId> + '_ {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.collision_index.locate_in_envelope_intersecting(&rect).map(|node| node.data)
    }

    // hint features

    pub fn get_base_y_sorting(&self, archetype_id: u16) -> Result<bool, BlockError> {
        let archetype = self.archetypes.get(archetype_id as usize).ok_or(BlockError::InvalidId)?;
        Ok(archetype.y_sorting)
    }

    pub fn get_base_hint_rect(&self, archetype_id: u16) -> Result<[Vec2; 2], BlockError> {
        let archetype = self.archetypes.get(archetype_id as usize).ok_or(BlockError::InvalidId)?;
        Ok(archetype.hint_rect(Default::default()).unwrap_or_default())
    }

    pub fn get_hint_rect(&self, id: BlockId) -> Result<[Vec2; 2], BlockError> {
        let block = self.get(id)?;
        let archetype = self.archetypes.get(block.archetype_id as usize).unwrap();
        Ok(archetype.hint_rect(block.coord).unwrap_or_default())
    }

    pub fn has_by_hint_point(&self, point: Vec2) -> bool {
        let point = [point.x, point.y];
        self.hint_index.locate_at_point(&point).is_some()
    }

    pub fn get_ids_by_hint_point(&self, point: Vec2) -> impl Iterator<Item = BlockId> + '_ {
        let point = [point.x, point.y];
        self.hint_index.locate_all_at_point(&point).map(|node| node.data)
    }

    pub fn has_by_hint_rect(&self, rect: [Vec2; 2]) -> bool {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.hint_index.locate_in_envelope_intersecting(&rect).next().is_some()
    }

    pub fn get_ids_by_hint_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = BlockId> + '_ {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.hint_index.locate_in_envelope_intersecting(&rect).map(|node| node.data)
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
        let _: BlockField = BlockField::new(BlockFieldInfo {
            blocks: vec![BlockInfo {
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
        let _: BlockField = BlockField::new(BlockFieldInfo {
            blocks: vec![BlockInfo {
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
        let _: BlockField = BlockField::new(BlockFieldInfo {
            blocks: vec![BlockInfo {
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
        let mut field: BlockField = BlockField::new(BlockFieldInfo {
            blocks: vec![
                BlockInfo {
                    display_name: "block_0".into(),
                    description: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockInfo {
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

        let id = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        assert_eq!(field.get_rect(id), Ok([IVec2::new(-1, 3), IVec2::new(-1, 3)]));

        let block = field.get(id).unwrap();
        assert_eq!(block.archetype_id, 1);
        assert_eq!(block.coord, IVec2::new(-1, 3));

        assert!(field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_id_by_point(IVec2::new(-1, 3)), Some(id));

        let block = field.remove(id).unwrap();
        assert_eq!(block.archetype_id, 1);
        assert_eq!(block.coord, IVec2::new(-1, 3));

        assert_eq!(field.get(id).unwrap_err(), BlockError::NotFound);
        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_id_by_point(IVec2::new(-1, 3)), None);
        assert_eq!(field.remove(id).unwrap_err(), BlockError::NotFound);

        assert_eq!(field.get_rect(id).unwrap_err(), BlockError::NotFound);
    }

    #[test]
    fn insert_block_with_invalid() {
        let mut field: BlockField = BlockField::new(BlockFieldInfo {
            blocks: vec![
                BlockInfo {
                    display_name: "block_0".into(),
                    description: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockInfo {
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
                archetype_id: 2,
                coord: IVec2::new(-1, 3),
                data: Default::default(),
                render_state: Default::default(),
            }),
            Err(BlockError::InvalidId)
        );
        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_id_by_point(IVec2::new(-1, 3)), None);

        let id = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        assert_eq!(
            field.insert(Block {
                archetype_id: 0,
                coord: IVec2::new(-1, 3),
                data: Default::default(),
                render_state: Default::default(),
            }),
            Err(BlockError::Conflict)
        );

        let block = field.get(id).unwrap();
        assert_eq!(block.archetype_id, 1);
        assert_eq!(block.coord, IVec2::new(-1, 3));

        assert!(field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_id_by_point(IVec2::new(-1, 3)), Some(id));
    }

    #[test]
    fn modify_block() {
        let mut field: BlockField = BlockField::new(BlockFieldInfo {
            blocks: vec![
                BlockInfo {
                    display_name: "block_0".into(),
                    description: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockInfo {
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

        let id = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();

        let id = field
            .modify(id, |block| block.coord = IVec2::new(-1, 4))
            .unwrap();

        let block = field.get(id).unwrap();
        assert_eq!(block.archetype_id, 1);
        assert_eq!(block.coord, IVec2::new(-1, 4));

        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_id_by_point(IVec2::new(-1, 3)), None);
        assert!(field.has_by_point(IVec2::new(-1, 4)));
        assert_eq!(field.get_id_by_point(IVec2::new(-1, 4)), Some(id));

        let id = field
            .modify(id, |block| block.render_state.variant = 1)
            .unwrap();

        let block = field.get(id).unwrap();
        assert_eq!(block.archetype_id, 1);
        assert_eq!(block.coord, IVec2::new(-1, 4));
        assert_eq!(block.render_state.variant, 1);

        let id = field.modify(id, |_| {}).unwrap();

        let block = field.get(id).unwrap();
        assert_eq!(block.archetype_id, 1);
        assert_eq!(block.coord, IVec2::new(-1, 4));
        assert_eq!(block.render_state.variant, 1);
    }

    #[test]
    fn modify_block_with_invalid() {
        let mut field: BlockField = BlockField::new(BlockFieldInfo {
            blocks: vec![
                BlockInfo {
                    display_name: "block_0".into(),
                    description: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockInfo {
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

        let id0 = field
            .insert(Block {
                archetype_id: 0,
                coord: IVec2::new(-1, 3),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        let id1 = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 4),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();

        assert_eq!(field.modify(id0, |block| block.archetype_id = 1), Err(BlockError::InvalidId));

        assert_eq!(field.modify(id0, |block| block.coord = IVec2::new(-1, 4)), Err(BlockError::Conflict));

        let block = field.get(id0).unwrap();
        assert_eq!(block.archetype_id, 0);
        assert_eq!(block.coord, IVec2::new(-1, 3));

        assert!(field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_id_by_point(IVec2::new(-1, 3)), Some(id0));

        field.remove(id1).unwrap();

        assert_eq!(field.modify(id1, |_| {}), Err(BlockError::NotFound));

        assert_eq!(field.get(id1).unwrap_err(), BlockError::NotFound);
        assert!(!field.has_by_point(IVec2::new(-1, 4)));
        assert_eq!(field.get_id_by_point(IVec2::new(-1, 4)), None);
    }

    #[test]
    fn modify_block_with_move() {
        let mut field: BlockField = BlockField::new(BlockFieldInfo {
            blocks: vec![
                BlockInfo {
                    display_name: "block_0".into(),
                    description: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockInfo {
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

        let id = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();

        let id = field
            .modify(id, |block| block.coord = IVec2::new(-1, 1000))
            .unwrap();

        let block = field.get(id).unwrap();
        assert_eq!(block.archetype_id, 1);
        assert_eq!(block.coord, IVec2::new(-1, 1000));

        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_id_by_point(IVec2::new(-1, 3)), None);
        assert!(field.has_by_point(IVec2::new(-1, 1000)));
        assert_eq!(field.get_id_by_point(IVec2::new(-1, 1000)), Some(id));
    }

    #[test]
    fn collision_block() {
        let mut field: BlockField = BlockField::new(BlockFieldInfo {
            blocks: vec![
                BlockInfo {
                    display_name: "block_0".into(),
                    description: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockInfo {
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

        let id0 = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        let id1 = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 4),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        let _ = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 5),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();

        assert_eq!(field.get_collision_rect(id0), Ok([Vec2::new(-1.0, 3.0), Vec2::new(0.0, 4.0)]));

        let point = Vec2::new(-1.0, 4.0);
        assert!(field.has_by_collision_point(point));
        let vec = field.get_ids_by_collision_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&id0));
        assert!(vec.contains(&id1));

        let rect = [Vec2::new(-1.0, 3.0), Vec2::new(-1.0, 4.0)];
        assert!(field.has_by_collision_rect(rect));
        let vec = field.get_ids_by_collision_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&id0));
        assert!(vec.contains(&id1));

        field.remove(id0).unwrap();
        assert_eq!(field.get_collision_rect(id0), Err(BlockError::NotFound));
    }

    #[test]
    fn hint_block() {
        let mut field: BlockField = BlockField::new(BlockFieldInfo {
            blocks: vec![
                BlockInfo {
                    display_name: "block_0".into(),
                    description: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockInfo {
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

        let id0 = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        let id1 = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 4),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        let _ = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 5),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();

        assert_eq!(field.get_hint_rect(id0), Ok([Vec2::new(-1.0, 3.0), Vec2::new(0.0, 4.0)]));

        let point = Vec2::new(-1.0, 4.0);
        assert!(field.has_by_hint_point(point));
        let vec = field.get_ids_by_hint_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&id0));
        assert!(vec.contains(&id1));

        let rect = [Vec2::new(-1.0, 3.0), Vec2::new(-1.0, 4.0)];
        assert!(field.has_by_hint_rect(rect));
        let vec = field.get_ids_by_hint_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&id0));
        assert!(vec.contains(&id1));

        field.remove(id0).unwrap();
        assert_eq!(field.get_hint_rect(id0), Err(BlockError::NotFound));
    }

    #[test]
    fn block_chunk() {
        let mut field: BlockField = BlockField::new(BlockFieldInfo {
            blocks: vec![
                BlockInfo {
                    display_name: "block_0".into(),
                    description: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                BlockInfo {
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

        let _= field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        let _ = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 4),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        let _ = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 5),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();

        assert!(field.get_ids_by_chunk_coord(IVec2::new(0, 0)).is_err());

        let ids = field.get_ids_by_chunk_coord(IVec2::new(-1, 0)).unwrap();
        assert_eq!(ids.count(), 3);
    }
}
