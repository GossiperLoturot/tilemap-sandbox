use glam::*;

use crate::geom::*;

pub type TileId = u64;

#[inline]
fn encode_id(chunk_id: u32, local_id: u16) -> TileId {
    (chunk_id as u64) << 32 | local_id as u64
}

#[inline]
fn decode_id(tile_id: TileId) -> (u32, u16) {
    ((tile_id >> 32) as u32, tile_id as u16)
}

#[inline]
fn encode_coord(coord: IVec2) -> u64 {
    (coord.x as u32 as u64) << 32 | coord.y as u32 as u64
}

// locality of reference
#[derive(Debug, Clone)]
struct TileSpatialData {
    rect: IRect2,
    collision_rect: Option<Rect2>,
}

#[derive(Debug, Clone)]
pub struct TileInfo {
    pub display_name: String,
    pub description: String,
    pub collision: bool,
}

#[derive(Debug, Clone)]
pub struct TileFieldInfo {
    pub tiles: Vec<TileInfo>,
}

#[derive(Debug, Clone)]
pub struct TileArchetype {
    pub display_name: String,
    pub description: String,
    pub collision: bool,
}

impl TileArchetype {
    #[inline]
    pub fn rect(coord: IVec2) -> IRect2 {
        IRect2::new(coord, coord)
    }

    #[inline]
    pub fn collision_rect(&self, coord: IVec2) -> Option<Rect2> {
        if !self.collision {
            return None;
        }

        Some(Rect2::new(coord.as_vec2(), coord.as_vec2() + 1.0))
    }

    #[inline]
    pub fn broad_rect(coord: IVec2) -> IRect2 {
        IRect2::new(coord, coord + 1)
    }
}

#[derive(Debug, Clone, Default)]
pub struct TileModify {
    pub variant: u8,
    pub tick: u32,
}

#[derive(Debug, Clone, Default)]
pub struct Tile {
    pub archetype_id: u16,
    pub coord: IVec2,
    pub variant: u8,
    pub tick: u32,
}

#[derive(Debug)]
pub struct TileChunk {
    pub version: u64,
    pub tiles: slab::Slab<Tile>,
}

#[derive(Debug)]
pub struct TileField {
    archetypes: Vec<TileArchetype>,
    chunks: Vec<TileChunk>,
    coord_index: ahash::AHashMap<u64, u32>,
    hgrid: HGrid<TileSpatialData>,
}

impl TileField {
    const CHUNK_SIZE: u32 = 32;

    pub fn new(info: TileFieldInfo) -> Self {
        let mut archetypes = vec![];

        for tile in info.tiles {
            archetypes.push(TileArchetype {
                display_name: tile.display_name,
                description: tile.description,
                collision: tile.collision,
            });
        }

        Self {
            archetypes,
            chunks: Default::default(),
            coord_index: Default::default(),
            hgrid: Default::default(),
        }
    }

    pub fn insert(&mut self, tile: Tile) -> Result<TileId, TileError> {
        let archetype = self.archetypes.get(tile.archetype_id as usize).ok_or(TileError::InvalidId)?;

        // check by spatial features
        if self.find_with_point(tile.coord).is_some() {
            return Err(TileError::Conflict);
        }

        let chunk_size = IVec2::splat(Self::CHUNK_SIZE as i32);
        let chunk_coord = tile.coord.div_euclid(chunk_size);
        let chunk_coord_ = encode_coord(chunk_coord);

        // get or allocate chunk
        let chunk_id = if let Some(chunk_id) = self.coord_index.get(&chunk_coord_) {
            *chunk_id
        } else {
            if self.chunks.len() >= u32::MAX as usize {
                panic!("capacity overflow");
            }
            let chunk_id = self.chunks.len() as u32;
            self.chunks.push(TileChunk {
                version: 0,
                tiles: Default::default(),
            });
            self.coord_index.insert(chunk_coord_, chunk_id);
            chunk_id
        };
        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();

        // key is guaranteed to be less than u16::MAX.
        if chunk.tiles.vacant_key() >= u16::MAX as usize {
            panic!("capacity overflow");
        }
        let local_id = chunk.tiles.vacant_key() as u16;
        let id = encode_id(chunk_id, local_id);

        // register spatial index
        let broad_rect = TileArchetype::broad_rect(tile.coord);
        self.hgrid.insert(broad_rect, id, TileSpatialData {
            rect: TileArchetype::rect(tile.coord),
            collision_rect: archetype.collision_rect(tile.coord),
        });

        chunk.tiles.insert(tile);
        chunk.version += 1;

        Ok(id)
    }

    pub fn remove(&mut self, id: TileId) -> Result<Tile, TileError> {
        let (chunk_id, local_id) = decode_id(id);

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let tile = chunk.tiles.try_remove(local_id as usize).ok_or(TileError::NotFound)?;
        chunk.version += 1;

        // unregister spatial index
        let broad_rect = TileArchetype::broad_rect(tile.coord);
        self.hgrid.remove(broad_rect, id);

        Ok(tile)
    }

