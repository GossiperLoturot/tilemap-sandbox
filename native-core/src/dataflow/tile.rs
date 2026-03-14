use glam::*;

use crate::geom::*;

pub type TileId = u64;

#[inline]
fn encode_address(chunk_id: u32, local_id: u32) -> u64 {
    (chunk_id as u64) << 32 | local_id as u64
}

#[inline]
fn decode_address(address: u64) -> (u32, u32) {
    ((address >> 32) as u32, address as u32)
}

#[inline]
fn encode_coord(coord: IVec2) -> u64 {
    (coord.x as u32 as u64) << 32 | coord.y as u32 as u64
}

// locality of reference
#[derive(Debug, Clone)]
pub struct TileSpatialData {
    pub rect: IRect2,
    pub collision_rect: Option<Rect2>,
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
pub struct Tile {
    pub coord: IVec2,
    pub archetype_id: u16,
    pub variant: u16,
    pub tick: u32,
}

#[derive(Debug)]
pub struct TileChunk {
    pub version: u64,
    pub tiles: Vec<Tile>,
    pub ids: Vec<u64>,
}

#[derive(Debug)]
pub struct TileField {
    archetypes: Vec<TileArchetype>,
    chunks: Vec<TileChunk>,
    coord_index: ahash::AHashMap<u64, u32>,
    id_index: slab::Slab<u64>,
    hgrid: HGrid<TileSpatialData>,
}

impl TileField {
    const CHUNK_SIZE: u32 = 32;

    pub fn new(info: TileFieldInfo) -> Self {
        let mut archetypes = vec![];

        assert!(info.tiles.len() <= u16::MAX as usize, "capacity overflow");
        for tile in info.tiles {
            archetypes.push(TileArchetype {
                collision: tile.collision,
            });
        }

        Self {
            archetypes,
            chunks: Default::default(),
            coord_index: Default::default(),
            id_index: Default::default(),
            hgrid: Default::default(),
        }
    }

    #[inline]
    fn alloc_chunk(&mut self, coord: IVec2) -> u32 {
        let chunk_coord = Self::find_chunk_coord_internal(coord);
        let chunk_coord_ = encode_coord(chunk_coord);

        if let Some(chunk_id) = self.coord_index.get(&chunk_coord_) {
            *chunk_id
        } else {
            assert!(self.chunks.len() <= u32::MAX as usize, "capacity overflow");
            let chunk_id = self.chunks.len() as u32;
            self.chunks.push(TileChunk {
                version: Default::default(),
                tiles: Default::default(),
                ids: Default::default(),
            });
            self.coord_index.insert(chunk_coord_, chunk_id);
            chunk_id
        }
    }

    pub fn insert(&mut self, tile: Tile) -> Result<TileId, TileError> {
        let chunk_id = self.alloc_chunk(tile.coord);

        // check by spatial features
        let archetype = self.archetypes.get(tile.archetype_id as usize).ok_or(TileError::InvalidId)?;
        if self.find_with_point(tile.coord).is_some() {
            return Err(TileError::Conflict);
        }

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        assert!(chunk.tiles.len() <= u32::MAX as usize, "capacity overflow");
        let local_id = chunk.tiles.len() as u32;
        let address = encode_address(chunk_id, local_id);
        let id = self.id_index.insert(address) as u64;

        // register spatial index
        let broad_rect = TileArchetype::broad_rect(tile.coord);
        self.hgrid.insert(broad_rect, id, TileSpatialData {
            rect: TileArchetype::rect(tile.coord),
            collision_rect: archetype.collision_rect(tile.coord),
        });

        chunk.tiles.push(tile);
        chunk.ids.push(id);
        chunk.version += 1;
        Ok(id)
    }

