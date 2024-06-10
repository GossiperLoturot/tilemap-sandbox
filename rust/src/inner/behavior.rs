use super::*;
use godot::log::godot_print;

// Behavior Plugin

pub type BehaviorKey = (BehaviorKind, u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BehaviorRelation {
    Global,
    Tile(u32),
    Block(u32),
    Entity(u32),
}

#[derive(Debug, Clone, strum_macros::EnumDiscriminants)]
#[strum_discriminants(name(BehaviorKind))]
#[strum_discriminants(derive(Hash))]
pub enum BehaviorInner {
    Unit,
    RandomWalk(RandomWalk),
    Generator(Generator),
}

#[derive(Debug, Clone)]
pub struct Behavior {
    pub inner: BehaviorInner,
    pub relation: BehaviorRelation,
}

impl<'a> From<&'a Behavior> for BehaviorKind {
    fn from(value: &'a Behavior) -> Self {
        (&value.inner).into()
    }
}

pub struct BehaviorRef<'a> {
    pub inner: &'a BehaviorInner,
    pub relation: &'a BehaviorRelation,
}

impl<'a> From<&'a Behavior> for BehaviorRef<'a> {
    fn from(value: &'a Behavior) -> Self {
        Self {
            inner: &value.inner,
            relation: &value.relation,
        }
    }
}

pub struct BehaviorMut<'a> {
    pub inner: &'a mut BehaviorInner,
    pub relation: &'a BehaviorRelation,
}

