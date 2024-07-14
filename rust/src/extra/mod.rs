use godot::prelude::*;

use crate::*;

#[path = "inner.rs"]
mod extra_inner;

// static class
#[derive(GodotClass)]
#[class(no_init)]
struct Action;

#[godot_api]
impl Action {
    #[func]
    fn before(mut world: Gd<World>) {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        extra_inner::before(&mut world.inner());
    }

    #[func]
    fn after(mut world: Gd<World>) {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        extra_inner::after(&mut world.inner());
    }

    #[func]
    fn forward(mut world: Gd<World>, delta_secs: f32) {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        extra_inner::forward(&mut world.inner(), delta_secs);
    }

    #[func]
    fn place_tile(mut world: Gd<World>, tile: Gd<Tile>) -> u32 {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let tile = tile.bind().inner_ref().clone();
        let key = extra_inner::place_tile(&mut world.inner(), tile).unwrap();
        key
    }

    #[func]
    fn break_tile(mut world: Gd<World>, tile_key: u32) -> Gd<Tile> {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let tile = extra_inner::break_tile(&mut world.inner(), tile_key).unwrap();
        Gd::from_object(Tile::new(tile))
    }

    #[func]
    fn place_block(mut world: Gd<World>, block: Gd<Block>) -> u32 {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let block = block.bind().inner_ref().clone();
        let key = extra_inner::place_block(&mut world.inner(), block).unwrap();
        key
    }

    #[func]
    fn break_block(mut world: Gd<World>, block_key: u32) -> Gd<Block> {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let block = extra_inner::break_block(&mut world.inner(), block_key).unwrap();
        Gd::from_object(Block::new(block))
    }

    #[func]
    fn place_entity(mut world: Gd<World>, entity: Gd<Entity>) -> u32 {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let entity = entity.bind().inner_ref().clone();
        let key = extra_inner::place_entity(&mut world.inner(), entity).unwrap();
        key
    }

    #[func]
    fn break_entity(mut world: Gd<World>, entity_key: u32) -> Gd<Entity> {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let entity = extra_inner::break_entity(&mut world.inner(), entity_key).unwrap();
        Gd::from_object(Entity::new(entity))
    }

    #[func]
    fn generate_chunk(mut world: Gd<World>, chunk_key: Vector2i) {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let chunk_key = [chunk_key.x, chunk_key.y];
        extra_inner::generate_chunk(&mut world.inner(), chunk_key);
    }

    #[func]
    fn call_move_entity(mut world: Gd<World>, entity_key: u32, new_location: Vector2) {
        let mut world = world.bind_mut();
        let mut world = world.as_mut();
        let new_location = [new_location.x, new_location.y];
        extra_inner::call_move_entity(&mut world.inner(), entity_key, new_location);
    }
}

// static class
#[derive(GodotClass)]
#[class(no_init)]
struct Callback;

#[godot_api]
impl Callback {
    #[func]
    fn new_generator(chunk_size: u32, size: u32) -> Gd<CallbackBundle> {
        let bundle = Box::new(extra_inner::Generator { chunk_size, size });
        Gd::from_object(CallbackBundle::new(bundle))
    }

    #[func]
    fn new_random_walk(
        entity_id: u32,
        min_rest_secs: f32,
        max_rest_secs: f32,
        min_distance: f32,
        max_distance: f32,
        speed: f32,
    ) -> Gd<CallbackBundle> {
        let bundle = Box::new(extra_inner::RandomWalk {
            entity_id,
            min_rest_secs,
            max_rest_secs,
            min_distance,
            max_distance,
            speed,
        });
        Gd::from_object(CallbackBundle::new(bundle))
    }

    #[func]
    fn new_random_walk_forward() -> Gd<CallbackBundle> {
        let bundle = Box::new(extra_inner::RandomWalkForward);
        Gd::from_object(CallbackBundle::new(bundle))
    }
}