    pub fn remove(&mut self, id: TileId) -> Result<Tile, TileError> {
        let address = self.id_index.try_remove(id as usize).ok_or(TileError::NotFound)?;
        let (chunk_id, local_id) = decode_address(address);

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let tile = chunk.tiles.swap_remove(local_id as usize);
        let _ = chunk.ids.swap_remove(local_id as usize);

        if let Some(id) = chunk.ids.get(local_id as usize) {
            *self.id_index.get_mut(*id as usize).unwrap() = address;
        }

        // unregister spatial index
        let broad_rect = TileArchetype::broad_rect(tile.coord);
        self.hgrid.remove(broad_rect, id);

        chunk.version += 1;
        Ok(tile)
    }

    pub fn modify_variant(&mut self, id: TileId, variant: u16) -> Result<(), TileError> {
        let address = *self.id_index.get(id as usize).ok_or(TileError::NotFound)?;
        let (chunk_id, local_id) = decode_address(address);

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let tile = chunk.tiles.get_mut(local_id as usize).unwrap();
        tile.variant = variant;
        chunk.version += 1;
        Ok(())
    }

    pub fn modify_tick(&mut self, id: TileId, tick: u32) -> Result<(), TileError> {
        let address = *self.id_index.get(id as usize).ok_or(TileError::NotFound)?;
        let (chunk_id, local_id) = decode_address(address);

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let tile = chunk.tiles.get_mut(local_id as usize).unwrap();
        tile.tick = tick;
        chunk.version += 1;
        Ok(())
    }

    pub fn r#move(&mut self, id: TileId, new_coord: IVec2) -> Result<(), TileError> {
        let address = *self.id_index.get(id as usize).ok_or(TileError::NotFound)?;
        let (chunk_id, local_id) = decode_address(address);

        let chunk = self.chunks.get(chunk_id as usize).unwrap();
        let tile = chunk.tiles.get(local_id as usize).unwrap();
        if tile.coord == new_coord {
            return Ok(());
        }

        // check by spatial features
        let archetype = self.archetypes.get(tile.archetype_id as usize).unwrap();
        if self.find_with_point(new_coord).is_some() {
            return Err(TileError::Conflict);
        }

        // update spatial index
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

        // move owner
        let chunk_coord = Self::find_chunk_coord_internal(tile.coord);
        let new_chunk_coord = Self::find_chunk_coord_internal(new_coord);
        if new_chunk_coord != chunk_coord {
            let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
            let tile = chunk.tiles.swap_remove(local_id as usize);
            let _ = chunk.ids.swap_remove(local_id as usize);

            if let Some(id) = chunk.ids.get(local_id as usize) {
                *self.id_index.get_mut(*id as usize).unwrap() = address;
            }
            chunk.version += 1;

            let new_chunk_id = self.alloc_chunk(new_coord);

            let new_chunk = self.chunks.get_mut(new_chunk_id as usize).unwrap();
            assert!(new_chunk.tiles.len() <= u32::MAX as usize, "capacity overflow");
            let new_local_id = new_chunk.tiles.len() as u32;
            let new_address = encode_address(new_chunk_id, new_local_id);
            *self.id_index.get_mut(id as usize).unwrap() = new_address;

            new_chunk.tiles.push(Tile { coord: new_coord, ..tile });
            new_chunk.ids.push(id);
            new_chunk.version += 1;
        } else {
            let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
            let tile = chunk.tiles.get_mut(local_id as usize).unwrap();
            tile.coord = new_coord;
            chunk.version += 1;
        }
        Ok(())
    }

    #[inline]
    pub fn get(&self, id: TileId) -> Result<&Tile, TileError> {
        let address = *self.id_index.get(id as usize).ok_or(TileError::NotFound)?;
        let (chunk_id, local_id) = decode_address(address);

        let chunk = self.chunks.get(chunk_id as usize).unwrap();
        let tile = chunk.tiles.get(local_id as usize).unwrap();

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
        coord.div_euclid(Vec2::splat(Self::CHUNK_SIZE as f32)).as_ivec2()
    }

    #[inline]
    fn find_chunk_coord_internal(coord: IVec2) -> IVec2 {
        coord.div_euclid(IVec2::splat(Self::CHUNK_SIZE as i32))
    }

