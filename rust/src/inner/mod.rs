pub use animal::*;
pub use feature::*;
pub use field::*;
pub use forwarder::*;
pub use generator::*;
pub use inventory::*;
pub use player::*;
pub use resource::*;
pub use time::*;

mod animal;
mod feature;
mod field;
mod forwarder;
mod generator;
mod inventory;
mod player;
mod resource;
mod time;

pub type Vec2 = [f32; 2];
pub type IVec2 = [i32; 2];

type RcVec<T> = std::rc::Rc<[T]>;

#[derive(Debug, Clone)]
pub struct RootDescriptor {
    pub tile_field: TileFieldDescriptor,
    pub block_field: BlockFieldDescriptor,
    pub entity_field: EntityFieldDescriptor,
    pub tile_features: RcVec<TileFeature>,
    pub block_features: RcVec<BlockFeature>,
    pub entity_features: RcVec<EntityFeature>,
}

#[derive(Debug)]
pub struct Root {
    tile_field: TileField,
    block_field: BlockField,
    entity_field: EntityField,
    tile_features: RcVec<TileFeature>,
    block_features: RcVec<BlockFeature>,
    entity_features: RcVec<EntityFeature>,
    resource_store: ResourceStore,
    time_store: TimeStore,
}

impl Root {
    #[inline]
    pub fn new(desc: RootDescriptor) -> Self {
        Self {
            tile_field: TileField::new(desc.tile_field),
            block_field: BlockField::new(desc.block_field),
            entity_field: EntityField::new(desc.entity_field),
            tile_features: desc.tile_features,
            block_features: desc.block_features,
            entity_features: desc.entity_features,
            resource_store: Default::default(),
            time_store: Default::default(),
        }
    }

    // tile

    pub fn tile_insert(&mut self, tile: field::Tile) -> Result<TileKey, FieldError> {
        let features = self.tile_features.clone();
        let feature = features
            .get(tile.id as usize)
            .ok_or(FieldError::InvalidId)?;
        let tile_key = self.tile_field.insert(tile)?;
        feature.after_place(self, tile_key);
        Ok(tile_key)
    }

    pub fn tile_remove(&mut self, tile_key: TileKey) -> Result<field::Tile, FieldError> {
        let features = self.tile_features.clone();
        let tile = self.tile_field.get(tile_key)?;
        let feature = features
            .get(tile.id as usize)
            .ok_or(FieldError::InvalidId)?;
        feature.before_break(self, tile_key);
        self.tile_field.remove(tile_key)
    }

    #[inline]
    pub fn tile_modify(
        &mut self,
        tile_key: TileKey,
        f: impl FnOnce(&mut field::Tile),
    ) -> Result<field::TileKey, FieldError> {
        self.tile_field.modify(tile_key, f)
    }

    #[inline]
    pub fn tile_get(&self, tile_key: TileKey) -> Result<&field::Tile, FieldError> {
        self.tile_field.get(tile_key)
    }

    #[inline]
    pub fn tile_get_chunk_size(&self) -> u32 {
        self.tile_field.get_chunk_size()
    }

    pub fn tile_get_chunk(&self, chunk_location: IVec2) -> Result<&field::TileChunk, FieldError> {
        let chunk_key = self
            .tile_field
            .get_by_chunk_location(chunk_location)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.tile_field.get_chunk(chunk_key).unwrap();
        Ok(chunk)
    }

    pub fn tile_forward_chunk(
        &mut self,
        chunk_location: IVec2,
        delta_secs: f32,
    ) -> Result<(), FieldError> {
        let chunk_key = self
            .tile_field
            .get_by_chunk_location(chunk_location)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.tile_field.get_chunk(chunk_key)?;

        let mut local_keys = vec![];
        for (local_key, _) in &chunk.tiles {
            local_keys.push(local_key as u32);
        }

        let features = self.tile_features.clone();
        for local_key in local_keys {
            let tile_key = (chunk_key, local_key);
            let tile = self.tile_field.get(tile_key).unwrap();
            let feature = features
                .get(tile.id as usize)
                .ok_or(FieldError::InvalidId)?;
            feature.forward(self, tile_key, delta_secs);
        }
        Ok(())
    }

    // tile spatial features

    #[inline]
    pub fn tile_has_by_point(&self, point: IVec2) -> bool {
        self.tile_field.has_by_point(point)
    }

    #[inline]
    pub fn tile_get_by_point(&self, point: IVec2) -> Option<TileKey> {
        self.tile_field.get_by_point(point)
    }

    // tile collision features

    #[inline]
    pub fn tile_get_collision_rect(&self, tile_key: TileKey) -> Result<[Vec2; 2], FieldError> {
        self.tile_field.get_collision_rect(tile_key)
    }

