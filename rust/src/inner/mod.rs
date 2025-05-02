pub use animal::*;
pub use feature::*;
pub use field::*;
pub use forwarder::*;
pub use generator::*;
pub use item::*;
pub use player::*;
pub use resource::*;
pub use time::*;

use glam::*;

mod animal;
mod feature;
mod field;
mod forwarder;
mod generator;
mod item;
mod player;
mod resource;
mod time;

type RcVec<T> = std::rc::Rc<[T]>;

#[derive(Debug)]
pub struct RootDescriptor {
    pub tile_field: TileFieldDescriptor,
    pub block_field: BlockFieldDescriptor,
    pub entity_field: EntityFieldDescriptor,
    pub item_storage: ItemStorageDescriptor,

    pub tile_features: RcVec<Box<dyn TileFeature>>,
    pub block_features: RcVec<Box<dyn BlockFeature>>,
    pub entity_features: RcVec<Box<dyn EntityFeature>>,
    pub item_features: RcVec<Box<dyn ItemFeature>>,
}

#[derive(Debug)]
pub struct Root {
    time_storage: TimeStorage,

    // structured data storage
    tile_field: TileField,
    block_field: BlockField,
    entity_field: EntityField,
    item_storage: ItemStorage,

    // readonly functional data storage
    tile_features: RcVec<Box<dyn TileFeature>>,
    block_features: RcVec<Box<dyn BlockFeature>>,
    entity_features: RcVec<Box<dyn EntityFeature>>,
    item_features: RcVec<Box<dyn ItemFeature>>,

    // external data storage
    resource_storage: ResourceStorage,
}

impl Root {
    #[inline]
    pub fn new(desc: RootDescriptor) -> Self {
        Self {
            time_storage: TimeStorage::new(),

            tile_field: TileField::new(desc.tile_field),
            block_field: BlockField::new(desc.block_field),
            entity_field: EntityField::new(desc.entity_field),
            item_storage: ItemStorage::new(desc.item_storage),

            tile_features: desc.tile_features,
            block_features: desc.block_features,
            entity_features: desc.entity_features,
            item_features: desc.item_features,

            resource_storage: ResourceStorage::new(),
        }
    }

    // time

    #[inline]
    pub fn tick_per_secs(&self) -> u64 {
        self.time_storage.tick_per_secs()
    }

    #[inline]
    pub fn get_tick(&self) -> u64 {
        self.time_storage.get_tick()
    }

    #[inline]
    pub fn forward_time(&mut self, delta_secs: f32) {
        self.time_storage.forward(delta_secs);
    }

    // tile

    pub fn insert_tile(&mut self, tile: field::Tile) -> Result<TileKey, RootError> {
        let features = self.tile_features.clone();
        let feature = features
            .get(tile.id as usize)
            .ok_or(FieldError::InvalidId)?;
        let tile_key = self.tile_field.insert(tile)?;
        feature.after_place(self, tile_key);
        Ok(tile_key)
    }

    pub fn remove_til(&mut self, tile_key: TileKey) -> Result<field::Tile, RootError> {
        let features = self.tile_features.clone();
        let tile = self.tile_field.get(tile_key)?;
        let feature = features
            .get(tile.id as usize)
            .ok_or(FieldError::InvalidId)?;
        feature.before_break(self, tile_key);
        let tile = self.tile_field.remove(tile_key)?;
        Ok(tile)
    }

    #[inline]
    pub fn modify_tile(
        &mut self,
        tile_key: TileKey,
        f: impl FnOnce(&mut field::Tile),
    ) -> Result<field::TileKey, FieldError> {
        self.tile_field.modify(tile_key, f)
    }

    #[inline]
    pub fn get_tile(&self, tile_key: TileKey) -> Result<&field::Tile, RootError> {
        let tile_key = self.tile_field.get(tile_key)?;
        Ok(tile_key)
    }

    #[inline]
    pub fn get_tile_chunk_size(&self) -> u32 {
        self.tile_field.get_chunk_size()
    }

    pub fn get_tile_chunk(&self, chunk_location: IVec2) -> Result<&field::TileChunk, RootError> {
        let chunk_key = self
            .tile_field
            .get_by_chunk_location(chunk_location)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.tile_field.get_chunk(chunk_key).unwrap();
        Ok(chunk)
    }

