use super::*;

// tile data/feature

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TileRenderParam {
    pub variant: u8,
    pub tick: u32,
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub enum TileData {
    #[default]
    Empty,
}

#[enum_dispatch::enum_dispatch]
pub trait TileFeatureTrait {
    fn after_place(&self, _root: &mut Root, _key: TileKey) -> Result<(), RootError> {
        Ok(())
    }

    fn before_break(&self, _root: &mut Root, _key: TileKey) -> Result<(), RootError> {
        Ok(())
    }

    fn forward(&self, _root: &mut Root, _key: TileKey, _delta_secs: f32) -> Result<(), RootError> {
        Ok(())
    }

    fn has_inventory(&self, _root: &Root, _key: TileKey) -> Result<bool, RootError> {
        Ok(false)
    }
    fn get_inventory(
        &self,
        _root: &Root,
        _key: TileKey,
    ) -> Result<Option<InventoryKey>, RootError> {
        Ok(None)
    }
}

#[enum_dispatch::enum_dispatch(TileFeatureTrait)]
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum TileFeature {
    Empty(EmptyTileFeature),
}

#[derive(Debug, Clone)]
pub struct EmptyTileFeature;

impl TileFeatureTrait for EmptyTileFeature {}

// block data/feature

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlockRenderParam {
    pub variant: u8,
    pub tick: u32,
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub enum BlockData {
    #[default]
    Empty,
}

#[enum_dispatch::enum_dispatch]
pub trait BlockFeatureTrait {
    fn after_place(&self, _root: &mut Root, _key: BlockKey) -> Result<(), RootError> {
        Ok(())
    }

    fn before_break(&self, _root: &mut Root, _key: BlockKey) -> Result<(), RootError> {
        Ok(())
    }

    fn forward(&self, _root: &mut Root, _key: BlockKey, _delta_secs: f32) -> Result<(), RootError> {
        Ok(())
    }

    fn has_inventory(&self, _root: &Root, _key: BlockKey) -> Result<bool, RootError> {
        Ok(false)
    }

    fn get_inventory(
        &self,
        _root: &Root,
        _key: BlockKey,
    ) -> Result<Option<InventoryKey>, RootError> {
        Ok(None)
    }
}

#[enum_dispatch::enum_dispatch(BlockFeatureTrait)]
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum BlockFeature {
    Empty(EmptyBlockFeature),
}

#[derive(Debug, Clone)]
pub struct EmptyBlockFeature;

impl BlockFeatureTrait for EmptyBlockFeature {}

// entity data/feature

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntityRenderParam {
    pub variant: u8,
    pub tick: u32,
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub enum EntityData {
    #[default]
    Empty,
    Animal(AnimalEntityData),
    Player(PlayerEntityData),
    Item(ItemEntityData),
}

#[enum_dispatch::enum_dispatch]
pub trait EntityFeatureTrait {
    fn after_place(&self, _root: &mut Root, _key: EntityKey) -> Result<(), RootError> {
        Ok(())
    }

    fn before_break(&self, _root: &mut Root, _key: EntityKey) -> Result<(), RootError> {
        Ok(())
    }

    fn forward(
        &self,
        _root: &mut Root,
        _key: EntityKey,
        _delta_secs: f32,
    ) -> Result<(), RootError> {
        Ok(())
    }

    fn has_inventory(&self, _root: &Root, _key: EntityKey) -> Result<bool, RootError> {
        Ok(false)
    }

    fn get_inventory(
        &self,
        _root: &Root,
        _key: EntityKey,
    ) -> Result<Option<InventoryKey>, RootError> {
        Ok(None)
    }

    fn can_pick_up(&self, _root: &Root, _inventory_key: InventoryKey) -> Result<bool, RootError> {
        Ok(false)
    }

    fn pick_up(
        &self,
        _root: &mut Root,
        _key: EntityKey,
        _inventory_key: InventoryKey,
    ) -> Result<(), RootError> {
        Ok(())
    }
}

#[enum_dispatch::enum_dispatch(EntityFeatureTrait)]
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum EntityFeature {
    Empty(EmptyEntityFeature),
    Animal(AnimalEntityFeature),
    Player(PlayerEntityFeature),
    Item(ItemEntityFeature),
}

#[derive(Debug, Clone)]
pub struct EmptyEntityFeature;

impl EntityFeatureTrait for EmptyEntityFeature {}

// item data/feature

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemRenderParam {
    pub variant: u8,
    pub tick: u32,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ItemData {
    Empty,
}

#[enum_dispatch::enum_dispatch]
pub trait ItemFeatureTrait {
    fn after_pick(&self, _root: &mut Root, _key: SlotKey) -> Result<(), RootError> {
        Ok(())
    }

    fn before_drop(&self, _root: &mut Root, _key: SlotKey) -> Result<(), RootError> {
        Ok(())
    }

    fn forward(&self, _root: &mut Root, _key: SlotKey) -> Result<(), RootError> {
        Ok(())
    }

    fn can_use(&self) -> Result<bool, RootError> {
        Ok(false)
    }

    fn r#use(&self, _root: &mut Root, _key: SlotKey) -> Result<(), RootError> {
        Ok(())
    }
}

#[enum_dispatch::enum_dispatch(ItemFeatureTrait)]
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ItemFeature {
    Empty(EmptyItemFeature),
}

#[derive(Debug, Clone)]
pub struct EmptyItemFeature;

impl ItemFeatureTrait for EmptyItemFeature {}
