use super::*;

type RectNode<T, U> = rstar::primitives::GeomWithData<rstar::primitives::Rectangle<T>, U>;

pub type EntityKey = (u32, u32);

#[derive(Debug, Clone)]
pub struct EntityDescriptor {
    pub display_name: String,
    pub description: String,
    pub collision_size: Vec2,
    pub collision_offset: Vec2,
    pub hint_size: Vec2,
    pub hint_offset: Vec2,
    pub z_along_y: bool,
}

#[derive(Debug, Clone)]
pub struct EntityFieldDescriptor {
    pub entities: Vec<EntityDescriptor>,
}

#[derive(Debug, Clone)]
pub struct EntityProperty {
    display_name: String,
    description: String,
    collision_size: Vec2,
    collision_offset: Vec2,
    hint_size: Vec2,
    hint_offset: Vec2,
    z_along_y: bool,
}

impl EntityProperty {
    #[rustfmt::skip]
    fn collision_rect(&self, location: Vec2) -> Option<[Vec2; 2]> {
        if self.collision_size.x * self.collision_size.y == 0.0 {
            return None;
        }

        Some([
            location + self.collision_offset,
            location + self.collision_offset + self.collision_size,
        ])
    }

    #[rustfmt::skip]
    fn hint_rect(&self, location: Vec2) -> Option<[Vec2; 2]> {
        if self.hint_size.x * self.hint_size.y == 0.0 {
            return None;
        }

        Some([
            location + self.hint_offset,
            location + self.hint_offset + self.hint_size,
        ])
    }
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub id: u16,
    pub location: Vec2,
    pub data: Box<dyn EntityData>,
    pub render_param: EntityRenderParam,
}

#[derive(Debug, Clone, Default)]
struct EntityChunk {
    version: u64,
    entities: slab::Slab<Entity>,
}

#[derive(Debug, Clone)]
pub struct EntityField {
    props: Vec<EntityProperty>,
    chunks: Vec<EntityChunk>,
    chunk_ref: ahash::AHashMap<IVec2, u32>,
    collision_ref: rstar::RTree<RectNode<[f32; 2], EntityKey>>,
    hint_ref: rstar::RTree<RectNode<[f32; 2], EntityKey>>,
}

impl EntityField {
    const CHUNK_SIZE: u32 = 32;

    pub fn new(desc: EntityFieldDescriptor) -> Self {
        let mut props = vec![];

        for entity in desc.entities {
            if entity.collision_size.x < 0.0 || entity.collision_size.y < 0.0 {
                panic!("collision size must be non-negative");
            }
            if entity.hint_size.x < 0.0 || entity.hint_size.y < 0.0 {
                panic!("hint size must be non-negative");
            }

            props.push(EntityProperty {
                display_name: entity.display_name,
                description: entity.description,
                collision_size: entity.collision_size,
                collision_offset: entity.collision_offset,
                hint_size: entity.hint_size,
                hint_offset: entity.hint_offset,
                z_along_y: entity.z_along_y,
            });
        }

        Self {
            props,
            chunks: Default::default(),
            chunk_ref: Default::default(),
            collision_ref: Default::default(),
            hint_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, entity: Entity) -> Result<EntityKey, EntityError> {
        let prop = self
            .props
            .get(entity.id as usize)
            .ok_or(EntityError::InvalidId)?;

        let chunk_size = Vec2::splat(Self::CHUNK_SIZE as f32);
        let chunk_location = entity.location.div_euclid(chunk_size).as_ivec2();

        // get or allocate chunk
        let chunk_key = if let Some(chunk_key) = self.chunk_ref.get(&chunk_location) {
            *chunk_key
        } else {
            if self.chunks.len() >= u32::MAX as usize {
                panic!("capacity overflow");
            }

            let chunk_key = self.chunks.len() as u32;
            self.chunks.push(EntityChunk {
                version: 0,
                entities: Default::default(),
            });
            self.chunk_ref.insert(chunk_location, chunk_key);
            chunk_key
        };

        let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();

        if chunk.entities.vacant_key() >= u32::MAX as usize {
            panic!("capacity overflow");
        }

        let local_key = chunk.entities.vacant_key() as u32;

        // collision features
        if let Some(rect) = prop.collision_rect(entity.location) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = rstar::primitives::GeomWithData::new(rect, (chunk_key, local_key));
            self.collision_ref.insert(node);
        }

        // hint features
        if let Some(rect) = prop.hint_rect(entity.location) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = rstar::primitives::GeomWithData::new(rect, (chunk_key, local_key));
            self.hint_ref.insert(node);
        }

        // entity_key is guaranteed to be less than u32::MAX.
        chunk.entities.insert(entity);
        chunk.version += 1;

