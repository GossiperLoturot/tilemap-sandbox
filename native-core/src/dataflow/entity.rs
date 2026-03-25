use glam::*;

use crate::geom::*;

pub type EntityId = u64;

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
pub struct EntitySpatialData {
    pub collision_rect: Option<Rect2>,
    pub hint_rect: Rect2,
}

#[derive(Debug, Clone)]
pub struct EntityInfo {
    pub display_name: String,
    pub description: String,
    pub collision_rect: Option<Rect2>,
    pub hint_rect: Rect2,
    pub y_sorting: bool,
}

#[derive(Debug, Clone)]
pub struct EntityFieldInfo {
    pub entities: Vec<EntityInfo>,
}

#[derive(Debug, Clone)]
pub struct EntityArchetype {
    pub collision_rect: Option<Rect2>,
    pub hint_rect: Rect2,
    pub broad_rect: IRect2,
    pub y_sorting: bool,
}

impl EntityArchetype {
    #[inline]
    pub fn collision_rect(&self, coord: Vec2) -> Option<Rect2> {
        self.collision_rect.map(|rect| rect + coord)
    }

    #[inline]
    pub fn hint_rect(&self, coord: Vec2) -> Rect2 {
        self.hint_rect + coord
    }

    #[inline]
    pub fn broad_rect(&self, coord: Vec2) -> IRect2 {
        self.broad_rect + coord.floor().as_ivec2()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Entity {
    pub coord: Vec2,
    pub archetype_id: u16,
    pub variant: u16,
    pub tick: u32,
}

#[derive(Debug)]
pub struct EntityChunk {
    pub version: u64,
    pub entities: Vec<Entity>,
    pub ids: Vec<EntityId>,
}

#[derive(Debug)]
pub struct EntityField {
    archetypes: Vec<EntityArchetype>,
    chunks: Vec<EntityChunk>,
    coord_index: ahash::AHashMap<u64, u32>,
    id_index: slab::Slab<u64>,
    hgrid: HGrid<EntitySpatialData>,
}

impl EntityField {
    const CHUNK_SIZE: u32 = 32;

    pub fn new(info: EntityFieldInfo) -> Self {
        let mut archetypes = vec![];

        assert!(info.entities.len() <= u16::MAX as usize, "capacity overflow");
        for entity in info.entities {
            let mut broad_rect = IRect2::new(IVec2::MAX, IVec2::MIN);

            if let Some(rect) = &entity.collision_rect {
                if rect.size().x < 0.0 || rect.size().y < 0.0 {
                    panic!("collision size must be non-negative");
                }
                broad_rect = broad_rect.maximum(rect.trunc_over().as_irect2());
            }

            if entity.hint_rect.size().x < 0.0 || entity.hint_rect.size().y < 0.0 {
                panic!("hint size must be non-negative");
            }
            broad_rect = broad_rect.maximum(entity.hint_rect.trunc_over().as_irect2());

            archetypes.push(EntityArchetype {
                collision_rect: entity.collision_rect,
                hint_rect: entity.hint_rect,
                broad_rect,
                y_sorting: entity.y_sorting,
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
    fn alloc_chunk(&mut self, coord: Vec2) -> u32 {
        let chunk_coord = Self::find_chunk_coord_internal(coord);
        let chunk_coord_ = encode_coord(chunk_coord);

        if let Some(chunk_id) = self.coord_index.get(&chunk_coord_) {
            *chunk_id
        } else {
            assert!(self.chunks.len() <= u32::MAX as usize, "capacity overflow");
            let chunk_id = self.chunks.len() as u32;
            self.chunks.push(EntityChunk {
                version: Default::default(),
                entities: Default::default(),
                ids: Default::default(),
            });
            self.coord_index.insert(chunk_coord_, chunk_id);
            chunk_id
        }
    }

    pub fn insert(&mut self, entity: Entity) -> Result<EntityId, EntityError> {
        let chunk_id = self.alloc_chunk(entity.coord);

        // check by spatial features
        let archetype = self.archetypes.get(entity.archetype_id as usize).ok_or(EntityError::InvalidId)?;

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        assert!(chunk.entities.len() <= u32::MAX as usize, "capacity overflow");
        let local_id = chunk.entities.len() as u32;
        let address = encode_address(chunk_id, local_id);
        let id = self.id_index.insert(address) as u64;

        // register spatial index
        let broad_rect = archetype.broad_rect(entity.coord);
        self.hgrid.insert(broad_rect, id, EntitySpatialData {
            collision_rect: archetype.collision_rect(entity.coord),
            hint_rect: archetype.hint_rect(entity.coord),
        });

        chunk.entities.push(entity);
        chunk.ids.push(id);
        chunk.version += 1;
        Ok(id)
    }

    pub fn remove(&mut self, id: EntityId) -> Result<Entity, EntityError> {
        let address = self.id_index.try_remove(id as usize).ok_or(EntityError::NotFound)?;
        let (chunk_id, local_id) = decode_address(address);

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let entity = chunk.entities.swap_remove(local_id as usize);
        let _ = chunk.ids.swap_remove(local_id as usize);

        if let Some(id) = chunk.ids.get(local_id as usize) {
            *self.id_index.get_mut(*id as usize).unwrap() = address;
        }

        // unregister spatial index
        let archetype = self.archetypes.get(entity.archetype_id as usize).unwrap();
        let broad_rect = archetype.broad_rect(entity.coord);
        self.hgrid.remove(broad_rect, id);

        chunk.version += 1;
        Ok(entity)
    }

    pub fn modify_variant(&mut self, id: EntityId, variant: u16) -> Result<(), EntityError> {
        let address = *self.id_index.get(id as usize).ok_or(EntityError::NotFound)?;
        let (chunk_id, local_id) = decode_address(address);

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let entity = chunk.entities.get_mut(local_id as usize).unwrap();
        entity.variant = variant;
        chunk.version += 1;
        Ok(())
    }

    pub fn modify_tick(&mut self, id: EntityId, tick: u32) -> Result<(), EntityError> {
        let address = *self.id_index.get(id as usize).ok_or(EntityError::NotFound)?;
        let (chunk_id, local_id) = decode_address(address);

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let entity = chunk.entities.get_mut(local_id as usize).unwrap();
        entity.tick = tick;
        chunk.version += 1;
        Ok(())
    }

    pub fn r#move(&mut self, id: EntityId, new_coord: Vec2) -> Result<EntityId, EntityError> {
        let address = *self.id_index.get(id as usize).ok_or(EntityError::NotFound)?;
        let (chunk_id, local_id) = decode_address(address);

        let chunk = self.chunks.get(chunk_id as usize).unwrap();
        let entity = chunk.entities.get(local_id as usize).unwrap();
        if entity.coord == new_coord {
            return Ok(id);
        }

        // check by spatial features
        let archetype = self.get_archetype(entity.archetype_id)?;

        // update spatial index
        let broad_rect = archetype.broad_rect(entity.coord);
        let new_broad_rect = archetype.broad_rect(new_coord);
        if self.hgrid.check_move(broad_rect, new_broad_rect) {
            let value = EntitySpatialData {
                collision_rect: archetype.collision_rect(new_coord),
                hint_rect: archetype.hint_rect(new_coord),
            };
            self.hgrid.remove(broad_rect, id);
            self.hgrid.insert(new_broad_rect, id, value);
        }

        // move owner
        let chunk_coord = Self::find_chunk_coord_internal(entity.coord);
        let new_chunk_coord = Self::find_chunk_coord_internal(new_coord);
        if chunk_coord != new_chunk_coord {
            let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
            let entity = chunk.entities.swap_remove(local_id as usize);
            let _ = chunk.ids.swap_remove(local_id as usize);

            if let Some(id) = chunk.ids.get(local_id as usize) {
                *self.id_index.get_mut(*id as usize).unwrap() = address;
            }
            chunk.version += 1;

            let new_chunk_id = self.alloc_chunk(new_coord);

            let new_chunk = self.chunks.get_mut(new_chunk_id as usize).unwrap();
            assert!(new_chunk.entities.len() <= u32::MAX as usize, "capacity overflow");
            let new_local_id = new_chunk.entities.len() as u32;
            let new_address = encode_address(new_chunk_id, new_local_id);
            *self.id_index.get_mut(id as usize).unwrap() = new_address;

            new_chunk.entities.push(Entity { coord: new_coord, ..entity });
            new_chunk.ids.push(id);
            new_chunk.version += 1;
        } else {
            let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
            let entity = chunk.entities.get_mut(local_id as usize).unwrap();
            entity.coord = new_coord;
            chunk.version += 1;
        }
        Ok(id)
    }

    #[inline]
    pub fn get(&self, id: EntityId) -> Result<&Entity, EntityError> {
        let address = *self.id_index.get(id as usize).ok_or(EntityError::NotFound)?;
        let (chunk_id, local_id) = decode_address(address);

        let chunk = self.chunks.get(chunk_id as usize).unwrap();
        let entity = chunk.entities.get(local_id as usize).unwrap();

        Ok(entity)
    }

    // archetype

    #[inline]
    pub fn get_archetype(&self, archetype_id: u16) -> Result<&EntityArchetype, EntityError> {
        self.archetypes.get(archetype_id as usize).ok_or(EntityError::InvalidId)
    }

    // transfer chunk data

    #[inline]
    pub fn find_chunk_coord(&self, coord: Vec2) -> IVec2 {
        coord.div_euclid(Vec2::splat(Self::CHUNK_SIZE as f32)).as_ivec2()
    }

    #[inline]
    fn find_chunk_coord_internal(coord: Vec2) -> IVec2 {
        coord.div_euclid(Vec2::splat(Self::CHUNK_SIZE as f32)).as_ivec2()
    }

    #[inline]
    pub fn get_chunk(&self, chunk_coord: IVec2) -> Result<&EntityChunk, EntityError> {
        let chunk_coord_ = encode_coord(chunk_coord);
        let chunk_id = *self.coord_index.get(&chunk_coord_).ok_or(EntityError::NotFound)?;
        let chunk = self.chunks.get(chunk_id as usize).unwrap();
        Ok(chunk)
    }

    // collision features

    #[inline]
    pub fn find_with_collision_point(&self, point: Vec2) -> impl Iterator<Item = (&EntityId, &EntitySpatialData)> {
        self.find_with_collision_rect(Rect2::new(point, point))
    }

    #[inline]
    pub fn find_with_collision_rect(&self, rect: Rect2) -> impl Iterator<Item = (&EntityId, &EntitySpatialData)> {
        self.hgrid.find(rect.trunc_over().as_irect2())
            .filter(move |(_, data)| data.collision_rect.map(|obj_rect| Intersects::intersects(&rect, &obj_rect)).unwrap_or(false))
    }

    // hint features

    #[inline]
    pub fn find_with_hint_point(&self, point: Vec2) -> impl Iterator<Item = (&EntityId, &EntitySpatialData)> {
        self.find_with_hint_rect(Rect2::new(point, point))
    }

    #[inline]
    pub fn find_with_hint_rect(&self, rect: Rect2) -> impl Iterator<Item = (&EntityId, &EntitySpatialData)> {
        self.hgrid.find(rect.trunc_over().as_irect2())
            .filter(move |(_, data)| Intersects::intersects(&rect, &data.hint_rect))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityError {
    NotFound,
    Conflict,
    InvalidId,
}

impl std::fmt::Display for EntityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "not found error"),
            Self::Conflict => write!(f, "conflict error"),
            Self::InvalidId => write!(f, "invalid id error"),
        }
    }
}

impl std::error::Error for EntityError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entity_field() -> EntityField {
        EntityField::new(EntityFieldInfo {
            entities: vec![
                EntityInfo {
                    display_name: "entity_0".into(),
                    description: "entity_0_desc".into(),
                    collision_rect: Some(Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0))),
                    hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                    y_sorting: false,
                },
                EntityInfo {
                    display_name: "entity_1".into(),
                    description: "entity_1_desc".into(),
                    collision_rect: Some(Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0))),
                    hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                    y_sorting: false,
                },
            ],
        })
    }

    #[test]
    #[should_panic]
    fn entity_field_with_invalid_collision() {
        EntityField::new(EntityFieldInfo {
            entities: vec![EntityInfo {
                display_name: "entity_0".into(),
                description: "entity_0_desc".into(),
                collision_rect: Some(Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(-1.0, -1.0))),
                hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)),
                y_sorting: false,
            }],
        });
    }

    #[test]
    #[should_panic]
    fn entity_field_with_invalid_hint() {
        EntityField::new(EntityFieldInfo {
            entities: vec![EntityInfo {
                display_name: "entity_0".into(),
                description: "entity_0_desc".into(),
                collision_rect: Some(Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0))),
                hint_rect: Rect2::new(Vec2::new(0.0, 0.0), Vec2::new(-1.0, -1.0)),
                y_sorting: false,
            }],
        });
    }

    #[test]
    fn crud_entity() {
        let mut field = make_entity_field();

        let id = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 3.0),
                ..Default::default()
            })
            .unwrap();

        let entity = field.get(id).unwrap();
        assert_eq!(entity.archetype_id, 1);
        assert_eq!(entity.coord, Vec2::new(-1.0, 3.0));

        let entity = field.remove(id).unwrap();
        assert_eq!(entity.archetype_id, 1);
        assert_eq!(entity.coord, Vec2::new(-1.0, 3.0));

        assert_eq!(field.get(id).unwrap_err(), EntityError::NotFound);
        assert_eq!(field.remove(id).unwrap_err(), EntityError::NotFound);
    }

    #[test]
    fn insert_entity_with_invalid() {
        let mut field = make_entity_field();

        assert_eq!(
            field.insert(Entity {
                archetype_id: 2,
                coord: Vec2::new(-1.0, 3.0),
                ..Default::default()
            }),
            Err(EntityError::InvalidId)
        );
    }

    #[test]
    fn modify_entity() {
        let mut field = make_entity_field();

        let id = field
            .insert(Entity {
                archetype_id: 0,
                coord: Vec2::new(-1.0, 4.0),
                ..Default::default()
            })
            .unwrap();

        let entity = field.get(id).unwrap();
        assert_eq!(entity.archetype_id, 0);
        assert_eq!(entity.coord, Vec2::new(-1.0, 4.0));

        field.modify_variant(id, 1).unwrap();

        let entity = field.get(id).unwrap();
        assert_eq!(entity.archetype_id, 0);
        assert_eq!(entity.coord, Vec2::new(-1.0, 4.0));
        assert_eq!(entity.variant, 1);
    }

    #[test]
    fn modify_entity_with_invalid() {
        let mut field = make_entity_field();

        let id0 = field
            .insert(Entity {
                archetype_id: 0,
                coord: Vec2::new(-1.0, 3.0),
                ..Default::default()
            })
            .unwrap();
        let id1 = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 4.0),
                ..Default::default()
            })
            .unwrap();

        let entity = field.get(id0).unwrap();
        assert_eq!(entity.archetype_id, 0);
        assert_eq!(entity.coord, Vec2::new(-1.0, 3.0));

        field.remove(id1).unwrap();
        assert_eq!(field.modify_variant(id1, 1), Err(EntityError::NotFound));
        assert_eq!(field.get(id1).unwrap_err(), EntityError::NotFound);
    }

    #[test]
    fn move_entity() {
        let mut field = make_entity_field();

        let id = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 3.0),
                ..Default::default()
            })
            .unwrap();

        let id = field.r#move(id, Vec2::new(-1.0, 1000.0)).unwrap();

        let entity = field.get(id).unwrap();
        assert_eq!(entity.archetype_id, 1);
        assert_eq!(entity.coord, Vec2::new(-1.0, 1000.0));
    }

    #[test]
    fn collision_entity() {
        let mut field = make_entity_field();

        let id0 = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 3.0),
                ..Default::default()
            })
            .unwrap();
        let id1 = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 4.0),
                ..Default::default()
            })
            .unwrap();
        let _ = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 5.0),
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
    fn hint_entity() {
        let mut field = make_entity_field();

        let id0 = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 3.0),
                ..Default::default()
            })
            .unwrap();
        let id1 = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 4.0),
                ..Default::default()
            })
            .unwrap();
        let _ = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 5.0),
                ..Default::default()
            })
            .unwrap();

        let point = Vec2::new(-1.0, 4.0);
        let vec = field.find_with_hint_point(point).map(|(id, _)| *id).collect::<Vec<_>>();
        assert!(vec.contains(&id0));
        assert!(vec.contains(&id1));

        let rect = Rect2::new(Vec2::new(-1.0, 3.0), Vec2::new(-1.0, 4.0));
        let vec = field.find_with_hint_rect(rect).map(|(id, _)| *id).collect::<Vec<_>>();
        assert!(vec.contains(&id0));
        assert!(vec.contains(&id1));
    }

    #[test]
    fn entity_chunk() {
        let mut field = make_entity_field();

        let _ = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 3.0),
                ..Default::default()
            })
            .unwrap();
        let _ = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 4.0),
                ..Default::default()
            })
            .unwrap();
        let _ = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 5.0),
                ..Default::default()
            })
            .unwrap();

        assert!(field.get_chunk(IVec2::new(0, 0)).is_err());

        let chunk = field.get_chunk(IVec2::new(-1, 0)).unwrap();
        assert_eq!(chunk.entities.len(), 3);
    }
}
