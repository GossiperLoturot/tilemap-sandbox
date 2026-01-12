use super::*;

pub type BlockId = (u32, u16);

#[derive(Debug, Clone)]
pub struct BlockInfo {
    pub display_name: String,
    pub description: String,
    pub size: IVec2,
    pub collision_rect: Rect2,
    pub hint_rect: Rect2,
    pub y_sorting: bool,
}

#[derive(Debug, Clone)]
pub struct BlockFieldInfo {
    pub blocks: Vec<BlockInfo>,
}

#[derive(Debug, Clone)]
pub struct BlockArchetype {
    pub display_name: String,
    pub description: String,
    pub size: IVec2,
    pub collision_rect: Rect2,
    pub hint_rect: Rect2,
    pub broad_rect: IRect2,
    pub y_sorting: bool,
}

impl BlockArchetype {
    fn rect(&self, coord: IVec2) -> IRect2 {
        IRect2::new(coord, coord + self.size[1] - 1)
    }

    fn collision_rect(&self, coord: IVec2) -> Rect2 {
        Rect2::new(coord.as_vec2(), coord.as_vec2()) + self.collision_rect
    }

    fn hint_rect(&self, coord: IVec2) -> Rect2 {
        Rect2::new(coord.as_vec2(), coord.as_vec2()) + self.hint_rect
    }
}

#[derive(Debug, Clone)]
pub struct BlockRenderState {
    pub variant: u8,
    pub tick: u32,
}

#[derive(Debug, Clone, Default)]
pub struct Block {
    pub archetype_id: u16,
    pub coord: IVec2,
    pub variant: u8,
    pub tick: u32,
}

#[derive(Debug, Clone)]
pub struct BlockChunk {
    pub version: u64,
    pub blocks: slab::Slab<Block>,
    coord_index: Vec<Option<BlockId>>,
    space_bins: [indexmap::IndexSet<TileId, ahash::RandomState>; 16],
}

#[derive(Debug, Clone)]
pub struct BlockField {
    archetypes: Vec<BlockArchetype>,
    chunks: Vec<BlockChunk>,
    coord_index: ahash::AHashMap<IVec2, u32>,
}

impl BlockField {
    const CHUNK_SIZE: u32 = 32;
    const BLOCK_LEN: u32 = Self::CHUNK_SIZE * Self::CHUNK_SIZE;

    pub fn new(desc: BlockFieldInfo) -> Self {
        let mut archetypes = vec![];

        for block in desc.blocks {
            if block.size.x <= 0 || block.size.y <= 0 {
                panic!("size must be positive");
            }
            let broad_rect = IRect2::new(IVec2::new(0, 0), block.size);

            if block.collision_rect.size().x < 0.0 || block.collision_rect.size().y < 0.0 {
                panic!("collision size must be non-negative");
            }
            let broad_rect = broad_rect.maximum(block.collision_rect.as_iaabb2());

            if block.hint_rect.size().x < 0.0 || block.hint_rect.size().y < 0.0 {
                panic!("hint size must be non-negative");
            }
            let broad_rect = broad_rect.maximum(block.hint_rect.as_iaabb2());

            archetypes.push(BlockArchetype {
                display_name: block.display_name,
                description: block.description,
                size: block.size,
                collision_rect: block.collision_rect,
                hint_rect: block.hint_rect,
                broad_rect,
                y_sorting: block.y_sorting,
            });
        }

        Self {
            archetypes,
            chunks: Default::default(),
            coord_index: Default::default(),
        }
    }

    pub fn insert(&mut self, block: Block) -> Result<BlockId, BlockError> {
        let archetype = self.archetypes.get(block.archetype_id as usize).ok_or(BlockError::InvalidId)?;

        // check by spatial features
        if self.find_with_rect(archetype.rect(block.coord)).next().is_some() {
            return Err(BlockError::Conflict);
        }

        let chunk_size = IVec2::splat(Self::CHUNK_SIZE as i32);
        let chunk_coord = block.coord.div_euclid(chunk_size);

        // get or allocate chunk
        let chunk_id = if let Some(chunk_id) = self.coord_index.get(&chunk_coord) {
            *chunk_id
        } else {
            if self.chunks.len() >= u32::MAX as usize {
                panic!("capacity overflow");
            }

            let chunk_id = self.chunks.len() as u32;
            self.chunks.push(BlockChunk {
                version: 0,
                blocks: Default::default(),
                coord_index: vec![None; Self::BLOCK_LEN as usize],
                space_bins: Default::default(),
            });
            self.coord_index.insert(chunk_coord, chunk_id);
            chunk_id
        };
        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();

        if chunk.blocks.vacant_key() >= u16::MAX as usize {
            panic!("capacity overflow");
        }
        let local_id = chunk.blocks.vacant_key() as u16;

        // // spatial features (dense)
        // let local_coord = block.coord.rem_euclid(chunk_size).as_u16vec2();
        // let local_coord_ = local_coord.y * Self::CHUNK_SIZE as u16 + local_coord.x;
        // *chunk.coord_index.get_mut(local_coord_ as usize).unwrap() = Some((chunk_id, local_id));
        // // spatial features (bin)
        // let space_bin_id = (local_coord.y >> 3) << 2 | (local_coord.x >> 3);
        // let space_bin = chunk.space_bins.get_mut(space_bin_id as usize).unwrap();
        // space_bin.insert((chunk_id, local_id));

        // block_key is guaranteed to be less than u32::MAX.
        chunk.blocks.insert(block);
        chunk.version += 1;

        Ok((chunk_id, local_id))
    }

