use glam::*;
use std::rc::Rc;

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

pub struct DataflowDescriptor {
    pub tile_field_desc: TileFieldInfo,
    pub block_field_desc: BlockFieldInfo,
    pub entity_field_desc: EntityFieldInfo,
    pub item_storage_desc: ItemStorageDescriptor,

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
    pub fn new(desc: DataflowDescriptor) -> Self {
        Self {
            time_storage: TimeStorage::new(),

            tile_field: TileField::new(desc.tile_field_desc),
            block_field: BlockField::new(desc.block_field_desc),
            entity_field: EntityField::new(desc.entity_field_desc),
            item_storage: ItemStorage::new(desc.item_storage_desc),

            tile_features: desc.tile_feature_builder.build(),
            block_features: desc.block_feature_builder.build(),
            entity_features: desc.entity_feature_builder.build(),
            item_features: desc.item_feature_builder.build(),

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

    pub fn insert_tile(&mut self, tile: tile::Tile) -> Result<TileId, DataflowError> {
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

    pub fn modify_tile(&mut self, tile_id: TileId, f: impl FnOnce(&mut tile::Tile)) -> Result<tile::TileId, DataflowError> {
        let tile_id = self.tile_field.modify(tile_id, f)?;
        Ok(tile_id)
    }

    pub fn get_tile(&self, tile_key: TileId) -> Result<&tile::Tile, DataflowError> {
        let tile_id = self.tile_field.get(tile_key)?;
        Ok(tile_id)
    }

    pub fn get_tile_chunk_coord(&self, point: Vec2) -> IVec2 {
        self.tile_field.get_chunk_coord(point)
    }

    pub fn get_tile_version_by_chunk_coord(&self, chunk_coord: IVec2) -> Result<u64, DataflowError> {
        let chunk = self
            .tile_field
            .get_version_by_chunk_coord(chunk_coord)?;
        Ok(chunk)
    }

    pub fn get_tile_ids_by_chunk_coord(&self, chunk_coord: IVec2) -> Result<impl Iterator<Item = BlockId>, DataflowError> {
        let chunk = self.tile_field.get_ids_by_chunk_coord(chunk_coord)?;
        Ok(chunk)
    }

    pub fn get_tile_display_name(&self, tile_id: TileId) -> Result<&str, DataflowError> {
        let display_name = self.tile_field.get_display_name(tile_id)?;
        Ok(display_name)
    }

    pub fn get_tile_description(&self, tile_id: TileId) -> Result<&str, DataflowError> {
        let description = self.tile_field.get_description(tile_id)?;
        Ok(description)
    }

    // tile spatial features

    pub fn get_tile_id_by_point(&self, point: IVec2) -> Option<TileId> {
        self.tile_field.get_id_by_point(point)
    }

    // tile collision features

    pub fn get_tile_ids_by_collision_point(&self, point: Vec2) -> Option<TileId> {
        self.tile_field.get_ids_by_collision_point(point)
    }

    // tile inventory

    pub fn get_tile_inventory(&self, tile_id: TileId) -> Result<Option<InventoryKey>, DataflowError> {
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

    pub fn insert_block(&mut self, block: block::Block) -> Result<BlockId, DataflowError> {
        let feature = self
            .get_block_feature::<Rc<dyn FieldFeature<Key = BlockId>>>(block.archetype_id)
            .cloned();
        let block_id = self.block_field.insert(block)?;
        let _ = feature.map(|f| f.after_place(self, block_id));
        Ok(block_id)
    }

    pub fn remove_block(&mut self, block_id: BlockId) -> Result<block::Block, DataflowError> {
        let block = self.block_field.get(block_id)?;
        let feature = self
            .get_block_feature::<Rc<dyn FieldFeature<Key = BlockId>>>(block.archetype_id)
            .cloned();
        let _ = feature.map(|f| f.before_break(self, block_id));
        let block = self.block_field.remove(block_id)?;
        Ok(block)
    }

    pub fn modify_block(&mut self, block_id: BlockId, f: impl FnOnce(&mut block::Block)) -> Result<block::BlockId, DataflowError> {
        let block_id = self.block_field.modify(block_id, f)?;
        Ok(block_id)
    }

    pub fn get_block(&self, block_id: BlockId) -> Result<&block::Block, DataflowError> {
        let block = self.block_field.get(block_id)?;
        Ok(block)
    }

    pub fn get_block_version_by_chunk_coord(&self, chunk_coord: IVec2) -> Result<u64, DataflowError> {
        let chunk = self.block_field.get_version_by_chunk_coord(chunk_coord)?;
        Ok(chunk)
    }

    pub fn get_block_ids_by_chunk_coord(&self, chunk_coord: IVec2) -> Result<impl Iterator<Item = BlockId>, DataflowError> {
        let chunk = self.block_field.get_ids_by_chunk_coord(chunk_coord)?;
        Ok(chunk)
    }

    pub fn get_block_display_name(&self, block_id: BlockId) -> Result<&str, DataflowError> {
        let display_name = self.block_field.get_display_name(block_id)?;
        Ok(display_name)
    }

    pub fn get_block_description(&self, block_id: BlockId) -> Result<&str, DataflowError> {
        let description = self.block_field.get_description(block_id)?;
        Ok(description)
    }

    // block spatial features

    pub fn get_block_base_rect(&self, archetype_id: u16) -> Result<[IVec2; 2], DataflowError> {
        let rect = self.block_field.get_base_rect(archetype_id)?;
        Ok(rect)
    }

    pub fn get_block_rect(&self, block_id: BlockId) -> Result<[IVec2; 2], DataflowError> {
        let rect = self.block_field.get_rect(block_id)?;
        Ok(rect)
    }

    pub fn has_block_by_point(&self, point: IVec2) -> bool {
        self.block_field.has_by_point(point)
    }

    pub fn get_block_id_by_point(&self, point: IVec2) -> Option<BlockId> {
        self.block_field.get_id_by_point(point)
    }

    pub fn has_block_by_rect(&self, rect: [IVec2; 2]) -> bool {
        self.block_field.has_by_rect(rect)
    }

    pub fn get_block_ids_by_rect(&self, rect: [IVec2; 2]) -> impl Iterator<Item = BlockId> + '_ {
        self.block_field.get_ids_by_rect(rect)
    }

    pub fn get_block_chunk_coord(&self, point: Vec2) -> IVec2 {
        self.block_field.get_chunk_coord(point)
    }

    // block collision features

    pub fn get_block_base_collision_rect(&self, archetype_id: u16) -> Result<[Vec2; 2], DataflowError> {
        let rect = self.block_field.get_base_collision_rect(archetype_id)?;
        Ok(rect)
    }

    pub fn get_block_collision_rect(&self, block_id: BlockId) -> Result<[Vec2; 2], DataflowError> {
        let rect = self.block_field.get_collision_rect(block_id)?;
        Ok(rect)
    }

    pub fn has_block_by_collision_point(&self, point: Vec2) -> bool {
        self.block_field.has_by_collision_point(point)
    }

    pub fn get_block_ids_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = BlockId> + '_ {
        self.block_field.get_ids_by_collision_point(point)
    }

    pub fn has_block_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        self.block_field.has_by_collision_rect(rect)
    }

    pub fn get_block_ids_by_collision_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = BlockId> + '_ {
        self.block_field.get_ids_by_collision_rect(rect)
    }