        Ok((chunk_key, local_key))
    }

    pub fn remove(&mut self, key: EntityKey) -> Result<Entity, EntityError> {
        let (chunk_key, local_key) = key;

        let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
        let entity = chunk
            .entities
            .try_remove(local_key as usize)
            .ok_or(EntityError::NotFound)?;
        chunk.version += 1;

        let prop = self.props.get(entity.id as usize).unwrap();

        // collision features
        if let Some(rect) = prop.collision_rect(entity.location) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = &rstar::primitives::GeomWithData::new(rect, key);
            self.collision_ref.remove(node).unwrap();
        }

        // hint features
        if let Some(rect) = prop.hint_rect(entity.location) {
            let p1 = [rect[0].x, rect[0].y];
            let p2 = [rect[1].x, rect[1].y];
            let rect = rstar::primitives::Rectangle::from_corners(p1, p2);
            let node = &rstar::primitives::GeomWithData::new(rect, key);
            self.hint_ref.remove(node).unwrap();
        }

        Ok(entity)
    }

    pub fn modify(
        &mut self,
        key: EntityKey,
        f: impl FnOnce(&mut Entity),
    ) -> Result<EntityKey, EntityError> {
        let (chunk_key, local_key) = key;

        let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
        let entity = chunk
            .entities
            .get_mut(local_key as usize)
            .ok_or(EntityError::NotFound)?;

        let mut new_entity = Entity {
            id: entity.id,
            location: entity.location,
            data: std::mem::take(&mut entity.data),
            render_param: entity.render_param.clone(),
        };
        f(&mut new_entity);

        if new_entity.id != entity.id {
            entity.data = new_entity.data;
            return Err(EntityError::InvalidId);
        }

        if new_entity.location != entity.location {
            self.remove(key).unwrap();
            let key = self.insert(new_entity).unwrap();
            return Ok(key);
        }

        if new_entity.render_param != entity.render_param {
            let chunk = self.chunks.get_mut(chunk_key as usize).unwrap();
            *chunk.entities.get_mut(local_key as usize).unwrap() = new_entity;
            chunk.version += 1;
            return Ok(key);
        }

        entity.data = new_entity.data;
        Ok(key)
    }

    pub fn get(&self, key: EntityKey) -> Result<&Entity, EntityError> {
        let (chunk_key, local_key) = key;

        let chunk = self.chunks.get(chunk_key as usize).unwrap();
        let entity = chunk
            .entities
            .get(local_key as usize)
            .ok_or(EntityError::NotFound)?;
        Ok(entity)
    }

    // transfer chunk data

    pub fn get_version_by_chunk_location(&self, chunk_location: IVec2) -> Result<u64, EntityError> {
        let chunk_key = self
            .chunk_ref
            .get(&chunk_location)
            .ok_or(EntityError::NotFound)?;
        let chunk = self.chunks.get(*chunk_key as usize).unwrap();

        Ok(chunk.version)
    }

    pub fn get_keys_by_chunk_location(
        &self,
        chunk_location: IVec2,
    ) -> Result<impl Iterator<Item = BlockKey>, EntityError> {
        let chunk_key = self
            .chunk_ref
            .get(&chunk_location)
            .ok_or(EntityError::NotFound)?;
        let chunk = self.chunks.get(*chunk_key as usize).unwrap();

        let keys = chunk
            .entities
            .iter()
            .map(move |(local_key, _)| (*chunk_key, local_key as u32));
        Ok(keys)
    }

    // property

    pub fn get_display_name(&self, key: EntityKey) -> Result<&str, EntityError> {
        let entity = self.get(key)?;
        let prop = self.props.get(entity.id as usize).unwrap();
        Ok(&prop.display_name)
    }

    pub fn get_description(&self, key: EntityKey) -> Result<&str, EntityError> {
        let entity = self.get(key)?;
        let prop = self.props.get(entity.id as usize).unwrap();
        Ok(&prop.description)
    }

    // spatial features

    pub fn get_chunk_location(&self, point: Vec2) -> IVec2 {
        let chunk_size = Vec2::splat(Self::CHUNK_SIZE as f32);
        point.div_euclid(chunk_size).as_ivec2()
    }

    // collision features

    pub fn get_base_collision_rect(&self, id: u16) -> Result<[Vec2; 2], EntityError> {
        let prop = self.props.get(id as usize).ok_or(EntityError::InvalidId)?;
        Ok(prop.collision_rect(Default::default()).unwrap_or_default())
    }

    pub fn get_collision_rect(&self, entity_key: EntityKey) -> Result<[Vec2; 2], EntityError> {
        let entity = self.get(entity_key)?;
        let prop = self.props.get(entity.id as usize).unwrap();
        Ok(prop.collision_rect(entity.location).unwrap_or_default())
    }

    pub fn has_by_collision_point(&self, point: Vec2) -> bool {
        let point = [point.x, point.y];
        self.collision_ref.locate_at_point(&point).is_some()
    }

    pub fn get_keys_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = EntityKey> + '_ {
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
    ) -> impl Iterator<Item = EntityKey> + '_ {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
    }

    // hint features

    pub fn get_base_z_along_y(&self, id: u16) -> Result<bool, EntityError> {
        let prop = self.props.get(id as usize).ok_or(EntityError::InvalidId)?;
        Ok(prop.z_along_y)
    }

    pub fn get_base_hint_rect(&self, id: u16) -> Result<[Vec2; 2], EntityError> {
        let prop = self.props.get(id as usize).ok_or(EntityError::InvalidId)?;
        Ok(prop.hint_rect(Default::default()).unwrap_or_default())
    }

    pub fn get_hint_rect(&self, entity_key: EntityKey) -> Result<[Vec2; 2], EntityError> {
        let entity = self.get(entity_key)?;
        let prop = self.props.get(entity.id as usize).unwrap();
        Ok(prop.hint_rect(entity.location).unwrap_or_default())
    }

    pub fn has_by_hint_point(&self, point: Vec2) -> bool {
        let point = [point.x, point.y];
        self.hint_ref.locate_at_point(&point).is_some()
    }

    pub fn get_keys_by_hint_point(&self, point: Vec2) -> impl Iterator<Item = EntityKey> + '_ {
        let point = [point.x, point.y];
        self.hint_ref
            .locate_all_at_point(&point)
            .map(|node| node.data)
    }

    pub fn has_by_hint_rect(&self, rect: [Vec2; 2]) -> bool {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    pub fn get_keys_by_hint_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = EntityKey> + '_ {
        let p1 = [rect[0].x, rect[0].y];
        let p2 = [rect[1].x, rect[1].y];
        let rect = rstar::AABB::from_corners(p1, p2);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| node.data)
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
        let _: EntityField = EntityField::new(EntityFieldDescriptor {
            entities: vec![EntityDescriptor {
                display_name: "entity_0".into(),
                description: "entity_0_desc".into(),
                collision_size: Vec2::new(-1.0, -1.0),
                collision_offset: Vec2::new(0.0, 0.0),
                hint_size: Vec2::new(1.0, 1.0),
                hint_offset: Vec2::new(0.0, 0.0),
                z_along_y: false,
            }],
        });
    }

    #[test]
    #[should_panic]
    fn entity_field_with_invalid_hint() {
        let _: EntityField = EntityField::new(EntityFieldDescriptor {
            entities: vec![EntityDescriptor {
                display_name: "entity_0".into(),
                description: "entity_0_desc".into(),
                collision_size: Vec2::new(1.0, 1.0),
                collision_offset: Vec2::new(0.0, 0.0),
                hint_size: Vec2::new(-1.0, -1.0),
                hint_offset: Vec2::new(0.0, 0.0),
                z_along_y: false,
            }],
        });
    }

    #[test]
    fn crud_entity() {
        let mut field: EntityField = EntityField::new(EntityFieldDescriptor {
            entities: vec![
                EntityDescriptor {
                    display_name: "entity_0".into(),
                    description: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                EntityDescriptor {
                    display_name: "entity_1".into(),
                    description: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        let key = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        let entity = field.get(key).unwrap();
        assert_eq!(entity.id, 1);
        assert_eq!(entity.location, Vec2::new(-1.0, 3.0));

        let entity = field.remove(key).unwrap();
        assert_eq!(entity.id, 1);
        assert_eq!(entity.location, Vec2::new(-1.0, 3.0));

        assert_eq!(field.get(key).unwrap_err(), EntityError::NotFound);
        assert_eq!(field.remove(key).unwrap_err(), EntityError::NotFound);
    }

    #[test]
    fn insert_entity_with_invalid() {
        let mut field: EntityField = EntityField::new(EntityFieldDescriptor {
            entities: vec![
                EntityDescriptor {
                    display_name: "entity_0".into(),
                    description: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                EntityDescriptor {
                    display_name: "entity_1".into(),
                    description: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        assert_eq!(
            field.insert(Entity {
                id: 2,
                location: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_param: Default::default(),
            }),
            Err(EntityError::InvalidId)
        );
    }

    #[test]
    fn modify_entity() {
        let mut field: EntityField = EntityField::new(EntityFieldDescriptor {
            entities: vec![
                EntityDescriptor {
                    display_name: "entity_0".into(),
                    description: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                EntityDescriptor {
                    display_name: "entity_1".into(),
                    description: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        let key = field
            .insert(Entity {
                id: 0,
                location: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        let key = field
            .modify(key, |entity| entity.location = Vec2::new(-1.0, 4.0))
            .unwrap();

        let entity = field.get(key).unwrap();
        assert_eq!(entity.id, 0);
        assert_eq!(entity.location, Vec2::new(-1.0, 4.0));

        let key = field
            .modify(key, |entity| entity.render_param.variant = 1)
            .unwrap();

        let entity = field.get(key).unwrap();
        assert_eq!(entity.id, 0);
        assert_eq!(entity.location, Vec2::new(-1.0, 4.0));
        assert_eq!(entity.render_param.variant, 1);

        let key = field.modify(key, |_| {}).unwrap();

        let entity = field.get(key).unwrap();
        assert_eq!(entity.id, 0);
        assert_eq!(entity.location, Vec2::new(-1.0, 4.0));
        assert_eq!(entity.render_param.variant, 1);
    }

    #[test]
    fn modify_entity_with_invalid() {
        let mut field: EntityField = EntityField::new(EntityFieldDescriptor {
            entities: vec![
                EntityDescriptor {
                    display_name: "entity_0".into(),
                    description: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                EntityDescriptor {
                    display_name: "entity_1".into(),
                    description: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        let key = field
            .insert(Entity {
                id: 0,
                location: Vec2::new(-1.0, 4.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        assert_eq!(
            field.modify(key, |entity| entity.id = 1),
            Err(EntityError::InvalidId)
        );

        field.remove(key).unwrap();
        assert_eq!(field.modify(key, |_| {}), Err(EntityError::NotFound));
        assert_eq!(field.get(key).unwrap_err(), EntityError::NotFound);
    }

    #[test]
    fn modify_entity_with_move() {
        let mut field: EntityField = EntityField::new(EntityFieldDescriptor {
            entities: vec![
                EntityDescriptor {
                    display_name: "entity_0".into(),
                    description: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                EntityDescriptor {
                    display_name: "entity_1".into(),
                    description: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        let key = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        let key = field
            .modify(key, |tile| tile.location = Vec2::new(-1.0, 1000.0))
            .unwrap();

        let entity = field.get(key).unwrap();
        assert_eq!(entity.id, 1);
        assert_eq!(entity.location, Vec2::new(-1.0, 1000.0));
    }

    #[test]
    fn collision_entity() {
        let mut field: EntityField = EntityField::new(EntityFieldDescriptor {
            entities: vec![
                EntityDescriptor {
                    display_name: "entity_0".into(),
                    description: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                EntityDescriptor {
                    display_name: "entity_1".into(),
                    description: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        let key_0 = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let key_1 = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 4.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let _key_2 = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 5.0),
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
        assert_eq!(field.get_collision_rect(key_0), Err(EntityError::NotFound));
    }

    #[test]
    fn hint_entity() {
        let mut field: EntityField = EntityField::new(EntityFieldDescriptor {
            entities: vec![
                EntityDescriptor {
                    display_name: "entity_0".into(),
                    description: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                EntityDescriptor {
                    display_name: "entity_1".into(),
                    description: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        let key_0 = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let key_1 = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 4.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let _key_2 = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 5.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        assert_eq!(
            field.get_hint_rect(key_0),
            Ok([Vec2::new(-1.0, 3.0), Vec2::new(0.0, 4.0)])
        );

        let point = Vec2::new(-1.0, 4.0);
        assert!(field.has_by_hint_point(point));
        let vec = field.get_keys_by_hint_point(point).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        let rect = [Vec2::new(-1.0, 3.0), Vec2::new(-1.0, 4.0)];
        assert!(field.has_by_hint_rect(rect));
        let vec = field.get_keys_by_hint_rect(rect).collect::<Vec<_>>();
        assert!(vec.contains(&key_0));
        assert!(vec.contains(&key_1));

        field.remove(key_0).unwrap();
        assert_eq!(field.get_hint_rect(key_0), Err(EntityError::NotFound));
    }

    #[test]
    fn entity_chunk() {
        let mut field: EntityField = EntityField::new(EntityFieldDescriptor {
            entities: vec![
                EntityDescriptor {
                    display_name: "entity_0".into(),
                    description: "entity_0_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
                EntityDescriptor {
                    display_name: "entity_1".into(),
                    description: "entity_1_desc".into(),
                    collision_size: Vec2::new(1.0, 1.0),
                    collision_offset: Vec2::new(0.0, 0.0),
                    hint_size: Vec2::new(1.0, 1.0),
                    hint_offset: Vec2::new(0.0, 0.0),
                    z_along_y: false,
                },
            ],
        });

        let _key0 = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 3.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let _key1 = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 4.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();
        let _key2 = field
            .insert(Entity {
                id: 1,
                location: Vec2::new(-1.0, 5.0),
                data: Default::default(),
                render_param: Default::default(),
            })
            .unwrap();

        assert!(field.get_keys_by_chunk_location(IVec2::new(0, 0)).is_err());

        let keys = field.get_keys_by_chunk_location(IVec2::new(-1, 0)).unwrap();
        assert_eq!(keys.count(), 3);
    }
}
