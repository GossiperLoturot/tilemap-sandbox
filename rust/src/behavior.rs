use crate::{block, entity, inner, tile};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct BehaviorFactory {
    inner: inner::BehaviorFactory,
}

#[godot_api]
impl BehaviorFactory {
    #[func]
    fn new_unit() -> Gd<Self> {
        let inner = inner::BehaviorFactory::Unit;
        Gd::from_init_fn(|_| Self { inner })
    }

    #[func]
    fn new_random_walk(
        min_rest_secs: f32,
        max_rest_secs: f32,
        min_distance: f32,
        max_distance: f32,
        speed: f32,
    ) -> Gd<Self> {
        let inner = inner::BehaviorFactory::RandomWalk(inner::RandomWalkFactory::new(
            min_rest_secs,
            max_rest_secs,
            min_distance,
            max_distance,
            speed,
        ));
        Gd::from_init_fn(|_| Self { inner })
    }

    #[func]
    fn new_generator() -> Gd<Self> {
        let inner = inner::BehaviorFactory::Generator(inner::GeneratorFactory::new());
        Gd::from_init_fn(|_| Self { inner })
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct BehaviorPluginDesc {
    #[export]
    tile_factories: Array<Gd<BehaviorFactory>>,
    #[export]
    block_factories: Array<Gd<BehaviorFactory>>,
    #[export]
    entity_factories: Array<Gd<BehaviorFactory>>,
}

#[godot_api]
impl BehaviorPluginDesc {
    #[func]
    fn new_from(
        tile_factories: Array<Gd<BehaviorFactory>>,
        block_factories: Array<Gd<BehaviorFactory>>,
        entity_factories: Array<Gd<BehaviorFactory>>,
    ) -> Gd<Self> {
        Gd::from_init_fn(|_| Self {
            tile_factories,
            block_factories,
            entity_factories,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct BehaviorPlugin {
    inner: inner::BehaviorPlugin,
    tile_field: Gd<tile::TileField>,
    block_field: Gd<block::BlockField>,
    entity_field: Gd<entity::EntityField>,
}

#[godot_api]
impl BehaviorPlugin {
    #[func]
    fn new_from(
        desc: Gd<BehaviorPluginDesc>,
        tile_field: Gd<tile::TileField>,
        block_field: Gd<block::BlockField>,
        entity_field: Gd<entity::EntityField>,
    ) -> Gd<Self> {
        let desc = desc.bind();
        let tile_factories = desc
            .tile_factories
            .iter_shared()
            .map(|factory| factory.bind().inner.clone())
            .collect::<Vec<_>>();
        let block_factories = desc
            .block_factories
            .iter_shared()
            .map(|factory| factory.bind().inner.clone())
            .collect::<Vec<_>>();
        let entity_factories = desc
            .entity_factories
            .iter_shared()
            .map(|factory| factory.bind().inner.clone())
            .collect::<Vec<_>>();

        Gd::from_init_fn(|_| Self {
            inner: inner::BehaviorPlugin::new(tile_factories, block_factories, entity_factories),
            tile_field,
            block_field,
            entity_field,
        })
    }

    #[func]
    fn place_tile(&mut self, tile: Gd<tile::Tile>) -> Option<Gd<tile::TileKey>> {
        let tile_field = &mut self.tile_field.bind_mut().inner;
        let tile = tile.bind().inner.clone();
        let key = self.inner.place_tile(tile_field, tile).ok()?;
        Some(Gd::from_init_fn(|_| tile::TileKey { inner: key }))
    }

    #[func]
    fn break_tile(&mut self, key: Gd<tile::TileKey>) -> Option<Gd<tile::Tile>> {
        let tile_field = &mut self.tile_field.bind_mut().inner;
        let key = key.bind().inner;
        let tile = self.inner.break_tile(tile_field, key).ok()?;
        Some(Gd::from_init_fn(|_| tile::Tile { inner: tile }))
    }

    #[func]
    fn place_block(&mut self, block: Gd<block::Block>) -> Option<Gd<block::BlockKey>> {
        let block_field = &mut self.block_field.bind_mut().inner;
        let block = block.bind().inner.clone();
        let key = self.inner.place_block(block_field, block).ok()?;
        Some(Gd::from_init_fn(|_| block::BlockKey { inner: key }))
    }

    #[func]
    fn break_block(&mut self, key: Gd<block::BlockKey>) -> Option<Gd<block::Block>> {
        let block_field = &mut self.block_field.bind_mut().inner;
        let key = key.bind().inner;
        let block = self.inner.break_block(block_field, key).ok()?;
        Some(Gd::from_init_fn(|_| block::Block { inner: block }))
    }

    #[func]
    fn place_entity(&mut self, entity: Gd<entity::Entity>) -> Option<Gd<entity::EntityKey>> {
        let entity_field = &mut self.entity_field.bind_mut().inner;
        let entity = entity.bind().inner.clone();
        let key = self.inner.place_entity(entity_field, entity).ok()?;
        Some(Gd::from_init_fn(|_| entity::EntityKey { inner: key }))
    }

    #[func]
    fn break_entity(&mut self, key: Gd<entity::EntityKey>) -> Option<Gd<entity::Entity>> {
        let entity_field = &mut self.entity_field.bind_mut().inner;
        let key = key.bind().inner;
        let entity = self.inner.break_entity(entity_field, key).ok()?;
        Some(Gd::from_init_fn(|_| entity::Entity { inner: entity }))
    }

    #[func]
    fn update(&mut self, delta_secs: f32) {
        let tile_field = &mut self.tile_field.bind_mut().inner;
        let block_field = &mut self.block_field.bind_mut().inner;
        let entity_field = &mut self.entity_field.bind_mut().inner;
        self.inner
            .update(tile_field, block_field, entity_field, delta_secs);
    }
}
