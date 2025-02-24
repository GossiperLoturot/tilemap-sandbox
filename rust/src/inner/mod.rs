pub use animal::*;
pub use feature::*;
pub use field::*;
pub use forwarder::*;
pub use gen::*;
pub use item::*;
pub use player::*;
pub use time::*;

use glam::*;

mod animal;
mod feature;
mod field;
mod forwarder;
mod gen;
mod item;
mod player;
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

    pub gen_resource: GenResourceDescriptor,
}

#[derive(Debug)]
pub struct Root {
    // isolated fields
    tile_field: TileField,
    block_field: BlockField,
    entity_field: EntityField,
    item_store: ItemStore,
    time_store: TimeStore,

    // readonly shared fields
    tile_features: RcVec<Box<dyn TileFeature>>,
    block_features: RcVec<Box<dyn BlockFeature>>,
    entity_features: RcVec<Box<dyn EntityFeature>>,
    item_features: RcVec<Box<dyn ItemFeature>>,

    // shared fields
    forwarder_resource: Option<ForwarderResource>,
    gen_resource: Option<GenResource>,
    player_resource: Option<PlayerResource>,
}

impl Root {
    #[inline]
    pub fn new(desc: RootDescriptor) -> Self {
        Self {
            tile_field: TileField::new(desc.tile_field),
            block_field: BlockField::new(desc.block_field),
            entity_field: EntityField::new(desc.entity_field),
            item_store: ItemStore::new(desc.item_store),
            time_store: TimeStore::new(),

            tile_features: desc.tile_features,
            block_features: desc.block_features,
            entity_features: desc.entity_features,
            item_features: desc.item_features,

            forwarder_resource: Some(ForwarderResource::new()),
            gen_resource: Some(GenResource::new(desc.gen_resource)),
            player_resource: Some(PlayerResource::new()),
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

    // tile inventory

    #[inline]
    pub fn tile_get_inventory(
        &self,
        tile_key: TileKey,
    ) -> Result<Option<InventoryKey>, FieldError> {
        let features = self.tile_features.clone();
        let tile = self.tile_field.get(tile_key)?;
        let feature = features
            .get(tile.id as usize)
            .ok_or(FieldError::InvalidId)?;
        Ok(feature.get_inventory(self, tile_key))
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

    // block spatial features

    #[inline]
    pub fn block_get_base_rect(&self, id: u16) -> Result<[IVec2; 2], FieldError> {
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

    // block inventory

    #[inline]
    pub fn block_get_inventory(
        &self,
        block_key: BlockKey,
    ) -> Result<Option<InventoryKey>, FieldError> {
        let features = self.block_features.clone();
        let block = self.block_field.get(block_key)?;
        let feature = features
            .get(block.id as usize)
            .ok_or(FieldError::InvalidId)?;
        Ok(feature.get_inventory(self, block_key))
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

    // entity inventory

    #[inline]
    pub fn entity_get_inventory(
        &self,
        entity_key: EntityKey,
    ) -> Result<Option<InventoryKey>, FieldError> {
        let features = self.entity_features.clone();
        let entity = self.entity_field.get(entity_key)?;
        let feature = features
            .get(entity.id as usize)
            .ok_or(FieldError::InvalidId)?;
        Ok(feature.get_inventory(self, entity_key))
    }

    // item

    #[inline]
    pub fn item_alloc_inventory(&mut self, slot_size: u32) -> Result<InventoryKey, ItemError> {
        self.item_store.alloc_inventory(slot_size)
    }

    #[inline]
    pub fn item_free_inventory(&mut self, inventory_key: InventoryKey) -> Result<(), ItemError> {
        self.item_store.free_inventory(inventory_key)
    }

    #[inline]
    pub fn item_get_inventory(
        &mut self,
        inventory_key: InventoryKey,
    ) -> Result<&Inventory, ItemError> {
        self.item_store.get_inventory(inventory_key)
    }

    #[inline]
    pub fn item_push_item(
        &mut self,
        inventory_key: InventoryKey,
        item: Item,
    ) -> Result<(), ItemError> {
        self.item_store.push_item(inventory_key, item)
    }

    #[inline]
    pub fn item_pop_item(&mut self, inventory_key: InventoryKey) -> Result<Item, ItemError> {
        self.item_store.pop_item(inventory_key)
    }

    #[inline]
    pub fn item_search_item(
        &mut self,
        inventory_key: InventoryKey,
        text: &str,
    ) -> Result<Vec<SlotKey>, ItemError> {
        self.item_store.search_item(inventory_key, text)
    }

    #[inline]
    pub fn item_insert_item(&mut self, slot_key: SlotKey, item: Item) -> Result<(), ItemError> {
        self.item_store.insert_item(slot_key, item)
    }

    #[inline]
    pub fn item_remove_item(&mut self, slot_key: SlotKey) -> Result<Item, ItemError> {
        self.item_store.remove_item(slot_key)
    }

    #[inline]
    pub fn item_modify_item(
        &mut self,
        slot_key: SlotKey,
        f: impl FnOnce(&mut Item),
    ) -> Result<(), ItemError> {
        self.item_store.modify_item(slot_key, f)
    }

    #[inline]
    pub fn item_use_item(&mut self, slot_key: SlotKey) -> Result<(), ItemError> {
        let features = self.item_features.clone();
        let item = self.item_store.get_item(slot_key)?;
        let feature = features
            .get(item.id as usize)
            .ok_or(ItemError::ItemInvalidId)?;
        feature.r#use(self, slot_key);
        Ok(())
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

    // others

    #[inline]
    fn exclusive<Field, Output>(
        &mut self,
        ref_fn: impl Fn(&mut Self) -> &mut Option<Field>,
        run_fn: impl Fn(&mut Field, &mut Self) -> Output,
    ) -> Option<Output> {
        let mut resource = ref_fn(self).take()?;
        let value = run_fn(&mut resource, self);
        *ref_fn(self) = Some(resource);
        Some(value)
    }

    #[inline]
    pub fn forwarder_exec_rect(
        &mut self,
        min_rect: [Vec2; 2],
        delta_secs: f32,
    ) -> Result<(), RootError> {
        self.exclusive(
            |root| &mut root.forwarder_resource,
            |resource, root| resource.exec_rect(root, min_rect, delta_secs),
        )
        .ok_or(RootError::ResourceBusy)?
    }

    #[inline]
    pub fn gen_exec_rect(&mut self, min_rect: [Vec2; 2]) -> Result<(), RootError> {
        self.exclusive(
            |root| &mut root.gen_resource,
            |resource, root| resource.exec_rect(root, min_rect),
        )
        .ok_or(RootError::ResourceBusy)?
    }

    #[inline]
    pub fn player_insert_current(&mut self, entity_key: EntityKey) -> Result<(), RootError> {
        self.exclusive(
            |root| &mut root.player_resource,
            |resource, _| resource.insert_current(entity_key),
        )
        .ok_or(RootError::ResourceBusy)?
    }

    #[inline]
    pub fn player_remove_current(&mut self) -> Result<EntityKey, RootError> {
        self.exclusive(
            |root| &mut root.player_resource,
            |resource, _| resource.remove_current(),
        )
        .ok_or(RootError::ResourceBusy)?
    }

    #[inline]
    pub fn player_get_current(&mut self) -> Result<EntityKey, RootError> {
        self.exclusive(
            |root| &mut root.player_resource,
            |resource, _| resource.get_current(),
        )
        .ok_or(RootError::ResourceBusy)?
    }

    #[inline]
    pub fn player_insert_input(&mut self, input: Vec2) -> Result<(), RootError> {
        self.exclusive(
            |root| &mut root.player_resource,
            |resource, _| resource.insert_input(input),
        )
        .ok_or(RootError::ResourceBusy)?
    }

    #[inline]
    pub fn player_remove_input(&mut self) -> Result<Vec2, RootError> {
        self.exclusive(
            |root| &mut root.player_resource,
            |resource, _| resource.remove_input(),
        )
        .ok_or(RootError::ResourceBusy)?
    }

    #[inline]
    pub fn player_get_input(&mut self) -> Result<Vec2, RootError> {
        self.exclusive(
            |root| &mut root.player_resource,
            |resource, _| resource.get_input(),
        )
        .ok_or(RootError::ResourceBusy)?
    }

    #[inline]
    pub fn player_get_current_location(&mut self) -> Result<Vec2, RootError> {
        self.exclusive(
            |root| &mut root.player_resource,
            |resource, root| resource.get_current_location(root),
        )
        .ok_or(RootError::ResourceBusy)?
    }
}

// error handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RootError {
    FieldError(FieldError),
    ItemError(ItemError),
    PlayerError(PlayerError),
    ResourceBusy,
}

impl std::fmt::Display for RootError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FieldError(e) => e.fmt(f),
            Self::ItemError(e) => e.fmt(f),
            Self::PlayerError(e) => e.fmt(f),
            Self::ResourceBusy => write!(f, "resource is busy"),
        }
    }
}

impl std::error::Error for RootError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::FieldError(e) => Some(e),
            Self::ItemError(e) => Some(e),
            Self::PlayerError(e) => Some(e),
            _ => None,
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

impl From<PlayerError> for RootError {
    fn from(e: PlayerError) -> Self {
        Self::PlayerError(e)
    }
}
