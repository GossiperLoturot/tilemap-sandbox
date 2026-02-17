use glam::*;

use crate::geom::*;

pub type EntityId = u64;

#[inline]
fn encode_id(chunk_id: u32, local_id: u16) -> EntityId {
    (chunk_id as u64) << 32 | local_id as u64
}

#[inline]
fn decode_id(tile_id: EntityId) -> (u32, u16) {
    ((tile_id >> 32) as u32, tile_id as u16)
}

#[inline]
fn encode_coord(coord: IVec2) -> u64 {
    (coord.x as u32 as u64) << 32 | coord.y as u32 as u64
}

#[derive(Clone, Debug)]
struct EntitySpatialData {
    collision_rect: Option<Rect2>,
    hint_rect: Rect2,
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
    pub display_name: String,
    pub description: String,
    pub collision_rect: Option<Rect2>,
    pub hint_rect: Rect2,
    pub broad_rect: IRect2,
    pub y_sorting: bool,
}

impl EntityArchetype {
    #[inline]
    pub fn collision_rect(&self, coord: Vec2) -> Option<Rect2> {
        self.collision_rect.map(|rect| coord + rect)
    }

    #[inline]
    pub fn hint_rect(&self, coord: Vec2) -> Rect2 {
        coord + self.hint_rect
    }

    #[inline]
    pub fn broad_rect(&self, coord: Vec2) -> IRect2 {
        coord.floor().as_ivec2() + self.broad_rect
    }
}

#[derive(Debug, Clone, Default)]
pub struct EntityModify {
    pub variant: u8,
    pub tick: u32,
}

#[derive(Debug, Clone, Default)]
pub struct Entity {
    pub archetype_id: u16,
    pub coord: Vec2,
    pub variant: u8,
    pub tick: u32,
}

#[derive(Debug)]
pub struct EntityChunk {
    pub version: u64,
    pub entities: slab::Slab<Entity>,
}

#[derive(Debug)]
pub struct EntityField {
    archetypes: Vec<EntityArchetype>,
    chunks: Vec<EntityChunk>,
    coord_index: ahash::AHashMap<u64, u32>,
    hgrid: HGrid<EntitySpatialData>,
}

impl EntityField {
    const CHUNK_SIZE: u32 = 32;

    pub fn new(info: EntityFieldInfo) -> Self {
        let mut archetyps = vec![];

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

            archetyps.push(EntityArchetype {
                display_name: entity.display_name,
                description: entity.description,
                collision_rect: entity.collision_rect,
                hint_rect: entity.hint_rect,
                broad_rect,
                y_sorting: entity.y_sorting,
            });
        }

