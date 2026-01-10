use super::*;

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
pub struct TileArchetype {
    pub display_name: String,
    pub description: String,
    pub collision: bool,
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
    pub render_state: TileRenderState,
}

#[derive(Debug, Clone)]
pub struct TileChunk {
    pub version: u64,
    pub tiles: slab::Slab<Tile>,
    dense_space: Vec<Option<TileId>>,
}

#[derive(Debug, Clone)]
pub struct TileField {
    archetypes: Vec<TileArchetype>,
    chunks: Vec<TileChunk>,
    chunk_index: ahash::AHashMap<IVec2, u32>,
}

impl TileField {
    const CHUNK_SIZE: u32 = 32;
    const TILE_LEN: u32 = Self::CHUNK_SIZE * Self::CHUNK_SIZE;

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
                dense_space: vec![None; Self::TILE_LEN as usize],
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
        let local_coord = tile.coord.rem_euclid(chunk_size);
        let dense_space = local_coord.y * Self::CHUNK_SIZE as i32 + local_coord.x;
        *chunk.dense_space.get_mut(dense_space as usize).unwrap() = Some((chunk_id, local_id));

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

        let chunk_size = IVec2::splat(Self::CHUNK_SIZE as i32);

        // spatial features
        let local_coord = tile.coord.rem_euclid(chunk_size);
        let spatial_id = local_coord.y * Self::CHUNK_SIZE as i32 + local_coord.x;
        *chunk.dense_space.get_mut(spatial_id as usize).unwrap() = None;

