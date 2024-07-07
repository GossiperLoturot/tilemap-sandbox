use crate::{block, entity, extra, inner, node, tile};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct DelegateStore {
    pub inner: inner::DelegateStore,
}

#[godot_api]
impl DelegateStore {
    #[func]
    fn new_from() -> Gd<Self> {
        let inner = Default::default();
        Gd::from_init_fn(|_| DelegateStore { inner })
    }
}

// Rename World to WorldServer to avoid duplicating the class name with the built-in class in Godot.
#[derive(GodotClass)]
#[class(no_init, base=RefCounted, rename=WorldServer)]
pub struct World {
    tile_field: Gd<tile::TileField>,
    block_field: Gd<block::BlockField>,
    entity_field: Gd<entity::EntityField>,
    node_store: Gd<node::NodeStore>,
    delegate_store: Gd<DelegateStore>,
}

#[godot_api]
impl World {
    #[func]
    fn new_from(
        tile_field: Gd<tile::TileField>,
        block_field: Gd<block::BlockField>,
        entity_field: Gd<entity::EntityField>,
        node_store: Gd<node::NodeStore>,
        delegate_store: Gd<DelegateStore>,
    ) -> Gd<Self> {
        Gd::from_init_fn(|_| Self {
            tile_field,
            block_field,
            entity_field,
            node_store,
            delegate_store,
        })
    }

    pub fn as_mut(&mut self) -> WorldMut {
        WorldMut {
            tile_field: self.tile_field.bind_mut(),
            block_field: self.block_field.bind_mut(),
            entity_field: self.entity_field.bind_mut(),
            node_store: self.node_store.bind_mut(),
            delegate_store: self.delegate_store.bind(),
        }
    }
}

pub struct WorldMut<'a> {
    tile_field: GdMut<'a, tile::TileField>,
    block_field: GdMut<'a, block::BlockField>,
    entity_field: GdMut<'a, entity::EntityField>,
    node_store: GdMut<'a, node::NodeStore>,
    delegate_store: GdRef<'a, DelegateStore>,
}

impl WorldMut<'_> {
    pub fn inner(&mut self) -> inner::World {
        inner::World {
            tile_field: &mut self.tile_field.inner,
            block_field: &mut self.block_field.inner,
            entity_field: &mut self.entity_field.inner,
            node_store: &mut self.node_store.inner,
            delegate_store: &self.delegate_store.inner,
        }
    }
}

// Static class
#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct Delegate;

#[godot_api]
impl Delegate {
    #[func]
    fn inserted_generator(
        mut delegate_store: Gd<DelegateStore>,
        id: u32,
        chunk_size: u32,
        size: u32,
    ) {
        let inner = &mut delegate_store.bind_mut().inner;
        let delegate = extra::GeneratorDelegate { chunk_size, size };
        delegate.inserted(inner, id);
    }

    #[func]
    fn inserted_randow_walk(
        mut delegate_store: Gd<DelegateStore>,
        id: u32,
        min_rest_secs: f32,
        max_rest_secs: f32,
        min_distance: f32,
        max_distance: f32,
        speed: f32,
    ) {
        let inner = &mut delegate_store.bind_mut().inner;
        let delegate = extra::RandomWalkDelegate {
            min_rest_secs,
            max_rest_secs,
            min_distance,
            max_distance,
            speed,
        };
        delegate.inserted(inner, id);
    }

    #[func]
    fn call_new(mut world: Gd<World>) {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        extra::new(&mut world.inner());
    }

    #[func]
    fn call_drop(mut world: Gd<World>) {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        extra::drop(&mut world.inner());
    }

    #[func]
    fn call_update(mut world: Gd<World>, delta_secs: f32) {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        extra::update(&mut world.inner(), delta_secs);
    }

    #[func]
    fn call_place_tile(mut world: Gd<World>, tile: Gd<tile::Tile>) -> Option<Gd<tile::TileKey>> {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let tile = tile.bind().inner.clone();
        let key = extra::place_tile(&mut world.inner(), tile).ok()?;
        Some(Gd::from_init_fn(|_| tile::TileKey { inner: key }))
    }

    #[func]
    fn call_break_tile(
        mut world: Gd<World>,
        tile_key: Gd<tile::TileKey>,
    ) -> Option<Gd<tile::Tile>> {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let key = tile_key.bind().inner;
        let tile = extra::break_tile(&mut world.inner(), key).ok()?;
        Some(Gd::from_init_fn(|_| tile::Tile { inner: tile }))
    }

    #[func]
    fn call_place_block(
        mut world: Gd<World>,
        block: Gd<block::Block>,
    ) -> Option<Gd<block::BlockKey>> {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let block = block.bind().inner.clone();
        let key = extra::place_block(&mut world.inner(), block).ok()?;
        Some(Gd::from_init_fn(|_| block::BlockKey { inner: key }))
    }

    #[func]
    fn call_break_block(
        mut world: Gd<World>,
        block_key: Gd<block::BlockKey>,
    ) -> Option<Gd<block::Block>> {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let key = block_key.bind().inner;
        let block = extra::break_block(&mut world.inner(), key).ok()?;
        Some(Gd::from_init_fn(|_| block::Block { inner: block }))
    }

    #[func]
    fn call_place_entity(
        mut world: Gd<World>,
        entity: Gd<entity::Entity>,
    ) -> Option<Gd<entity::EntityKey>> {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let entity = entity.bind().inner.clone();
        let key = extra::place_entity(&mut world.inner(), entity).ok()?;
        Some(Gd::from_init_fn(|_| entity::EntityKey { inner: key }))
    }

    #[func]
    fn call_break_entity(
        mut world: Gd<World>,
        entity_key: Gd<entity::EntityKey>,
    ) -> Option<Gd<entity::Entity>> {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let key = entity_key.bind().inner;
        let entity = extra::break_entity(&mut world.inner(), key).ok()?;
        Some(Gd::from_init_fn(|_| entity::Entity { inner: entity }))
    }

    #[func]
    fn call_generate_chunk(mut world: Gd<World>, chunk_key: Vector2i) {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let chunk_key = [chunk_key.x, chunk_key.y];
        extra::generate_chunk(&mut world.inner(), chunk_key);
    }
}
