use super::*;

type RectNode<T, U> = rstar::primitives::GeomWithData<rstar::primitives::Rectangle<T>, U>;

pub type EntityId = (u32, u32);

#[derive(Debug, Clone)]
pub struct EntityInfo {
    pub display_name: String,
    pub description: String,
    pub collision_size: Vec2,
    pub collision_offset: Vec2,
    pub hint_size: Vec2,
    pub hint_offset: Vec2,
    pub y_sorting: bool,
}

#[derive(Debug, Clone)]
pub struct EntityFieldInfo {
    pub entities: Vec<EntityInfo>,
}

#[derive(Debug, Clone)]
pub struct EntityArchetype {
    display_name: String,
    description: String,
    collision_size: Vec2,
    collision_offset: Vec2,
    hint_size: Vec2,
    hint_offset: Vec2,
    y_sorting: bool,
}

impl EntityArchetype {
    #[rustfmt::skip]
    fn collision_rect(&self, coord: Vec2) -> Option<[Vec2; 2]> {
        if self.collision_size.x * self.collision_size.y == 0.0 {
            return None;
        }

        Some([coord + self.collision_offset, coord + self.collision_offset + self.collision_size])
    }

    #[rustfmt::skip]
    fn hint_rect(&self, coord: Vec2) -> Option<[Vec2; 2]> {
        if self.hint_size.x * self.hint_size.y == 0.0 {
            return None;
        }

        Some([coord + self.hint_offset, coord + self.hint_offset + self.hint_size])
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntityRenderState {
    pub variant: u8,
    pub tick: u32,
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub archetype_id: u16,
    pub coord: Vec2,
    pub data: Box<dyn EntityData>,  // TODO: removal
    pub render_state: EntityRenderState,
}

#[derive(Debug, Clone, Default)]
struct EntityChunk {
    version: u64,
    entities: slab::Slab<Entity>,
}

#[derive(Debug, Clone)]
pub struct EntityField {
    archetypes: Vec<EntityArchetype>,
    chunks: Vec<EntityChunk>,
    chunk_index: ahash::AHashMap<IVec2, u32>,
    collision_index: rstar::RTree<RectNode<[f32; 2], EntityId>>,
    hint_index: rstar::RTree<RectNode<[f32; 2], EntityId>>,
}

impl EntityField {
    const CHUNK_SIZE: u32 = 32;

    pub fn new(desc: EntityFieldInfo) -> Self {
        let mut archetyps = vec![];

        for entity in desc.entities {
            if entity.collision_size.x < 0.0 || entity.collision_size.y < 0.0 {
                panic!("collision size must be non-negative");
            }
            if entity.hint_size.x < 0.0 || entity.hint_size.y < 0.0 {
                panic!("hint size must be non-negative");
            }

            archetyps.push(EntityArchetype {
                display_name: entity.display_name,
                description: entity.description,
                collision_size: entity.collision_size,
                collision_offset: entity.collision_offset,
                hint_size: entity.hint_size,
                hint_offset: entity.hint_offset,
                y_sorting: entity.y_sorting,
            });
        }

        Self {
            archetypes: archetyps,
            chunks: Default::default(),
            chunk_index: Default::default(),
            collision_index: Default::default(),
            hint_index: Default::default(),
        }
    }

    pub fn insert(&mut self, entity: Entity) -> Result<EntityId, EntityError> {
        let archetype = self
            .archetypes
            .get(entity.archetype_id as usize)
            .ok_or(EntityError::InvalidId)?;

        let chunk_size = Vec2::splat(Self::CHUNK_SIZE as f32);
        let chunk_coord = entity.coord.div_euclid(chunk_size).as_ivec2();

        // get or allocate chunk
        let chunk_id = if let Some(chunk_id) = self.chunk_index.get(&chunk_coord) {
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
            self.chunk_index.insert(chunk_coord, chunk_id);
            chunk_id
        };

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();

        if chunk.entities.vacant_key() >= u32::MAX as usize {
            panic!("capacity overflow");
        }

        let local_id = chunk.entities.vacant_key() as u32;

        // collision features
        if let Some(rect) = archetype.collision_rect(entity.coord) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = rstar::primitives::GeomWithData::new(rect, (chunk_id, local_id));
            self.collision_index.insert(node);
        }

        // hint features
        if let Some(rect) = archetype.hint_rect(entity.coord) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = rstar::primitives::GeomWithData::new(rect, (chunk_id, local_id));
            self.hint_index.insert(node);
        }

        // entity_key is guaranteed to be less than u32::MAX.
        chunk.entities.insert(entity);
        chunk.version += 1;

        Ok((chunk_id, local_id))
    }