    pub fn modify(&mut self, id: TileId, f: impl FnOnce(&mut TileModify)) -> Result<TileId, TileError> {
        let (chunk_id, local_id) = decode_id(id);

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let tile = chunk.tiles.get_mut(local_id as usize).ok_or(TileError::NotFound)?;

        let mut tile_modify = TileModify { variant: tile.variant, tick: tile.tick };
        f(&mut tile_modify);
        tile.variant = tile_modify.variant;
        tile.tick = tile_modify.tick;

        chunk.version += 1;
        Ok(id)
    }

    pub fn r#move(&mut self, id: TileId, new_coord: IVec2) -> Result<TileId, TileError> {
        let (chunk_id, local_id) = decode_id(id);

        let chunk = self.chunks.get(chunk_id as usize).unwrap();
        let tile = chunk.tiles.get(local_id as usize).ok_or(TileError::NotFound)?;
        if tile.coord == new_coord {
            return Ok(id);
        }

        // move spatial memory
        let chunk_size = IVec2::splat(Self::CHUNK_SIZE as i32);
        let chunk_coord = tile.coord.div_euclid(chunk_size);
        let new_chunk_coord = new_coord.div_euclid(chunk_size);
        if new_chunk_coord != chunk_coord {
            let new_id = self.insert(Tile { coord: new_coord, ..tile.clone() })?;
            self.remove(id).unwrap(); // for transaction rollback
            return Ok(new_id)
        }

        // update spatial index
        let archetype = self.get_archetype(tile.archetype_id)?;
        let broad_rect = TileArchetype::broad_rect(tile.coord);
        let new_broad_rect = TileArchetype::broad_rect(new_coord);
        if self.hgrid.check_move(broad_rect, new_broad_rect) {
            let value = TileSpatialData {
                rect: TileArchetype::rect(new_coord),
                collision_rect: archetype.collision_rect(new_coord),
            };
            self.hgrid.remove(broad_rect, id);
            self.hgrid.insert(new_broad_rect, id, value);
        }

        // move in same spatial memory
        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let tile = chunk.tiles.get_mut(local_id as usize).unwrap();
        tile.coord = new_coord;

        chunk.version += 1;
        Ok(id)
    }

    #[inline]
    pub fn get(&self, id: TileId) -> Result<&Tile, TileError> {
        let (chunk_id, local_id) = decode_id(id);
        let chunk = self.chunks.get(chunk_id as usize).unwrap();
        let tile = chunk.tiles.get(local_id as usize).ok_or(TileError::NotFound)?;
        Ok(tile)
    }

    // archetype

    #[inline]
    pub fn get_archetype(&self, archetype_id: u16) -> Result<&TileArchetype, TileError> {
        self.archetypes.get(archetype_id as usize).ok_or(TileError::InvalidId)
    }

    // transfer chunk data

    #[inline]
    pub fn find_chunk_coord(&self, coord: Vec2) -> IVec2 {
        let chunk_size = Vec2::splat(Self::CHUNK_SIZE as f32);
        coord.div_euclid(chunk_size).as_ivec2()
    }

    #[inline]
    pub fn get_chunk(&self, chunk_coord: IVec2) -> Result<&TileChunk, TileError> {
        let chunk_coord_ = encode_coord(chunk_coord);
        let chunk_id = self.coord_index.get(&chunk_coord_).ok_or(TileError::NotFound)?;
        let chunk = self.chunks.get(*chunk_id as usize).unwrap();
        Ok(chunk)
    }

    // spatial features

    #[inline]
    pub fn find_with_point(&self, point: IVec2) -> Option<TileId> {
        self.find_with_rect(IRect2::new(point, point)).next()
    }

    #[inline]
    pub fn find_with_rect(&self, rect: IRect2) -> impl Iterator<Item = TileId> + '_ {
        self.hgrid.find(rect)
            .map(|(id, data)| (id, data.rect))
            .filter(move |(_, obj_rect)| Intersects::intersects(&rect, obj_rect))
            .map(|(id, _)| *id)
    }

    // collision features

    #[inline]
    pub fn find_with_collision_point(&self, point: Vec2) -> impl Iterator<Item = TileId> + '_ {
        self.find_with_collision_rect(Rect2::new(point, point))
    }

    #[inline]
    pub fn find_with_collision_rect(&self, rect: Rect2) -> impl Iterator<Item = TileId> + '_ {
        self.hgrid.find(rect.trunc_over().as_irect2())
            .filter_map(|(id, data)| data.collision_rect.map(|obj_rect| (id, obj_rect)))
            .filter(move |(_, obj_rect)| Intersects::intersects(&rect, obj_rect))
            .map(|(id, _)| *id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TileError {
    NotFound,
    Conflict,
    InvalidId,
}

impl std::fmt::Display for TileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "not found error"),
            Self::Conflict => write!(f, "conflict error"),
            Self::InvalidId => write!(f, "invalid id error"),
        }
    }
}

