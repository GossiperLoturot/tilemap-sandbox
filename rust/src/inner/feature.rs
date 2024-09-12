use super::*;

#[derive(Debug, Clone)]
pub struct TileFeatureEmpty;

impl TileFeatureTrait for TileFeatureEmpty {
    fn after_place(&self, _root: &mut Root, _key: TileKey) {}

    fn before_break(&self, _root: &mut Root, _key: TileKey) {}

    fn forward(&self, _root: &mut Root, _key: TileKey, _delta_secs: f32) {}
}

#[derive(Debug, Clone)]
pub struct BlockFeatureEmpty;

impl BlockFeatureTrait for BlockFeatureEmpty {
    fn after_place(&self, _root: &mut Root, _key: BlockKey) {}

    fn before_break(&self, _root: &mut Root, _key: BlockKey) {}

    fn forward(&self, _root: &mut Root, _key: BlockKey, _delta_secs: f32) {}
}

#[derive(Debug, Clone)]
pub struct EntityFeatureEmpty;

impl EntityFeatureTrait for EntityFeatureEmpty {
    fn after_place(&self, _root: &mut Root, _key: EntityKey) {}

    fn before_break(&self, _root: &mut Root, _key: EntityKey) {}

    fn forward(&self, _root: &mut Root, _key: EntityKey, _delta_secs: f32) {}
}
