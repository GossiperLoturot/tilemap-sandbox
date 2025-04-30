pub use animal::*;
pub use feature::*;
pub use field::*;
pub use forwarder::*;
pub use gen::*;
pub use item::*;
pub use player::*;
pub use resource::*;
pub use time::*;

use glam::*;

mod animal;
mod feature;
mod field;
mod forwarder;
mod gen;
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
    pub item_store: ItemStoreDescriptor,

    pub tile_features: RcVec<Box<dyn TileFeature>>,
    pub block_features: RcVec<Box<dyn BlockFeature>>,
    pub entity_features: RcVec<Box<dyn EntityFeature>>,
    pub item_features: RcVec<Box<dyn ItemFeature>>,
}

#[derive(Debug)]
pub struct Root {
    time_store: TimeStore,

    // isolated data
    tile_field: TileField,
    block_field: BlockField,
    entity_field: EntityField,
    item_store: ItemStore,

    // readonly shared data
    tile_features: RcVec<Box<dyn TileFeature>>,
    block_features: RcVec<Box<dyn BlockFeature>>,
    entity_features: RcVec<Box<dyn EntityFeature>>,
    item_features: RcVec<Box<dyn ItemFeature>>,

    // shared data
    resources: Resources,
}

impl Root {
    #[inline]
    pub fn new(desc: RootDescriptor) -> Self {
        Self {
            time_store: TimeStore::new(),

            tile_field: TileField::new(desc.tile_field),
            block_field: BlockField::new(desc.block_field),
            entity_field: EntityField::new(desc.entity_field),
            item_store: ItemStore::new(desc.item_store),

            tile_features: desc.tile_features,
            block_features: desc.block_features,
            entity_features: desc.entity_features,
            item_features: desc.item_features,

            resources: Resources::new(),
        }
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

    // tile

    pub fn tile_insert(&mut self, tile: field::Tile) -> Result<TileKey, RootError> {
        let features = self.tile_features.clone();
        let feature = features
            .get(tile.id as usize)
            .ok_or(FieldError::InvalidId)?;
        let tile_key = self.tile_field.insert(tile)?;
        feature.after_place(self, tile_key);
        Ok(tile_key)
    }

    pub fn tile_remove(&mut self, tile_key: TileKey) -> Result<field::Tile, RootError> {
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
    pub fn tile_modify(
        &mut self,
        tile_key: TileKey,
        f: impl FnOnce(&mut field::Tile),
    ) -> Result<field::TileKey, FieldError> {
        self.tile_field.modify(tile_key, f)
    }

    #[inline]
    pub fn tile_get(&self, tile_key: TileKey) -> Result<&field::Tile, RootError> {
        let tile_key = self.tile_field.get(tile_key)?;
        Ok(tile_key)
    }

    #[inline]
    pub fn tile_get_chunk_size(&self) -> u32 {
        self.tile_field.get_chunk_size()
    }

    pub fn tile_get_chunk(&self, chunk_location: IVec2) -> Result<&field::TileChunk, RootError> {
        let chunk_key = self
            .tile_field
            .get_by_chunk_location(chunk_location)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.tile_field.get_chunk(chunk_key).unwrap();
        Ok(chunk)
    }

    pub fn tile_get_name_text(&self, tile_key: TileKey) -> Result<&str, RootError> {
        let name_text = self.tile_field.get_name_text(tile_key)?;
        Ok(name_text)
    }

    pub fn tile_get_desc_text(&self, tile_key: TileKey) -> Result<&str, RootError> {
        let desc_text = self.tile_field.get_desc_text(tile_key)?;
        Ok(desc_text)
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
    pub fn tile_get_collision_rect(&self, tile_key: TileKey) -> Result<[Vec2; 2], RootError> {
        let rect = self.tile_field.get_collision_rect(tile_key)?;
        Ok(rect)
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

    // tile inventory

    #[inline]
    pub fn tile_get_inventory(&self, tile_key: TileKey) -> Result<Option<InventoryKey>, RootError> {
        let features = self.tile_features.clone();
        let tile = self.tile_field.get(tile_key)?;
        let feature = features
            .get(tile.id as usize)
            .ok_or(FieldError::InvalidId)?;
        Ok(feature.get_inventory(self, tile_key))
    }

    // block

    pub fn block_insert(&mut self, block: field::Block) -> Result<BlockKey, RootError> {
        let features = self.block_features.clone();
        let feature = features
            .get(block.id as usize)
            .ok_or(FieldError::InvalidId)?;
        let block_key = self.block_field.insert(block)?;
        feature.after_place(self, block_key);
        Ok(block_key)
    }

    pub fn block_remove(&mut self, block_key: BlockKey) -> Result<field::Block, RootError> {
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
    pub fn block_modify(
        &mut self,
        block_key: BlockKey,
        f: impl FnOnce(&mut field::Block),
    ) -> Result<field::BlockKey, FieldError> {
        self.block_field.modify(block_key, f)
    }

    #[inline]
    pub fn block_get(&self, block_key: BlockKey) -> Result<&field::Block, RootError> {
        let block = self.block_field.get(block_key)?;
        Ok(block)
    }

    #[inline]
    pub fn block_get_chunk_size(&self) -> u32 {
        self.block_field.get_chunk_size()
    }

    pub fn block_get_chunk(&self, chunk_location: IVec2) -> Result<&field::BlockChunk, RootError> {
        let chunk_key = self
            .tile_field
            .get_by_chunk_location(chunk_location)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.block_field.get_chunk(chunk_key).unwrap();
        Ok(chunk)
    }

    pub fn block_get_name_text(&self, block_key: BlockKey) -> Result<&str, RootError> {
        let name_text = self.block_field.get_name_text(block_key)?;
        Ok(name_text)
    }

    pub fn block_get_desc_text(&self, block_key: BlockKey) -> Result<&str, RootError> {
        let desc_text = self.block_field.get_desc_text(block_key)?;
        Ok(desc_text)
    }

    // block spatial features

    #[inline]
    pub fn block_get_base_rect(&self, id: u16) -> Result<[IVec2; 2], RootError> {
        let rect = self.block_field.get_base_rect(id)?;
        Ok(rect)
    }

    #[inline]
    pub fn block_get_rect(&self, block_key: BlockKey) -> Result<[IVec2; 2], RootError> {
        let rect = self.block_field.get_rect(block_key)?;
        Ok(rect)
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
    pub fn block_get_base_collision_rect(&self, id: u16) -> Result<[Vec2; 2], RootError> {
        let rect = self.block_field.get_base_collision_rect(id)?;
        Ok(rect)
    }

    #[inline]
    pub fn block_get_collision_rect(&self, block_key: BlockKey) -> Result<[Vec2; 2], RootError> {
        let rect = self.block_field.get_collision_rect(block_key)?;
        Ok(rect)
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
    pub fn block_get_base_z_along_y(&self, id: u16) -> Result<bool, RootError> {
        let z_along_y = self.block_field.get_base_z_along_y(id)?;
        Ok(z_along_y)
    }

    pub fn block_get_base_hint_rect(&self, id: u16) -> Result<[Vec2; 2], RootError> {
        let block = self.block_field.get_base_hint_rect(id)?;
        Ok(block)
    }

    #[inline]
    pub fn block_get_hint_rect(&self, block_key: BlockKey) -> Result<[Vec2; 2], RootError> {
        let block = self.block_field.get_hint_rect(block_key)?;
        Ok(block)
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

    // block inventory

    #[inline]
    pub fn block_get_inventory(
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

    pub fn entity_insert(&mut self, entity: field::Entity) -> Result<EntityKey, RootError> {
        let features = self.entity_features.clone();
        let feature = features
            .get(entity.id as usize)
            .ok_or(FieldError::InvalidId)?;
        let entity_key = self.entity_field.insert(entity)?;
        feature.after_place(self, entity_key);
        Ok(entity_key)
    }

    pub fn entity_remove(&mut self, entity_key: EntityKey) -> Result<field::Entity, RootError> {
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
    pub fn entity_modify(
        &mut self,
        entity_key: EntityKey,
        f: impl FnOnce(&mut field::Entity),
    ) -> Result<field::EntityKey, RootError> {
        let entity_key = self.entity_field.modify(entity_key, f)?;
        Ok(entity_key)
    }

    #[inline]
    pub fn entity_get(&self, entity_key: EntityKey) -> Result<&field::Entity, RootError> {
        let entity = self.entity_field.get(entity_key)?;
        Ok(entity)
    }

    #[inline]
    pub fn entity_get_chunk_size(&self) -> u32 {
        self.entity_field.get_chunk_size()
    }

    pub fn entity_get_chunk(&self, chunk_key: IVec2) -> Result<&field::EntityChunk, RootError> {
        let chunk_key = self
            .entity_field
            .get_by_chunk_location(chunk_key)
            .ok_or(FieldError::NotFound)?;
        let chunk = self.entity_field.get_chunk(chunk_key).unwrap();
        Ok(chunk)
    }

    pub fn entity_get_name_text(&self, entity_key: EntityKey) -> Result<&str, RootError> {
        let name_text = self.entity_field.get_name_text(entity_key)?;
        Ok(name_text)
    }

    pub fn entity_get_desc_text(&self, entity_key: EntityKey) -> Result<&str, RootError> {
        let desc_text = self.entity_field.get_desc_text(entity_key)?;
        Ok(desc_text)
    }

    // entity collision features

    #[inline]
    pub fn entity_get_base_collision_rect(&self, id: u16) -> Result<[Vec2; 2], RootError> {
        let rect = self.entity_field.get_base_collision_rect(id)?;
        Ok(rect)
    }

    #[inline]
    pub fn entity_get_collision_rect(&self, entity_key: EntityKey) -> Result<[Vec2; 2], RootError> {
        let rect = self.entity_field.get_collision_rect(entity_key)?;
        Ok(rect)
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
    pub fn entity_get_base_z_along_y(&self, id: u16) -> Result<bool, RootError> {
        let z_along_y = self.entity_field.get_base_z_along_y(id)?;
        Ok(z_along_y)
    }

    pub fn entity_get_base_hint_rect(&self, id: u16) -> Result<[Vec2; 2], RootError> {
        let rect = self.entity_field.get_base_hint_rect(id)?;
        Ok(rect)
    }

    #[inline]
    pub fn entity_get_hint_rect(&self, entity_key: EntityKey) -> Result<[Vec2; 2], RootError> {
        let rect = self.entity_field.get_hint_rect(entity_key)?;
        Ok(rect)
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

    // entity inventory

    #[inline]
    pub fn entity_get_inventory(
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
    pub fn item_insert_inventory(&mut self, id: u16) -> Result<InventoryKey, RootError> {
        let inventory_key = self.item_store.insert_inventory(id)?;
        Ok(inventory_key)
    }

    #[inline]
    pub fn item_remove_inventory(&mut self, inventory_key: InventoryKey) -> Result<u16, RootError> {
        let id = self.item_store.remove_inventory(inventory_key)?;
        Ok(id)
    }

    #[inline]
    pub fn item_get_inventory(&self, inventory_key: InventoryKey) -> Result<&Inventory, RootError> {
        let inventory = self.item_store.get_inventory(inventory_key)?;
        Ok(inventory)
    }

    #[inline]
    pub fn item_push_item(
        &mut self,
        inventory_key: InventoryKey,
        item: Item,
    ) -> Result<(), RootError> {
        self.item_store.push_item(inventory_key, item)?;
        Ok(())
    }

    #[inline]
    pub fn item_pop_item(&mut self, inventory_key: InventoryKey) -> Result<Item, RootError> {
        let item = self.item_store.pop_item(inventory_key)?;
        Ok(item)
    }

    #[inline]
    pub fn item_search_item(
        &self,
        inventory_key: InventoryKey,
        text: &str,
    ) -> Result<Vec<SlotKey>, RootError> {
        let item_key = self.item_store.search_item(inventory_key, text)?;
        Ok(item_key)
    }

    #[inline]
    pub fn item_insert_item(&mut self, slot_key: SlotKey, item: Item) -> Result<(), RootError> {
        self.item_store.insert_item(slot_key, item)?;
        Ok(())
    }

    #[inline]
    pub fn item_remove_item(&mut self, slot_key: SlotKey) -> Result<Item, RootError> {
        let item = self.item_store.remove_item(slot_key)?;
        Ok(item)
    }

    #[inline]
    pub fn item_modify_item(
        &mut self,
        slot_key: SlotKey,
        f: impl FnOnce(&mut Item),
    ) -> Result<(), RootError> {
        self.item_store.modify_item(slot_key, f)?;
        Ok(())
    }

    #[inline]
    pub fn item_use_item(&mut self, slot_key: SlotKey) -> Result<(), RootError> {
        let features = self.item_features.clone();
        let item = self.item_store.get_item(slot_key)?;
        let feature = features
            .get(item.id as usize)
            .ok_or(ItemError::ItemInvalidId)?;
        feature.r#use(self, slot_key);
        Ok(())
    }

    // resources

    #[inline]
    pub fn insert_resources<T: 'static>(&mut self, resource: T) -> Result<(), RootError> {
        self.resources.insert::<T>(resource)?;
        Ok(())
    }

    #[inline]
    pub fn remove_resources<T: 'static>(&mut self) -> Result<T, RootError> {
        let resource = self.resources.remove::<T>()?;
        Ok(resource)
    }

    #[inline]
    pub fn find_resources<T: 'static>(&self) -> Result<ResourceCell<T>, RootError> {
        let resource = self.resources.find::<T>()?;
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