    pub fn remove(&mut self, id: BlockId) -> Result<Block, BlockError> {
        let (chunk_id, local_id) = id;

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let block = chunk.blocks.try_remove(local_id as usize).ok_or(BlockError::NotFound)?;
        chunk.version += 1;

        // let chunk_size = IVec2::splat(Self::CHUNK_SIZE as i32);
        //
        // // spatial features (dense)
        // let local_coord = block.coord.rem_euclid(chunk_size).as_u16vec2();
        // let local_coord_ = local_coord.y * Self::CHUNK_SIZE as u16 + local_coord.x;
        // *chunk.coord_index.get_mut(local_coord_ as usize).unwrap() = None;
        // // spatial features (bin)
        // let space_bin_id = (local_coord.y >> 3) << 2 | (local_coord.x >> 3);
        // let space_bin = chunk.space_bins.get_mut(space_bin_id as usize).unwrap();
        // space_bin.swap_remove(&(chunk_id, local_id));

        Ok(block)
    }

    pub fn modify(&mut self, id: BlockId, f: impl FnOnce(&mut BlockRenderState)) -> Result<BlockId, BlockError> {
        let (chunk_id, local_id) = id;
        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let block = chunk.blocks.get_mut(local_id as usize).ok_or(BlockError::NotFound)?;
        let mut render_state = BlockRenderState { variant: block.variant, tick: block.tick };
        f(&mut render_state);
        block.variant = render_state.variant;
        block.tick = render_state.tick;
        chunk.version += 1;
        Ok(id)
    }

    pub fn get(&self, id: BlockId) -> Result<&Block, BlockError> {
        let (chunk_id, local_id) = id;
        let chunk = self.chunks.get(chunk_id as usize).unwrap();
        let block = chunk.blocks.get(local_id as usize).ok_or(BlockError::NotFound)?;
        Ok(block)
    }

    // archetype

    pub fn get_archetype(&self, archetype_id: u16) -> Result<&BlockArchetype, BlockError> {
        self.archetypes.get(archetype_id as usize).ok_or(BlockError::InvalidId)
    }

    // transfer chunk data

    pub fn find_chunk_coord(&self, coord: Vec2) -> IVec2 {
        let chunk_size = Vec2::splat(Self::CHUNK_SIZE as f32);
        coord.div_euclid(chunk_size).as_ivec2()
    }

    pub fn get_chunk(&self, chunk_coord: IVec2) -> Result<&BlockChunk, BlockError> {
        let chunk_id = self.coord_index.get(&chunk_coord).ok_or(BlockError::NotFound)?;
        let chunk = self.chunks.get(*chunk_id as usize).unwrap();
        Ok(chunk)
    }

    // common spatial features

    #[inline]
    fn broad_find_with_point(&self, coord: IVec2) -> Option<(BlockId, &Block)> {
        let chunk_size = IVec2::splat(Self::CHUNK_SIZE as i32);
        let chunk_coord = coord.div_euclid(chunk_size);
        let chunk_id = self.coord_index.get(&chunk_coord)?;
        let chunk = self.chunks.get(*chunk_id as usize).unwrap();

        let local_coord = coord.rem_euclid(chunk_size).as_u16vec2();
        let local_coord_ = local_coord.y * Self::CHUNK_SIZE as u16 + local_coord.x;
        let (chunk_id, local_id) = (*chunk.coord_index.get(local_coord_ as usize).unwrap())?;
        let block = chunk.blocks.get(local_id as usize).unwrap();

        Some(((chunk_id, local_id), block))
    }

