use glam::*;
use std::rc::Rc;

use crate::geom::*;

pub use data::*;
pub use feature::*;
pub use item::*;
pub use resource::*;
pub use time::*;
pub use tile::*;
pub use block::*;
pub use entity::*;

mod data;
mod feature;
mod item;
mod resource;
mod time;
mod tile;
mod block;
mod entity;

pub struct DataflowInfo {
    pub tile_field: TileFieldInfo,
    pub block_field: BlockFieldInfo,
    pub entity_field: EntityFieldInfo,
    pub item_storage: ItemStorageInfo,

    pub tile_feature_builder: FeatureMatrixBuilder,
    pub block_feature_builder: FeatureMatrixBuilder,
    pub entity_feature_builder: FeatureMatrixBuilder,
    pub item_feature_builder: FeatureMatrixBuilder,
}

pub struct Dataflow {
    time_storage: TimeStorage,

    // structured data storage
    tile_field: TileField,
    block_field: BlockField,
    entity_field: EntityField,
    item_storage: ItemStorage,

    // readonly functional data storage
    tile_features: FeatureMatrix,
    block_features: FeatureMatrix,
    entity_features: FeatureMatrix,
    item_features: FeatureMatrix,

    // external data storage
    resource_storage: ResourceStorage,
}

impl Dataflow {
    pub fn new(info: DataflowInfo) -> Self {
        Self {
            time_storage: TimeStorage::new(),

            tile_field: TileField::new(info.tile_field),
            block_field: BlockField::new(info.block_field),
            entity_field: EntityField::new(info.entity_field),
            item_storage: ItemStorage::new(info.item_storage),

            tile_features: info.tile_feature_builder.build(),
            block_features: info.block_feature_builder.build(),
            entity_features: info.entity_feature_builder.build(),
            item_features: info.item_feature_builder.build(),

            resource_storage: ResourceStorage::new(),
        }
    }

    // time

    pub fn tick_per_secs(&self) -> u64 {
        self.time_storage.get_tick_per_secs()
    }

    pub fn get_tick(&self) -> u64 {
        self.time_storage.get_tick()
    }

    pub fn forward_time(&mut self, delta_secs: f32) {
        self.time_storage.forward(delta_secs);
    }

    // tile

