use glam::*;

use crate::geom::*;

pub use item::*;
pub use resource::*;
pub use time::*;
pub use tile::*;
pub use block::*;
pub use entity::*;

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
    pub inventory_system: InventorySystemInfo,
}

pub struct Dataflow {
    time_storage: TimeStorage,

    // structured data storage
    tile_field: TileField,
    block_field: BlockField,
    entity_field: EntityField,
    inventory_system: InventorySystem,

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
            inventory_system: InventorySystem::new(info.inventory_system),

            resource_storage: ResourceStorage::new(),
        }
    }

    // time

    #[inline]
    pub fn tick_per_secs(&self) -> u64 {
        self.time_storage.get_tick_per_secs()
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

    #[inline]
    pub fn insert_tile(&mut self, tile: Tile) -> Result<TileId, DataflowError> {
        let tile_id = self.tile_field.insert(tile)?;
        Ok(tile_id)
    }

    #[inline]
    pub fn remove_til(&mut self, tile_id: TileId) -> Result<Tile, DataflowError> {
        let tile = self.tile_field.remove(tile_id)?;
        Ok(tile)
    }

    #[inline]
    pub fn modify_tile(&mut self, tile_id: TileId, f: impl FnOnce(&mut TileModify)) -> Result<TileId, DataflowError> {
        let tile_id = self.tile_field.modify(tile_id, f)?;
        Ok(tile_id)
    }

    #[inline]
    pub fn get_tile(&self, tile_key: TileId) -> Result<&Tile, DataflowError> {
        let tile_id = self.tile_field.get(tile_key)?;
        Ok(tile_id)
    }

    #[inline]
    pub fn find_tile_chunk_coord(&self, point: Vec2) -> IVec2 {
        self.tile_field.find_chunk_coord(point)
    }

    #[inline]
    pub fn get_tile_chunk(&self, chunk_coord: IVec2) -> Result<&TileChunk, DataflowError> {
        let chunk = self.tile_field.get_chunk(chunk_coord)?;
        Ok(chunk)
    }

    #[inline]
    pub fn get_tile_archetype(&self, archetype_id: u16) -> Result<&TileArchetype, DataflowError> {
        let archetype = self.tile_field.get_archetype(archetype_id)?;
        Ok(archetype)
    }

    // tile spatial features

    #[inline]
    pub fn find_tile_with_point(&self, point: IVec2) -> Option<TileId> {
        self.tile_field.find_with_point(point)
    }

    #[inline]
    pub fn find_tile_with_rect(&self, rect: IRect2) -> impl Iterator<Item = TileId> + '_ {
        self.tile_field.find_with_rect(rect)
    }

    // tile collision features

    #[inline]
    pub fn find_tile_with_collision_point(&self, coord: Vec2) -> impl Iterator<Item = TileId> + '_ {
        self.tile_field.find_with_collision_point(coord)
    }

    #[inline]
    pub fn find_tile_with_collision_rect(&self, rect: Rect2) -> impl Iterator<Item = TileId> + '_ {
        self.tile_field.find_with_collision_rect(rect)
    }

    // block

    #[inline]
    pub fn insert_block(&mut self, block: Block) -> Result<BlockId, DataflowError> {
        let block_id = self.block_field.insert(block)?;
        Ok(block_id)
    }

    #[inline]
    pub fn remove_block(&mut self, block_id: BlockId) -> Result<Block, DataflowError> {
        let block = self.block_field.remove(block_id)?;
        Ok(block)
    }

    #[inline]
    pub fn modify_block(&mut self, block_id: BlockId, f: impl FnOnce(&mut BlockModify)) -> Result<BlockId, DataflowError> {
        let block_id = self.block_field.modify(block_id, f)?;
        Ok(block_id)
    }

    #[inline]
    pub fn get_block(&self, block_id: BlockId) -> Result<&Block, DataflowError> {
        let block = self.block_field.get(block_id)?;
        Ok(block)
    }

    #[inline]
    pub fn find_block_chunk_coord(&self, point: Vec2) -> IVec2 {
        self.block_field.find_chunk_coord(point)
    }

    #[inline]
    pub fn get_block_chunk(&self, chunk_coord: IVec2) -> Result<&BlockChunk, DataflowError> {
        let chunk = self.block_field.get_chunk(chunk_coord)?;
        Ok(chunk)
    }

    #[inline]
    pub fn get_block_archetype(&self, archetype_id: u16) -> Result<&BlockArchetype, DataflowError> {
        let archetype = self.block_field.get_archetype(archetype_id)?;
        Ok(archetype)
    }

    // block spatial features

    #[inline]
    pub fn find_block_with_point(&self, point: IVec2) -> Option<BlockId> {
        self.block_field.find_with_point(point)
    }

    #[inline]
    pub fn find_block_with_rect(&self, rect: IRect2) -> impl Iterator<Item = BlockId> + '_ {
        self.block_field.find_with_rect(rect)
    }

    // block collision features

    #[inline]
    pub fn find_block_collision_point(&self, point: Vec2) -> impl Iterator<Item = BlockId> + '_ {
        self.block_field.find_with_collision_point(point)
    }

    #[inline]
    pub fn find_block_with_collision_rect(&self, rect: Rect2) -> impl Iterator<Item = BlockId> + '_ {
        self.block_field.find_with_collision_rect(rect)
    }

    // block hint features

    #[inline]
    pub fn find_block_with_hint_point(&self, point: Vec2) -> impl Iterator<Item = BlockId> + '_ {
        self.block_field.find_with_hint_point(point)
    }

    #[inline]
    pub fn find_block_with_hint_rect(&self, rect: Rect2) -> impl Iterator<Item = BlockId> + '_ {
        self.block_field.find_with_hint_rect(rect)
    }

    // entity

    #[inline]
    pub fn insert_entity(&mut self, entity: Entity) -> Result<EntityId, DataflowError> {
        let entity_id = self.entity_field.insert(entity)?;
        Ok(entity_id)
    }

    #[inline]
    pub fn remove_entity(&mut self, entity_id: EntityId) -> Result<Entity, DataflowError> {
        let entity = self.entity_field.remove(entity_id)?;
        Ok(entity)
    }

    #[inline]
    pub fn modify_entity(&mut self, entity_id: EntityId, f: impl FnOnce(&mut EntityModify)) -> Result<EntityId, DataflowError> {
        let entity_id = self.entity_field.modify(entity_id, f)?;
        Ok(entity_id)
    }

    #[inline]
    pub fn move_entity(&mut self, entity_id: EntityId, new_coord: Vec2) -> Result<EntityId, DataflowError> {
        let entity_id = self.entity_field.r#move(entity_id, new_coord)?;
        Ok(entity_id)
    }

    #[inline]
    pub fn get_entity(&self, entity_id: EntityId) -> Result<&Entity, DataflowError> {
        let entity = self.entity_field.get(entity_id)?;
        Ok(entity)
    }

    #[inline]
    pub fn find_entity_chunk_coord(&self, point: Vec2) -> IVec2 {
        self.entity_field.find_chunk_coord(point)
    }

    #[inline]
    pub fn get_entity_chunk(&self, chunk_coord: IVec2) -> Result<&EntityChunk, DataflowError> {
        let chunk = self.entity_field.get_chunk(chunk_coord)?;
        Ok(chunk)
    }

    #[inline]
    pub fn get_entity_archetype(&self, archetype_id: u16) -> Result<&EntityArchetype, DataflowError> {
        let archetype = self.entity_field.get_archetype(archetype_id)?;
        Ok(archetype)
    }

    // entity collision features

    #[inline]
    pub fn find_entity_with_collision_point(&self, point: Vec2) -> impl Iterator<Item = EntityId> + '_ {
        self.entity_field.find_with_collision_point(point)
    }

    #[inline]
    pub fn find_entity_with_collision_rect(&self, rect: Rect2) -> impl Iterator<Item = EntityId> + '_ {
        self.entity_field.find_with_collision_rect(rect)
    }

    // entity hint features

    #[inline]
    pub fn find_entity_with_hint_point(&self, point: Vec2) -> impl Iterator<Item = EntityId> + '_ {
        self.entity_field.find_with_hint_point(point)
    }

    #[inline]
    pub fn find_entity_with_hint_rect(&self, rect: Rect2) -> impl Iterator<Item = EntityId> + '_ {
        self.entity_field.find_with_hint_rect(rect)
    }

    // item

    pub fn insert_inventory(&mut self, archetype_id: u16) -> Result<InventoryId, DataflowError> {
        let inventory_key = self.inventory_system.insert_inventory(archetype_id)?;
        Ok(inventory_key)
    }

    pub fn remove_inventory(&mut self, inventory_id: InventoryId) -> Result<u16, DataflowError> {
        let id = self.inventory_system.remove_inventory(inventory_id)?;
        Ok(id)
    }

    pub fn get_inventory(&self, inventory_id: InventoryId) -> Result<&Inventory, DataflowError> {
        let inventory = self.inventory_system.get_inventory(inventory_id)?;
        Ok(inventory)
    }

    pub fn push_item_to_inventory(&mut self, inventory_id: InventoryId, item: Item) -> Result<(), DataflowError> {
        self.inventory_system.push_item_to_inventory(inventory_id, item)?;
        Ok(())
    }

    pub fn pop_item_from_inventory(&mut self, inventory_id: InventoryId) -> Result<Item, DataflowError> {
        let item = self.inventory_system.pop_item_from_inventory(inventory_id)?;
        Ok(item)
    }

    pub fn insert_item(&mut self, item_id: ItemId, item: Item) -> Result<(), DataflowError> {
        self.inventory_system.insert_item(item_id, item)?;
        Ok(())
    }

    pub fn remove_item(&mut self, item_id: ItemId) -> Result<Item, DataflowError> {
        let item = self.inventory_system.remove_item(item_id)?;
        Ok(item)
    }

    pub fn modify_item(&mut self, item_id: ItemId, f: impl FnOnce(&mut ItemModify)) -> Result<(), DataflowError> {
        self.inventory_system.modify_item(item_id, f)?;
        Ok(())
    }

    pub fn swap_item(&mut self, from_item_id: ItemId, to_item_id: ItemId) -> Result<(), DataflowError> {
        self.inventory_system.swap_item(from_item_id, to_item_id)?;
        Ok(())
    }

    pub fn get_item(&self, slot_id: ItemId) -> Result<&Item, DataflowError> {
        let item = self.inventory_system.get_item(slot_id)?;
        Ok(item)
    }

    pub fn get_item_archetype(&self, archetype_id: u16) -> Result<&ItemArchetype, DataflowError> {
        let archetype = self.inventory_system.get_item_archetype(archetype_id)?;
        Ok(archetype)
    }

    pub fn get_inventory_archetype(&self, archetype_id: u16) -> Result<&InventoryArchetype, DataflowError> {
        let archetype = self.inventory_system.get_inventory_archetype(archetype_id)?;
        Ok(archetype)
    }

    // resources

    #[inline]
    pub fn insert_resources<T>(&mut self, resource: T) -> Result<(), DataflowError> where T: Resource + 'static,
    {
        self.resource_storage.insert::<T>(resource)?;
        Ok(())
    }

    #[inline]
    pub fn remove_resources<T>(&mut self) -> Result<T, DataflowError> where T: Resource + 'static,
    {
        let resource = self.resource_storage.remove::<T>()?;
        Ok(resource)
    }

    #[inline]
    pub fn find_resources<T>(&self) -> Result<ResourceCell<T>, DataflowError> where T: Resource + 'static,
    {
        let resource = self.resource_storage.find::<T>()?;
        Ok(resource)
    }
}

// error handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataflowError {
    TileError(TileError),
    BlockError(BlockError),
    EntityError(EntityError),
    ItemError(ItemError),
    ResourceError(ResourceError),
}

impl std::fmt::Display for DataflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TileError(e) => e.fmt(f),
            Self::BlockError(e) => e.fmt(f),
            Self::EntityError(e) => e.fmt(f),
            Self::ItemError(e) => e.fmt(f),
            Self::ResourceError(e) => e.fmt(f),
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