    pub fn get_tile_name_text(&self, tile_key: TileKey) -> Result<&str, RootError> {
        let name_text = self.tile_field.get_name_text(tile_key)?;
        Ok(name_text)
    }

    pub fn get_tile_desc_text(&self, tile_key: TileKey) -> Result<&str, RootError> {
        let desc_text = self.tile_field.get_desc_text(tile_key)?;
        Ok(desc_text)
    }

    // tile spatial features

    #[inline]
    pub fn has_tile_by_point(&self, point: IVec2) -> bool {
        self.tile_field.has_by_point(point)
    }

    #[inline]
    pub fn get_tile_by_point(&self, point: IVec2) -> Option<TileKey> {
        self.tile_field.get_by_point(point)
    }

    // tile collision features

    #[inline]
    pub fn get_tile_collision_rect(&self, tile_key: TileKey) -> Result<[Vec2; 2], RootError> {
        let rect = self.tile_field.get_collision_rect(tile_key)?;
        Ok(rect)
    }

    #[inline]
    pub fn has_tile_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        self.tile_field.has_by_collision_rect(rect)
    }

    #[inline]
    pub fn get_tile_by_collision_rect(
        &self,
        rect: [Vec2; 2],
    ) -> impl Iterator<Item = TileKey> + '_ {
        self.tile_field.get_by_collision_rect(rect)
    }

    #[inline]
    pub fn has_tile_by_collision_point(&self, point: Vec2) -> bool {
        self.tile_field.has_by_collision_point(point)
    }

    #[inline]
    pub fn get_tile_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = TileKey> + '_ {
        self.tile_field.get_by_collision_point(point)
    }

    // tile inventory

    #[inline]
    pub fn get_tile_inventory(&self, tile_key: TileKey) -> Result<Option<InventoryKey>, RootError> {
        let features = self.tile_features.clone();
        let tile = self.tile_field.get(tile_key)?;
        let feature = features
            .get(tile.id as usize)
            .ok_or(FieldError::InvalidId)?;
        Ok(feature.get_inventory(self, tile_key))
    }

    // block

    pub fn insert_block(&mut self, block: field::Block) -> Result<BlockKey, RootError> {
        let features = self.block_features.clone();
        let feature = features
            .get(block.id as usize)
            .ok_or(FieldError::InvalidId)?;
        let block_key = self.block_field.insert(block)?;
        feature.after_place(self, block_key);
        Ok(block_key)
    }

    pub fn remove_block(&mut self, block_key: BlockKey) -> Result<field::Block, RootError> {
        let features = self.block_features.clone();
        let block = self.block_field.get(block_key)?;
        let feature = features
            .get(block.id as usize)
            .ok_or(FieldError::InvalidId)?;
        feature.before_break(self, block_key);
        let block = self.block_field.remove(block_key)?;
        Ok(block)
    }

    #[inline]
    pub fn modify_block(
        &mut self,
        block_key: BlockKey,
        f: impl FnOnce(&mut field::Block),
    ) -> Result<field::BlockKey, FieldError> {
        self.block_field.modify(block_key, f)
    }

    #[inline]
    pub fn get_block(&self, block_key: BlockKey) -> Result<&field::Block, RootError> {
        let block = self.block_field.get(block_key)?;
        Ok(block)
    }

    #[inline]
    pub fn get_block_chunk_size(&self) -> u32 {
        self.block_field.get_chunk_size()
    }

    pub fn get_block_chunk(&self, chunk_location: IVec2) -> Result<&field::BlockChunk, RootError> {
        let chunk_key = self
            .tile_field
            .get_by_chunk_location(chunk_location)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.block_field.get_chunk(chunk_key).unwrap();
        Ok(chunk)
    }

    pub fn get_block_name_text(&self, block_key: BlockKey) -> Result<&str, RootError> {
        let name_text = self.block_field.get_name_text(block_key)?;
        Ok(name_text)
    }

    pub fn get_block_desc_text(&self, block_key: BlockKey) -> Result<&str, RootError> {
        let desc_text = self.block_field.get_desc_text(block_key)?;
        Ok(desc_text)
    }

    // block spatial features

    #[inline]
    pub fn get_block_base_rect(&self, id: u16) -> Result<[IVec2; 2], RootError> {
        let rect = self.block_field.get_base_rect(id)?;
        Ok(rect)
    }

    #[inline]
    pub fn get_block_rect(&self, block_key: BlockKey) -> Result<[IVec2; 2], RootError> {
        let rect = self.block_field.get_rect(block_key)?;
        Ok(rect)
    }

    #[inline]
    pub fn has_block_by_point(&self, point: IVec2) -> bool {
        self.block_field.has_by_point(point)
    }

    #[inline]
    pub fn get_block_by_point(&self, point: IVec2) -> Option<BlockKey> {
        self.block_field.get_by_point(point)
    }

    #[inline]
    pub fn has_block_by_rect(&self, rect: [IVec2; 2]) -> bool {
        self.block_field.has_by_rect(rect)
    }

    #[inline]
    pub fn get_block_by_rect(&self, rect: [IVec2; 2]) -> impl Iterator<Item = BlockKey> + '_ {
        self.block_field.get_by_rect(rect)
    }

    // block collision features

    #[inline]
    pub fn get_block_base_collision_rect(&self, id: u16) -> Result<[Vec2; 2], RootError> {
        let rect = self.block_field.get_base_collision_rect(id)?;
        Ok(rect)
    }

    #[inline]
    pub fn get_block_collision_rect(&self, block_key: BlockKey) -> Result<[Vec2; 2], RootError> {
        let rect = self.block_field.get_collision_rect(block_key)?;
        Ok(rect)
    }

    #[inline]
    pub fn has_block_by_collision_point(&self, point: Vec2) -> bool {
        self.block_field.has_by_collision_point(point)
    }

    #[inline]
    pub fn get_block_by_collision_point(&self, point: Vec2) -> impl Iterator<Item = BlockKey> + '_ {
        self.block_field.get_by_collision_point(point)
    }

    #[inline]
    pub fn has_block_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        self.block_field.has_by_collision_rect(rect)
    }

    #[inline]
    pub fn get_block_by_collision_rect(
        &self,
        rect: [Vec2; 2],
    ) -> impl Iterator<Item = BlockKey> + '_ {
        self.block_field.get_by_collision_rect(rect)
    }

    // block hint features

    #[inline]
    pub fn get_block_base_z_along_y(&self, id: u16) -> Result<bool, RootError> {
        let z_along_y = self.block_field.get_base_z_along_y(id)?;
        Ok(z_along_y)
    }

    pub fn get_block_base_hint_rect(&self, id: u16) -> Result<[Vec2; 2], RootError> {
        let block = self.block_field.get_base_hint_rect(id)?;
        Ok(block)
    }

    #[inline]
    pub fn get_block_hint_rect(&self, block_key: BlockKey) -> Result<[Vec2; 2], RootError> {
        let block = self.block_field.get_hint_rect(block_key)?;
        Ok(block)
    }

    #[inline]
    pub fn has_block_by_hint_point(&self, point: Vec2) -> bool {
        self.block_field.has_by_hint_point(point)
    }

    #[inline]
    pub fn get_block_by_hint_point(&self, point: Vec2) -> impl Iterator<Item = BlockKey> + '_ {
        self.block_field.get_by_hint_point(point)
    }

    #[inline]
    pub fn has_block_by_hint_rect(&self, rect: [Vec2; 2]) -> bool {
        self.block_field.has_by_hint_rect(rect)
    }

    #[inline]
    pub fn get_block_by_hint_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = BlockKey> + '_ {
        self.block_field.get_by_hint_rect(rect)
    }

    // block inventory

    #[inline]
    pub fn get_block_inventory(
        &self,
        block_key: BlockKey,
    ) -> Result<Option<InventoryKey>, RootError> {
        let features = self.block_features.clone();
        let block = self.block_field.get(block_key)?;
        let feature = features
            .get(block.id as usize)
            .ok_or(FieldError::InvalidId)?;
        Ok(feature.get_inventory(self, block_key))
    }

    // entity

    pub fn insert_entity(&mut self, entity: field::Entity) -> Result<EntityKey, RootError> {
        let features = self.entity_features.clone();
        let feature = features
            .get(entity.id as usize)
            .ok_or(FieldError::InvalidId)?;
        let entity_key = self.entity_field.insert(entity)?;
        feature.after_place(self, entity_key);
        Ok(entity_key)
    }

    pub fn remove_entity(&mut self, entity_key: EntityKey) -> Result<field::Entity, RootError> {
        let features = self.entity_features.clone();
        let entity = self.entity_field.get(entity_key)?;
        let feature = features
            .get(entity.id as usize)
            .ok_or(FieldError::InvalidId)?;
        feature.before_break(self, entity_key);
        let entity = self.entity_field.remove(entity_key)?;
        Ok(entity)
    }

    #[inline]
    pub fn modify_entity(
        &mut self,
        entity_key: EntityKey,
        f: impl FnOnce(&mut field::Entity),
    ) -> Result<field::EntityKey, RootError> {
        let entity_key = self.entity_field.modify(entity_key, f)?;
        Ok(entity_key)
    }

    #[inline]
    pub fn get_entity(&self, entity_key: EntityKey) -> Result<&field::Entity, RootError> {
        let entity = self.entity_field.get(entity_key)?;
        Ok(entity)
    }

    #[inline]
    pub fn get_entity_chunk_size(&self) -> u32 {
        self.entity_field.get_chunk_size()
    }

    pub fn get_entity_chunk(&self, chunk_key: IVec2) -> Result<&field::EntityChunk, RootError> {
        let chunk_key = self
            .entity_field
            .get_by_chunk_location(chunk_key)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.entity_field.get_chunk(chunk_key).unwrap();
        Ok(chunk)
    }

    pub fn get_entity_name_text(&self, entity_key: EntityKey) -> Result<&str, RootError> {
        let name_text = self.entity_field.get_name_text(entity_key)?;
        Ok(name_text)
    }

    pub fn get_entity_desc_text(&self, entity_key: EntityKey) -> Result<&str, RootError> {
        let desc_text = self.entity_field.get_desc_text(entity_key)?;
        Ok(desc_text)
    }

    // entity collision features

    #[inline]
    pub fn get_entity_base_collision_rect(&self, id: u16) -> Result<[Vec2; 2], RootError> {
        let rect = self.entity_field.get_base_collision_rect(id)?;
        Ok(rect)
    }

    #[inline]
    pub fn get_entity_collision_rect(&self, entity_key: EntityKey) -> Result<[Vec2; 2], RootError> {
        let rect = self.entity_field.get_collision_rect(entity_key)?;
        Ok(rect)
    }

    #[inline]
    pub fn has_entity_by_collision_point(&self, point: Vec2) -> bool {
        self.entity_field.has_by_collision_point(point)
    }

    #[inline]
    pub fn get_entity_by_collision_point(
        &self,
        point: Vec2,
    ) -> impl Iterator<Item = EntityKey> + '_ {
        self.entity_field.get_by_collision_point(point)
    }

    #[inline]
    pub fn has_entity_by_collision_rect(&self, rect: [Vec2; 2]) -> bool {
        self.entity_field.has_by_collision_rect(rect)
    }

    #[inline]
    pub fn get_entity_by_collision_rect(
        &self,
        rect: [Vec2; 2],
    ) -> impl Iterator<Item = EntityKey> + '_ {
        self.entity_field.get_by_collision_rect(rect)
    }

    // entity hint features

    #[inline]
    pub fn get_entity_base_z_along_y(&self, id: u16) -> Result<bool, RootError> {
        let z_along_y = self.entity_field.get_base_z_along_y(id)?;
        Ok(z_along_y)
    }

    pub fn get_entity_base_hint_rect(&self, id: u16) -> Result<[Vec2; 2], RootError> {
        let rect = self.entity_field.get_base_hint_rect(id)?;
        Ok(rect)
    }

    #[inline]
    pub fn get_entity_hint_rect(&self, entity_key: EntityKey) -> Result<[Vec2; 2], RootError> {
        let rect = self.entity_field.get_hint_rect(entity_key)?;
        Ok(rect)
    }

    #[inline]
    pub fn has_entity_by_hint_point(&self, point: Vec2) -> bool {
        self.entity_field.has_by_hint_point(point)
    }

    #[inline]
    pub fn get_entity_by_hint_point(&self, point: Vec2) -> impl Iterator<Item = EntityKey> + '_ {
        self.entity_field.get_by_hint_point(point)
    }

    #[inline]
    pub fn has_entity_by_hint_rect(&self, rect: [Vec2; 2]) -> bool {
        self.entity_field.has_by_hint_rect(rect)
    }

    #[inline]
    pub fn get_entity_by_hint_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = EntityKey> + '_ {
        self.entity_field.get_by_hint_rect(rect)
    }

    // entity inventory

    #[inline]
    pub fn get_inventory_by_entity(
        &self,
        entity_key: EntityKey,
    ) -> Result<Option<InventoryKey>, RootError> {
        let features = self.entity_features.clone();
        let entity = self.entity_field.get(entity_key)?;
        let feature = features
            .get(entity.id as usize)
            .ok_or(FieldError::InvalidId)?;
        Ok(feature.get_inventory(self, entity_key))
    }

    // item

    #[inline]
    pub fn insert_inventory(&mut self, id: u16) -> Result<InventoryKey, RootError> {
        let inventory_key = self.item_storage.insert_inventory(id)?;
        Ok(inventory_key)
    }

    #[inline]
    pub fn remove_inventory(&mut self, inventory_key: InventoryKey) -> Result<u16, RootError> {
        let id = self.item_storage.remove_inventory(inventory_key)?;
        Ok(id)
    }

    #[inline]
    pub fn get_inventory(&self, inventory_key: InventoryKey) -> Result<&Inventory, RootError> {
        let inventory = self.item_storage.get_inventory(inventory_key)?;
        Ok(inventory)
    }

    #[inline]
    pub fn push_item(&mut self, inventory_key: InventoryKey, item: Item) -> Result<(), RootError> {
        self.item_storage.push_item(inventory_key, item)?;
        Ok(())
    }

    #[inline]
    pub fn pop_item(&mut self, inventory_key: InventoryKey) -> Result<Item, RootError> {
        let item = self.item_storage.pop_item(inventory_key)?;
        Ok(item)
    }

    #[inline]
    pub fn search_item(
        &self,
        inventory_key: InventoryKey,
        text: &str,
    ) -> Result<Vec<SlotKey>, RootError> {
        let item_key = self.item_storage.search_item(inventory_key, text)?;
        Ok(item_key)
    }

    #[inline]
    pub fn insert_item(&mut self, slot_key: SlotKey, item: Item) -> Result<(), RootError> {
        self.item_storage.insert_item(slot_key, item)?;
        Ok(())
    }

    #[inline]
    pub fn remove_item(&mut self, slot_key: SlotKey) -> Result<Item, RootError> {
        let item = self.item_storage.remove_item(slot_key)?;
        Ok(item)
    }

    #[inline]
    pub fn modify_item(
        &mut self,
        slot_key: SlotKey,
        f: impl FnOnce(&mut Item),
    ) -> Result<(), RootError> {
        self.item_storage.modify_item(slot_key, f)?;
        Ok(())
    }

    #[inline]
    pub fn use_item(&mut self, slot_key: SlotKey) -> Result<(), RootError> {
        let features = self.item_features.clone();
        let item = self.item_storage.get_item(slot_key)?;
        let feature = features
            .get(item.id as usize)
            .ok_or(ItemError::ItemInvalidId)?;
        feature.r#use(self, slot_key);
        Ok(())
    }

    // resources

    #[inline]
    pub fn insert_resources<T>(&mut self, resource: T) -> Result<(), RootError>
    where
        T: Resource + 'static,
    {
        self.resource_storage.insert::<T>(resource)?;
        Ok(())
    }

    #[inline]
    pub fn remove_resources<T>(&mut self) -> Result<T, RootError>
    where
        T: Resource + 'static,
    {
        let resource = self.resource_storage.remove::<T>()?;
        Ok(resource)
    }

    #[inline]
    pub fn find_resources<T>(&self) -> Result<ResourceCell<T>, RootError>
    where
        T: Resource + 'static,
    {
        let resource = self.resource_storage.find::<T>()?;
        Ok(resource)
    }
}

// error handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RootError {
    FieldError(FieldError),
    ItemError(ItemError),
    ResourceError(ResourceError),
}

impl std::fmt::Display for RootError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FieldError(e) => e.fmt(f),
            Self::ItemError(e) => e.fmt(f),
            Self::ResourceError(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for RootError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::FieldError(e) => Some(e),
            Self::ItemError(e) => Some(e),
            Self::ResourceError(e) => Some(e),
        }
    }
}

impl From<FieldError> for RootError {
    fn from(e: FieldError) -> Self {
        Self::FieldError(e)
    }
}

impl From<ItemError> for RootError {
    fn from(e: ItemError) -> Self {
        Self::ItemError(e)
    }
}

impl From<ResourceError> for RootError {
    fn from(e: ResourceError) -> Self {
        Self::ResourceError(e)
    }
}