    pub fn remove(&mut self, id: EntityId) -> Result<Entity, EntityError> {
        let (chunk_id, local_id) = id;

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let entity = chunk
            .entities
            .try_remove(local_id as usize)
            .ok_or(EntityError::NotFound)?;
        chunk.version += 1;

        let archetype = self.archetypes.get(entity.archetype_id as usize).unwrap();

        // collision features
        if let Some(rect) = archetype.collision_rect(entity.coord) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = &rstar::primitives::GeomWithData::new(rect, id);
            self.collision_index.remove(node).unwrap();
        }

        // hint features
        if let Some(rect) = archetype.hint_rect(entity.coord) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = &rstar::primitives::GeomWithData::new(rect, id);
            self.hint_index.remove(node).unwrap();
        }

        Ok(entity)
    }

    pub fn modify(
        &mut self,
        id: EntityId,
        f: impl FnOnce(&mut Entity),
    ) -> Result<EntityId, EntityError> {
        let (chunk_id, local_id) = id;

        let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
        let entity = chunk
            .entities
            .get_mut(local_id as usize)
            .ok_or(EntityError::NotFound)?;

        let mut new_entity = Entity {
            archetype_id: entity.archetype_id,
            coord: entity.coord,
            data: std::mem::take(&mut entity.data),
            render_state: entity.render_state.clone(),
        };
        f(&mut new_entity);

        if new_entity.archetype_id != entity.archetype_id {
            entity.data = new_entity.data;
            return Err(EntityError::InvalidId);
        }

        if new_entity.coord != entity.coord {
            self.remove(id).unwrap();
            let new_id = self.insert(new_entity).unwrap();
            return Ok(new_id);
        }

        if new_entity.render_state != entity.render_state {
            let chunk = self.chunks.get_mut(chunk_id as usize).unwrap();
            *chunk.entities.get_mut(local_id as usize).unwrap() = new_entity;
            chunk.version += 1;
            return Ok(id);
        }

        entity.data = new_entity.data;
        Ok(id)
    }

    pub fn get(&self, id: EntityId) -> Result<&Entity, EntityError> {
        let (chunk_id, local_id) = id;

        let chunk = self.chunks.get(chunk_id as usize).unwrap();
        let entity = chunk
            .entities
            .get(local_id as usize)
            .ok_or(EntityError::NotFound)?;
        Ok(entity)
    }

    // transfer chunk data

    pub fn get_version_by_chunk_coord(&self, chunk_coord: IVec2) -> Result<u64, EntityError> {
        let chunk_id = self
            .chunk_index
            .get(&chunk_coord)
            .ok_or(EntityError::NotFound)?;
        let chunk = self.chunks.get(*chunk_id as usize).unwrap();

        Ok(chunk.version)
    }

    pub fn get_ids_by_chunk_coord(&self, chunk_coord: IVec2) -> Result<impl Iterator<Item = EntityId>, EntityError> {
        let chunk_id = self
            .chunk_index
            .get(&chunk_coord)
            .ok_or(EntityError::NotFound)?;
        let chunk = self.chunks.get(*chunk_id as usize).unwrap();

        let ids = chunk
            .entities
            .iter()
            .map(move |(local_id, _)| (*chunk_id, local_id as u32));
        Ok(ids)
    }

    // property

    pub fn get_display_name(&self, id: EntityId) -> Result<&str, EntityError> {
        let entity = self.get(id)?;
        let archetype = self.archetypes.get(entity.archetype_id as usize).unwrap();
        Ok(&archetype.display_name)
    }

    pub fn get_description(&self, id: EntityId) -> Result<&str, EntityError> {
        let entity = self.get(id)?;
        let archetype = self.archetypes.get(entity.archetype_id as usize).unwrap();
        Ok(&archetype.description)
    }

    // spatial features

    pub fn get_chunk_coord(&self, point: Vec2) -> IVec2 {
        let chunk_size = Vec2::splat(Self::CHUNK_SIZE as f32);
        point.div_euclid(chunk_size).as_ivec2()
    }

    // collision features

    pub fn get_base_collision_rect(&self, archetype_id: u16) -> Result<[Vec2; 2], EntityError> {
        let archetype = self.archetypes.get(archetype_id as usize).ok_or(EntityError::InvalidId)?;
        Ok(archetype.collision_rect(Default::default()).unwrap_or_default())
    }

    pub fn get_collision_rect(&self, id: EntityId) -> Result<[Vec2; 2], EntityError> {
        let entity = self.get(id)?;
        let archetype = self.archetypes.get(entity.archetype_id as usize).unwrap();
        Ok(archetype.collision_rect(entity.coord).unwrap_or_default())
    }

    pub fn has_by_collision_point(&self, point: Vec2) -> bool {
        let point = [point.x, point.y];
        self.collision_index.locate_at_point(&point).is_some()
    }

    pub fn get_ids_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = EntityId> + '_ {
        let point = [point.x, point.y];
        self.collision_index.locate_all_at_point(&point).map(|node| node.data)
    }

    pub fn has_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.collision_index.locate_in_envelope_intersecting(&rect).next().is_some()
    }

    pub fn get_ids_by_collision_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = EntityId> + '_ {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.collision_index.locate_in_envelope_intersecting(&rect).map(|node| node.data)
    }

    // hint features

    pub fn get_base_y_sorting(&self, archetype_id: u16) -> Result<bool, EntityError> {
        let archetype = self.archetypes.get(archetype_id as usize).ok_or(EntityError::InvalidId)?;
        Ok(archetype.y_sorting)
    }

    pub fn get_base_hint_rect(&self, archetype_id: u16) -> Result<[Vec2; 2], EntityError> {
        let archetype = self.archetypes.get(archetype_id as usize).ok_or(EntityError::InvalidId)?;
        Ok(archetype.hint_rect(Default::default()).unwrap_or_default())
    }

    pub fn get_hint_rect(&self, id: EntityId) -> Result<[Vec2; 2], EntityError> {
        let entity = self.get(id)?;
        let archetype = self.archetypes.get(entity.archetype_id as usize).unwrap();
        Ok(archetype.hint_rect(entity.coord).unwrap_or_default())
    }

    pub fn has_by_hint_point(&self, point: Vec2) -> bool {
        let point = [point.x, point.y];
        self.hint_index.locate_at_point(&point).is_some()
    }

    pub fn get_ids_by_hint_point(&self, point: Vec2) -> impl Iterator<Item = EntityId> + '_ {
        let point = [point.x, point.y];
        self.hint_index.locate_all_at_point(&point).map(|node| node.data)
    }

    pub fn has_by_hint_rect(&self, rect: [Vec2; 2]) -> bool {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.hint_index.locate_in_envelope_intersecting(&rect).next().is_some()
    }

    pub fn get_ids_by_hint_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = EntityId> + '_ {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.hint_index.locate_in_envelope_intersecting(&rect).map(|node| node.data)
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

    #[test]
    #[should_panic]
    fn entity_field_with_invalid_collision() {
        let _: EntityField = EntityField::new(EntityFieldInfo {
            entities: vec![EntityInfo {
                display_name: "entity_0".into(),
                description: "entity_0_desc".into(),
                collision_size: Vec2::new(-1.0, -1.0),
                collision_offset: Vec2::new(0.0, 0.0),
                hint_size: Vec2::new(1.0, 1.0),
                hint_offset: Vec2::new(0.0, 0.0),
                y_sorting: false,
            }],
        });
    }

    #[test]
    #[should_panic]
    fn entity_field_with_invalid_hint() {
        let _: EntityField = EntityField::new(EntityFieldInfo {
            entities: vec![EntityInfo {
                display_name: "entity_0".into(),
                description: "entity_0_desc".into(),
                collision_size: Vec2::new(1.0, 1.0),
                collision_offset: Vec2::new(0.0, 0.0),
                hint_size: Vec2::new(-1.0, -1.0),
                hint_offset: Vec2::new(0.0, 0.0),
                y_sorting: false,
            }],
        });
    }

    #[test]
    fn crud_entity() {
        let mut field: EntityField = EntityField::new(EntityFieldInfo {
            entities: vec![
                EntityInfo {
                    display_name: "entity_0".into(),
                    description: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    y_sorting: false,
                },
                EntityInfo {
                    display_name: "entity_1".into(),
                    description: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    y_sorting: false,
                },
            ],
        });

        let id = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_state: Default::default(),
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
        let mut field: EntityField = EntityField::new(EntityFieldInfo {
            entities: vec![
                EntityInfo {
                    display_name: "entity_0".into(),
                    description: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    y_sorting: false,
                },
                EntityInfo {
                    display_name: "entity_1".into(),
                    description: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    y_sorting: false,
                },
            ],
        });

        assert_eq!(
            field.insert(Entity {
                archetype_id: 2,
                coord: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_state: Default::default(),
            }),
            Err(EntityError::InvalidId)
        );
    }

    #[test]
    fn modify_entity() {
        let mut field: EntityField = EntityField::new(EntityFieldInfo {
            entities: vec![
                EntityInfo {
                    display_name: "entity_0".into(),
                    description: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    y_sorting: false,
                },
                EntityInfo {
                    display_name: "entity_1".into(),
                    description: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    y_sorting: false,
                },
            ],
        });

        let id = field
            .insert(Entity {
                archetype_id: 0,
                coord: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();

        let id = field
            .modify(id, |entity| entity.coord = Vec2::new(-1.0, 4.0))
            .unwrap();

        let entity = field.get(id).unwrap();
        assert_eq!(entity.archetype_id, 0);
        assert_eq!(entity.coord, Vec2::new(-1.0, 4.0));

        let id = field
            .modify(id, |entity| entity.render_state.variant = 1)
            .unwrap();

        let entity = field.get(id).unwrap();
        assert_eq!(entity.archetype_id, 0);
        assert_eq!(entity.coord, Vec2::new(-1.0, 4.0));
        assert_eq!(entity.render_state.variant, 1);

        let id = field.modify(id, |_| {}).unwrap();

        let entity = field.get(id).unwrap();
        assert_eq!(entity.archetype_id, 0);
        assert_eq!(entity.coord, Vec2::new(-1.0, 4.0));
        assert_eq!(entity.render_state.variant, 1);
    }

    #[test]
    fn modify_entity_with_invalid() {
        let mut field: EntityField = EntityField::new(EntityFieldInfo {
            entities: vec![
                EntityInfo {
                    display_name: "entity_0".into(),
                    description: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    y_sorting: false,
                },
                EntityInfo {
                    display_name: "entity_1".into(),
                    description: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    y_sorting: false,
                },
            ],
        });

        let id = field
            .insert(Entity {
                archetype_id: 0,
                coord: Vec2::new(-1.0, 4.0),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();

        assert_eq!(field.modify(id, |entity| entity.archetype_id = 1), Err(EntityError::InvalidId));

        field.remove(id).unwrap();
        assert_eq!(field.modify(id, |_| {}), Err(EntityError::NotFound));
        assert_eq!(field.get(id).unwrap_err(), EntityError::NotFound);
    }

    #[test]
    fn modify_entity_with_move() {
        let mut field: EntityField = EntityField::new(EntityFieldInfo {
            entities: vec![
                EntityInfo {
                    display_name: "entity_0".into(),
                    description: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    y_sorting: false,
                },
                EntityInfo {
                    display_name: "entity_1".into(),
                    description: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    y_sorting: false,
                },
            ],
        });

        let id = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();

        let id = field
            .modify(id, |tile| tile.coord = Vec2::new(-1.0, 1000.0))
            .unwrap();

        let entity = field.get(id).unwrap();
        assert_eq!(entity.archetype_id, 1);
        assert_eq!(entity.coord, Vec2::new(-1.0, 1000.0));
    }

    #[test]
    fn collision_entity() {
        let mut field: EntityField = EntityField::new(EntityFieldInfo {
            entities: vec![
                EntityInfo {
                    display_name: "entity_0".into(),
                    description: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    y_sorting: false,
                },
                EntityInfo {
                    display_name: "entity_1".into(),
                    description: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    y_sorting: false,
                },
            ],
        });

        let id0 = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        let id1 = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 4.0),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        let _ = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 5.0),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();

        assert_eq!(field.get_collision_rect(id0), Ok([Vec2::new(-1.0, 3.0), Vec2::new(0.0, 4.0)]));

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
        assert_eq!(field.get_collision_rect(id0), Err(EntityError::NotFound));
    }

    #[test]
    fn hint_entity() {
        let mut field: EntityField = EntityField::new(EntityFieldInfo {
            entities: vec![
                EntityInfo {
                    display_name: "entity_0".into(),
                    description: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    y_sorting: false,
                },
                EntityInfo {
                    display_name: "entity_1".into(),
                    description: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    y_sorting: false,
                },
            ],
        });

        let id0 = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        let id1 = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 4.0),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        let _ = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 5.0),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();

        assert_eq!(field.get_hint_rect(id0), Ok([Vec2::new(-1.0, 3.0), Vec2::new(0.0, 4.0)]));

        let point = Vec2::new(-1.0, 4.0);
        assert!(field.has_by_hint_point(point));
        let vec = field.get_ids_by_hint_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&id0));
        assert!(vec.contains(&id1));

        let rect = [Vec2::new(-1.0, 3.0), Vec2::new(-1.0, 4.0)];
        assert!(field.has_by_hint_rect(rect));
        let vec = field.get_ids_by_hint_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&id0));
        assert!(vec.contains(&id1));

        field.remove(id0).unwrap();
        assert_eq!(field.get_hint_rect(id0), Err(EntityError::NotFound));
    }

    #[test]
    fn entity_chunk() {
        let mut field: EntityField = EntityField::new(EntityFieldInfo {
            entities: vec![
                EntityInfo {
                    display_name: "entity_0".into(),
                    description: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    y_sorting: false,
                },
                EntityInfo {
                    display_name: "entity_1".into(),
                    description: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    y_sorting: false,
                },
            ],
        });

        let _ = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        let _ = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 4.0),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();
        let _ = field
            .insert(Entity {
                archetype_id: 1,
                coord: Vec2::new(-1.0, 5.0),
                data: Default::default(),
                render_state: Default::default(),
            })
            .unwrap();

        assert!(field.get_ids_by_chunk_coord(IVec2::new(0, 0)).is_err());

        let ids = field.get_ids_by_chunk_coord(IVec2::new(-1, 0)).unwrap();
        assert_eq!(ids.count(), 3);
    }
}