    #[inline]
    fn broad_find_with_rect(&self, rect: IRect2) -> impl Iterator<Item = (BlockId, &Block)> + '_ {
        let chunk_rect = rect.div_euclid_i32(Self::CHUNK_SIZE as i32);
        let query = chunk_rect
            .into_iter_ivec2()
            .filter_map(move |chunk_coord| {
                let chunk_id = self.coord_index.get(&chunk_coord)?;
                let chunk = self.chunks.get(*chunk_id as usize).unwrap();
                Some((chunk_coord, chunk))
            });
        let clamp = IRect2::new(IVec2::ZERO, IVec2::ONE) * (Self::CHUNK_SIZE - 1) as i32;
        query.flat_map(move |(chunk_coord, chunk)| {
            let bin_rect = (rect - chunk_coord * Self::CHUNK_SIZE as i32).minimum(clamp) >> 3;
            bin_rect
                .into_iter_ivec2()
                .flat_map(move |bin_coord| {
                    let space_bin_id = (bin_coord.y << 2) | bin_coord.x;
                    let space_bin = chunk.space_bins.get(space_bin_id as usize).unwrap();
                    space_bin.iter().map(move |&(chunk_id, local_id)| {
                        let block = chunk.blocks.get(local_id as usize).unwrap();
                        ((chunk_id, local_id), block)
                    })
                })
        })
    }

    // spatial features

    pub fn find_with_point(&self, coord: IVec2) -> Option<BlockId> {
        self.broad_find_with_point(coord).map(|(id, _)| id)
    }

    pub fn find_with_rect(&self, rect: IRect2) -> impl Iterator<Item = BlockId> + '_ {
        self.broad_find_with_rect(rect)
            .filter(move |(_, block)| {
                let archetype = &self.archetypes[block.archetype_id as usize];
                rect.intersects(archetype.rect(block.coord))
            })
            .map(|(id, _)| id)
    }

    // collision features

    pub fn find_with_collision_point(&self, coord: Vec2) -> impl Iterator<Item = BlockId> + '_ {
        self.find_with_collision_rect(Rect2::new(coord, coord))
    }

    pub fn find_with_collision_rect(&self, rect: Rect2) -> impl Iterator<Item = BlockId> + '_ {
        self.broad_find_with_rect(rect.as_iaabb2())
            .filter(move |(_, block)| {
                let archetype = &self.archetypes[block.archetype_id as usize];
                rect.intersects(archetype.collision_rect(block.coord))
            })
            .map(|(id, _)| id)
    }

    // hint features

    pub fn find_with_hint_point(&self, coord: Vec2) -> impl Iterator<Item = BlockId> + '_ {
        self.find_with_hint_rect(Rect2::new(coord, coord))
    }

    pub fn find_with_hint_rect(&self, rect: Rect2) -> impl Iterator<Item = BlockId> + '_ {
        self.broad_find_with_rect(rect.as_iaabb2())
            .filter(move |(_, block)| {
                let archetype = &self.archetypes[block.archetype_id as usize];
                rect.intersects(archetype.hint_rect(block.coord))
            })
            .map(|(id, _)| id)
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

    fn make_block_field() -> BlockField {
        BlockField::new(BlockFieldInfo {
            blocks: vec![
                BlockInfo {
                    display_name: "block_0".into(),
                    description: "block_0_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                    hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                    y_sorting: false,
                },
                BlockInfo {
                    display_name: "block_1".into(),
                    description: "block_1_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                    hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                    y_sorting: false,
                },
            ],
        })
    }

    #[test]
    #[should_panic]
    fn block_field_with_invalid() {
        let _: BlockField = BlockField::new(BlockFieldInfo {
            blocks: vec![BlockInfo {
                display_name: "block_0".into(),
                description: "block_0_desc".into(),
                size: IVec2::new(-1, -1),
                collision_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                y_sorting: false,
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
                collision_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(-1.0, -1.0)),
                hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                y_sorting: false,
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
                collision_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(-1.0, -1.0)),
                y_sorting: false,
            }],
        });
    }

    #[test]
    fn crud_block() {
        let mut field = make_block_field();

        let id = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                ..Default::default()
            })
            .unwrap();

        let block = field.get(id).unwrap();
        assert_eq!(block.archetype_id, 1);
        assert_eq!(block.coord, IVec2::new(-1, 3));

        assert_eq!(field.find_with_point(IVec2::new(-1, 3)), Some(id));

        let block = field.remove(id).unwrap();
        assert_eq!(block.archetype_id, 1);
        assert_eq!(block.coord, IVec2::new(-1, 3));

        assert_eq!(field.get(id).unwrap_err(), BlockError::NotFound);
        assert_eq!(field.find_with_point(IVec2::new(-1, 3)), None);
        assert_eq!(field.remove(id).unwrap_err(), BlockError::NotFound);
    }

    #[test]
    fn insert_block_with_invalid() {
        let mut field = make_block_field();

        assert_eq!(
            field.insert(Block {
                archetype_id: 2,
                coord: IVec2::new(-1, 3),
                ..Default::default()
            }),
            Err(BlockError::InvalidId)
        );
        assert_eq!(field.find_with_point(IVec2::new(-1, 3)), None);

        let id = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(
            field.insert(Block {
                archetype_id: 0,
                coord: IVec2::new(-1, 3),
                ..Default::default()
            }),
            Err(BlockError::Conflict)
        );

        let block = field.get(id).unwrap();
        assert_eq!(block.archetype_id, 1);
        assert_eq!(block.coord, IVec2::new(-1, 3));

        assert_eq!(field.find_with_point(IVec2::new(-1, 3)), Some(id));
    }

    #[test]
    fn modify_block() {
        let mut field = make_block_field();

        let id = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 4),
                ..Default::default()
            })
            .unwrap();

        let block = field.get(id).unwrap();
        assert_eq!(block.archetype_id, 1);
        assert_eq!(block.coord, IVec2::new(-1, 4));

        assert_eq!(field.find_with_point(IVec2::new(-1, 3)), None);
        assert_eq!(field.find_with_point(IVec2::new(-1, 4)), Some(id));

        let id = field
            .modify(id, |render_state| render_state.variant = 1)
            .unwrap();

        let block = field.get(id).unwrap();
        assert_eq!(block.archetype_id, 1);
        assert_eq!(block.coord, IVec2::new(-1, 4));
        assert_eq!(block.variant, 1);

        let id = field.modify(id, |_| {}).unwrap();

        let block = field.get(id).unwrap();
        assert_eq!(block.archetype_id, 1);
        assert_eq!(block.coord, IVec2::new(-1, 4));
        assert_eq!(block.variant, 1);
    }

    #[test]
    fn modify_block_with_invalid() {
        let mut field = make_block_field();

        let id0 = field
            .insert(Block {
                archetype_id: 0,
                coord: IVec2::new(-1, 3),
                ..Default::default()
            })
            .unwrap();
        let id1 = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 4),
                ..Default::default()
            })
            .unwrap();

        let block = field.get(id0).unwrap();
        assert_eq!(block.archetype_id, 0);
        assert_eq!(block.coord, IVec2::new(-1, 3));

        assert_eq!(field.find_with_point(IVec2::new(-1, 3)), Some(id0));

        field.remove(id1).unwrap();
        assert_eq!(field.modify(id1, |_| {}), Err(BlockError::NotFound));
        assert_eq!(field.get(id1).unwrap_err(), BlockError::NotFound);
        assert_eq!(field.find_with_point(IVec2::new(-1, 4)), None);
    }

    #[test]
    fn move_block() {
        let mut field = make_block_field();

        let id = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                ..Default::default()
            })
            .unwrap();

        field.remove(id).unwrap();
        let id = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 1000),
                ..Default::default()
            })
            .unwrap();

        let block = field.get(id).unwrap();
        assert_eq!(block.archetype_id, 1);
        assert_eq!(block.coord, IVec2::new(-1, 1000));

        assert_eq!(field.find_with_point(IVec2::new(-1, 3)), None);
        assert_eq!(field.find_with_point(IVec2::new(-1, 1000)), Some(id));
    }

    #[test]
    fn collision_block() {
        let mut field = make_block_field();

        let id0 = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                ..Default::default()
            })
            .unwrap();
        let id1 = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 4),
                ..Default::default()
            })
            .unwrap();
        let _ = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 5),
                ..Default::default()
            })
            .unwrap();

        let point = Vec2::new(-1.0, 4.0);
        let vec = field.find_with_collision_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&id0));
        assert!(vec.contains(&id1));

        let rect = Rect2::new(Vec2::new(-1.0, 3.0), Vec2::new(-1.0, 4.0));
        let vec = field.find_with_collision_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&id0));
        assert!(vec.contains(&id1));
    }

    #[test]
    fn hint_block() {
        let mut field = make_block_field();

        let id0 = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                ..Default::default()
            })
            .unwrap();
        let id1 = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 4),
                ..Default::default()
            })
            .unwrap();
        let _ = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 5),
                ..Default::default()
            })
            .unwrap();

        let point = Vec2::new(-1.0, 4.0);
        let vec = field.find_with_hint_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&id0));
        assert!(vec.contains(&id1));

        let rect = Rect2::new(Vec2::new(-1.0, 3.0), Vec2::new(-1.0, 4.0));
        let vec = field.find_with_hint_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&id0));
        assert!(vec.contains(&id1));
    }

    #[test]
    fn block_chunk() {
        let mut field = make_block_field();

        let _= field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                ..Default::default()
            })
            .unwrap();
        let _ = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 4),
                ..Default::default()
            })
            .unwrap();
        let _ = field
            .insert(Block {
                archetype_id: 1,
                coord: IVec2::new(-1, 5),
                ..Default::default()
            })
            .unwrap();

        assert!(field.get_chunk(IVec2::new(0, 0)).is_err());

        let chunk = field.get_chunk(IVec2::new(-1, 0)).unwrap();
        assert_eq!(chunk.blocks.len(), 3);
    }
}