impl std::error::Error for TileError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tile_field() -> TileField {
        TileField::new(TileFieldInfo {
            tiles: vec![
                TileInfo {
                    display_name: "tile_0".into(),
                    description: "tile_0_desc".into(),
                    collision: true,
                },
                TileInfo {
                    display_name: "tile_1".into(),
                    description: "tile_1_desc".into(),
                    collision: true,
                },
            ],
        })
    }

    #[test]
    fn crud_tile() {
        let mut field = make_tile_field();

        let id = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                ..Default::default()
            })
            .unwrap();

        let tile = field.get(id).unwrap();
        assert_eq!(tile.archetype_id, 1);
        assert_eq!(tile.coord, IVec2::new(-1, 3));

        assert_eq!(field.find_with_point(IVec2::new(-1, 3)), Some(id));

        let tile = field.remove(id).unwrap();
        assert_eq!(tile.archetype_id, 1);
        assert_eq!(tile.coord, IVec2::new(-1, 3));

        assert_eq!(field.get(id).unwrap_err(), TileError::NotFound);
        assert_eq!(field.find_with_point(IVec2::new(-1, 3)), None);
        assert_eq!(field.remove(id).unwrap_err(), TileError::NotFound);
    }

    #[test]
    fn insert_tile_with_invalid() {
        let mut field = make_tile_field();

        assert_eq!(
            field.insert(Tile {
                archetype_id: 2,
                coord: IVec2::new(-1, 3),
                ..Default::default()
            }),
            Err(TileError::InvalidId)
        );
        assert_eq!(field.find_with_point(IVec2::new(-1, 3)), None);

        let id = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(
            field.insert(Tile {
                archetype_id: 0,
                coord: IVec2::new(-1, 3),
                ..Default::default()
            }),
            Err(TileError::Conflict)
        );

        let tile = field.get(id).unwrap();
        assert_eq!(tile.archetype_id, 1);
        assert_eq!(tile.coord, IVec2::new(-1, 3));

        assert_eq!(field.find_with_point(IVec2::new(-1, 3)), Some(id));
    }

    #[test]
    fn modify_tile() {
        let mut field = make_tile_field();

        let id = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 4),
                ..Default::default()
            })
            .unwrap();

        let tile = field.get(id).unwrap();
        assert_eq!(tile.archetype_id, 1);
        assert_eq!(tile.coord, IVec2::new(-1, 4));

        assert_eq!(field.find_with_point(IVec2::new(-1, 3)), None);
        assert_eq!(field.find_with_point(IVec2::new(-1, 4)), Some(id));

        let id = field
            .modify(id, |tile_modify| tile_modify.variant = 1)
            .unwrap();

        let tile = field.get(id).unwrap();
        assert_eq!(tile.archetype_id, 1);
        assert_eq!(tile.coord, IVec2::new(-1, 4));
        assert_eq!(tile.variant, 1);

        let id = field.modify(id, |_| {}).unwrap();

        let tile = field.get(id).unwrap();
        assert_eq!(tile.archetype_id, 1);
        assert_eq!(tile.coord, IVec2::new(-1, 4));
        assert_eq!(tile.variant, 1);
    }

    #[test]
    fn modify_tile_with_invalid() {
        let mut field = make_tile_field();

        let id0 = field
            .insert(Tile {
                archetype_id: 0,
                coord: IVec2::new(-1, 3),
                ..Default::default()
            })
            .unwrap();
        let id1 = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 4),
                ..Default::default()
            })
            .unwrap();

        let tile = field.get(id0).unwrap();
        assert_eq!(tile.archetype_id, 0);
        assert_eq!(tile.coord, IVec2::new(-1, 3));

        assert_eq!(field.find_with_point(IVec2::new(-1, 3)), Some(id0));

        field.remove(id1).unwrap();
        assert_eq!(field.modify(id1, |_| {}), Err(TileError::NotFound));
        assert_eq!(field.get(id1).unwrap_err(), TileError::NotFound);
        assert_eq!(field.find_with_point(IVec2::new(-1, 4)), None);
    }

    #[test]
    fn move_tile() {
        let mut field = make_tile_field();

        let id = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                ..Default::default()
            })
            .unwrap();

        let id = field.r#move(id, IVec2::new(-1, 1000)).unwrap();

        let tile = field.get(id).unwrap();
        assert_eq!(tile.archetype_id, 1);
        assert_eq!(tile.coord, IVec2::new(-1, 1000));

        assert_eq!(field.find_with_point(IVec2::new(-1, 3)), None);
        assert_eq!(field.find_with_point(IVec2::new(-1, 1000)), Some(id));
    }

    #[test]
    fn collision_tile() {
        let mut field = make_tile_field();

        let id0 = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                ..Default::default()
            })
            .unwrap();
        let id1 = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 4),
                ..Default::default()
            })
            .unwrap();
        let _ = field
            .insert(Tile {
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
    fn tile_chunk() {
        let mut field = make_tile_field();

        let _ = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                ..Default::default()
            })
            .unwrap();
        let _ = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 4),
                ..Default::default()
            })
            .unwrap();
        let _ = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 5),
                ..Default::default()
            })
            .unwrap();

        assert!(field.get_chunk(IVec2::new(0, 0)).is_err());

        let chunk = field.get_chunk(IVec2::new(-1, 0)).unwrap();
        assert_eq!(chunk.tiles.len(), 3);
    }
}
