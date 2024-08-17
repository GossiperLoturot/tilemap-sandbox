pub use field::*;
pub use flow::*;
pub use resource::*;
pub use tag::*;

mod field;
mod flow;
mod resource;
mod tag;

pub type Vec2 = [f32; 2];
pub type IVec2 = [i32; 2];

pub struct RootDescriptor {
    pub tile_field: TileFieldDescriptor,
    pub block_field: BlockFieldDescriptor,
    pub entity_field: EntityFieldDescriptor,
    pub flow_store: FlowStoreDescriptor,
}

pub struct Root {
    tile_field: TileField,
    block_field: BlockField,
    entity_field: EntityField,
    resource_store: ResourceStore,
    tag_store: TagStore,
    flow_store: FlowStore,
}

impl Root {
    #[inline]
    pub fn new(desc: RootDescriptor) -> Self {
        Self {
            tile_field: TileField::new(desc.tile_field),
            block_field: BlockField::new(desc.block_field),
            entity_field: EntityField::new(desc.entity_field),
            resource_store: Default::default(),
            tag_store: Default::default(),
            flow_store: FlowStore::new(desc.flow_store),
        }
    }

    // tile

    #[inline]
    pub fn tile_insert(&mut self, tile: field::Tile) -> Result<TileKey, FieldError> {
        self.tile_field.insert(tile)
    }

    #[inline]
    pub fn tile_remove(&mut self, tile_key: TileKey) -> Result<field::Tile, FieldError> {
        self.tile_field.remove(tile_key)
    }

    #[inline]
    pub fn tile_modify(
        &mut self,
        tile_key: TileKey,
        f: impl Fn(&mut field::Tile),
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

    #[inline]
    pub fn tile_get_chunk(&self, chunk_key: IVec2) -> Result<&field::TileChunk, FieldError> {
        self.tile_field.get_chunk(chunk_key)
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

    #[inline]
    pub fn block_insert(&mut self, block: field::Block) -> Result<BlockKey, FieldError> {
        self.block_field.insert(block)
    }

    #[inline]
    pub fn block_remove(&mut self, block_key: BlockKey) -> Result<field::Block, FieldError> {
        self.block_field.remove(block_key)
    }

    #[inline]
    pub fn block_modify(
        &mut self,
        block_key: BlockKey,
        f: impl Fn(&mut field::Block),
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

    #[inline]
    pub fn block_get_chunk(&self, chunk_key: IVec2) -> Result<&field::BlockChunk, FieldError> {
        self.block_field.get_chunk(chunk_key)
    }

    // block spatial features

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

    #[inline]
    pub fn entity_insert(&mut self, entity: field::Entity) -> Result<EntityKey, FieldError> {
        self.entity_field.insert(entity)
    }

    #[inline]
    pub fn entity_remove(&mut self, entity_key: EntityKey) -> Result<field::Entity, FieldError> {
        self.entity_field.remove(entity_key)
    }

    #[inline]
    pub fn entity_modify(
        &mut self,
        entity_key: EntityKey,
        f: impl Fn(&mut field::Entity),
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

    #[inline]
    pub fn entity_get_chunk(&self, chunk_key: IVec2) -> Result<&field::EntityChunk, FieldError> {
        self.entity_field.get_chunk(chunk_key)
    }

    // entity collision features

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

    // resource

    #[inline]
    pub fn resource_insert<T: 'static>(&mut self, value: T) -> Option<()> {
        self.resource_store.insert(value)
    }

    #[inline]
    pub fn resource_remove<T: 'static>(&mut self) -> Option<T> {
        self.resource_store.remove::<T>()
    }

    #[inline]
    pub fn resource_has<T: 'static>(&self) -> bool {
        self.resource_store.has::<T>()
    }

    #[inline]
    pub fn resource_get<T: 'static>(&self) -> Option<&T> {
        self.resource_store.get::<T>()
    }