    pub fn get_tile_feature<T: 'static>(&self, id: u16) -> Result<&T, DataflowError> {
        let feature = self.tile_features.get::<T>(id)?;
        Ok(feature)
    }

    pub fn insert_tile(&mut self, tile: Tile) -> Result<TileId, DataflowError> {
        let feature = self
            .get_tile_feature::<Rc<dyn FieldFeature<Key = TileId>>>(tile.archetype_id)
            .cloned();
        let tile_id = self.tile_field.insert(tile)?;
        let _ = feature.map(|f| f.after_place(self, tile_id));
        Ok(tile_id)
    }

    pub fn remove_til(&mut self, tile_id: TileId) -> Result<tile::Tile, DataflowError> {
        let tile = self.tile_field.get(tile_id)?;
        let feature = self
            .get_tile_feature::<Rc<dyn FieldFeature<Key = TileId>>>(tile.archetype_id)
            .cloned();
        let _ = feature.map(|f| f.before_break(self, tile_id));
        let tile = self.tile_field.remove(tile_id)?;
        Ok(tile)
    }

    pub fn modify_tile(&mut self, tile_id: TileId, f: impl FnOnce(&mut TileRenderState)) -> Result<TileId, DataflowError> {
        let tile_id = self.tile_field.modify(tile_id, f)?;
        Ok(tile_id)
    }

    pub fn get_tile(&self, tile_key: TileId) -> Result<&Tile, DataflowError> {
        let tile_id = self.tile_field.get(tile_key)?;
        Ok(tile_id)
    }

    pub fn find_tile_chunk_coord(&self, point: Vec2) -> IVec2 {
        self.tile_field.find_chunk_coord(point)
    }

    pub fn get_tile_chunk(&self, chunk_coord: IVec2) -> Result<&TileChunk, DataflowError> {
        let chunk = self.tile_field.get_chunk(chunk_coord)?;
        Ok(chunk)
    }

    pub fn get_tile_archetype(&self, archetype_id: u16) -> Result<&TileArchetype, DataflowError> {
        let archetype = self.tile_field.get_archetype(archetype_id)?;
        Ok(archetype)
    }

    // tile spatial features

    pub fn find_tile_with_point(&self, point: IVec2) -> Option<TileId> {
        self.tile_field.find_with_point(point)
    }

    pub fn find_tile_with_rect(&self, rect: IRect2) -> impl Iterator<Item = TileId> + '_ {
        self.tile_field.find_with_rect(rect)
    }

    // tile collision features

    pub fn find_tile_with_collision_point(&self, coord: Vec2) -> impl Iterator<Item = TileId> + '_ {
        self.tile_field.find_with_collision_point(coord)
    }

    pub fn find_tile_with_collision_rect(&self, rect: Rect2) -> impl Iterator<Item = TileId> + '_ {
        self.tile_field.find_with_collision_rect(rect)
    }

    // tile inventory

    pub fn get_tile_inventory(&self, tile_id: TileId) -> Result<Option<InventoryId>, DataflowError> {
        let tile = self.tile_field.get(tile_id)?;
        let feature = self
            .get_tile_feature::<Rc<dyn InventoryFeature<Key = TileId>>>(tile.archetype_id)
            .cloned();
        let inventory = feature.map(|f| f.get_inventory(self, tile_id)).ok();
        Ok(inventory)
    }

    // block

    pub fn get_block_feature<T: 'static>(&self, id: u16) -> Result<&T, DataflowError> {
        let feature = self.block_features.get::<T>(id)?;
        Ok(feature)
    }

    pub fn insert_block(&mut self, block: Block) -> Result<BlockId, DataflowError> {
        let feature = self
            .get_block_feature::<Rc<dyn FieldFeature<Key = BlockId>>>(block.archetype_id)
            .cloned();
        let block_id = self.block_field.insert(block)?;
        let _ = feature.map(|f| f.after_place(self, block_id));
        Ok(block_id)
    }

    pub fn remove_block(&mut self, block_id: BlockId) -> Result<Block, DataflowError> {
        let block = self.block_field.get(block_id)?;
        let feature = self
            .get_block_feature::<Rc<dyn FieldFeature<Key = BlockId>>>(block.archetype_id)
            .cloned();
        let _ = feature.map(|f| f.before_break(self, block_id));
        let block = self.block_field.remove(block_id)?;
        Ok(block)
    }

    pub fn modify_block(&mut self, block_id: BlockId, f: impl FnOnce(&mut BlockRenderState)) -> Result<BlockId, DataflowError> {
        let block_id = self.block_field.modify(block_id, f)?;
        Ok(block_id)
    }

    pub fn get_block(&self, block_id: BlockId) -> Result<&Block, DataflowError> {
        let block = self.block_field.get(block_id)?;
        Ok(block)
    }

    pub fn find_block_chunk_coord(&self, point: Vec2) -> IVec2 {
        self.block_field.find_chunk_coord(point)
    }

    pub fn get_block_chunk(&self, chunk_coord: IVec2) -> Result<&BlockChunk, DataflowError> {
        let chunk = self.block_field.get_chunk(chunk_coord)?;
        Ok(chunk)
    }

    pub fn get_block_archetype(&self, archetype_id: u16) -> Result<&BlockArchetype, DataflowError> {
        let archetype = self.block_field.get_archetype(archetype_id)?;
        Ok(archetype)
    }

    // block spatial features

    pub fn find_block_with_point(&self, point: IVec2) -> Option<BlockId> {
        self.block_field.find_with_point(point)
    }

    pub fn find_block_with_rect(&self, rect: IRect2) -> impl Iterator<Item = BlockId> + '_ {
        self.block_field.find_with_rect(rect)
    }

    // block collision features

    pub fn find_block_collision_point(&self, point: Vec2) -> impl Iterator<Item = BlockId> + '_ {
        self.block_field.find_with_collision_point(point)
    }

    pub fn find_block_with_collision_rect(&self, rect: Rect2) -> impl Iterator<Item = BlockId> + '_ {
        self.block_field.find_with_collision_rect(rect)
    }

    // block hint features

    pub fn find_block_with_hint_point(&self, point: Vec2) -> impl Iterator<Item = BlockId> + '_ {
        self.block_field.find_with_hint_point(point)
    }

    pub fn find_block_with_hint_rect(&self, rect: Rect2) -> impl Iterator<Item = BlockId> + '_ {
        self.block_field.find_with_hint_rect(rect)
    }

    // block inventory

    pub fn get_block_inventory(&self, block_id: BlockId) -> Result<Option<InventoryId>, DataflowError> {
        let block = self.block_field.get(block_id)?;
        let feature = self
            .get_block_feature::<Rc<dyn InventoryFeature<Key = BlockId>>>(block.archetype_id)
            .cloned();
        let inventory = feature.map(|f| f.get_inventory(self, block_id)).ok();
        Ok(inventory)
    }

    // entity

    pub fn get_entity_feature<T: 'static>(&self, id: u16) -> Result<&T, DataflowError> {
        let feature = self.entity_features.get::<T>(id)?;
        Ok(feature)
    }

    pub fn insert_entity(&mut self, entity: entity::Entity) -> Result<EntityId, DataflowError> {
        let feature = self
            .get_entity_feature::<Rc<dyn FieldFeature<Key = EntityId>>>(entity.archetype_id)
            .cloned();
        let entity_id = self.entity_field.insert(entity)?;
        let _ = feature.map(|f| f.after_place(self, entity_id));
        Ok(entity_id)
    }

    pub fn remove_entity(&mut self, entity_id: EntityId) -> Result<Entity, DataflowError> {
        let entity = self.entity_field.get(entity_id)?;
        let feature = self
            .get_entity_feature::<Rc<dyn FieldFeature<Key = EntityId>>>(entity.archetype_id)
            .cloned();
        let _ = feature.map(|f| f.before_break(self, entity_id));
        let entity = self.entity_field.remove(entity_id)?;
        Ok(entity)
    }

    pub fn modify_entity(&mut self, entity_id: EntityId, f: impl FnOnce(&mut EntityRenderState)) -> Result<EntityId, DataflowError> {
        let entity_id = self.entity_field.modify(entity_id, f)?;
        Ok(entity_id)
    }

    pub fn get_entity(&self, entity_id: EntityId) -> Result<&Entity, DataflowError> {
        let entity = self.entity_field.get(entity_id)?;
        Ok(entity)
    }

    pub fn find_entity_chunk_coord(&self, point: Vec2) -> IVec2 {
        self.entity_field.find_chunk_coord(point)
    }

    pub fn get_entity_chunk(&self, chunk_coord: IVec2) -> Result<&EntityChunk, DataflowError> {
        let chunk = self.entity_field.get_chunk(chunk_coord)?;
        Ok(chunk)
    }

    pub fn get_entity_archetype(&self, archetype_id: u16) -> Result<&EntityArchetype, DataflowError> {
        let archetype = self.entity_field.get_archetype(archetype_id)?;
        Ok(archetype)
    }

    // entity collision features

    pub fn find_entity_with_collision_point(&self, point: Vec2) -> impl Iterator<Item = EntityId> + '_ {
        self.entity_field.find_with_collision_point(point)
    }

    pub fn find_entity_with_collision_rect(&self, rect: Rect2) -> impl Iterator<Item = EntityId> + '_ {
        self.entity_field.find_with_collision_rect(rect)
    }

    // entity hint features

    pub fn find_entity_with_hint_point(&self, point: Vec2) -> impl Iterator<Item = EntityId> + '_ {
        self.entity_field.find_with_hint_point(point)
    }

    pub fn find_entity_with_hint_rect(&self, rect: Rect2) -> impl Iterator<Item = EntityId> + '_ {
        self.entity_field.find_with_hint_rect(rect)
    }

    // entity inventory

    pub fn get_inventory_by_entity(&self, entity_id: EntityId) -> Result<Option<InventoryId>, DataflowError> {
        let entity = self.entity_field.get(entity_id)?;
        let feature = self
            .get_entity_feature::<Rc<dyn InventoryFeature<Key = EntityId>>>(entity.archetype_id)
            .cloned();
        let inventory = feature.map(|f| f.get_inventory(self, entity_id)).ok();
        Ok(inventory)
    }

    // item

    pub fn get_item_feature<T: 'static>(&self, id: u16) -> Result<&T, DataflowError> {
        let feature = self.item_features.get::<T>(id)?;
        Ok(feature)
    }

    pub fn insert_inventory(&mut self, archetype_id: u16) -> Result<InventoryId, DataflowError> {
        let inventory_key = self.item_storage.insert_inventory(archetype_id)?;
        Ok(inventory_key)
    }

    pub fn remove_inventory(&mut self, inventory_id: InventoryId) -> Result<u16, DataflowError> {
        let id = self.item_storage.remove_inventory(inventory_id)?;
        Ok(id)
    }

    pub fn get_inventory(&self, inventory_id: InventoryId) -> Result<&Inventory, DataflowError> {
        let inventory = self.item_storage.get_inventory(inventory_id)?;
        Ok(inventory)
    }

    pub fn push_item_to_inventory(&mut self, inventory_id: InventoryId, item: Item) -> Result<(), DataflowError> {
        self.item_storage.push_item_to_inventory(inventory_id, item)?;
        Ok(())
    }

    pub fn pop_item_from_inventory(&mut self, inventory_id: InventoryId) -> Result<Item, DataflowError> {
        let item = self.item_storage.pop_item_from_inventory(inventory_id)?;
        Ok(item)
    }

    pub fn insert_item(&mut self, slot_id: SlotId, item: Item) -> Result<(), DataflowError> {
        self.item_storage.insert_item(slot_id, item)?;
        Ok(())
    }

    pub fn remove_item(&mut self, slot_id: SlotId) -> Result<Item, DataflowError> {
        let item = self.item_storage.remove_item(slot_id)?;
        Ok(item)
    }

    pub fn modify_item(&mut self, slot_id: SlotId, f: impl FnOnce(&mut Item)) -> Result<(), DataflowError> {
        self.item_storage.modify_item(slot_id, f)?;
        Ok(())
    }

    pub fn swap_item(&mut self, src_slot_id: SlotId, dst_slot_id: SlotId) -> Result<(), DataflowError> {
        self.item_storage.swap_item(src_slot_id, dst_slot_id)?;
        Ok(())
    }

    pub fn get_item(&self, slot_id: SlotId) -> Result<&Item, DataflowError> {
        let item = self.item_storage.get_item(slot_id)?;
        Ok(item)
    }

    pub fn get_item_archetype(&self, archetype_id: u16) -> Result<&ItemArchetype, DataflowError> {
        let archetype = self.item_storage.get_item_archetype(archetype_id)?;
        Ok(archetype)
    }

    // resources

    pub fn insert_resources<T>(&mut self, resource: T) -> Result<(), DataflowError> where T: Resource + 'static,
    {
        self.resource_storage.insert::<T>(resource)?;
        Ok(())
    }

    pub fn remove_resources<T>(&mut self) -> Result<T, DataflowError> where T: Resource + 'static,
    {
        let resource = self.resource_storage.remove::<T>()?;
        Ok(resource)
    }

    pub fn find_resources<T>(&self) -> Result<ResourceCell<T>, DataflowError> where T: Resource + 'static,
    {
        let resource = self.resource_storage.find::<T>()?;
        Ok(resource)
    }
}

