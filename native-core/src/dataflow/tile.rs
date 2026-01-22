use glam::*;

use crate::geom::*;

pub type TileId = (u32, u16);

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
    pub fn rect(coord: IVec2) -> IRect2 {
        IRect2::new(coord, coord)
    }

    pub fn collision_rect(coord: IVec2) -> Rect2 {
        Rect2::new(coord.as_vec2(), coord.as_vec2() + 1.0)
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
    coord_index: std::collections::HashMap<IVec2, u32, ahash::RandomState>,
    broad_tree: BroadTree<TileId>,
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
            broad_tree: Default::default(),
        }
    }

    pub fn insert(&mut self, tile: Tile) -> Result<TileId, TileError> {
        let _ = self.archetypes.get(tile.archetype_id as usize).ok_or(TileError::InvalidId)?;

        // check by spatial features
        if self.find_with_point(tile.coord).is_some() {
            return Err(TileError::Conflict);
        }

        let chunk_size = IVec2::splat(Self::CHUNK_SIZE as i32);
        let chunk_coord = tile.coord.div_euclid(chunk_size);

        // get or allocate chunk
        let chunk_id = if let Some(chunk_id) = self.coord_index.get(&chunk_coord) {
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
            self.coord_index.insert(chunk_coord, chunk_id);
            chunk_id
        };
        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();

        // key is guaranteed to be less than u16::MAX.
        if chunk.tiles.vacant_key() >= u16::MAX as usize {
            panic!("capacity overflow");
        }
        let local_id = chunk.tiles.vacant_key() as u16;

        // register spatial index
        let rect = tile.coord + IRect2::new(IVec2::ZERO, IVec2::ONE);
        self.broad_tree.insert(rect, (chunk_id, local_id));

        chunk.tiles.insert(tile);
        chunk.version += 1;

        Ok((chunk_id, local_id))
    }

    pub fn remove(&mut self, id: TileId) -> Result<Tile, TileError> {
        let (chunk_id, local_id) = id;

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let tile = chunk.tiles.try_remove(local_id as usize).ok_or(TileError::NotFound)?;
        chunk.version += 1;

        // unregister spatial index
        let rect = tile.coord + IRect2::new(IVec2::ZERO, IVec2::ONE);
        self.broad_tree.remove(rect, (chunk_id, local_id));

        Ok(tile)
    }

    pub fn modify(&mut self, id: TileId, f: impl FnOnce(&mut TileModify)) -> Result<TileId, TileError> {
        let (chunk_id, local_id) = id;
        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let tile = chunk.tiles.get_mut(local_id as usize).ok_or(TileError::NotFound)?;
        let mut tile_modify = TileModify { variant: tile.variant, tick: tile.tick };
        f(&mut tile_modify);
        tile.variant = tile_modify.variant;
        tile.tick = tile_modify.tick;
        chunk.version += 1;
        Ok(id)
    }

    pub fn get(&self, id: TileId) -> Result<&Tile, TileError> {
        let (chunk_id, local_id) = id;
        let chunk = self.chunks.get(chunk_id as usize).unwrap();
        let tile = chunk.tiles.get(local_id as usize).ok_or(TileError::NotFound)?;
        Ok(tile)
    }

    // archetype

    pub fn get_archetype(&self, archetype_id: u16) -> Result<&TileArchetype, TileError> {
        self.archetypes.get(archetype_id as usize).ok_or(TileError::InvalidId)
    }

    // transfer chunk data

    pub fn find_chunk_coord(&self, coord: Vec2) -> IVec2 {
        let chunk_size = Vec2::splat(Self::CHUNK_SIZE as f32);
        coord.div_euclid(chunk_size).as_ivec2()
    }

    pub fn get_chunk(&self, chunk_coord: IVec2) -> Result<&TileChunk, TileError> {
        let chunk_id = self.coord_index.get(&chunk_coord).ok_or(TileError::NotFound)?;
        let chunk = self.chunks.get(*chunk_id as usize).unwrap();
        Ok(chunk)
    }

    // spatial features

    pub fn find_with_point(&self, point: IVec2) -> Option<TileId> {
        self.find_with_rect(IRect2::new(point, point)).next()
    }

    pub fn find_with_rect(&self, rect: IRect2) -> impl Iterator<Item = TileId> + '_ {
        self.broad_tree.find(rect)
            .filter(move |id| {
                let tile = self.get(*id).unwrap();
                Intersects::intersects(&rect, &TileArchetype::rect(tile.coord))
            })
    }

    // collision features

    pub fn find_with_collision_point(&self, point: Vec2) -> impl Iterator<Item = TileId> + '_ {
        self.find_with_collision_rect(Rect2::new(point, point))
    }

    pub fn find_with_collision_rect(&self, rect: Rect2) -> impl Iterator<Item = TileId> + '_ {
        self.broad_tree.find(rect.floor().as_irect2())
            .filter(move |id| {
                let tile = self.get(*id).unwrap();
                let archetype = &self.archetypes[tile.archetype_id as usize];
                archetype.collision && Intersects::intersects(&rect, &TileArchetype::collision_rect(tile.coord))
            })
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

        field.remove(id).unwrap();
        let id = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 1000),
                ..Default::default()
            })
            .unwrap();

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
