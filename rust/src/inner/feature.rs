use super::*;

// tile data/feature

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TileRenderParam {
    pub variant: Option<u8>,
    pub tick: Option<u32>,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum TileData {}

#[enum_dispatch::enum_dispatch]
pub trait TileFeatureTrait {
    fn after_place(&self, root: &mut Root, key: TileKey);
    fn before_break(&self, root: &mut Root, key: TileKey);
    fn forward(&self, root: &mut Root, key: TileKey, delta_secs: f32);

    fn get_inventory(&self, root: &Root, key: TileKey) -> Option<InventoryKey>;
}

#[enum_dispatch::enum_dispatch(TileFeatureTrait)]
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum TileFeature {
    Empty(EmptyTileFeature),
}

#[derive(Debug, Clone)]
pub struct EmptyTileFeature;

impl TileFeatureTrait for EmptyTileFeature {
    fn after_place(&self, _root: &mut Root, _key: TileKey) {}
    fn before_break(&self, _root: &mut Root, _key: TileKey) {}
    fn forward(&self, _root: &mut Root, _key: TileKey, _delta_secs: f32) {}

    fn get_inventory(&self, _root: &Root, _key: TileKey) -> Option<InventoryKey> {
        None
    }
}

// block data/feature

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlockRenderParam {
    pub variant: Option<u8>,
    pub tick: Option<u32>,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum BlockData {}

#[enum_dispatch::enum_dispatch]
pub trait BlockFeatureTrait {
    fn after_place(&self, root: &mut Root, key: BlockKey);
    fn before_break(&self, root: &mut Root, key: BlockKey);
    fn forward(&self, root: &mut Root, key: BlockKey, delta_secs: f32);

    fn get_inventory(&self, root: &Root, key: BlockKey) -> Option<InventoryKey>;
}

#[enum_dispatch::enum_dispatch(BlockFeatureTrait)]
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum BlockFeature {
    Empty(EmptyBlockFeature),
}

#[derive(Debug, Clone)]
pub struct EmptyBlockFeature;

impl BlockFeatureTrait for EmptyBlockFeature {
    fn after_place(&self, _root: &mut Root, _key: BlockKey) {}
    fn before_break(&self, _root: &mut Root, _key: BlockKey) {}
    fn forward(&self, _root: &mut Root, _key: BlockKey, _delta_secs: f32) {}

    fn get_inventory(&self, _root: &Root, _key: BlockKey) -> Option<InventoryKey> {
        None
    }
}

// entity data/feature

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntityRenderParam {
    pub variant: Option<u8>,
    pub tick: Option<u32>,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum EntityData {
    Animal(AnimalEntityData),
    Player(PlayerEntityData),
}

#[enum_dispatch::enum_dispatch]
pub trait EntityFeatureTrait {
    fn after_place(&self, root: &mut Root, key: TileKey);
    fn before_break(&self, root: &mut Root, key: TileKey);
    fn forward(&self, root: &mut Root, key: TileKey, delta_secs: f32);

    fn get_inventory(&self, root: &Root, key: TileKey) -> Option<InventoryKey>;
}

#[enum_dispatch::enum_dispatch(EntityFeatureTrait)]
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum EntityFeature {
    Empty(EmptyEntityFeature),
    Animal(AnimalEntityFeature),
    Player(PlayerEntityFeature),
}

#[derive(Debug, Clone)]
pub struct EmptyEntityFeature;

impl EntityFeatureTrait for EmptyEntityFeature {
    fn after_place(&self, _root: &mut Root, _key: EntityKey) {}
    fn before_break(&self, _root: &mut Root, _key: EntityKey) {}
    fn forward(&self, _root: &mut Root, _key: EntityKey, _delta_secs: f32) {}

    fn get_inventory(&self, _root: &Root, _key: EntityKey) -> Option<InventoryKey> {
        None
    }
}

// item data/feature

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemRenderParam {
    pub variant: Option<u8>,
    pub tick: Option<u32>,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ItemData {}

#[enum_dispatch::enum_dispatch]
pub trait ItemFeatureTrait {
    fn after_pick(&self, _root: &mut Root, _key: u32) {}
    fn before_drop(&self, _root: &mut Root, _key: u32) {}
    fn forward(&self, _root: &mut Root, _key: u32) {}

    fn r#use(&self, _root: &mut Root, _key: u32) {}
}

#[enum_dispatch::enum_dispatch(ItemFeatureTrait)]
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ItemFeature {
    Empty(EmptyItemFeature),
}

#[derive(Debug, Clone)]
pub struct EmptyItemFeature;

impl ItemFeatureTrait for EmptyItemFeature {
    fn after_pick(&self, _root: &mut Root, _key: u32) {}
    fn before_drop(&self, _root: &mut Root, _key: u32) {}
    fn forward(&self, _root: &mut Root, _key: u32) {}

    fn r#use(&self, _root: &mut Root, _key: u32) {}
}
