use super::*;

// tile render param

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TileRenderParam {
    pub variant: u8,
    pub tick: u32,
    pub override_color: u32,
}

// tile data

pub trait TileData: dyn_clone::DynClone + downcast_rs::Downcast + std::fmt::Debug {}

dyn_clone::clone_trait_object!(TileData);

downcast_rs::impl_downcast!(TileData);

impl TileData for () {}

impl Default for Box<dyn TileData> {
    fn default() -> Self {
        // Dangling pointer
        Box::new(())
    }
}

// tile feature

pub trait TileFeature: std::fmt::Debug {
    /// Invoked after place tile with no extra args.
    /// If you want to invoke with extra args, you can modify data after place.
    ///
    /// # Panic
    ///
    /// Panic if tile is not found or mismatch id.
    fn after_place(&self, _dataflow: &mut Dataflow, _key: TileKey) {}

    /// Invoked before break tile with no extra args.
    /// If you want to invoke with extra args, you can modify data before break.
    ///
    /// # Panic
    ///
    /// panic if tile is not found or mismatch id.
    fn before_break(&self, _dataflow: &mut Dataflow, _key: TileKey) {}

    /// Invoked every frame.
    ///
    /// # Panic
    ///
    /// panic if tile is not found or mismatch id.
    fn forward(&self, _dataflow: &mut Dataflow, _key: TileKey, _delta_secs: f32) {}

    /// Check if tile has inventory.
    ///
    /// # Panic
    ///
    /// panic if tile is not found or mismatch id.
    fn has_inventory(&self, _dataflow: &Dataflow, _key: TileKey) -> bool {
        false
    }

    /// Get inventory key.
    ///
    /// # Panic
    ///
    /// panic if tile is not found or mismatch id.
    fn get_inventory(&self, _dataflow: &Dataflow, _key: TileKey) -> Option<InventoryKey> {
        None
    }
}

impl TileFeature for () {}

impl Default for Box<dyn TileFeature> {
    fn default() -> Self {
        // Dangling pointer
        Box::new(())
    }
}

// block render param

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlockRenderParam {
    pub variant: u8,
    pub tick: u32,
    pub override_color: u32,
}

// block data

pub trait BlockData: dyn_clone::DynClone + downcast_rs::Downcast + std::fmt::Debug {}

dyn_clone::clone_trait_object!(BlockData);

downcast_rs::impl_downcast!(BlockData);

impl BlockData for () {}

impl Default for Box<dyn BlockData> {
    fn default() -> Self {
        // Dangling pointer
        Box::new(())
    }
}

// block feature

pub trait BlockFeature: std::fmt::Debug {
    /// Invoked after place block with no extra args.
    /// If you want to invoke with extra args, you can modify data after place.
    ///
    /// # Panic
    ///
    /// Panic if block is not found or mismatch id.
    fn after_place(&self, _dataflow: &mut Dataflow, _key: BlockKey) {}

    /// Invoked before break block with no extra args.
    /// If you want to invoke with extra args, you can modify data before break.
    ///
    /// # Panic
    ///
    /// panic if block is not found or mismatch id.
    fn before_break(&self, _dataflow: &mut Dataflow, _key: BlockKey) {}

    /// Invoked every frame.
    ///
    /// # Panic
    ///
    /// panic if block is not found or mismatch id.
    fn forward(&self, _dataflow: &mut Dataflow, _key: BlockKey, _delta_secs: f32) {}

    /// Check if block has inventory.
    ///
    /// # Panic
    ///
    /// panic if block is not found or mismatch id.
    fn has_inventory(&self, _dataflow: &Dataflow, _key: BlockKey) -> bool {
        false
    }

    /// Get inventory key.
    ///
    /// # Panic
    ///
    /// panic if block is not found or mismatch id.
    fn get_inventory(&self, _dataflow: &Dataflow, _key: BlockKey) -> Option<InventoryKey> {
        None
    }
}

