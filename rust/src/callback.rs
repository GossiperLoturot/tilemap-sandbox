use crate::{block, entity, extra, inner, tile};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct CallbackBundle {
    pub r#ref: inner::CallbackRef,
    pub bundle: Box<dyn inner::CallbackBundle>,
}

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct CallbackStoreBuilder {
    inner: inner::CallbackStoreBuilder,
}

#[godot_api]
impl CallbackStoreBuilder {
    #[func]
    fn insert_bundle(&mut self, bundle: Gd<CallbackBundle>) {
        let r#ref = bundle.bind().r#ref;
        let bundle = &bundle.bind().bundle;
        bundle.insert(r#ref, &mut self.inner);
    }

    #[func]
    fn build(&mut self) -> Gd<CallbackStore> {
        let builder = std::mem::take(&mut self.inner);
        let store = builder.build();
        Gd::from_init_fn(|_| CallbackStore { inner: store })
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct CallbackStore {
    pub inner: inner::CallbackStore,
}

// extra

// static class
#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct Action;

#[godot_api]
impl Action {
    #[func]
    fn before(mut world: Gd<crate::World>) {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        extra::before(&mut world.inner());
    }

    #[func]
    fn after(mut world: Gd<crate::World>) {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        extra::after(&mut world.inner());
    }

    #[func]
    fn forward(mut world: Gd<crate::World>, delta_secs: f32) {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        extra::forward(&mut world.inner(), delta_secs);
    }

    #[func]
    fn place_tile(mut world: Gd<crate::World>, tile: Gd<tile::Tile>) -> Option<Gd<tile::TileKey>> {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let tile = tile.bind().inner.clone();
        let key = extra::place_tile(&mut world.inner(), tile).ok()?;
        Some(Gd::from_init_fn(|_| tile::TileKey { inner: key }))
    }

    #[func]
    fn break_tile(
        mut world: Gd<crate::World>,
        tile_key: Gd<tile::TileKey>,
    ) -> Option<Gd<tile::Tile>> {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let key = tile_key.bind().inner;
        let tile = extra::break_tile(&mut world.inner(), key).ok()?;
        Some(Gd::from_init_fn(|_| tile::Tile { inner: tile }))
    }

    #[func]
    fn place_block(
        mut world: Gd<crate::World>,
        block: Gd<block::Block>,
    ) -> Option<Gd<block::BlockKey>> {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let block = block.bind().inner.clone();
        let key = extra::place_block(&mut world.inner(), block).ok()?;
        Some(Gd::from_init_fn(|_| block::BlockKey { inner: key }))
    }

    #[func]
    fn break_block(
        mut world: Gd<crate::World>,
        block_key: Gd<block::BlockKey>,
    ) -> Option<Gd<block::Block>> {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let key = block_key.bind().inner;
        let block = extra::break_block(&mut world.inner(), key).ok()?;
        Some(Gd::from_init_fn(|_| block::Block { inner: block }))
    }

    #[func]
    fn place_entity(
        mut world: Gd<crate::World>,
        entity: Gd<entity::Entity>,
    ) -> Option<Gd<entity::EntityKey>> {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let entity = entity.bind().inner.clone();
        let key = extra::place_entity(&mut world.inner(), entity).ok()?;
        Some(Gd::from_init_fn(|_| entity::EntityKey { inner: key }))
    }

    #[func]
    fn break_entity(
        mut world: Gd<crate::World>,
        entity_key: Gd<entity::EntityKey>,
    ) -> Option<Gd<entity::Entity>> {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let key = entity_key.bind().inner;
        let entity = extra::break_entity(&mut world.inner(), key).ok()?;
        Some(Gd::from_init_fn(|_| entity::Entity { inner: entity }))
    }

    #[func]
    fn generate_chunk(mut world: Gd<crate::World>, chunk_key: Vector2i) {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let chunk_key = [chunk_key.x, chunk_key.y];
        extra::generate_chunk(&mut world.inner(), chunk_key);
    }
}

// static class
#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct Callback;

#[godot_api]
impl Callback {
    #[func]
    fn new_generator(chunk_size: u32, size: u32) -> Gd<CallbackBundle> {
        let r#ref = inner::CallbackRef::Global;
        let bundle = Box::new(extra::Generator { chunk_size, size });
        Gd::from_init_fn(|_| CallbackBundle { r#ref, bundle })
    }

    #[func]
    fn new_random_walk(
        entity_key: u32,
        min_rest_secs: f32,
        max_rest_secs: f32,
        min_distance: f32,
        max_distance: f32,
        speed: f32,
    ) -> Gd<CallbackBundle> {
        let r#ref = inner::CallbackRef::Entity(entity_key);
        let bundle = Box::new(extra::RandomWalk {
            min_rest_secs,
            max_rest_secs,
            min_distance,
            max_distance,
            speed,
        });
        Gd::from_init_fn(|_| CallbackBundle { r#ref, bundle })
    }

    #[func]
    fn new_random_walk_forward() -> Gd<CallbackBundle> {
        let r#ref = inner::CallbackRef::Global;
        let bundle = Box::new(extra::RandomWalkForward);
        Gd::from_init_fn(|_| CallbackBundle { r#ref, bundle })
    }
}
