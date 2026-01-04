use super::*;

type RectNode<T, U> = rstar::primitives::GeomWithData<rstar::primitives::Rectangle<T>, U>;

pub type TileKey = (u32, u32);

#[derive(Debug, Clone)]
pub struct TileDescriptor {
    pub display_name: String,
    pub description: String,
    pub collision: bool,
}

#[derive(Debug, Clone)]
pub struct TileFieldDescriptor {
    pub tiles: Vec<TileDescriptor>,
}

#[derive(Debug, Clone)]
struct TileProperty {
    display_name: String,
    discription: String,
    collision: bool,
}

impl TileProperty {
    #[rustfmt::skip]
    fn collision_rect(&self, location: IVec2) -> Option<[Vec2; 2]> {
        if !self.collision {
            return None;
        }

        Some([
            location.as_vec2(),
            location.as_vec2() + 1.0,
        ])
    }
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub id: u16,
    pub location: IVec2,
    pub data: Box<dyn TileData>,
    pub render_param: TileRenderParam,
}

#[derive(Debug, Clone)]
struct TileChunk {
    version: u64,
    tiles: slab::Slab<Tile>,
}

#[derive(Debug, Clone)]
pub struct TileField {
    props: Vec<TileProperty>,
    chunks: Vec<TileChunk>,
    chunk_ref: ahash::AHashMap<IVec2, u32>,
    spatial_ref: ahash::AHashMap<IVec2, TileKey>,
    collision_ref: rstar::RTree<RectNode<[f32; 2], TileKey>>,
}

impl TileField {
    const CHUNK_SIZE: u32 = 32;

    pub fn new(desc: TileFieldDescriptor) -> Self {
        let mut props = vec![];

        for tile in desc.tiles {
            props.push(TileProperty {
                display_name: tile.display_name,
                discription: tile.description,
                collision: tile.collision,
            });
        }

        Self {
            props,
            chunks: Default::default(),
            chunk_ref: Default::default(),
            spatial_ref: Default::default(),
            collision_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, tile: Tile) -> Result<TileKey, TileError> {
        let prop = self
            .props
            .get(tile.id as usize)
            .ok_or(TileError::InvalidId)?;

        // check by spatial features
        if self.has_by_point(tile.location) {
            return Err(TileError::Conflict);
        }

        let chunk_size = IVec2::splat(Self::CHUNK_SIZE as i32);
        let chunk_location = tile.location.div_euclid(chunk_size);

        // get or allocate chunk
        let chunk_key = if let Some(chunk_key) = self.chunk_ref.get(&chunk_location) {
            *chunk_key
        } else {
            if self.chunks.len() >= u32::MAX as usize {
                panic!("capacity overflow");
            }

            let chunk_key = self.chunks.len() as u32;
            self.chunks.push(TileChunk {
                version: 0,
                tiles: Default::default(),
            });
            self.chunk_ref.insert(chunk_location, chunk_key);
            chunk_key
        };

        let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();

        if chunk.tiles.vacant_key() >= u32::MAX as usize {
            panic!("capacity overflow");
        }

        let local_key = chunk.tiles.vacant_key() as u32;

        // spatial features
        self.spatial_ref
            .insert(tile.location, (chunk_key, local_key));

        // collision features
        if let Some(rect) = prop.collision_rect(tile.location) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = rstar::primitives::GeomWithData::new(rect, (chunk_key, local_key));
            self.collision_ref.insert(node);
        }

        // key is guaranteed to be less than u32::MAX.
        chunk.tiles.insert(tile);
        chunk.version += 1;

        Ok((chunk_key, local_key))
    }