// feature

pub trait FieldFeature {
    type Key;
    fn after_place(&self, dataflow: &mut Dataflow, key: Self::Key);
    fn before_break(&self, dataflow: &mut Dataflow, key: Self::Key);
}

pub trait InventoryFeature {
    type Key;
    fn get_inventory(&self, dataflow: &Dataflow, key: Self::Key) -> InventoryId;
}

// error handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataflowError {
    TileError(TileError),
    BlockError(BlockError),
    EntityError(EntityError),
    ItemError(ItemError),
    ResourceError(ResourceError),
    FeatureError(FeatureError),
}

impl std::fmt::Display for DataflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TileError(e) => e.fmt(f),
            Self::BlockError(e) => e.fmt(f),
            Self::EntityError(e) => e.fmt(f),
            Self::ItemError(e) => e.fmt(f),
            Self::ResourceError(e) => e.fmt(f),
            Self::FeatureError(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for DataflowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::TileError(e) => Some(e),
            Self::BlockError(e) => Some(e),
            Self::EntityError(e) => Some(e),
            Self::ItemError(e) => Some(e),
            Self::ResourceError(e) => Some(e),
            Self::FeatureError(e) => Some(e),
        }
    }
}

impl From<TileError> for DataflowError {
    fn from(e: TileError) -> Self {
        Self::TileError(e)
    }
}

impl From<BlockError> for DataflowError {
    fn from(e: BlockError) -> Self {
        Self::BlockError(e)
    }
}

impl From<EntityError> for DataflowError {
    fn from(e: EntityError) -> Self {
        Self::EntityError(e)
    }
}

impl From<ItemError> for DataflowError {
    fn from(e: ItemError) -> Self {
        Self::ItemError(e)
    }
}

impl From<ResourceError> for DataflowError {
    fn from(e: ResourceError) -> Self {
        Self::ResourceError(e)
    }
}

impl From<FeatureError> for DataflowError {
    fn from(e: FeatureError) -> Self {
        Self::FeatureError(e)
    }
}
