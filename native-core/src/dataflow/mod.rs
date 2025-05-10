use glam::*;
use std::rc::Rc;

pub use data::*;
pub use feature::*;
pub use field::*;
pub use item::*;
pub use resource::*;
pub use time::*;

mod data;
mod feature;
mod field;
mod item;
mod resource;
mod time;

pub struct DataflowDescriptor {
    pub tile_field_desc: TileFieldDescriptor,
    pub block_field_desc: BlockFieldDescriptor,
    pub entity_field_desc: EntityFieldDescriptor,
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

    pub fn insert_tile(&mut self, tile: field::Tile) -> Result<TileKey, DataflowError> {
        let feature = self
            .get_tile_feature::<Rc<dyn FieldFeature<Key = TileKey>>>(tile.id)
            .cloned();
        let tile_key = self.tile_field.insert(tile)?;
        let _ = feature.map(|f| f.after_place(self, tile_key));
        Ok(tile_key)
    }

    pub fn remove_til(&mut self, tile_key: TileKey) -> Result<field::Tile, DataflowError> {
        let tile = self.tile_field.get(tile_key)?;
        let feature = self
            .get_tile_feature::<Rc<dyn FieldFeature<Key = TileKey>>>(tile.id)
            .cloned();
        let _ = feature.map(|f| f.before_break(self, tile_key));
        let tile = self.tile_field.remove(tile_key)?;
        Ok(tile)
    }

    pub fn modify_tile(
        &mut self,
        tile_key: TileKey,
        f: impl FnOnce(&mut field::Tile),
    ) -> Result<field::TileKey, FieldError> {
        self.tile_field.modify(tile_key, f)
    }

    pub fn get_tile(&self, tile_key: TileKey) -> Result<&field::Tile, DataflowError> {
        let tile_key = self.tile_field.get(tile_key)?;
        Ok(tile_key)
    }

    pub fn get_tile_version_by_chunk_location(
        &self,
        chunk_location: IVec2,
    ) -> Result<u64, DataflowError> {
        let chunk = self
            .tile_field
            .get_version_by_chunk_location(chunk_location)?;
        Ok(chunk)
    }

    pub fn get_tile_keys_by_chunk_location(
        &self,
        chunk_location: IVec2,
    ) -> Result<impl Iterator<Item = BlockKey>, DataflowError> {
        let chunk = self.tile_field.get_keys_by_chunk_location(chunk_location)?;
        Ok(chunk)
    }

    pub fn get_tile_display_name(&self, tile_key: TileKey) -> Result<&str, DataflowError> {
        let display_name = self.tile_field.get_display_name(tile_key)?;
        Ok(display_name)
    }

    pub fn get_tile_description(&self, tile_key: TileKey) -> Result<&str, DataflowError> {
        let description = self.tile_field.get_description(tile_key)?;
        Ok(description)
    }

    // tile spatial features

    pub fn has_tile_by_point(&self, point: IVec2) -> bool {
        self.tile_field.has_by_point(point)
    }

    pub fn get_tile_key_by_point(&self, point: IVec2) -> Option<TileKey> {
        self.tile_field.get_key_by_point(point)
    }

    pub fn get_tile_chunk_location(&self, point: Vec2) -> IVec2 {
        self.tile_field.get_chunk_location(point)
    }

    // tile collision features

    pub fn get_tile_collision_rect(&self, tile_key: TileKey) -> Result<[Vec2; 2], DataflowError> {
        let rect = self.tile_field.get_collision_rect(tile_key)?;
        Ok(rect)
    }

