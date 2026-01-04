use super::*;

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