    #[inline]
    pub fn resource_get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.resource_store.get_mut::<T>()
    }

    // tag

    #[inline]
    pub fn tag_insert<T: 'static>(
        &mut self,
        r#ref: RefKey,
        spc: SpaceKey,
        tag: T,
    ) -> Option<TagKey> {
        self.tag_store.insert(r#ref, spc, tag)
    }

    #[inline]
    pub fn tag_remove<T: 'static>(&mut self, tag_key: TagKey) -> Option<(RefKey, SpaceKey, T)> {
        self.tag_store.remove::<T>(tag_key)
    }

    #[inline]
    pub fn tag_modify<T: 'static>(
        &mut self,
        tag_key: TagKey,
        f: impl FnOnce(&mut RefKey, &mut SpaceKey, &mut T),
    ) -> Option<()> {
        self.tag_store.modify::<T>(tag_key, f)
    }

    #[inline]
    pub fn tag_remove_by_ref(&mut self, r#ref: RefKey) -> Option<()> {
        self.tag_store.remove_by_ref(r#ref)
    }

    #[inline]
    pub fn tag_modify_ref_by_ref(&mut self, r#ref: RefKey, new_ref: RefKey) -> Option<()> {
        self.tag_store.modify_ref_by_ref(r#ref, new_ref)
    }

    #[inline]
    pub fn tag_modify_spc_by_ref(&mut self, r#ref: RefKey, new_spc: SpaceKey) -> Option<()> {
        self.tag_store.modify_spc_by_ref(r#ref, new_spc)
    }

    #[inline]
    pub fn tag_get<T: 'static>(&self, tag_key: TagKey) -> Option<(&RefKey, &SpaceKey, &T)> {
        self.tag_store.get::<T>(tag_key)
    }

    #[inline]
    pub fn tag_iter<T: 'static>(&self) -> impl Iterator<Item = &TagKey> {
        self.tag_store.iter::<T>()
    }

    #[inline]
    pub fn tag_detach_iter<T: 'static>(&self) -> Vec<TagKey> {
        self.tag_store.detach_iter::<T>()
    }

    #[inline]
    pub fn tag_one_by_ref<T: 'static>(&self, r#ref: RefKey) -> Option<&TagKey> {
        self.tag_store.one_by_ref::<T>(r#ref)
    }

    #[inline]
    pub fn tag_iter_by_rect<T: 'static>(
        &self,
        rect: [SpaceKey; 2],
    ) -> impl Iterator<Item = &TagKey> {
        self.tag_store.iter_by_rect::<T>(rect)
    }

    #[inline]
    pub fn tag_detach_iter_by_rect<T: 'static>(&self, rect: [SpaceKey; 2]) -> Vec<TagKey> {
        self.tag_store.detach_iter_by_rect::<T>(rect)
    }

    // flow

    #[inline]
    pub fn flow_iter<T: 'static>(&self) -> impl Iterator<Item = &T> {
        self.flow_store.iter::<T>()
    }

    #[inline]
    pub fn flow_one<T: 'static>(&self) -> Option<&T> {
        self.flow_store.one::<T>()
    }

    #[inline]
    pub fn flow_iter_by_ref<T: 'static>(&self, r#ref: FlowRef) -> impl Iterator<Item = &T> {
        self.flow_store.iter_by_ref::<T>(r#ref)
    }

    #[inline]
    pub fn flow_one_by_ref<T: 'static>(&self, r#ref: FlowRef) -> Option<&T> {
        self.flow_store.one_by_ref::<T>(r#ref)
    }
}

// Error Handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldError {
    NotFound,
    Conflict,
    InvalidId,
}

impl std::fmt::Display for FieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldError::NotFound => write!(f, "not found error"),
            FieldError::Conflict => write!(f, "conflict error"),
            FieldError::InvalidId => write!(f, "invalid id error"),
        }
    }
}

impl std::error::Error for FieldError {}
