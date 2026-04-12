use glam::*;

use crate::geom::*;

pub type BlockId = u64;

#[inline]
fn encode_address(page_id: u32, local_id: u32) -> u64 {
    (page_id as u64) << 32 | local_id as u64
}

#[inline]
fn decode_address(address: u64) -> (u32, u32) {
    ((address >> 32) as u32, address as u32)
}

#[inline]
fn encode_coord(coord: IVec2) -> u64 {
    (coord.x as u32 as u64) << 32 | coord.y as u32 as u64
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
        self.collision_rect.map(|rect| rect + coord.as_vec2())
    }

    #[inline]
    pub fn hint_rect(&self, coord: IVec2) -> Rect2 {
         self.hint_rect + coord.as_vec2()
    }

    #[inline]
    pub fn broad_rect(&self, coord: IVec2) -> IRect2 {
        self.broad_rect + coord
    }
}

#[derive(Debug, Clone, Default)]
pub struct Block {
    pub coord: IVec2,
    pub archetype_id: u16,
    pub variant: u16,
    pub tick: u32,
}

#[derive(Debug)]
pub struct BlockPage {
    pub version: u64,
    pub blocks: Vec<Block>,
    pub ids: Vec<BlockId>,
}

#[derive(Debug)]
pub struct BlockField {
    archetypes: Vec<BlockArchetype>,

    id_index: slab::Slab<u64>,
    pages: Vec<BlockPage>,

    coord_index: ahash::AHashMap<u64, u32>,
    spatial_index: SpatialIndex,
}

impl BlockField {
    const PAGE_SIZE: u32 = 32;

