use crate::inner;

#[derive(Clone)]
pub struct Feature;

impl inner::Feature for Feature {
    type Tile = TileFeature;
    type Block = BlockFeature;
    type Entity = EntityFeature;
}

#[derive(Clone)]
pub struct TileFeature;

impl inner::TileFeature<Feature> for TileFeature {
    type Item = ();

    fn after_place(&self, _root: &mut inner::Root<Feature>, _key: inner::TileKey) {}
    fn before_break(&self, _root: &mut inner::Root<Feature>, _key: inner::TileKey) {}
    fn forward(&self, _root: &mut inner::Root<Feature>, _key: inner::TileKey) {}
}

#[derive(Clone)]
pub struct BlockFeature;

impl inner::BlockFeature<Feature> for BlockFeature {
    type Item = ();

    fn after_place(&self, _root: &mut inner::Root<Feature>, _key: inner::TileKey) {}
    fn before_break(&self, _root: &mut inner::Root<Feature>, _key: inner::TileKey) {}
    fn forward(&self, _root: &mut inner::Root<Feature>, _key: inner::TileKey) {}
}

#[derive(Clone)]
pub struct EntityFeature;

impl inner::EntityFeature<Feature> for EntityFeature {
    type Item = ();

    fn after_place(&self, _root: &mut inner::Root<Feature>, _key: inner::TileKey) {}
    fn before_break(&self, _root: &mut inner::Root<Feature>, _key: inner::TileKey) {}
    fn forward(&self, _root: &mut inner::Root<Feature>, _key: inner::TileKey) {}
}
