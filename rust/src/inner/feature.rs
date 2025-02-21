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
    /// Invoked after place tile with no extra args.
    /// If you want to invoke with extra args, you can modify data after place.
    ///
    /// # Panic
    ///
    /// Panic if tile is not found or mismatch id.
    fn after_place(&self, _root: &mut Root, _key: TileKey) {}

    /// Invoked before break tile with no extra args.
    /// If you want to invoke with extra args, you can modify data before break.
    ///
    /// # Panic
    ///
    /// panic if tile is not found or mismatch id.
    fn before_break(&self, _root: &mut Root, _key: TileKey) {}

    /// Invoked every frame.
    ///
    /// # Panic
    ///
    /// panic if tile is not found or mismatch id.
    fn forward(&self, _root: &mut Root, _key: TileKey, _delta_secs: f32) {}

    /// Check if tile has inventory.
    ///
    /// # Panic
    ///
    /// panic if tile is not found or mismatch id.
    fn has_inventory(&self, _root: &Root, _key: TileKey) -> bool {
        false
    }

    /// Get inventory key.
    ///
    /// # Panic
    ///
    /// panic if tile is not found or mismatch id.
    fn get_inventory(&self, _root: &Root, _key: TileKey) -> Option<InventoryKey> {
        None
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
    /// Invoked after place block with no extra args.
    /// If you want to invoke with extra args, you can modify data after place.
    ///
    /// # Panic
    ///
    /// Panic if block is not found or mismatch id.
    fn after_place(&self, _root: &mut Root, _key: BlockKey) {}

    /// Invoked before break block with no extra args.
    /// If you want to invoke with extra args, you can modify data before break.
    ///
    /// # Panic
    ///
    /// panic if block is not found or mismatch id.
    fn before_break(&self, _root: &mut Root, _key: BlockKey) {}

    /// Invoked every frame.
    ///
    /// # Panic
    ///
    /// panic if block is not found or mismatch id.
    fn forward(&self, _root: &mut Root, _key: BlockKey, _delta_secs: f32) {}

    /// Check if block has inventory.
    ///
    /// # Panic
    ///
    /// panic if block is not found or mismatch id.
    fn has_inventory(&self, _root: &Root, _key: BlockKey) -> bool {
        false
    }

    /// Get inventory key.
    ///
    /// # Panic
    ///
    /// panic if block is not found or mismatch id.
    fn get_inventory(&self, _root: &Root, _key: BlockKey) -> Option<InventoryKey> {
        None
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
    /// Invoked after place entity with no extra args.
    /// If you want to invoke with extra args, you can modify data after place.
    ///
    /// # Panic
    ///
    /// Panic if entity is not found or mismatch id.
    fn after_place(&self, _root: &mut Root, _key: EntityKey) {}

    /// Invoked before break entity with no extra args.
    /// If you want to invoke with extra args, you can modify data before break.
    ///
    /// # Panic
    ///
    /// panic if entity is not found or mismatch id.
    fn before_break(&self, _root: &mut Root, _key: EntityKey) {}

    /// Invoked every frame.
    ///
    /// # Panic
    ///
    /// panic if entity is not found or mismatch id.
    fn forward(&self, _root: &mut Root, _key: EntityKey, _delta_secs: f32) {}

    /// Check if entity has inventory.
    ///
    /// # Panic
    ///
    /// panic if entity is not found or mismatch id.
    fn has_inventory(&self, _root: &Root, _key: EntityKey) -> bool {
        false
    }

    /// Get inventory key.
    ///
    /// # Panic
    ///
    /// panic if entity is not found or mismatch id.
    fn get_inventory(&self, _root: &Root, _key: EntityKey) -> Option<InventoryKey> {
        None
    }

    /// Check if can pick up entity.
    ///
    /// # Panic
    ///
    /// panic if entity is not found or mismatch id.
    fn can_pick_up(&self, _root: &Root, _key: EntityKey, _inventory_key: InventoryKey) -> bool {
        false
    }

    /// Pick up entity.
    ///
    /// # Panic
    ///
    /// panic if entity is not found or mismatch id.
    fn pick_up(&self, _root: &mut Root, _key: EntityKey, _inventory_key: InventoryKey) {}
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
    fn after_pick(&self, _root: &mut Root, _key: SlotKey) {}

    fn before_drop(&self, _root: &mut Root, _key: SlotKey) {}

    fn forward(&self, _root: &mut Root, _key: SlotKey) {}

    fn r#use(&self, _root: &mut Root, _key: SlotKey) {}
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