    pub fn new(info: BlockFieldInfo) -> Self {
        let mut archetypes = vec![];

        assert!(info.blocks.len() <= u16::MAX as usize, "capacity overflow");
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
                size: block.size,
                collision_rect: block.collision_rect,
                hint_rect: block.hint_rect,
                broad_rect,
                y_sorting: block.y_sorting,
            });
        }

        Self {
            archetypes,
            pages: Default::default(),
            coord_index: Default::default(),
            id_index: Default::default(),
            spatial_index: Default::default(),
        }
    }

    #[inline]
    fn alloc_page(&mut self, coord: IVec2) -> u32 {
        let page_coord = Self::find_page_coord_ivec2(coord);
        let page_key = encode_coord(page_coord);

        if let Some(page_id) = self.coord_index.get(&page_key) {
            *page_id
        } else {
            assert!(self.pages.len() <= u32::MAX as usize, "capacity overflow");
            let page_id = self.pages.len() as u32;
            self.pages.push(BlockPage {
                version: Default::default(),
                blocks: Default::default(),
                ids: Default::default(),
            });
            self.coord_index.insert(page_key, page_id);
            page_id
        }
    }

    pub fn insert(&mut self, block: Block) -> Result<BlockId, BlockError> {
        let page_id = self.alloc_page(block.coord);

        // check by spatial features
        let archetype = self.archetypes.get(block.archetype_id as usize).ok_or(BlockError::InvalidId)?;
        if self.find_with_rect(archetype.rect(block.coord)).next().is_some() {
            return Err(BlockError::Conflict);
        }

        let page = self.pages.get_mut(page_id as usize).unwrap();
        assert!(page.blocks.len() <= u32::MAX as usize, "capacity overflow");
        let local_id = page.blocks.len() as u32;
        let address = encode_address(page_id, local_id);
        let id = self.id_index.insert(address) as u64;

        // register spatial index
        let rect = archetype.rect(block.coord);
        self.spatial_index.insert(rect, id);

        page.blocks.push(block);
        page.ids.push(id);
        page.version += 1;
        Ok(id)
    }

    pub fn remove(&mut self, id: BlockId) -> Result<Block, BlockError> {
        let address = self.id_index.try_remove(id as usize).ok_or(BlockError::NotFound)?;
        let (page_id, local_id) = decode_address(address);

        let page = self.pages.get_mut(page_id as usize).unwrap();
        let block = page.blocks.swap_remove(local_id as usize);
        let _ = page.ids.swap_remove(local_id as usize);

        if let Some(id) = page.ids.get(local_id as usize) {
            *self.id_index.get_mut(*id as usize).unwrap() = address;
        }

        // unregister spatial index
        let archetype = self.archetypes.get(block.archetype_id as usize).unwrap();
        let rect = archetype.rect(block.coord);
        self.spatial_index.remove(rect);

        page.version += 1;
        Ok(block)
    }

    pub fn modify_variant(&mut self, id: BlockId, variant: u16) -> Result<(), BlockError> {
        let address = *self.id_index.get(id as usize).ok_or(BlockError::NotFound)?;
        let (page_id, local_id) = decode_address(address);

        let page = self.pages.get_mut(page_id as usize).unwrap();
        let block = page.blocks.get_mut(local_id as usize).unwrap();
        block.variant = variant;
        page.version += 1;
        Ok(())
    }

    pub fn modify_tick(&mut self, id: BlockId, tick: u32) -> Result<(), BlockError> {
        let address = *self.id_index.get(id as usize).ok_or(BlockError::NotFound)?;
        let (page_id, local_id) = decode_address(address);

        let page = self.pages.get_mut(page_id as usize).unwrap();
        let block = page.blocks.get_mut(local_id as usize).unwrap();
        block.tick = tick;
        page.version += 1;
        Ok(())
    }

    pub fn r#move(&mut self, id: BlockId, new_coord: IVec2) -> Result<(), BlockError> {
        let address = *self.id_index.get(id as usize).ok_or(BlockError::NotFound)?;
        let (page_id, local_id) = decode_address(address);

        let page = self.pages.get(page_id as usize).unwrap();
        let block = page.blocks.get(local_id as usize).unwrap();
        if block.coord == new_coord {
            return Ok(());
        }

        // check by spatial features
        let archetype = self.archetypes.get(block.archetype_id as usize).unwrap();
        if self.find_with_rect(archetype.rect(new_coord)).find(|v| **v != id).is_some() {
            return Err(BlockError::Conflict);
        }

        // update spatial index
        let rect = archetype.rect(block.coord);
        let new_rect = archetype.rect(new_coord);
        self.spatial_index.remove(rect);
        self.spatial_index.insert(new_rect, id);

        // move owner
        let page_coord = Self::find_page_coord_ivec2(block.coord);
        let new_page_coord = Self::find_page_coord_ivec2(new_coord);
        if page_coord != new_page_coord {
            let page = self.pages.get_mut(page_id as usize).unwrap();
            let block = page.blocks.swap_remove(local_id as usize);
            let _ = page.ids.swap_remove(local_id as usize);

            if let Some(id) = page.ids.get(local_id as usize) {
                *self.id_index.get_mut(*id as usize).unwrap() = address;
            }
            page.version += 1;

            let new_page_id = self.alloc_page(new_coord);

            let new_page = self.pages.get_mut(new_page_id as usize).unwrap();
            assert!(new_page.blocks.len() <= u32::MAX as usize, "capacity overflow");
            let new_local_id = new_page.blocks.len() as u32;
            let new_address = encode_address(new_page_id, new_local_id);
            *self.id_index.get_mut(id as usize).unwrap() = new_address;

            new_page.blocks.push(Block { coord: new_coord, ..block });
            new_page.ids.push(id);
            new_page.version += 1;
        } else {
            let page = self.pages.get_mut(page_id as usize).unwrap();
            let block = page.blocks.get_mut(local_id as usize).unwrap();
            block.coord = new_coord;
            page.version += 1;
        }
        Ok(())
    }

    #[inline]
    pub fn get(&self, id: BlockId) -> Result<&Block, BlockError> {
        let address = *self.id_index.get(id as usize).ok_or(BlockError::NotFound)?;
        let (page_id, local_id) = decode_address(address);

        let page = self.pages.get(page_id as usize).unwrap();
        let block = page.blocks.get(local_id as usize).unwrap();

        Ok(block)
    }

    // archetype

    #[inline]
    pub fn get_archetype(&self, archetype_id: u16) -> Result<&BlockArchetype, BlockError> {
        self.archetypes.get(archetype_id as usize).ok_or(BlockError::InvalidId)
    }

    // transfer page data

    #[inline]
    pub fn find_page_coord(coord: Vec2) -> IVec2 {
        coord.div_euclid(Vec2::splat(Self::PAGE_SIZE as f32)).as_ivec2()
    }

    #[inline]
    fn find_page_coord_ivec2(coord: IVec2) -> IVec2 {
        coord.div_euclid(IVec2::splat(Self::PAGE_SIZE as i32))
    }

    #[inline]
    pub fn get_page(&self, page_coord: IVec2) -> Result<&BlockPage, BlockError> {
        let page_key = encode_coord(page_coord);
        let page_id = *self.coord_index.get(&page_key).ok_or(BlockError::NotFound)?;
        let page = self.pages.get(page_id as usize).unwrap();
        Ok(page)
    }

    // spatial features

    #[inline]
    pub fn find_with_point(&self, point: IVec2) -> Option<&BlockId> {
        self.spatial_index.find_point(point)
    }

    #[inline]
    pub fn find_with_rect(&self, rect: IRect2) -> impl Iterator<Item = &BlockId> {
        self.spatial_index.find_rect(rect)
    }

    // collision features

    #[inline]
    pub fn find_with_collision_point(&self, point: Vec2) -> impl Iterator<Item = &BlockId> {
        self.find_with_collision_rect(Rect2::new(point, point))
    }

    #[inline]
    pub fn find_with_collision_rect(&self, rect: Rect2) -> impl Iterator<Item = &BlockId> {
        std::iter::empty()
    }

    // hint features

    #[inline]
    pub fn find_with_hint_point(&self, point: Vec2) -> impl Iterator<Item = &BlockId> {
        self.find_with_hint_rect(Rect2::new(point, point))
    }

    #[inline]
    pub fn find_with_hint_rect(&self, rect: Rect2) -> impl Iterator<Item = &BlockId> {
        std::iter::empty()
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

        let query = field.find_with_point(IVec2::new(-1, 3)).map(|id| *id);
        assert_eq!(query, Some(id));

        let block = field.remove(id).unwrap();
        assert_eq!(block.archetype_id, 1);
        assert_eq!(block.coord, IVec2::new(-1, 3));

        assert_eq!(field.get(id).unwrap_err(), BlockError::NotFound);
        let query = field.find_with_point(IVec2::new(-1, 3)).map(|id| *id);
        assert_eq!(query, None);
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

        let query = field.find_with_point(IVec2::new(-1, 3)).map(|id| *id);
        assert_eq!(query, None);

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

        let query = field.find_with_point(IVec2::new(-1, 3)).map(|id| *id);
        assert_eq!(query, Some(id));
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

        let query = field.find_with_point(IVec2::new(-1, 3)).map(|id| *id);
        assert_eq!(query, None);
        let query = field.find_with_point(IVec2::new(-1, 4)).map(|id| *id);
        assert_eq!(query, Some(id));

        field.modify_variant(id, 1).unwrap();

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

        let query = field.find_with_point(IVec2::new(-1, 3)).map(|id| *id);
        assert_eq!(query, Some(id0));

        field.remove(id1).unwrap();
        assert_eq!(field.modify_variant(id1, 1), Err(BlockError::NotFound));
        assert_eq!(field.get(id1).unwrap_err(), BlockError::NotFound);
        let query = field.find_with_point(IVec2::new(-1, 4)).map(|id| *id);
        assert_eq!(query, None);
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

        field.r#move(id, IVec2::new(-1, 1000)).unwrap();

        let block = field.get(id).unwrap();
        assert_eq!(block.archetype_id, 1);
        assert_eq!(block.coord, IVec2::new(-1, 1000));

        let query = field.find_with_point(IVec2::new(-1, 3)).map(|id| *id);
        assert_eq!(query, None);
        let query = field.find_with_point(IVec2::new(-1, 1000)).map(|id| *id);
        assert_eq!(query, Some(id));
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
        let vec = field.find_with_collision_point(point).map(|id| *id).collect::<Vec<_>>();
        assert!(vec.contains(&id0));
        assert!(vec.contains(&id1));

        let rect = Rect2::new(Vec2::new(-1.0, 3.0), Vec2::new(-1.0, 4.0));
        let vec = field.find_with_collision_rect(rect).map(|id| *id).collect::<Vec<_>>();
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
        let vec = field.find_with_hint_point(point).map(|id| *id).collect::<Vec<_>>();
        assert!(vec.contains(&id0));
        assert!(vec.contains(&id1));

        let rect = Rect2::new(Vec2::new(-1.0, 3.0), Vec2::new(-1.0, 4.0));
        let vec = field.find_with_hint_rect(rect).map(|id| *id).collect::<Vec<_>>();
        assert!(vec.contains(&id0));
        assert!(vec.contains(&id1));
    }

    #[test]
    fn block_page() {
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

        assert!(field.get_page(IVec2::new(0, 0)).is_err());

        let page = field.get_page(IVec2::new(-1, 0)).unwrap();
        assert_eq!(page.blocks.len(), 3);
    }
}
