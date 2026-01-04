use super::*;

type RectNode<T, U> = rstar::primitives::GeomWithData<rstar::primitives::Rectangle<T>, U>;

pub type TileId = (u32, u32);

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
struct TileArchetype {
    display_name: String,
    description: String,
    collision: bool,
}

impl TileArchetype {
    #[rustfmt::skip]
    fn collision_rect(&self, coord: IVec2) -> Option<[Vec2; 2]> {
        if !self.collision {
            return None;
        }

        Some([coord.as_vec2(), coord.as_vec2() + 1.0])
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TileRenderState {
    pub variant: u8,
    pub tick: u32,
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub archetype_id: u16,
    pub coord: IVec2,
    pub data: Box<dyn TileData>,  // TODO: removal
    pub render_state: TileRenderState,
}

#[derive(Debug, Clone)]
struct TileChunk {
    version: u64,
    tiles: slab::Slab<Tile>,
}

#[derive(Debug, Clone)]
pub struct TileField {
    archetypes: Vec<TileArchetype>,
    chunks: Vec<TileChunk>,
    chunk_index: ahash::AHashMap<IVec2, u32>,
    spatial_index: ahash::AHashMap<IVec2, TileId>,
    collision_index: rstar::RTree<RectNode<[f32; 2], TileId>>,
}

impl TileField {
    const CHUNK_SIZE: u32 = 32;

    pub fn new(desc: TileFieldInfo) -> Self {
        let mut archetypes = vec![];

        for tile in desc.tiles {
            archetypes.push(TileArchetype {
                display_name: tile.display_name,
                description: tile.description,
                collision: tile.collision,
            });
        }

        Self {
            archetypes,
            chunks: Default::default(),
            chunk_index: Default::default(),
            spatial_index: Default::default(),
            collision_index: Default::default(),
        }
    }

    pub fn insert(&mut self, tile: Tile) -> Result<TileId, TileError> {
        let archetype = self
            .archetypes
            .get(tile.archetype_id as usize)
            .ok_or(TileError::InvalidId)?;

        // check by spatial features
        if self.has_by_point(tile.coord) {
            return Err(TileError::Conflict);
        }

        let chunk_size = IVec2::splat(Self::CHUNK_SIZE as i32);
        let chunk_coord = tile.coord.div_euclid(chunk_size);

        // get or allocate chunk
        let chunk_id = if let Some(chunk_id) = self.chunk_index.get(&chunk_coord) {
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
            self.chunk_index.insert(chunk_coord, chunk_id);
            chunk_id
        };

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();

        if chunk.tiles.vacant_key() >= u32::MAX as usize {
            panic!("capacity overflow");
        }

        let local_id = chunk.tiles.vacant_key() as u32;

        // spatial features
        self.spatial_index.insert(tile.coord, (chunk_id, local_id));

        // collision features
        if let Some(rect) = archetype.collision_rect(tile.coord) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = rstar::primitives::GeomWithData::new(rect, (chunk_id, local_id));
            self.collision_index.insert(node);
        }

        // key is guaranteed to be less than u32::MAX.
        chunk.tiles.insert(tile);
        chunk.version += 1;

        Ok((chunk_id, local_id))
    }

    pub fn remove(&mut self, id: TileId) -> Result<Tile, TileError> {
        let (chunk_id, local_id) = id;

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let tile = chunk
            .tiles
            .try_remove(local_id as usize)
            .ok_or(TileError::NotFound)?;
        chunk.version += 1;

        let archetype = self.archetypes.get(tile.archetype_id as usize).unwrap();

        // spatial features
        self.spatial_index.remove(&tile.coord).unwrap();

        // collision features
        if let Some(rect) = archetype.collision_rect(tile.coord) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = rstar::primitives::GeomWithData::new(rect, id);
            self.collision_index.remove(&node).unwrap();
        }

        Ok(tile)
    }

    pub fn modify(
        &mut self,
        id: TileId,
        f: impl FnOnce(&mut Tile),
    ) -> Result<TileId, TileError> {
        let (chunk_id, local_id) = id;

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let tile = chunk
            .tiles
            .get_mut(local_id as usize)
            .ok_or(TileError::NotFound)?;

        let mut new_tile = Tile {
            archetype_id: tile.archetype_id,
            coord: tile.coord,
            data: std::mem::take(&mut tile.data),
            render_state: tile.render_state.clone(),
        };
        f(&mut new_tile);

        if new_tile.archetype_id != tile.archetype_id {
            tile.data = new_tile.data;
            return Err(TileError::InvalidId);
        }

        if new_tile.coord != tile.coord {
            // check by spatial features
            if self.has_by_point(new_tile.coord) {
                return Err(TileError::Conflict);
            }

            self.remove(id).unwrap();
            let new_id = self.insert(new_tile).unwrap();
            return Ok(new_id);
        }

        if new_tile.render_state != tile.render_state {
            let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
            *chunk.tiles.get_mut(local_id as usize).unwrap() = new_tile;
            chunk.version += 1;
            return Ok(id);
        }

        tile.data = new_tile.data;
        Ok(id)
    }

