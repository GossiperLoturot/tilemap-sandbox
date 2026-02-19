use glam::*;

use crate::geom::*;

pub type BlockId = u64;

#[inline]
fn encode_id(chunk_id: u32, local_id: u16) -> BlockId {
    (chunk_id as u64) << 32 | local_id as u64
}

#[inline]
fn decode_id(tile_id: BlockId) -> (u32, u16) {
    ((tile_id >> 32) as u32, tile_id as u16)
}

#[inline]
fn encode_coord(coord: IVec2) -> u64 {
    (coord.x as u32 as u64) << 32 | coord.y as u32 as u64
}

#[derive(Clone, Debug)]
struct BlockSpatialData {
    rect: IRect2,
    collision_rect: Option<Rect2>,
    hint_rect: Rect2,
}

#[derive(Debug, Clone)]
pub struct BlockInfo {
    pub display_name: String,
    pub description: String,
    pub size: IVec2,
    pub collision_rect: Option<Rect2>,
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
    pub collision_rect: Option<Rect2>,
    pub hint_rect: Rect2,
    pub broad_rect: IRect2,
    pub y_sorting: bool,
}

impl BlockArchetype {
    #[inline]
    pub fn rect(&self, coord: IVec2) -> IRect2 {
        IRect2::new(coord, coord + self.size - 1)
    }

    #[inline]
    pub fn collision_rect(&self, coord: IVec2) -> Option<Rect2> {
        self.collision_rect.map(|rect| coord.as_vec2() + rect)
    }

    #[inline]
    pub fn hint_rect(&self, coord: IVec2) -> Rect2 {
        coord.as_vec2() + self.hint_rect
    }

    #[inline]
    pub fn broad_rect(&self, coord: IVec2) -> IRect2 {
        coord + self.broad_rect
    }
}

#[derive(Debug, Clone, Default)]
pub struct BlockModify {
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

#[derive(Debug)]
pub struct BlockChunk {
    pub version: u64,
    pub blocks: slab::Slab<Block>,
}

#[derive(Debug)]
pub struct BlockField {
    archetypes: Vec<BlockArchetype>,
    chunks: Vec<BlockChunk>,
    coord_index: ahash::AHashMap<u64, u32>,
    hgrid: HGrid<BlockSpatialData>,
}

impl BlockField {
    const CHUNK_SIZE: u32 = 32;

    pub fn new(info: BlockFieldInfo) -> Self {
        let mut archetypes = vec![];

        for block in info.blocks {
            if block.size.x <= 0 || block.size.y <= 0 {
                panic!("size must be positive");
            }
            let mut broad_rect = IRect2::new(IVec2::ZERO, block.size);

            if let Some(rect) = &block.collision_rect {
                if rect.size().x < 0.0 || rect.size().y < 0.0 {
                    panic!("collision size must be non-negative");
                }
                broad_rect = broad_rect.maximum(rect.trunc_over().as_irect2());
            }

            if block.hint_rect.size().x < 0.0 || block.hint_rect.size().y < 0.0 {
                panic!("hint size must be non-negative");
            }
            broad_rect = broad_rect.maximum(block.hint_rect.trunc_over().as_irect2());

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
            hgrid: Default::default(),
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
        let chunk_coord_ = encode_coord(chunk_coord);

        // get or allocate chunk
        let chunk_id = if let Some(chunk_id) = self.coord_index.get(&chunk_coord_) {
            *chunk_id
        } else {
            if self.chunks.len() >= u32::MAX as usize {
                panic!("capacity overflow");
            }

            let chunk_id = self.chunks.len() as u32;
            self.chunks.push(BlockChunk {
                version: Default::default(),
                blocks: Default::default(),
            });
            self.coord_index.insert(chunk_coord_, chunk_id);
            chunk_id
        };
        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();

        // block_key is guaranteed to be less than u16::MAX.
        if chunk.blocks.vacant_key() >= u16::MAX as usize {
            panic!("capacity overflow");
        }
        let local_id = chunk.blocks.vacant_key() as u16;
        let id = encode_id(chunk_id, local_id);

        // register spatial index
        let broad_rect = archetype.broad_rect(block.coord);
        self.hgrid.insert(broad_rect, id, BlockSpatialData {
            rect: archetype.rect(block.coord),
            collision_rect: archetype.collision_rect(block.coord),
            hint_rect: archetype.hint_rect(block.coord),
        });

        chunk.blocks.insert(block);
        chunk.version += 1;

        Ok(id)
    }

    pub fn remove(&mut self, id: BlockId) -> Result<Block, BlockError> {
        let (chunk_id, local_id) = decode_id(id);

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let block = chunk.blocks.try_remove(local_id as usize).ok_or(BlockError::NotFound)?;
        chunk.version += 1;

        // unregister spatial index
        let archetype = self.archetypes.get(block.archetype_id as usize).unwrap();
        let broad_rect = archetype.broad_rect(block.coord);
        self.hgrid.remove(broad_rect, id);

        Ok(block)
    }

    pub fn modify(&mut self, id: BlockId, f: impl FnOnce(&mut BlockModify)) -> Result<BlockId, BlockError> {
        let (chunk_id, local_id) = decode_id(id);

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let block = chunk.blocks.get_mut(local_id as usize).ok_or(BlockError::NotFound)?;

        let mut block_modify = BlockModify { variant: block.variant, tick: block.tick };
        f(&mut block_modify);
        block.variant = block_modify.variant;
        block.tick = block_modify.tick;

        chunk.version += 1;
        Ok(id)
    }

    pub fn get(&self, id: BlockId) -> Result<&Block, BlockError> {
        let (chunk_id, local_id) = decode_id(id);
        let chunk = self.chunks.get(chunk_id as usize).unwrap();
        let block = chunk.blocks.get(local_id as usize).ok_or(BlockError::NotFound)?;
        Ok(block)
    }

    // archetype

    #[inline]
    pub fn get_archetype(&self, archetype_id: u16) -> Result<&BlockArchetype, BlockError> {
        self.archetypes.get(archetype_id as usize).ok_or(BlockError::InvalidId)
    }

    // transfer chunk data

    #[inline]
    pub fn find_chunk_coord(&self, coord: Vec2) -> IVec2 {
        let chunk_size = Vec2::splat(Self::CHUNK_SIZE as f32);
        coord.div_euclid(chunk_size).as_ivec2()
    }

    #[inline]
    pub fn get_chunk(&self, chunk_coord: IVec2) -> Result<&BlockChunk, BlockError> {
        let chunk_coord_ = encode_coord(chunk_coord);
        let chunk_id = self.coord_index.get(&chunk_coord_).ok_or(BlockError::NotFound)?;
        let chunk = self.chunks.get(*chunk_id as usize).unwrap();
        Ok(chunk)
    }

    // spatial features

    #[inline]
    pub fn find_with_point(&self, point: IVec2) -> Option<BlockId> {
        self.find_with_rect(IRect2::new(point, point)).next()
    }

    #[inline]
    pub fn find_with_rect(&self, rect: IRect2) -> impl Iterator<Item = BlockId> + '_ {
        self.hgrid.find(rect)
            .map(|(id, data)| (id, data.rect))
            .filter(move |(_, obj_rect)| Intersects::intersects(&rect, obj_rect))
            .map(|(id, _)| *id)
    }