    // block hint features

    pub fn get_block_base_y_sorting(&self, archetype_id: u16) -> Result<bool, DataflowError> {
        let y_sorting = self.block_field.get_base_y_sorting(archetype_id)?;
        Ok(y_sorting)
    }

    pub fn get_block_base_hint_rect(&self, archetype_id: u16) -> Result<[Vec2; 2], DataflowError> {
        let block = self.block_field.get_base_hint_rect(archetype_id)?;
        Ok(block)
    }

    pub fn get_block_hint_rect(&self, block_id: BlockId) -> Result<[Vec2; 2], DataflowError> {
        let block = self.block_field.get_hint_rect(block_id)?;
        Ok(block)
    }

    pub fn has_block_by_hint_point(&self, point: Vec2) -> bool {
        self.block_field.has_by_hint_point(point)
    }

    pub fn get_block_ids_by_hint_point(&self, point: Vec2) -> impl Iterator<Item = BlockId> + '_ {
        self.block_field.get_ids_by_hint_point(point)
    }

    pub fn has_block_by_hint_rect(&self, rect: [Vec2; 2]) -> bool {
        self.block_field.has_by_hint_rect(rect)
    }

    pub fn get_block_ids_by_hint_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = BlockId> + '_ {
        self.block_field.get_ids_by_hint_rect(rect)
    }

    // block inventory

    pub fn get_block_inventory(&self, block_id: BlockId) -> Result<Option<InventoryKey>, DataflowError> {
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

    pub fn remove_entity(&mut self, entity_id: EntityId) -> Result<entity::Entity, DataflowError> {
        let entity = self.entity_field.get(entity_id)?;
        let feature = self
            .get_entity_feature::<Rc<dyn FieldFeature<Key = EntityId>>>(entity.archetype_id)
            .cloned();
        let _ = feature.map(|f| f.before_break(self, entity_id));
        let entity = self.entity_field.remove(entity_id)?;
        Ok(entity)
    }

    pub fn modify_entity(&mut self, entity_id: EntityId, f: impl FnOnce(&mut entity::Entity)) -> Result<entity::EntityId, DataflowError> {
        let entity_id = self.entity_field.modify(entity_id, f)?;
        Ok(entity_id)
    }

    pub fn get_entity(&self, entity_id: EntityId) -> Result<&entity::Entity, DataflowError> {
        let entity = self.entity_field.get(entity_id)?;
        Ok(entity)
    }

    pub fn get_entity_version_by_chunk_coord(&self, chunk_location: IVec2) -> Result<u64, DataflowError> {
        let chunk = self.entity_field.get_version_by_chunk_coord(chunk_location)?;
        Ok(chunk)
    }

    pub fn get_entity_ids_by_chunk_coord(&self, chunk_location: IVec2) -> Result<impl Iterator<Item = BlockId>, DataflowError> {
        let chunk = self.entity_field.get_ids_by_chunk_coord(chunk_location)?;
        Ok(chunk)
    }

    pub fn get_entity_display_name(&self, entity_id: EntityId) -> Result<&str, DataflowError> {
        let display_name = self.entity_field.get_display_name(entity_id)?;
        Ok(display_name)
    }

    pub fn get_entity_description(&self, entity_id: EntityId) -> Result<&str, DataflowError> {
        let description = self.entity_field.get_description(entity_id)?;
        Ok(description)
    }

    // entity spatial features

    pub fn get_entity_chunk_coord(&self, point: Vec2) -> IVec2 {
        self.entity_field.get_chunk_coord(point)
    }

    // entity collision features

    pub fn get_entity_base_collision_rect(&self, archetype_id: u16) -> Result<[Vec2; 2], DataflowError> {
        let rect = self.entity_field.get_base_collision_rect(archetype_id)?;
        Ok(rect)
    }

    pub fn get_entity_collision_rect(&self, entity_id: EntityId) -> Result<[Vec2; 2], DataflowError> {
        let rect = self.entity_field.get_collision_rect(entity_id)?;
        Ok(rect)
    }

    pub fn has_entity_by_collision_point(&self, point: Vec2) -> bool {
        self.entity_field.has_by_collision_point(point)
    }

    pub fn get_entity_ids_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = EntityId> + '_ {
        self.entity_field.get_ids_by_collision_point(point)
    }

    pub fn has_entity_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        self.entity_field.has_by_collision_rect(rect)
    }

    pub fn get_entity_ids_by_collision_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = EntityId> + '_ {
        self.entity_field.get_ids_by_collision_rect(rect)
    }

    // entity hint features

    pub fn get_entity_base_y_sorting(&self, archetype_id: u16) -> Result<bool, DataflowError> {
        let y_sorting = self.entity_field.get_base_y_sorting(archetype_id)?;
        Ok(y_sorting)
    }

    pub fn get_entity_base_hint_rect(&self, archetype_id: u16) -> Result<[Vec2; 2], DataflowError> {
        let rect = self.entity_field.get_base_hint_rect(archetype_id)?;
        Ok(rect)
    }

    pub fn get_entity_hint_rect(&self, entity_id: EntityId) -> Result<[Vec2; 2], DataflowError> {
        let rect = self.entity_field.get_hint_rect(entity_id)?;
        Ok(rect)
    }

    pub fn has_entity_by_hint_point(&self, point: Vec2) -> bool {
        self.entity_field.has_by_hint_point(point)
    }

    pub fn get_entity_ids_by_hint_point(&self, point: Vec2) -> impl Iterator<Item = EntityId> + '_ {
        self.entity_field.get_ids_by_hint_point(point)
    }

    pub fn has_entity_by_hint_rect(&self, rect: [Vec2; 2]) -> bool {
        self.entity_field.has_by_hint_rect(rect)
    }

    pub fn get_entity_ids_by_hint_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = EntityId> + '_ {
        self.entity_field.get_ids_by_hint_rect(rect)
    }

    // entity inventory

    pub fn get_inventory_by_entity(&self, entity_id: EntityId) -> Result<Option<InventoryKey>, DataflowError> {
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

    pub fn insert_inventory(&mut self, id: u16) -> Result<InventoryKey, DataflowError> {
        let inventory_key = self.item_storage.insert_inventory(id)?;
        Ok(inventory_key)
    }

    pub fn remove_inventory(&mut self, inventory_key: InventoryKey) -> Result<u16, DataflowError> {
        let id = self.item_storage.remove_inventory(inventory_key)?;
        Ok(id)
    }

    pub fn get_inventory(&self, inventory_key: InventoryKey) -> Result<&Inventory, DataflowError> {
        let inventory = self.item_storage.get_inventory(inventory_key)?;
        Ok(inventory)
    }

    pub fn push_item_to_inventory(
        &mut self,
        inventory_key: InventoryKey,
        item: Item,
    ) -> Result<(), DataflowError> {
        self.item_storage
            .push_item_to_inventory(inventory_key, item)?;
        Ok(())
    }

    pub fn pop_item_from_inventory(
        &mut self,
        inventory_key: InventoryKey,
    ) -> Result<Item, DataflowError> {
        let item = self.item_storage.pop_item_from_inventory(inventory_key)?;
        Ok(item)
    }

    pub fn search_item_in_inventory(
        &self,
        inventory_key: InventoryKey,
        text: &str,
    ) -> Result<Vec<SlotKey>, DataflowError> {
        let item_key = self
            .item_storage
            .search_item_in_inventory(inventory_key, text)?;
        Ok(item_key)
    }

    pub fn insert_item(&mut self, slot_key: SlotKey, item: Item) -> Result<(), DataflowError> {
        self.item_storage.insert_item(slot_key, item)?;
        Ok(())
    }

    pub fn remove_item(&mut self, slot_key: SlotKey) -> Result<Item, DataflowError> {
        let item = self.item_storage.remove_item(slot_key)?;
        Ok(item)
    }

    pub fn modify_item(
        &mut self,
        slot_key: SlotKey,
        f: impl FnOnce(&mut Item),
    ) -> Result<(), DataflowError> {
        self.item_storage.modify_item(slot_key, f)?;
        Ok(())
    }

    pub fn swap_item(
        &mut self,
        slot_key_a: SlotKey,
        slot_key_b: SlotKey,
    ) -> Result<(), DataflowError> {
        self.item_storage.swap_item(slot_key_a, slot_key_b)?;
        Ok(())
    }

    pub fn get_item(&self, slot_key: SlotKey) -> Result<&Item, DataflowError> {
        let item = self.item_storage.get_item(slot_key)?;
        Ok(item)
    }

    pub fn get_item_display_name(&self, slot_key: SlotKey) -> Result<&str, DataflowError> {
        let display_name = self.item_storage.get_item_display_name(slot_key)?;
        Ok(display_name)
    }

    pub fn get_item_description(&self, slot_key: SlotKey) -> Result<&str, DataflowError> {
        let description = self.item_storage.get_item_description(slot_key)?;
        Ok(description)
    }

    // resources

    pub fn insert_resources<T>(&mut self, resource: T) -> Result<(), DataflowError>
    where
        T: Resource + 'static,
    {
        self.resource_storage.insert::<T>(resource)?;
        Ok(())
    }

    pub fn remove_resources<T>(&mut self) -> Result<T, DataflowError>
    where
        T: Resource + 'static,
    {
        let resource = self.resource_storage.remove::<T>()?;
        Ok(resource)
    }

    pub fn find_resources<T>(&self) -> Result<ResourceCell<T>, DataflowError>
    where
        T: Resource + 'static,
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
    fn get_inventory(&self, dataflow: &Dataflow, key: Self::Key) -> InventoryKey;
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