impl<'a> From<&'a mut Behavior> for BehaviorMut<'a> {
    fn from(value: &'a mut Behavior) -> Self {
        Self {
            inner: &mut value.inner,
            relation: &value.relation,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BehaviorFactory {
    Unit,
    RandomWalk(RandomWalkFactory),
    Generator(GeneratorFactory),
}

impl<'a> From<&'a BehaviorFactory> for BehaviorInner {
    fn from(value: &'a BehaviorFactory) -> Self {
        match value {
            BehaviorFactory::Unit => Self::Unit,
            BehaviorFactory::RandomWalk(factory) => Self::RandomWalk(factory.create()),
            BehaviorFactory::Generator(factory) => Self::Generator(factory.create()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BehaviorPlugin {
    tile_factories: Vec<BehaviorFactory>,
    block_factories: Vec<BehaviorFactory>,
    entity_factories: Vec<BehaviorFactory>,
    ecs: ecs_tiny::ECS<Behavior, BehaviorKind>,
    relation_ref: ahash::AHashMap<BehaviorRelation, u32>,
}

impl BehaviorPlugin {
    pub fn new(
        tile_factories: Vec<BehaviorFactory>,
        block_factories: Vec<BehaviorFactory>,
        entity_factories: Vec<BehaviorFactory>,
    ) -> Self {
        Self {
            tile_factories,
            block_factories,
            entity_factories,
            ecs: Default::default(),
            relation_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, behavior: Behavior) -> Result<BehaviorKey, FieldError> {
        let entity_key = self
            .relation_ref
            .entry(behavior.relation)
            .or_insert_with(|| self.ecs.insert_entity());

        let comp_key = self.ecs.insert_comp(*entity_key, behavior).check();
        Ok(comp_key)
    }

    pub fn remove(&mut self, key: BehaviorKey) -> Result<Behavior, FieldError> {
        self.ecs.remove_comp(key).ok_or(FieldError::NotFound)
    }

    pub fn get(&self, key: BehaviorKey) -> Result<BehaviorRef, FieldError> {
        let behavior = self.ecs.get_comp(key).ok_or(FieldError::NotFound)?;
        Ok(behavior.into())
    }

    pub fn get_mut(&mut self, key: BehaviorKey) -> Result<BehaviorMut, FieldError> {
        let behavior = self.ecs.get_comp_mut(key).ok_or(FieldError::NotFound)?;
        Ok(behavior.into())
    }

    pub fn iter(
        &self,
        kind: BehaviorKind,
    ) -> Result<impl Iterator<Item = BehaviorRef>, FieldError> {
        let iter = self.ecs.iter_comp(kind).ok_or(FieldError::NotFound)?;
        Ok(iter.map(|behavior| behavior.into()))
    }

    pub fn iter_mut(
        &mut self,
        kind: BehaviorKind,
    ) -> Result<impl Iterator<Item = BehaviorMut>, FieldError> {
        let iter = self.ecs.iter_comp_mut(kind).ok_or(FieldError::NotFound)?;
        Ok(iter.map(|behavior| behavior.into()))
    }

    pub fn iter_by_relation(
        &self,
        relation: BehaviorRelation,
        kind: BehaviorKind,
    ) -> Result<impl Iterator<Item = BehaviorRef>, FieldError> {
        let entity_key = self
            .relation_ref
            .get(&relation)
            .ok_or(FieldError::NotFound)?;
        let iter = self.ecs.iter_comp_by_entity(*entity_key, kind).check();
        Ok(iter.map(|behavior| behavior.into()))
    }

    pub fn iter_mut_by_relation(
        &mut self,
        relation: BehaviorRelation,
        kind: BehaviorKind,
    ) -> Result<impl Iterator<Item = BehaviorMut>, FieldError> {
        let entity_key = self
            .relation_ref
            .get_mut(&relation)
            .ok_or(FieldError::NotFound)?;
        let iter = self.ecs.iter_comp_mut_by_entity(*entity_key, kind).check();
        Ok(iter.map(|behavior| behavior.into()))
    }

    pub fn remove_by_relation(&mut self, relation: BehaviorRelation) -> Result<(), FieldError> {
        let entity_key = self
            .relation_ref
            .remove(&relation)
            .ok_or(FieldError::NotFound)?;
        self.ecs.remove_entity(entity_key).check();
        Ok(())
    }

    pub fn update(
        &mut self,
        tile_field: &mut TileField,
        block_field: &mut BlockField,
        entity_field: &mut EntityField,
        delta_secs: f32,
    ) {
        Generator::update(self, tile_field, block_field, entity_field, delta_secs);
        RandomWalk::update(self, tile_field, block_field, entity_field, delta_secs);
    }

    pub fn place_tile(
        &mut self,
        tile_field: &mut TileField,
        tile: Tile,
    ) -> Result<u32, FieldError> {
        let factory = self
            .tile_factories
            .get(tile.id as usize)
            .ok_or(FieldError::InvalidId)?;

        let key = tile_field.insert(tile)?;
        let relation = BehaviorRelation::Tile(key);

        let inner = factory.into();
        self.insert(Behavior { inner, relation }).check();

        Ok(key)
    }

    pub fn break_tile(&mut self, tile_field: &mut TileField, key: u32) -> Result<Tile, FieldError> {
        let tile = tile_field.remove(key)?;
        let relation = BehaviorRelation::Tile(key);
        self.remove_by_relation(relation).check();
        Ok(tile)
    }

    pub fn place_block(
        &mut self,
        block_field: &mut BlockField,
        block: Block,
    ) -> Result<u32, FieldError> {
        let factory = self
            .block_factories
            .get(block.id as usize)
            .ok_or(FieldError::InvalidId)?;

        let key = block_field.insert(block)?;
        let relation = BehaviorRelation::Block(key);

        let inner = factory.into();
        self.insert(Behavior { inner, relation }).check();

        Ok(key)
    }

    pub fn break_block(
        &mut self,
        block_field: &mut BlockField,
        key: u32,
    ) -> Result<Block, FieldError> {
        let block = block_field.remove(key)?;
        let relation = BehaviorRelation::Block(key);
        self.remove_by_relation(relation).check();
        Ok(block)
    }

    pub fn place_entity(
        &mut self,
        entity_field: &mut EntityField,
        entity: Entity,
    ) -> Result<u32, FieldError> {
        let factory = self
            .entity_factories
            .get(entity.id as usize)
            .ok_or(FieldError::InvalidId)?;

        let key = entity_field.insert(entity)?;
        let relation = BehaviorRelation::Entity(key);

        let inner = factory.into();
        self.insert(Behavior { inner, relation }).check();

        Ok(key)
    }

    pub fn break_entity(
        &mut self,
        entity_field: &mut EntityField,
        key: u32,
    ) -> Result<Entity, FieldError> {
        let entity = entity_field.remove(key)?;
        let relation = BehaviorRelation::Entity(key);
        self.remove_by_relation(relation).check();
        Ok(entity)
    }
}

// Behavior Plugin Extra

#[derive(Debug, Clone)]
pub struct Generator {}

impl Generator {
    fn update(
        behavior_plguin: &mut BehaviorPlugin,
        _tile_field: &mut TileField,
        _block_field: &mut BlockField,
        entity_field: &mut EntityField,
        _delta_secs: f32,
    ) {
        let Ok(mut iter) = behavior_plguin.iter_mut(BehaviorKind::Generator) else {
            return;
        };

        let Some(generator) = iter.next() else {
            return;
        };

        if iter.next().is_some() {
            panic!("generator behavior must be one element.");
        }

        let BehaviorRelation::Entity(relation) = *generator.relation else {
            return;
        };

        let entity = entity_field.get(relation).check();

        godot_print!("{:?}", entity.location);
    }
}

#[derive(Debug, Clone)]
pub struct GeneratorFactory {}

impl GeneratorFactory {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create(&self) -> Generator {
        Generator {}
    }
}

#[derive(Debug, Clone)]
enum RandomWalkState {
    Init,
    WaitStart,
    Wait(f32),
    TripStart,
    Trip(Vec2),
}

#[derive(Debug, Clone)]
pub struct RandomWalk {
    min_rest_secs: f32,
    max_rest_secs: f32,
    min_distance: f32,
    max_distance: f32,
    speed: f32,
    state: RandomWalkState,
}

impl RandomWalk {
    fn update(
        behavior_plguin: &mut BehaviorPlugin,
        _tile_field: &mut TileField,
        block_field: &mut BlockField,
        entity_field: &mut EntityField,
        delta_secs: f32,
    ) {
        use rand::Rng;

        let Ok(iter) = behavior_plguin.iter_mut(BehaviorKind::RandomWalk) else {
            return;
        };

        for behavior in iter {
            let BehaviorInner::RandomWalk(inner) = behavior.inner else {
                unreachable!("invalid inner");
            };

            let BehaviorRelation::Entity(relation) = *behavior.relation else {
                unreachable!("invalid relation");
            };

            match inner.state {
                RandomWalkState::Init => {
                    inner.state = RandomWalkState::WaitStart;
                }
                RandomWalkState::WaitStart => {
                    let secs =
                        rand::thread_rng().gen_range(inner.min_rest_secs..inner.max_rest_secs);
                    inner.state = RandomWalkState::Wait(secs);
                }
                RandomWalkState::Wait(secs) => {
                    let new_secs = secs - delta_secs;
                    if new_secs <= 0.0 {
                        inner.state = RandomWalkState::TripStart;
                    } else {
                        inner.state = RandomWalkState::Wait(new_secs);
                    }
                }
                RandomWalkState::TripStart => {
                    let entity = entity_field.get(relation).check();

                    let distance =
                        rand::thread_rng().gen_range(inner.min_distance..inner.max_distance);
                    let direction = rand::thread_rng().gen_range(0.0..std::f32::consts::PI * 2.0);
                    let destination = [
                        entity.location[0] + distance * direction.cos(),
                        entity.location[1] + distance * direction.sin(),
                    ];

                    inner.state = RandomWalkState::Trip(destination);
                }
                RandomWalkState::Trip(destination) => {
                    let entity = entity_field.get(relation).check();

                    if entity.location == destination {
                        inner.state = RandomWalkState::WaitStart;
                        return;
                    }

                    let diff = [
                        destination[0] - entity.location[0],
                        destination[1] - entity.location[1],
                    ];
                    let distance = (diff[0].powi(2) + diff[1].powi(2)).sqrt();
                    let direction = [diff[0] / distance, diff[1] / distance];
                    let delta_distance = distance.min(inner.speed * delta_secs);
                    let location = [
                        entity.location[0] + direction[0] * delta_distance,
                        entity.location[1] + direction[1] * delta_distance,
                    ];

                    if move_entity(block_field, entity_field, relation, location).is_ok() {
                        inner.state = RandomWalkState::Trip(destination);
                    } else {
                        inner.state = RandomWalkState::WaitStart;
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct RandomWalkFactory {
    min_rest_secs: f32,
    max_rest_secs: f32,
    min_distance: f32,
    max_distance: f32,
    speed: f32,
}

impl RandomWalkFactory {
    pub fn new(
        min_rest_secs: f32,
        max_rest_secs: f32,
        min_distance: f32,
        max_distance: f32,
        speed: f32,
    ) -> Self {
        Self {
            min_rest_secs,
            max_rest_secs,
            min_distance,
            max_distance,
            speed,
        }
    }

    fn create(&self) -> RandomWalk {
        RandomWalk {
            min_rest_secs: self.min_rest_secs,
            max_rest_secs: self.max_rest_secs,
            min_distance: self.min_distance,
            max_distance: self.max_distance,
            speed: self.speed,
            state: RandomWalkState::Init,
        }
    }
}