impl BlockFeature for () {}

impl Default for Box<dyn BlockFeature> {
    fn default() -> Self {
        // Dangling pointer
        Box::new(())
    }
}

// entity render param

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntityRenderParam {
    pub variant: u8,
    pub tick: u32,
    pub override_color: u32,
}

// entity data

pub trait EntityData: dyn_clone::DynClone + downcast_rs::Downcast + std::fmt::Debug {}

dyn_clone::clone_trait_object!(EntityData);

downcast_rs::impl_downcast!(EntityData);

impl EntityData for () {}

impl Default for Box<dyn EntityData> {
    fn default() -> Self {
        // Dangling pointer
        Box::new(())
    }
}

// entity feature

pub trait EntityFeature: std::fmt::Debug {
    /// Invoked after place entity with no extra args.
    /// If you want to invoke with extra args, you can modify data after place.
    ///
    /// # Panic
    ///
    /// Panic if entity is not found or mismatch id.
    fn after_place(&self, _dataflow: &mut Dataflow, _key: EntityKey) {}

    /// Invoked before break entity with no extra args.
    /// If you want to invoke with extra args, you can modify data before break.
    ///
    /// # Panic
    ///
    /// panic if entity is not found or mismatch id.
    fn before_break(&self, _dataflow: &mut Dataflow, _key: EntityKey) {}

    /// Invoked every frame.
    ///
    /// # Panic
    ///
    /// panic if entity is not found or mismatch id.
    fn forward(&self, _dataflow: &mut Dataflow, _key: EntityKey, _delta_secs: f32) {}

    /// Check if entity has inventory.
    ///
    /// # Panic
    ///
    /// panic if entity is not found or mismatch id.
    fn has_inventory(&self, _dataflow: &Dataflow, _key: EntityKey) -> bool {
        false
    }

    /// Get inventory key.
    ///
    /// # Panic
    ///
    /// panic if entity is not found or mismatch id.
    fn get_inventory(&self, _dataflow: &Dataflow, _key: EntityKey) -> Option<InventoryKey> {
        None
    }

    /// Check if can pick up entity.
    ///
    /// # Panic
    ///
    /// panic if entity is not found or mismatch id.
    fn can_pick_up(
        &self,
        _dataflow: &Dataflow,
        _key: EntityKey,
        _inventory_key: InventoryKey,
    ) -> bool {
        false
    }

    /// Pick up entity.
    ///
    /// # Panic
    ///
    /// panic if entity is not found or mismatch id.
    fn pick_up(&self, _dataflow: &mut Dataflow, _key: EntityKey, _inventory_key: InventoryKey) {}
}

impl EntityFeature for () {}

impl Default for Box<dyn EntityFeature> {
    fn default() -> Self {
        // Dangling pointer
        Box::new(())
    }
}

// item render param

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemRenderParam {
    pub variant: u8,
    pub tick: u32,
}

// item data

pub trait ItemData: dyn_clone::DynClone + downcast_rs::Downcast + std::fmt::Debug {}

dyn_clone::clone_trait_object!(ItemData);

downcast_rs::impl_downcast!(ItemData);

impl ItemData for () {}

impl Default for Box<dyn ItemData> {
    fn default() -> Self {
        // Dangling pointer
        Box::new(())
    }
}

// item feature

pub trait ItemFeature: std::fmt::Debug {
    fn after_pick(&self, _dataflow: &mut Dataflow, _key: SlotKey) {}

    fn before_drop(&self, _dataflow: &mut Dataflow, _key: SlotKey) {}

    fn forward(&self, _dataflow: &mut Dataflow, _key: SlotKey) {}

    fn r#use(&self, _dataflow: &mut Dataflow, _key: SlotKey) {}
}

impl ItemFeature for () {}

impl Default for Box<dyn ItemFeature> {
    fn default() -> Self {
        // Dangling pointer
        Box::new(())
    }
}