    pub fn has_tile_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        self.tile_field.has_by_collision_rect(rect)
    }

    pub fn get_tile_keys_by_collision_rect(
        &self,
        rect: [Vec2; 2],
    ) -> impl Iterator<Item = TileKey> + '_ {
        self.tile_field.get_keys_by_collision_rect(rect)
    }

    pub fn has_tile_by_collision_point(&self, point: Vec2) -> bool {
        self.tile_field.has_by_collision_point(point)
    }

    pub fn get_tile_keys_by_collision_point(
        &self,
        point: Vec2,
    ) -> impl Iterator<Item = TileKey> + '_ {
        self.tile_field.get_keys_by_collision_point(point)
    }

    // tile inventory

    pub fn get_tile_inventory(
        &self,
        tile_key: TileKey,
    ) -> Result<Option<InventoryKey>, DataflowError> {
        let tile = self.tile_field.get(tile_key)?;
        let feature = self
            .get_tile_feature::<Rc<dyn InventoryFeature<Key = TileKey>>>(tile.id)
            .cloned();
        let inventory = feature.map(|f| f.get_inventory(self, tile_key)).ok();
        Ok(inventory)
    }

    // block

    pub fn get_block_feature<T: 'static>(&self, id: u16) -> Result<&T, DataflowError> {
        let feature = self.block_features.get::<T>(id)?;
        Ok(feature)
    }

    pub fn insert_block(&mut self, block: field::Block) -> Result<BlockKey, DataflowError> {
        let feature = self
            .get_block_feature::<Rc<dyn FieldFeature<Key = BlockKey>>>(block.id)
            .cloned();
        let block_key = self.block_field.insert(block)?;
        let _ = feature.map(|f| f.after_place(self, block_key));
        Ok(block_key)
    }

    pub fn remove_block(&mut self, block_key: BlockKey) -> Result<field::Block, DataflowError> {
        let block = self.block_field.get(block_key)?;
        let feature = self
            .get_block_feature::<Rc<dyn FieldFeature<Key = BlockKey>>>(block.id)
            .cloned();
        let _ = feature.map(|f| f.before_break(self, block_key));
        let block = self.block_field.remove(block_key)?;
        Ok(block)
    }

    pub fn modify_block(
        &mut self,
        block_key: BlockKey,
        f: impl FnOnce(&mut field::Block),
    ) -> Result<field::BlockKey, FieldError> {
        self.block_field.modify(block_key, f)
    }

    pub fn get_block(&self, block_key: BlockKey) -> Result<&field::Block, DataflowError> {
        let block = self.block_field.get(block_key)?;
        Ok(block)
    }

    pub fn get_block_version_by_chunk_location(
        &self,
        chunk_location: IVec2,
    ) -> Result<u64, DataflowError> {
        let chunk = self
            .block_field
            .get_version_by_chunk_location(chunk_location)?;
        Ok(chunk)
    }

    pub fn get_block_keys_by_chunk_location(
        &self,
        chunk_location: IVec2,
    ) -> Result<impl Iterator<Item = BlockKey>, DataflowError> {
        let chunk = self
            .block_field
            .get_keys_by_chunk_location(chunk_location)?;
        Ok(chunk)
    }

    pub fn get_block_display_name(&self, block_key: BlockKey) -> Result<&str, DataflowError> {
        let display_name = self.block_field.get_display_name(block_key)?;
        Ok(display_name)
    }

    pub fn get_block_description(&self, block_key: BlockKey) -> Result<&str, DataflowError> {
        let description = self.block_field.get_description(block_key)?;
        Ok(description)
    }

    // block spatial features

    pub fn get_block_base_rect(&self, id: u16) -> Result<[IVec2; 2], DataflowError> {
        let rect = self.block_field.get_base_rect(id)?;
        Ok(rect)
    }

    pub fn get_block_rect(&self, block_key: BlockKey) -> Result<[IVec2; 2], DataflowError> {
        let rect = self.block_field.get_rect(block_key)?;
        Ok(rect)
    }

    pub fn has_block_by_point(&self, point: IVec2) -> bool {
        self.block_field.has_by_point(point)
    }

    pub fn get_block_key_by_point(&self, point: IVec2) -> Option<BlockKey> {
        self.block_field.get_key_by_point(point)
    }

    pub fn has_block_by_rect(&self, rect: [IVec2; 2]) -> bool {
        self.block_field.has_by_rect(rect)
    }

    pub fn get_block_keys_by_rect(&self, rect: [IVec2; 2]) -> impl Iterator<Item = BlockKey> + '_ {
        self.block_field.get_keys_by_rect(rect)
    }

    pub fn get_block_chunk_location(&self, point: Vec2) -> IVec2 {
        self.block_field.get_chunk_location(point)
    }

    // block collision features

    pub fn get_block_base_collision_rect(&self, id: u16) -> Result<[Vec2; 2], DataflowError> {
        let rect = self.block_field.get_base_collision_rect(id)?;
        Ok(rect)
    }

    pub fn get_block_collision_rect(
        &self,
        block_key: BlockKey,
    ) -> Result<[Vec2; 2], DataflowError> {
        let rect = self.block_field.get_collision_rect(block_key)?;
        Ok(rect)
    }

    pub fn has_block_by_collision_point(&self, point: Vec2) -> bool {
        self.block_field.has_by_collision_point(point)
    }

    pub fn get_block_keys_by_collision_point(
        &self,
        point: Vec2,
    ) -> impl Iterator<Item = BlockKey> + '_ {
        self.block_field.get_keys_by_collision_point(point)
    }

    pub fn has_block_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        self.block_field.has_by_collision_rect(rect)
    }

    pub fn get_block_keys_by_collision_rect(
        &self,
        rect: [Vec2; 2],
    ) -> impl Iterator<Item = BlockKey> + '_ {
        self.block_field.get_keys_by_collision_rect(rect)
    }

    // block hint features

    pub fn get_block_base_z_along_y(&self, id: u16) -> Result<bool, DataflowError> {
        let z_along_y = self.block_field.get_base_z_along_y(id)?;
        Ok(z_along_y)
    }

    pub fn get_block_base_hint_rect(&self, id: u16) -> Result<[Vec2; 2], DataflowError> {
        let block = self.block_field.get_base_hint_rect(id)?;
        Ok(block)
    }

    pub fn get_block_hint_rect(&self, block_key: BlockKey) -> Result<[Vec2; 2], DataflowError> {
        let block = self.block_field.get_hint_rect(block_key)?;
        Ok(block)
    }

    pub fn has_block_by_hint_point(&self, point: Vec2) -> bool {
        self.block_field.has_by_hint_point(point)
    }

    pub fn get_block_keys_by_hint_point(&self, point: Vec2) -> impl Iterator<Item = BlockKey> + '_ {
        self.block_field.get_keys_by_hint_point(point)
    }

    pub fn has_block_by_hint_rect(&self, rect: [Vec2; 2]) -> bool {
        self.block_field.has_by_hint_rect(rect)
    }

    pub fn get_block_keys_by_hint_rect(
        &self,
        rect: [Vec2; 2],
    ) -> impl Iterator<Item = BlockKey> + '_ {
        self.block_field.get_keys_by_hint_rect(rect)
    }

    // block inventory

    pub fn get_block_inventory(
        &self,
        block_key: BlockKey,
    ) -> Result<Option<InventoryKey>, DataflowError> {
        let block = self.block_field.get(block_key)?;
        let feature = self
            .get_block_feature::<Rc<dyn InventoryFeature<Key = BlockKey>>>(block.id)
            .cloned();
        let inventory = feature.map(|f| f.get_inventory(self, block_key)).ok();
        Ok(inventory)
    }

    // entity

    pub fn get_entity_feature<T: 'static>(&self, id: u16) -> Result<&T, DataflowError> {
        let feature = self.entity_features.get::<T>(id)?;
        Ok(feature)
    }

    pub fn insert_entity(&mut self, entity: field::Entity) -> Result<EntityKey, DataflowError> {
        let feature = self
            .get_entity_feature::<Rc<dyn FieldFeature<Key = EntityKey>>>(entity.id)
            .cloned();
        let entity_key = self.entity_field.insert(entity)?;
        let _ = feature.map(|f| f.after_place(self, entity_key));
        Ok(entity_key)
    }

    pub fn remove_entity(&mut self, entity_key: EntityKey) -> Result<field::Entity, DataflowError> {
        let entity = self.entity_field.get(entity_key)?;
        let feature = self
            .get_entity_feature::<Rc<dyn FieldFeature<Key = EntityKey>>>(entity.id)
            .cloned();
        let _ = feature.map(|f| f.before_break(self, entity_key));
        let entity = self.entity_field.remove(entity_key)?;
        Ok(entity)
    }

    pub fn modify_entity(
        &mut self,
        entity_key: EntityKey,
        f: impl FnOnce(&mut field::Entity),
    ) -> Result<field::EntityKey, DataflowError> {
        let entity_key = self.entity_field.modify(entity_key, f)?;
        Ok(entity_key)
    }

    pub fn get_entity(&self, entity_key: EntityKey) -> Result<&field::Entity, DataflowError> {
        let entity = self.entity_field.get(entity_key)?;
        Ok(entity)
    }

    pub fn get_entity_version_by_chunk_location(
        &self,
        chunk_location: IVec2,
    ) -> Result<u64, DataflowError> {
        let chunk = self
            .entity_field
            .get_version_by_chunk_location(chunk_location)?;
        Ok(chunk)
    }

    pub fn get_entity_keys_by_chunk_location(
        &self,
        chunk_location: IVec2,
    ) -> Result<impl Iterator<Item = BlockKey>, DataflowError> {
        let chunk = self
            .entity_field
            .get_keys_by_chunk_location(chunk_location)?;
        Ok(chunk)
    }

    pub fn get_entity_display_name(&self, entity_key: EntityKey) -> Result<&str, DataflowError> {
        let display_name = self.entity_field.get_display_name(entity_key)?;
        Ok(display_name)
    }

    pub fn get_entity_description(&self, entity_key: EntityKey) -> Result<&str, DataflowError> {
        let description = self.entity_field.get_description(entity_key)?;
        Ok(description)
    }

    // entity spatial features

    pub fn get_entity_chunk_location(&self, point: Vec2) -> IVec2 {
        self.entity_field.get_chunk_location(point)
    }

    // entity collision features

    pub fn get_entity_base_collision_rect(&self, id: u16) -> Result<[Vec2; 2], DataflowError> {
        let rect = self.entity_field.get_base_collision_rect(id)?;
        Ok(rect)
    }

    pub fn get_entity_collision_rect(
        &self,
        entity_key: EntityKey,
    ) -> Result<[Vec2; 2], DataflowError> {
        let rect = self.entity_field.get_collision_rect(entity_key)?;
        Ok(rect)
    }

    pub fn has_entity_by_collision_point(&self, point: Vec2) -> bool {
        self.entity_field.has_by_collision_point(point)
    }

    pub fn get_entity_keys_by_collision_point(
        &self,
        point: Vec2,
    ) -> impl Iterator<Item = EntityKey> + '_ {
        self.entity_field.get_keys_by_collision_point(point)
    }

    pub fn has_entity_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        self.entity_field.has_by_collision_rect(rect)
    }

    pub fn get_entity_keys_by_collision_rect(
        &self,
        rect: [Vec2; 2],
    ) -> impl Iterator<Item = EntityKey> + '_ {
        self.entity_field.get_keys_by_collision_rect(rect)
    }

    // entity hint features

    pub fn get_entity_base_z_along_y(&self, id: u16) -> Result<bool, DataflowError> {
        let z_along_y = self.entity_field.get_base_z_along_y(id)?;
        Ok(z_along_y)
    }

    pub fn get_entity_base_hint_rect(&self, id: u16) -> Result<[Vec2; 2], DataflowError> {
        let rect = self.entity_field.get_base_hint_rect(id)?;
        Ok(rect)
    }

    pub fn get_entity_hint_rect(&self, entity_key: EntityKey) -> Result<[Vec2; 2], DataflowError> {
        let rect = self.entity_field.get_hint_rect(entity_key)?;
        Ok(rect)
    }

    pub fn has_entity_by_hint_point(&self, point: Vec2) -> bool {
        self.entity_field.has_by_hint_point(point)
    }

    pub fn get_entity_keys_by_hint_point(
        &self,
        point: Vec2,
    ) -> impl Iterator<Item = EntityKey> + '_ {
        self.entity_field.get_keys_by_hint_point(point)
    }

    pub fn has_entity_by_hint_rect(&self, rect: [Vec2; 2]) -> bool {
        self.entity_field.has_by_hint_rect(rect)
    }

    pub fn get_entity_keys_by_hint_rect(
        &self,
        rect: [Vec2; 2],
    ) -> impl Iterator<Item = EntityKey> + '_ {
        self.entity_field.get_keys_by_hint_rect(rect)
    }

    // entity inventory

    pub fn get_inventory_by_entity(
        &self,
        entity_key: EntityKey,
    ) -> Result<Option<InventoryKey>, DataflowError> {
        let entity = self.entity_field.get(entity_key)?;
        let feature = self
            .get_entity_feature::<Rc<dyn InventoryFeature<Key = EntityKey>>>(entity.id)
            .cloned();
        let inventory = feature.map(|f| f.get_inventory(self, entity_key)).ok();
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
    FieldError(FieldError),
    ItemError(ItemError),
    ResourceError(ResourceError),
    FeatureError(FeatureError),
}

impl std::fmt::Display for DataflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FieldError(e) => e.fmt(f),
            Self::ItemError(e) => e.fmt(f),
            Self::ResourceError(e) => e.fmt(f),
            Self::FeatureError(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for DataflowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::FieldError(e) => Some(e),
            Self::ItemError(e) => Some(e),
            Self::ResourceError(e) => Some(e),
            Self::FeatureError(e) => Some(e),
        }
    }
}

impl From<FieldError> for DataflowError {
    fn from(e: FieldError) -> Self {
        Self::FieldError(e)
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