    #[inline]
    pub fn get_chunk(&self, chunk_coord: IVec2) -> Result<&TileChunk, TileError> {
        let chunk_coord_ = encode_coord(chunk_coord);
        let chunk_id = *self.coord_index.get(&chunk_coord_).ok_or(TileError::NotFound)?;
        let chunk = self.chunks.get(chunk_id as usize).unwrap();
        Ok(chunk)
    }

    // spatial features

    #[inline]
    pub fn find_with_point(&self, point: IVec2) -> Option<(&TileId, &TileSpatialData)> {
        self.find_with_rect(IRect2::new(point, point)).next()
    }

    #[inline]
    pub fn find_with_rect(&self, rect: IRect2) -> impl Iterator<Item = (&TileId, &TileSpatialData)> {
        self.hgrid.find(rect)
            .filter(move |(_, data)| Intersects::intersects(&rect, &data.rect))
    }

    // collision features

    #[inline]
    pub fn find_with_collision_point(&self, point: Vec2) -> impl Iterator<Item = (&TileId, &TileSpatialData)> {
        self.find_with_collision_rect(Rect2::new(point, point))
    }

    #[inline]
    pub fn find_with_collision_rect(&self, rect: Rect2) -> impl Iterator<Item = (&TileId, &TileSpatialData)> {
        self.hgrid.find(rect.trunc_over().as_irect2())
            .filter(move |(_, data)| data.collision_rect.map(|obj_rect| Intersects::intersects(&rect, &obj_rect)).unwrap_or(false))
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

        let query = field.find_with_point(IVec2::new(-1, 3)).map(|(id, _)| *id);
        assert_eq!(query, Some(id));

        let tile = field.remove(id).unwrap();
        assert_eq!(tile.archetype_id, 1);
        assert_eq!(tile.coord, IVec2::new(-1, 3));

        assert_eq!(field.get(id).unwrap_err(), TileError::NotFound);
        let query = field.find_with_point(IVec2::new(-1, 3)).map(|(id, _)| *id);
        assert_eq!(query, None);
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

        let query = field.find_with_point(IVec2::new(-1, 3)).map(|(id, _)| *id);
        assert_eq!(query, None);

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

        let query = field.find_with_point(IVec2::new(-1, 3)).map(|(id, _)| *id);
        assert_eq!(query, Some(id));
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

        let query = field.find_with_point(IVec2::new(-1, 3)).map(|(id, _)| *id);
        assert_eq!(query, None);
        let query = field.find_with_point(IVec2::new(-1, 4)).map(|(id, _)| *id);
        assert_eq!(query, Some(id));

        field.modify_variant(id, 1).unwrap();

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

        let query = field.find_with_point(IVec2::new(-1, 3)).map(|(id, _)| *id);
        assert_eq!(query, Some(id0));

        field.remove(id1).unwrap();
        assert_eq!(field.modify_variant(id1, 1), Err(TileError::NotFound));
        assert_eq!(field.get(id1).unwrap_err(), TileError::NotFound);
        let query = field.find_with_point(IVec2::new(-1, 4)).map(|(id, _)| *id);
        assert_eq!(query, None);
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

        field.r#move(id, IVec2::new(-1, 1000)).unwrap();

        let tile = field.get(id).unwrap();
        assert_eq!(tile.archetype_id, 1);
        assert_eq!(tile.coord, IVec2::new(-1, 1000));

        let query = field.find_with_point(IVec2::new(-1, 3)).map(|(id, _)| *id);
        assert_eq!(query, None);
        let query = field.find_with_point(IVec2::new(-1, 1000)).map(|(id, _)| *id);
        assert_eq!(query, Some(id));
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
        let vec = field.find_with_collision_point(point).map(|(id, _)| *id).collect::<Vec<_>>();
        assert!(vec.contains(&id0));
        assert!(vec.contains(&id1));

        let rect = Rect2::new(Vec2::new(-1.0, 3.0), Vec2::new(-1.0, 4.0));
        let vec = field.find_with_collision_rect(rect).map(|(id, _)| *id).collect::<Vec<_>>();
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