    pub fn get(&self, id: TileId) -> Result<&Tile, TileError> {
        let (chunk_id, local_id) = id;

        let chunk = self.chunks.get(chunk_id as usize).unwrap();
        let tile = chunk
            .tiles
            .get(local_id as usize)
            .ok_or(TileError::NotFound)?;
        Ok(tile)
    }

    // transfer chunk data

    pub fn get_version_by_chunk_coord(&self, chunk_coord: IVec2) -> Result<u64, TileError> {
        let chunk_id = self
            .chunk_index
            .get(&chunk_coord)
            .ok_or(TileError::NotFound)?;
        let chunk = self.chunks.get(*chunk_id as usize).unwrap();

        Ok(chunk.version)
    }

    pub fn get_ids_by_chunk_coord(&self, chunk_coord: IVec2) -> Result<impl Iterator<Item = BlockId>, TileError> {
        let chunk_id = self
            .chunk_index
            .get(&chunk_coord)
            .ok_or(TileError::NotFound)?;
        let chunk = self.chunks.get(*chunk_id as usize).unwrap();

        let ids = chunk
            .tiles
            .iter()
            .map(move |(local_id, _)| (*chunk_id, local_id as u32));
        Ok(ids)
    }

    // property

    pub fn get_display_name(&self, id: TileId) -> Result<&str, TileError> {
        let tile = self.get(id)?;
        let archetype = self.archetypes.get(tile.archetype_id as usize).unwrap();
        Ok(&archetype.display_name)
    }

    pub fn get_description(&self, id: TileId) -> Result<&str, TileError> {
        let tile = self.get(id)?;
        let archetype = self.archetypes.get(tile.archetype_id as usize).unwrap();
        Ok(&archetype.description)
    }

    // spatial features

    pub fn has_by_point(&self, point: IVec2) -> bool {
        self.spatial_index.contains_key(&point)
    }

    pub fn get_id_by_point(&self, point: IVec2) -> Option<TileId> {
        self.spatial_index.get(&point).copied()
    }

    pub fn get_chunk_coord(&self, point: Vec2) -> IVec2 {
        let chunk_size = Vec2::splat(Self::CHUNK_SIZE as f32);
        point.div_euclid(chunk_size).as_ivec2()
    }

    // collision features

    pub fn get_collision_rect(&self, id: TileId) -> Result<[Vec2; 2], TileError> {
        let tile = self.get(id)?;
        let archetype = self.archetypes.get(tile.archetype_id as usize).unwrap();
        Ok(archetype.collision_rect(tile.coord).unwrap_or_default())
    }

    pub fn has_by_collision_point(&self, point: Vec2) -> bool {
        let point = [point.x, point.y];
        self.collision_index.locate_at_point(&point).is_some()
    }

    pub fn get_ids_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = TileId> + '_ {
        let point = [point.x, point.y];
        self.collision_index
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    pub fn has_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.collision_index
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    pub fn get_ids_by_collision_rect(
        &self,
        rect: [Vec2; 2],
    ) -> impl Iterator<Item = TileId> + '_ {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.collision_index
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
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