    #[inline]
    pub fn tile_has_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        self.tile_field.has_by_collision_rect(rect)
    }

    #[inline]
    pub fn tile_get_by_collision_rect(
        &self,
        rect: [Vec2; 2],
    ) -> impl Iterator<Item = TileKey> + '_ {
        self.tile_field.get_by_collision_rect(rect)
    }

    #[inline]
    pub fn tile_has_by_collision_point(&self, point: Vec2) -> bool {
        self.tile_field.has_by_collision_point(point)
    }

    #[inline]
    pub fn tile_get_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = TileKey> + '_ {
        self.tile_field.get_by_collision_point(point)
    }

    // block

    pub fn block_insert(&mut self, block: field::Block) -> Result<BlockKey, FieldError> {
        let features = self.block_features.clone();
        let feature = features
            .get(block.id as usize)
            .ok_or(FieldError::InvalidId)?;
        let block_key = self.block_field.insert(block)?;
        feature.after_place(self, block_key);
        Ok(block_key)
    }

    pub fn block_remove(&mut self, block_key: BlockKey) -> Result<field::Block, FieldError> {
        let features = self.block_features.clone();
        let block = self.block_field.get(block_key)?;
        let feature = features
            .get(block.id as usize)
            .ok_or(FieldError::InvalidId)?;
        feature.before_break(self, block_key);
        self.block_field.remove(block_key)
    }

    #[inline]
    pub fn block_modify(
        &mut self,
        block_key: BlockKey,
        f: impl FnOnce(&mut field::Block),
    ) -> Result<field::BlockKey, FieldError> {
        self.block_field.modify(block_key, f)
    }

    #[inline]
    pub fn block_get(&self, block_key: BlockKey) -> Result<&field::Block, FieldError> {
        self.block_field.get(block_key)
    }

    #[inline]
    pub fn block_get_chunk_size(&self) -> u32 {
        self.block_field.get_chunk_size()
    }

    pub fn block_get_chunk(&self, chunk_location: IVec2) -> Result<&field::BlockChunk, FieldError> {
        let chunk_key = self
            .tile_field
            .get_by_chunk_location(chunk_location)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.block_field.get_chunk(chunk_key).unwrap();
        Ok(chunk)
    }

    pub fn block_forward_chunk(
        &mut self,
        chunk_location: IVec2,
        delta_secs: f32,
    ) -> Result<(), FieldError> {
        let chunk_key = self
            .block_field
            .get_by_chunk_location(chunk_location)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.block_field.get_chunk(chunk_key)?;

        let mut local_keys = vec![];
        for (local_key, _) in &chunk.blocks {
            local_keys.push(local_key as u32);
        }

        let features = self.block_features.clone();
        for local_key in local_keys {
            let block_key = (chunk_key, local_key);
            let block = self.block_field.get(block_key).unwrap();
            let feature = features
                .get(block.id as usize)
                .ok_or(FieldError::InvalidId)?;
            feature.forward(self, block_key, delta_secs);
        }
        Ok(())
    }

    // block spatial features

    #[inline]
    pub fn block_get_base_rect(&self, id: u16) -> Result<[[i32; 2]; 2], FieldError> {
        self.block_field.get_base_rect(id)
    }

    #[inline]
    pub fn block_get_rect(&self, block_key: BlockKey) -> Result<[IVec2; 2], FieldError> {
        self.block_field.get_rect(block_key)
    }

    #[inline]
    pub fn block_has_by_point(&self, point: IVec2) -> bool {
        self.block_field.has_by_point(point)
    }

    #[inline]
    pub fn block_get_by_point(&self, point: IVec2) -> Option<BlockKey> {
        self.block_field.get_by_point(point)
    }

    #[inline]
    pub fn block_has_by_rect(&self, rect: [IVec2; 2]) -> bool {
        self.block_field.has_by_rect(rect)
    }

    #[inline]
    pub fn block_get_by_rect(&self, rect: [IVec2; 2]) -> impl Iterator<Item = BlockKey> + '_ {
        self.block_field.get_by_rect(rect)
    }

    // block collision features

    #[inline]
    pub fn block_get_base_collision_rect(&self, id: u16) -> Result<[Vec2; 2], FieldError> {
        self.block_field.get_base_collision_rect(id)
    }

    #[inline]
    pub fn block_get_collision_rect(&self, block_key: BlockKey) -> Result<[Vec2; 2], FieldError> {
        self.block_field.get_collision_rect(block_key)
    }

    #[inline]
    pub fn block_has_by_collision_point(&self, point: Vec2) -> bool {
        self.block_field.has_by_collision_point(point)
    }

    #[inline]
    pub fn block_get_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = BlockKey> + '_ {
        self.block_field.get_by_collision_point(point)
    }

    #[inline]
    pub fn block_has_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        self.block_field.has_by_collision_rect(rect)
    }

    #[inline]
    pub fn block_get_by_collision_rect(
        &self,
        rect: [Vec2; 2],
    ) -> impl Iterator<Item = BlockKey> + '_ {
        self.block_field.get_by_collision_rect(rect)
    }

    // block hint features

    #[inline]
    pub fn block_get_base_hint_rect(&self, id: u16) -> Result<[Vec2; 2], FieldError> {
        self.block_field.get_base_hint_rect(id)
    }

    #[inline]
    pub fn block_get_hint_rect(&self, block_key: BlockKey) -> Result<[Vec2; 2], FieldError> {
        self.block_field.get_hint_rect(block_key)
    }

    #[inline]
    pub fn block_has_by_hint_point(&self, point: Vec2) -> bool {
        self.block_field.has_by_hint_point(point)
    }

    #[inline]
    pub fn block_get_by_hint_point(&self, point: Vec2) -> impl Iterator<Item = BlockKey> + '_ {
        self.block_field.get_by_hint_point(point)
    }

    #[inline]
    pub fn block_has_by_hint_rect(&self, rect: [Vec2; 2]) -> bool {
        self.block_field.has_by_hint_rect(rect)
    }

    #[inline]
    pub fn block_get_by_hint_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = BlockKey> + '_ {
        self.block_field.get_by_hint_rect(rect)
    }

    // entity

    pub fn entity_insert(&mut self, entity: field::Entity) -> Result<EntityKey, FieldError> {
        let features = self.entity_features.clone();
        let feature = features
            .get(entity.id as usize)
            .ok_or(FieldError::InvalidId)?;
        let entity_key = self.entity_field.insert(entity)?;
        feature.after_place(self, entity_key);
        Ok(entity_key)
    }

    pub fn entity_remove(&mut self, entity_key: EntityKey) -> Result<field::Entity, FieldError> {
        let features = self.entity_features.clone();
        let entity = self.entity_field.get(entity_key)?;
        let feature = features
            .get(entity.id as usize)
            .ok_or(FieldError::InvalidId)?;
        feature.before_break(self, entity_key);
        self.entity_field.remove(entity_key)
    }

    #[inline]
    pub fn entity_modify(
        &mut self,
        entity_key: EntityKey,
        f: impl FnOnce(&mut field::Entity),
    ) -> Result<field::EntityKey, FieldError> {
        self.entity_field.modify(entity_key, f)
    }

    #[inline]
    pub fn entity_get(&self, entity_key: EntityKey) -> Result<&field::Entity, FieldError> {
        self.entity_field.get(entity_key)
    }

    #[inline]
    pub fn entity_get_chunk_size(&self) -> u32 {
        self.entity_field.get_chunk_size()
    }

    pub fn entity_get_chunk(&self, chunk_key: IVec2) -> Result<&field::EntityChunk, FieldError> {
        let chunk_key = self
            .entity_field
            .get_by_chunk_location(chunk_key)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.entity_field.get_chunk(chunk_key).unwrap();
        Ok(chunk)
    }

    pub fn entity_forward_chunk(
        &mut self,
        chunk_location: IVec2,
        delta_secs: f32,
    ) -> Result<(), FieldError> {
        let chunk_key = self
            .entity_field
            .get_by_chunk_location(chunk_location)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.entity_field.get_chunk(chunk_key)?;

        let mut local_keys = vec![];
        for (local_key, _) in &chunk.entities {
            local_keys.push(local_key as u32);
        }

        let features = self.entity_features.clone();
        for local_key in local_keys {
            let entity_key = (chunk_key, local_key);
            let entity = self.entity_field.get(entity_key).unwrap();
            let feature = features
                .get(entity.id as usize)
                .ok_or(FieldError::InvalidId)?;
            feature.forward(self, entity_key, delta_secs);
        }
        Ok(())
    }

    // entity collision features

    #[inline]
    pub fn entity_get_base_collision_rect(&self, id: u16) -> Result<[Vec2; 2], FieldError> {
        self.entity_field.get_base_collision_rect(id)
    }

    #[inline]
    pub fn entity_get_collision_rect(
        &self,
        entity_key: EntityKey,
    ) -> Result<[Vec2; 2], FieldError> {
        self.entity_field.get_collision_rect(entity_key)
    }

    #[inline]
    pub fn entity_has_by_collision_point(&self, point: Vec2) -> bool {
        self.entity_field.has_by_collision_point(point)
    }

    #[inline]
    pub fn entity_get_by_collision_point(
        &self,
        point: Vec2,
    ) -> impl Iterator<Item = EntityKey> + '_ {
        self.entity_field.get_by_collision_point(point)
    }

    #[inline]
    pub fn entity_has_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        self.entity_field.has_by_collision_rect(rect)
    }

    #[inline]
    pub fn entity_get_by_collision_rect(
        &self,
        rect: [Vec2; 2],
    ) -> impl Iterator<Item = EntityKey> + '_ {
        self.entity_field.get_by_collision_rect(rect)
    }

    // entity hint features

    #[inline]
    pub fn entity_get_base_hint_rect(&self, id: u16) -> Result<[Vec2; 2], FieldError> {
        self.entity_field.get_base_hint_rect(id)
    }

    #[inline]
    pub fn entity_get_hint_rect(&self, entity_key: EntityKey) -> Result<[Vec2; 2], FieldError> {
        self.entity_field.get_hint_rect(entity_key)
    }

    #[inline]
    pub fn entity_has_by_hint_point(&self, point: Vec2) -> bool {
        self.entity_field.has_by_hint_point(point)
    }

    #[inline]
    pub fn entity_get_by_hint_point(&self, point: Vec2) -> impl Iterator<Item = EntityKey> + '_ {
        self.entity_field.get_by_hint_point(point)
    }

    #[inline]
    pub fn entity_has_by_hint_rect(&self, rect: [Vec2; 2]) -> bool {
        self.entity_field.has_by_hint_rect(rect)
    }

    #[inline]
    pub fn entity_get_by_hint_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = EntityKey> + '_ {
        self.entity_field.get_by_hint_rect(rect)
    }

    // time

    #[inline]
    pub fn time_tick_per_secs(&self) -> u64 {
        self.time_store.tick_per_secs()
    }

    #[inline]
    pub fn time_tick(&self) -> u64 {
        self.time_store.tick()
    }

    #[inline]
    pub fn time_forward(&mut self, delta_secs: f32) {
        self.time_store.forward(delta_secs);
    }

    // resource

    #[inline]
    pub fn resource_insert<R: 'static>(&mut self, value: R) -> Result<(), ResourceError> {
        self.resource_store.insert(value)
    }

    #[inline]
    pub fn resource_remove<R: 'static>(&mut self) -> Result<R, ResourceError> {
        self.resource_store.remove::<R>()
    }

    #[inline]
    pub fn resource_has<R: 'static>(&self) -> bool {
        self.resource_store.has::<R>()
    }

    #[inline]
    pub fn resource_get<R: 'static>(&self) -> Result<&R, ResourceError> {
        self.resource_store.get::<R>()
    }

    #[inline]
    pub fn resource_get_mut<R: 'static>(&mut self) -> Result<&mut R, ResourceError> {
        self.resource_store.get_mut::<R>()
    }

    // execute

    #[inline]
    pub fn forwarder_init(&mut self) -> Result<(), ForwarderError> {
        ForwarderResource::init(self)
    }

    #[inline]
    pub fn forwarder_exec_rect(
        &mut self,
        min_rect: [Vec2; 2],
        delta_secs: f32,
    ) -> Result<(), ForwarderError> {
        ForwarderResource::exec_rect(self, min_rect, delta_secs)
    }

    #[inline]
    pub fn generator_init(
        &mut self,
        desc: GeneratorResourceDescriptor,
    ) -> Result<(), GeneratorError> {
        GeneratorResource::init(self, desc)
    }

    #[inline]
    pub fn generator_exec_rect(&mut self, min_rect: [Vec2; 2]) -> Result<(), GeneratorError> {
        GeneratorResource::exec_rect(self, min_rect)
    }

    #[inline]
    pub fn player_init(&mut self) -> Result<(), PlayerError> {
        PlayerResource::init(self)
    }

    #[inline]
    pub fn player_input(&mut self, input: Vec2) -> Result<(), PlayerError> {
        PlayerResource::input(self, input)
    }

    #[inline]
    pub fn player_location(&mut self) -> Result<Vec2, PlayerError> {
        PlayerResource::location(self)
    }

    #[inline]
    pub fn inventory_init(&mut self) -> Result<(), InventoryError> {
        InventoryResource::init(self)
    }

    #[inline]
    pub fn inventory_insert(
        &mut self,
        inventory: Inventory,
    ) -> Result<InventoryKey, InventoryError> {
        InventoryResource::insert(self, inventory)
    }

    #[inline]
    pub fn inventory_remove(
        &mut self,
        inventory_key: InventoryKey,
    ) -> Result<Inventory, InventoryError> {
        InventoryResource::remove(self, inventory_key)
    }
}