    // collision features

    #[inline]
    pub fn find_with_collision_point(&self, point: Vec2) -> impl Iterator<Item = BlockId> + '_ {
        self.find_with_collision_rect(Rect2::new(point, point))
    }

    #[inline]
    pub fn find_with_collision_rect(&self, rect: Rect2) -> impl Iterator<Item = BlockId> + '_ {
        self.hgrid.find(rect.trunc_over().as_irect2())
            .filter_map(|(id, data)| data.collision_rect.map(|obj_rect| (id, obj_rect)))
            .filter(move |(_, obj_rect)| Intersects::intersects(&rect, obj_rect))
            .map(|(id, _)| *id)
    }

    // hint features

    #[inline]
    pub fn find_with_hint_point(&self, point: Vec2) -> impl Iterator<Item = BlockId> + '_ {
        self.find_with_hint_rect(Rect2::new(point, point))
    }

    #[inline]
    pub fn find_with_hint_rect(&self, rect: Rect2) -> impl Iterator<Item = BlockId> + '_ {
        self.hgrid.find(rect.trunc_over().as_irect2())
            .map(|(id, data)| (id, data.hint_rect))
            .filter(move |(_, obj_rect)| Intersects::intersects(&rect, obj_rect))
            .map(|(id, _)| *id)
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
                    collision_rect: Some(Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0))),
                    hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                    y_sorting: false,
                },
                BlockInfo {
                    display_name: "block_1".into(),
                    description: "block_1_desc".into(),
                    size: IVec2::new(1, 1),
                    collision_rect: Some(Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0))),
                    hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                    y_sorting: false,
                },
            ],
        })
    }

    #[test]
    #[should_panic]
    fn block_field_with_invalid() {
        BlockField::new(BlockFieldInfo {
            blocks: vec![BlockInfo {
                display_name: "block_0".into(),
                description: "block_0_desc".into(),
                size: IVec2::new(-1, -1),
                collision_rect: Some(Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0))),
                hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                y_sorting: false,
            }],
        });
    }

    #[test]
    #[should_panic]
    fn block_field_with_invalid_collision() {
        BlockField::new(BlockFieldInfo {
            blocks: vec![BlockInfo {
                display_name: "block_0".into(),
                description: "block_0_desc".into(),
                size: IVec2::new(1, 1),
                collision_rect: Some(Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(-1.0, -1.0))),
                hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                y_sorting: false,
            }],
        });
    }

    #[test]
    #[should_panic]
    fn block_field_with_invalid_hint() {
        BlockField::new(BlockFieldInfo {
            blocks: vec![BlockInfo {
                display_name: "block_0".into(),
                description: "block_0_desc".into(),
                size: IVec2::new(1, 1),
                collision_rect: Some(Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0))),
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
            .modify(id, |block_modify| block_modify.variant = 1)
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