    #[test]
    fn crud_tile() {
        let mut field: TileField = TileField::new(TileFieldInfo {
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
        });

        let id = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();

        let tile = field.get(id).unwrap();
        assert_eq!(tile.archetype_id, 1);
        assert_eq!(tile.coord, IVec2::new(-1, 3));

        assert!(field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_id_by_point(IVec2::new(-1, 3)), Some(id));

        let tile = field.remove(id).unwrap();
        assert_eq!(tile.archetype_id, 1);
        assert_eq!(tile.coord, IVec2::new(-1, 3));

        assert_eq!(field.get(id).unwrap_err(), TileError::NotFound);
        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_id_by_point(IVec2::new(-1, 3)), None);
        assert_eq!(field.remove(id).unwrap_err(), TileError::NotFound);
    }

    #[test]
    fn insert_tile_with_invalid() {
        let mut field: TileField = TileField::new(TileFieldInfo {
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
        });

        assert_eq!(
            field.insert(Tile {
                archetype_id: 2,
                coord: IVec2::new(-1, 3),
                data: Default::default(),
                render_state: Default::default(),
            }),
            Err(TileError::InvalidId)
        );
        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_id_by_point(IVec2::new(-1, 3)), None);

        let id = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        assert_eq!(
            field.insert(Tile {
                archetype_id: 0,
                coord: IVec2::new(-1, 3),
                data: Default::default(),
                render_state: Default::default(),
            }),
            Err(TileError::Conflict)
        );

        let tile = field.get(id).unwrap();
        assert_eq!(tile.archetype_id, 1);
        assert_eq!(tile.coord, IVec2::new(-1, 3));

        assert!(field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_id_by_point(IVec2::new(-1, 3)), Some(id));
    }

    #[test]
    fn modify_tile() {
        let mut field: TileField = TileField::new(TileFieldInfo {
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
        });

        let id = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();

        let id = field
            .modify(id, |tile| tile.coord = IVec2::new(-1, 4))
            .unwrap();

        let tile = field.get(id).unwrap();
        assert_eq!(tile.archetype_id, 1);
        assert_eq!(tile.coord, IVec2::new(-1, 4));

        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_id_by_point(IVec2::new(-1, 3)), None);
        assert!(field.has_by_point(IVec2::new(-1, 4)));
        assert_eq!(field.get_id_by_point(IVec2::new(-1, 4)), Some(id));

        let id = field
            .modify(id, |tile| tile.render_state.variant = 1)
            .unwrap();

        let tile = field.get(id).unwrap();
        assert_eq!(tile.archetype_id, 1);
        assert_eq!(tile.coord, IVec2::new(-1, 4));
        assert_eq!(tile.render_state.variant, 1);

        let id = field.modify(id, |_| {}).unwrap();

        let tile = field.get(id).unwrap();
        assert_eq!(tile.archetype_id, 1);
        assert_eq!(tile.coord, IVec2::new(-1, 4));
        assert_eq!(tile.render_state.variant, 1);
    }

    #[test]
    fn modify_tile_with_invalid() {
        let mut field: TileField = TileField::new(TileFieldInfo {
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
        });

        let id0 = field
            .insert(Tile {
                archetype_id: 0,
                coord: IVec2::new(-1, 3),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        let id1 = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 4),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();

        assert_eq!(
            field.modify(id0, |tile| tile.archetype_id = 1),
            Err(TileError::InvalidId)
        );

        assert_eq!(
            field.modify(id0, |tile| tile.coord = IVec2::new(-1, 4)),
            Err(TileError::Conflict)
        );

        let tile = field.get(id0).unwrap();
        assert_eq!(tile.archetype_id, 0);
        assert_eq!(tile.coord, IVec2::new(-1, 3));

        assert!(field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_id_by_point(IVec2::new(-1, 3)), Some(id0));

        field.remove(id1).unwrap();
        assert_eq!(field.modify(id1, |_| {}), Err(TileError::NotFound));
        assert_eq!(field.get(id1).unwrap_err(), TileError::NotFound);
        assert!(!field.has_by_point(IVec2::new(-1, 4)));
        assert_eq!(field.get_id_by_point(IVec2::new(-1, 4)), None);
    }

    #[test]
    fn modify_tile_with_move() {
        let mut field: TileField = TileField::new(TileFieldInfo {
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
        });

        let id = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();

        let id = field
            .modify(id, |tile| tile.coord = IVec2::new(-1, 1000))
            .unwrap();

        let tile = field.get(id).unwrap();
        assert_eq!(tile.archetype_id, 1);
        assert_eq!(tile.coord, IVec2::new(-1, 1000));

        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_id_by_point(IVec2::new(-1, 3)), None);
        assert!(field.has_by_point(IVec2::new(-1, 1000)));
        assert_eq!(field.get_id_by_point(IVec2::new(-1, 1000)), Some(id));
    }

    #[test]
    fn collision_tile() {
        let mut field: TileField = TileField::new(TileFieldInfo {
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
        });

        let id0 = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        let id1 = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 4),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        let _ = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 5),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();

        assert_eq!(
            field.get_collision_rect(id0),
            Ok([Vec2::new(-1.0, 3.0), Vec2::new(0.0, 4.0)])
        );

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
        assert_eq!(field.get_collision_rect(id0), Err(TileError::NotFound));
    }

    #[test]
    fn tile_chunk() {
        let mut field: TileField = TileField::new(TileFieldInfo {
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
        });

        let _ = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        let _ = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 4),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        let _ = field
            .insert(Tile {
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