    pub fn remove(&mut self, key: TileKey) -> Result<Tile, TileError> {
        let (chunk_key, local_key) = key;

        let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
        let tile = chunk
            .tiles
            .try_remove(local_key as usize)
            .ok_or(TileError::NotFound)?;
        chunk.version += 1;

        let prop = self.props.get(tile.id as usize).unwrap();

        // spatial features
        self.spatial_ref.remove(&tile.location).unwrap();

        // collision features
        if let Some(rect) = prop.collision_rect(tile.location) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.remove(&node).unwrap();
        }

        Ok(tile)
    }

    pub fn modify(
        &mut self,
        key: TileKey,
        f: impl FnOnce(&mut Tile),
    ) -> Result<TileKey, TileError> {
        let (chunk_key, local_key) = key;

        let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
        let tile = chunk
            .tiles
            .get_mut(local_key as usize)
            .ok_or(TileError::NotFound)?;

        let mut new_tile = Tile {
            id: tile.id,
            location: tile.location,
            data: std::mem::take(&mut tile.data),
            render_param: tile.render_param.clone(),
        };
        f(&mut new_tile);

        if new_tile.id != tile.id {
            tile.data = new_tile.data;
            return Err(TileError::InvalidId);
        }

        if new_tile.location != tile.location {
            // check by spatial features
            if self.has_by_point(new_tile.location) {
                return Err(TileError::Conflict);
            }

            self.remove(key).unwrap();
            let key = self.insert(new_tile).unwrap();
            return Ok(key);
        }

        if new_tile.render_param != tile.render_param {
            let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
            *chunk.tiles.get_mut(local_key as usize).unwrap() = new_tile;
            chunk.version += 1;
            return Ok(key);
        }

        tile.data = new_tile.data;
        Ok(key)
    }

    pub fn get(&self, key: TileKey) -> Result<&Tile, TileError> {
        let (chunk_key, local_key) = key;

        let chunk = self.chunks.get(chunk_key as usize).unwrap();
        let tile = chunk
            .tiles
            .get(local_key as usize)
            .ok_or(TileError::NotFound)?;
        Ok(tile)
    }

    // transfer chunk data

    pub fn get_version_by_chunk_location(&self, chunk_location: IVec2) -> Result<u64, TileError> {
        let chunk_key = self
            .chunk_ref
            .get(&chunk_location)
            .ok_or(TileError::NotFound)?;
        let chunk = self.chunks.get(*chunk_key as usize).unwrap();

        Ok(chunk.version)
    }

    pub fn get_keys_by_chunk_location(
        &self,
        chunk_location: IVec2,
    ) -> Result<impl Iterator<Item = BlockKey>, TileError> {
        let chunk_key = self
            .chunk_ref
            .get(&chunk_location)
            .ok_or(TileError::NotFound)?;
        let chunk = self.chunks.get(*chunk_key as usize).unwrap();

        let keys = chunk
            .tiles
            .iter()
            .map(move |(local_key, _)| (*chunk_key, local_key as u32));
        Ok(keys)
    }

    // property

    pub fn get_display_name(&self, key: TileKey) -> Result<&str, TileError> {
        let tile = self.get(key)?;
        let prop = self.props.get(tile.id as usize).unwrap();
        Ok(&prop.display_name)
    }

    pub fn get_description(&self, key: TileKey) -> Result<&str, TileError> {
        let tile = self.get(key)?;
        let prop = self.props.get(tile.id as usize).unwrap();
        Ok(&prop.discription)
    }

    // spatial features

    pub fn has_by_point(&self, point: IVec2) -> bool {
        self.spatial_ref.contains_key(&point)
    }

    pub fn get_key_by_point(&self, point: IVec2) -> Option<TileKey> {
        self.spatial_ref.get(&point).copied()
    }

    pub fn get_chunk_location(&self, point: Vec2) -> IVec2 {
        let chunk_size = Vec2::splat(Self::CHUNK_SIZE as f32);
        point.div_euclid(chunk_size).as_ivec2()
    }

    // collision features

    pub fn get_collision_rect(&self, tile_key: TileKey) -> Result<[Vec2; 2], TileError> {
        let tile = self.get(tile_key)?;
        let prop = self.props.get(tile.id as usize).unwrap();
        Ok(prop.collision_rect(tile.location).unwrap_or_default())
    }

    pub fn has_by_collision_point(&self, point: Vec2) -> bool {
        let point = [point.x, point.y];
        self.collision_ref.locate_at_point(&point).is_some()
    }

    pub fn get_keys_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = TileKey> + '_ {
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
    ) -> impl Iterator<Item = TileKey> + '_ {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.collision_ref
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
        let mut field: TileField = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor {
                    display_name: "tile_0".into(),
                    description: "tile_0_desc".into(),
                    collision: true,
                },
                TileDescriptor {
                    display_name: "tile_1".into(),
                    description: "tile_1_desc".into(),
                    collision: true,
                },
            ],
        });

        let key = field
            .insert(Tile {
                id: 1,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        let tile = field.get(key).unwrap();
        assert_eq!(tile.id, 1);
        assert_eq!(tile.location, IVec2::new(-1, 3));

        assert!(field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_key_by_point(IVec2::new(-1, 3)), Some(key));

        let tile = field.remove(key).unwrap();
        assert_eq!(tile.id, 1);
        assert_eq!(tile.location, IVec2::new(-1, 3));

        assert_eq!(field.get(key).unwrap_err(), TileError::NotFound);
        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_key_by_point(IVec2::new(-1, 3)), None);
        assert_eq!(field.remove(key).unwrap_err(), TileError::NotFound);
    }

    #[test]
    fn insert_tile_with_invalid() {
        let mut field: TileField = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor {
                    display_name: "tile_0".into(),
                    description: "tile_0_desc".into(),
                    collision: true,
                },
                TileDescriptor {
                    display_name: "tile_1".into(),
                    description: "tile_1_desc".into(),
                    collision: true,
                },
            ],
        });

        assert_eq!(
            field.insert(Tile {
                id: 2,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            }),
            Err(TileError::InvalidId)
        );
        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_key_by_point(IVec2::new(-1, 3)), None);

        let key = field
            .insert(Tile {
                id: 1,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        assert_eq!(
            field.insert(Tile {
                id: 0,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            }),
            Err(TileError::Conflict)
        );

        let tile = field.get(key).unwrap();
        assert_eq!(tile.id, 1);
        assert_eq!(tile.location, IVec2::new(-1, 3));

        assert!(field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_key_by_point(IVec2::new(-1, 3)), Some(key));
    }

    #[test]
    fn modify_tile() {
        let mut field: TileField = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor {
                    display_name: "tile_0".into(),
                    description: "tile_0_desc".into(),
                    collision: true,
                },
                TileDescriptor {
                    display_name: "tile_1".into(),
                    description: "tile_1_desc".into(),
                    collision: true,
                },
            ],
        });

        let key = field
            .insert(Tile {
                id: 1,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        let key = field
            .modify(key, |tile| tile.location = IVec2::new(-1, 4))
            .unwrap();

        let tile = field.get(key).unwrap();
        assert_eq!(tile.id, 1);
        assert_eq!(tile.location, IVec2::new(-1, 4));

        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_key_by_point(IVec2::new(-1, 3)), None);
        assert!(field.has_by_point(IVec2::new(-1, 4)));
        assert_eq!(field.get_key_by_point(IVec2::new(-1, 4)), Some(key));

        let key = field
            .modify(key, |tile| tile.render_param.variant = 1)
            .unwrap();

        let tile = field.get(key).unwrap();
        assert_eq!(tile.id, 1);
        assert_eq!(tile.location, IVec2::new(-1, 4));
        assert_eq!(tile.render_param.variant, 1);

        let key = field.modify(key, |_| {}).unwrap();

        let tile = field.get(key).unwrap();
        assert_eq!(tile.id, 1);
        assert_eq!(tile.location, IVec2::new(-1, 4));
        assert_eq!(tile.render_param.variant, 1);
    }

    #[test]
    fn modify_tile_with_invalid() {
        let mut field: TileField = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor {
                    display_name: "tile_0".into(),
                    description: "tile_0_desc".into(),
                    collision: true,
                },
                TileDescriptor {
                    display_name: "tile_1".into(),
                    description: "tile_1_desc".into(),
                    collision: true,
                },
            ],
        });

        let key_0 = field
            .insert(Tile {
                id: 0,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let key_1 = field
            .insert(Tile {
                id: 1,
                location: IVec2::new(-1, 4),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        assert_eq!(
            field.modify(key_0, |tile| tile.id = 1),
            Err(TileError::InvalidId)
        );

        assert_eq!(
            field.modify(key_0, |tile| tile.location = IVec2::new(-1, 4)),
            Err(TileError::Conflict)
        );

        let tile = field.get(key_0).unwrap();
        assert_eq!(tile.id, 0);
        assert_eq!(tile.location, IVec2::new(-1, 3));

        assert!(field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_key_by_point(IVec2::new(-1, 3)), Some(key_0));

        field.remove(key_1).unwrap();
        assert_eq!(field.modify(key_1, |_| {}), Err(TileError::NotFound));
        assert_eq!(field.get(key_1).unwrap_err(), TileError::NotFound);
        assert!(!field.has_by_point(IVec2::new(-1, 4)));
        assert_eq!(field.get_key_by_point(IVec2::new(-1, 4)), None);
    }

    #[test]
    fn modify_tile_with_move() {
        let mut field: TileField = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor {
                    display_name: "tile_0".into(),
                    description: "tile_0_desc".into(),
                    collision: true,
                },
                TileDescriptor {
                    display_name: "tile_1".into(),
                    description: "tile_1_desc".into(),
                    collision: true,
                },
            ],
        });

        let key = field
            .insert(Tile {
                id: 1,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        let key = field
            .modify(key, |tile| tile.location = IVec2::new(-1, 1000))
            .unwrap();

        let tile = field.get(key).unwrap();
        assert_eq!(tile.id, 1);
        assert_eq!(tile.location, IVec2::new(-1, 1000));

        assert!(!field.has_by_point(IVec2::new(-1, 3)));
        assert_eq!(field.get_key_by_point(IVec2::new(-1, 3)), None);
        assert!(field.has_by_point(IVec2::new(-1, 1000)));
        assert_eq!(field.get_key_by_point(IVec2::new(-1, 1000)), Some(key));
    }

    #[test]
    fn collision_tile() {
        let mut field: TileField = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor {
                    display_name: "tile_0".into(),
                    description: "tile_0_desc".into(),
                    collision: true,
                },
                TileDescriptor {
                    display_name: "tile_1".into(),
                    description: "tile_1_desc".into(),
                    collision: true,
                },
            ],
        });

        let key_0 = field
            .insert(Tile {
                id: 1,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let key_1 = field
            .insert(Tile {
                id: 1,
                location: IVec2::new(-1, 4),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let _key_2 = field
            .insert(Tile {
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
        assert_eq!(field.get_collision_rect(key_0), Err(TileError::NotFound));
    }

    #[test]
    fn tile_chunk() {
        let mut field: TileField = TileField::new(TileFieldDescriptor {
            tiles: vec![
                TileDescriptor {
                    display_name: "tile_0".into(),
                    description: "tile_0_desc".into(),
                    collision: true,
                },
                TileDescriptor {
                    display_name: "tile_1".into(),
                    description: "tile_1_desc".into(),
                    collision: true,
                },
            ],
        });

        let _key0 = field
            .insert(Tile {
                id: 1,
                location: IVec2::new(-1, 3),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let _key1 = field
            .insert(Tile {
                id: 1,
                location: IVec2::new(-1, 4),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let _key2 = field
            .insert(Tile {
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