        Ok(tile)
    }

    pub fn modify(&mut self, id: TileId, f: impl FnOnce(&mut TileRenderState)) -> Result<TileId, TileError> {
        let (chunk_id, local_id) = id;
        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let tile = chunk.tiles.get_mut(local_id as usize).ok_or(TileError::NotFound)?;
        f(&mut tile.render_state);
        chunk.version += 1;
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

    // archetype

    pub fn get_archetype(&self, archetype_id: u16) -> Result<&TileArchetype, TileError> {
        let archetype = self.archetypes.get(archetype_id as usize).unwrap();
        Ok(archetype)
    }

    // transfer chunk data

    pub fn find_chunk_coord(&self, coord: Vec2) -> IVec2 {
        let chunk_size = Vec2::splat(Self::CHUNK_SIZE as f32);
        coord.div_euclid(chunk_size).as_ivec2()
    }

    pub fn get_chunk(&self, chunk_coord: IVec2) -> Result<&TileChunk, TileError> {
        let chunk_id = self.chunk_index.get(&chunk_coord).ok_or(TileError::NotFound)?;
        let chunk = self.chunks.get(*chunk_id as usize).unwrap();
        Ok(chunk)
    }

    // spatial features

    pub fn find_with_point(&self, coord: IVec2) -> Option<TileId> {
        let chunk_size = IVec2::splat(Self::CHUNK_SIZE as i32);
        let chunk_coord = coord.div_euclid(chunk_size);
        let chunk_id = self.chunk_index.get(&chunk_coord)?;
        let chunk = self.chunks.get(*chunk_id as usize).unwrap();

        let local_coord = coord.rem_euclid(chunk_size);
        let dense_space_id = local_coord.y * Self::CHUNK_SIZE as i32 + local_coord.x;
        *chunk.dense_space.get(dense_space_id as usize).unwrap()
    }

    pub fn find_with_rect(&self, rect: [IVec2; 2]) -> impl Iterator<Item = TileId> + '_ {
        let chunk_size = IVec2::splat(Self::CHUNK_SIZE as i32);
        let chunk_coord = [rect[0].div_euclid(chunk_size), rect[1].div_euclid(chunk_size)];

        let dim_min = IVec2::ZERO;
        let dim_max = IVec2::splat(Self::CHUNK_SIZE as i32 - 1);

        let query = (chunk_coord[0].y..=chunk_coord[1].y).flat_map(move |y| {
            (chunk_coord[0].x..=chunk_coord[1].x).filter_map(move |x| {
                let chunk_coord = IVec2::new(x, y);
                let chunk_id = self.chunk_index.get(&chunk_coord)?;
                let chunk = self.chunks.get(*chunk_id as usize).unwrap();
                Some((chunk_coord, chunk))
            })
        });
        query.flat_map(move |(chunk_coord, chunk)| {
            let local_min = (rect[0] - chunk_coord * chunk_size).clamp(dim_min, dim_max);
            let local_max = (rect[1] - chunk_coord * chunk_size).clamp(dim_min, dim_max);
            (local_min.y..=local_max.y).flat_map(move |y| {
                (local_min.x..=local_max.x).filter_map(move |x| {
                    let dense_space_id = y * Self::CHUNK_SIZE as i32 + x;
                    *chunk.dense_space.get(dense_space_id as usize).unwrap()
                })
            })
        })
    }

    // collision features

    pub fn find_with_collision_point(&self, coord: Vec2) -> Option<TileId> {
        let coord = coord.floor().as_ivec2();
        self.find_with_point(coord).filter(|id| {
            let tile = self.get(*id).unwrap();
            let archetype = self.archetypes.get(tile.archetype_id as usize).unwrap();
            archetype.collision
        })
    }

    pub fn find_with_collision_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = TileId> + '_ {
        let coord_min = rect[0].floor().as_ivec2();
        let coord_max = rect[1].ceil().as_ivec2();
        self.find_with_rect([coord_min, coord_max]).filter(move |id| {
            let tile = self.get(*id).unwrap();
            let archetype = self.archetypes.get(tile.archetype_id as usize).unwrap();
            archetype.collision
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
                render_state: Default::default(),
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
                render_state: Default::default(),
            }),
            Err(TileError::InvalidId)
        );
        assert_eq!(field.find_with_point(IVec2::new(-1, 3)), None);

        let id = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 3),
                render_state: Default::default(),
            })
            .unwrap();
        assert_eq!(
            field.insert(Tile {
                archetype_id: 0,
                coord: IVec2::new(-1, 3),
                render_state: Default::default(),
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
                coord: IVec2::new(-1, 4),
                render_state: Default::default(),
            })
            .unwrap();

        let tile = field.get(id).unwrap();
        assert_eq!(tile.archetype_id, 1);
        assert_eq!(tile.coord, IVec2::new(-1, 4));

        assert_eq!(field.find_with_point(IVec2::new(-1, 3)), None);
        assert_eq!(field.find_with_point(IVec2::new(-1, 4)), Some(id));

        let id = field
            .modify(id, |render_state| render_state.variant = 1)
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
                render_state: Default::default(),
            })
            .unwrap();
        let id1 = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 4),
                render_state: Default::default(),
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
                render_state: Default::default(),
            })
            .unwrap();

        field.remove(id).unwrap();
        let id = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 1000),
                render_state: Default::default(),
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
                render_state: Default::default(),
            })
            .unwrap();
        let id1 = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 4),
                render_state: Default::default(),
            })
            .unwrap();
        let _ = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 5),
                render_state: Default::default(),
            })
            .unwrap();

        let coord = Vec2::new(-1.0, 4.0);
        assert_eq!(field.find_with_collision_point(coord), Some(id1));

        let rect = [Vec2::new(-1.0, 3.0), Vec2::new(-1.0, 4.0)];
        let vec = field.find_with_collision_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&id0));
        assert!(vec.contains(&id1));
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
                render_state: Default::default(),
            })
            .unwrap();
        let _ = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 4),
                render_state: Default::default(),
            })
            .unwrap();
        let _ = field
            .insert(Tile {
                archetype_id: 1,
                coord: IVec2::new(-1, 5),
                render_state: Default::default(),
            })
            .unwrap();

        assert!(field.get_chunk(IVec2::new(0, 0)).is_err());

        let ids = field.get_chunk(IVec2::new(-1, 0)).unwrap();
        assert_eq!(ids.tiles.len(), 3);
    }
}