        Self {
            archetypes: archetyps,
            chunks: Default::default(),
            coord_index: Default::default(),
            hgrid: Default::default(),
        }
    }

    pub fn insert(&mut self, entity: Entity) -> Result<EntityId, EntityError> {
        let archetype = self.archetypes.get(entity.archetype_id as usize).ok_or(EntityError::InvalidId)?;

        let chunk_size = Vec2::splat(Self::CHUNK_SIZE as f32);
        let chunk_coord = entity.coord.div_euclid(chunk_size).as_ivec2();
        let chunk_coord_ = encode_coord(chunk_coord);

        // get or allocate chunk
        let chunk_id = if let Some(chunk_id) = self.coord_index.get(&chunk_coord_) {
            *chunk_id
        } else {
            if self.chunks.len() >= u32::MAX as usize {
                panic!("capacity overflow");
            }

            let chunk_id = self.chunks.len() as u32;
            self.chunks.push(EntityChunk {
                version: 0,
                entities: Default::default(),
            });
            self.coord_index.insert(chunk_coord_, chunk_id);
            chunk_id
        };
        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();

        // entity_key is guaranteed to be less than u16::MAX.
        if chunk.entities.vacant_key() >= u16::MAX as usize {
            panic!("capacity overflow");
        }
        let local_id = chunk.entities.vacant_key() as u16;
        let id = encode_id(chunk_id, local_id);

        // register spatial index
        let broad_rect = archetype.broad_rect(entity.coord);
        self.hgrid.insert(broad_rect, id, EntitySpatialData {
            collision_rect: archetype.collision_rect(entity.coord),
            hint_rect: archetype.hint_rect(entity.coord),
        });

        chunk.entities.insert(entity);
        chunk.version += 1;

        Ok(id)
    }

    pub fn remove(&mut self, id: EntityId) -> Result<Entity, EntityError> {
        let (chunk_id, local_id) = decode_id(id);

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let entity = chunk.entities.try_remove(local_id as usize).ok_or(EntityError::NotFound)?;
        chunk.version += 1;

        // unregister spatial index
        let archetype = self.archetypes.get(entity.archetype_id as usize).unwrap();
        let broad_rect = archetype.broad_rect(entity.coord);
        self.hgrid.remove(broad_rect, id);

        Ok(entity)
    }

    pub fn modify(&mut self, id: EntityId, f: impl FnOnce(&mut EntityModify)) -> Result<EntityId, EntityError> {
        let (chunk_id, local_id) = decode_id(id);

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let entity = chunk.entities.get_mut(local_id as usize).ok_or(EntityError::NotFound)?;

        let mut entity_modify = EntityModify { variant: entity.variant, tick: entity.tick };
        f(&mut entity_modify);
        entity.variant = entity_modify.variant;
        entity.tick = entity_modify.tick;
        chunk.version += 1;
        Ok(id)
    }

    pub fn get(&self, id: EntityId) -> Result<&Entity, EntityError> {
        let (chunk_id, local_id) = decode_id(id);
        let chunk = self.chunks.get(chunk_id as usize).unwrap();
        let entity = chunk.entities.get(local_id as usize).ok_or(EntityError::NotFound)?;
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
        let chunk_size = Vec2::splat(Self::CHUNK_SIZE as f32);
        coord.div_euclid(chunk_size).as_ivec2()
    }

    #[inline]
    pub fn get_chunk(&self, chunk_coord: IVec2) -> Result<&EntityChunk, EntityError> {
        let chunk_coord_ = encode_coord(chunk_coord);
        let chunk_id = self.coord_index.get(&chunk_coord_).ok_or(EntityError::NotFound)?;
        let chunk = self.chunks.get(*chunk_id as usize).unwrap();
        Ok(chunk)
    }

    // collision features

    #[inline]
    pub fn find_with_collision_point(&self, point: Vec2) -> impl Iterator<Item = EntityId> + '_ {
        self.find_with_collision_rect(Rect2::new(point, point))
    }

    #[inline]
    pub fn find_with_collision_rect(&self, rect: Rect2) -> impl Iterator<Item = EntityId> + '_ {
        self.hgrid.find(rect.trunc_over().as_irect2())
            .filter_map(|(id, data)| data.collision_rect.map(|obj_rect| (id, obj_rect)))
            .filter(move |(_, obj_rect)| Intersects::intersects(&rect, obj_rect))
            .map(|(id, _)| *id)
    }

    // hint features

    #[inline]
    pub fn find_with_hint_point(&self, point: Vec2) -> impl Iterator<Item = EntityId> + '_ {
        self.find_with_hint_rect(Rect2::new(point, point))
    }

    #[inline]
    pub fn find_with_hint_rect(&self, rect: Rect2) -> impl Iterator<Item = EntityId> + '_ {
        self.hgrid.find(rect.trunc_over().as_irect2())
            .map(|(id, data)| (id, data.hint_rect))
            .filter(move |(_, obj_rect)| Intersects::intersects(&rect, obj_rect))
            .map(|(id, _)| *id)
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

        let id = field
            .modify(id, |entity| entity.variant = 1)
            .unwrap();

        let entity = field.get(id).unwrap();
        assert_eq!(entity.archetype_id, 0);
        assert_eq!(entity.coord, Vec2::new(-1.0, 4.0));
        assert_eq!(entity.variant, 1);

        let id = field.modify(id, |_| {}).unwrap();

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
        assert_eq!(field.modify(id1, |_| {}), Err(EntityError::NotFound));
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

        field.remove(id).unwrap();
        let id = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 1000.0),
                ..Default::default()
            })
            .unwrap();

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
        let vec = field.find_with_collision_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&id0));
        assert!(vec.contains(&id1));

        let rect = Rect2::new(Vec2::new(-1.0, 3.0), Vec2::new(-1.0, 4.0));
        let vec = field.find_with_collision_rect(rect).collect::<Vec<_>>();
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
        let vec = field.find_with_hint_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&id0));
        assert!(vec.contains(&id1));

        let rect = Rect2::new(Vec2::new(-1.0, 3.0), Vec2::new(-1.0, 4.0));
        let vec = field.find_with_hint_rect(rect).collect::<Vec<_>>();
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
