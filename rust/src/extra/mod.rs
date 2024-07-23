use godot::prelude::*;

use crate::*;

#[path = "inner.rs"]
mod extra_inner;

// static class
#[derive(GodotClass)]
#[class(no_init)]
struct Actions;

#[godot_api]
impl Actions {
    #[func]
    fn before(mut root: Gd<Root>) {
        let mut root = root.bind_mut();
        let mut root = root.as_mut();
        extra_inner::before(&mut root.inner());
    }

    #[func]
    fn after(mut root: Gd<Root>) {
        let mut root = root.bind_mut();
        let mut root = root.as_mut();
        extra_inner::after(&mut root.inner());
    }

    #[func]
    fn forward(mut root: Gd<Root>, delta_secs: f32) {
        let mut root = root.bind_mut();
        let mut root = root.as_mut();
        extra_inner::forward(&mut root.inner(), delta_secs);
    }

    #[func]
    fn forward_local(mut root: Gd<Root>, delta_secs: f32, rect: Rect2) {
        let mut root = root.bind_mut();
        let mut root = root.as_mut();

        #[rustfmt::skip]
        let rect = [[
            rect.position.x,
            rect.position.y, ], [
            rect.position.x + rect.size.x,
            rect.position.y + rect.size.y,
        ]];

        extra_inner::forward_local(&mut root.inner(), delta_secs, rect);
    }

    #[func]
    fn place_tile(mut root: Gd<Root>, tile: Gd<Tile>) -> u32 {
        let mut root = root.bind_mut();
        let mut root = root.as_mut();
        let tile = tile.bind().inner_ref().clone();
        let key = extra_inner::place_tile(&mut root.inner(), tile).unwrap();
        key
    }

    #[func]
    fn break_tile(mut root: Gd<Root>, tile_key: u32) -> Gd<Tile> {
        let mut root = root.bind_mut();
        let mut root = root.as_mut();
        let tile = extra_inner::break_tile(&mut root.inner(), tile_key).unwrap();
        Gd::from_object(Tile::new(tile))
    }

    #[func]
    fn place_block(mut root: Gd<Root>, block: Gd<Block>) -> u32 {
        let mut root = root.bind_mut();
        let mut root = root.as_mut();
        let block = block.bind().inner_ref().clone();
        let key = extra_inner::place_block(&mut root.inner(), block).unwrap();
        key
    }

    #[func]
    fn break_block(mut root: Gd<Root>, block_key: u32) -> Gd<Block> {
        let mut root = root.bind_mut();
        let mut root = root.as_mut();
        let block = extra_inner::break_block(&mut root.inner(), block_key).unwrap();
        Gd::from_object(Block::new(block))
    }

    #[func]
    fn place_entity(mut root: Gd<Root>, entity: Gd<Entity>) -> u32 {
        let mut root = root.bind_mut();
        let mut root = root.as_mut();
        let entity = entity.bind().inner_ref().clone();
        let key = extra_inner::place_entity(&mut root.inner(), entity).unwrap();
        key
    }

    #[func]
    fn break_entity(mut root: Gd<Root>, entity_key: u32) -> Gd<Entity> {
        let mut root = root.bind_mut();
        let mut root = root.as_mut();
        let entity = extra_inner::break_entity(&mut root.inner(), entity_key).unwrap();
        Gd::from_object(Entity::new(entity))
    }

    #[func]
    fn generate_chunk(mut root: Gd<Root>, rect: Rect2) {
        let mut root = root.bind_mut();
        let mut root = root.as_mut();

        #[rustfmt::skip]
        let rect = [[
            rect.position.x,
            rect.position.y, ], [
            rect.position.x + rect.size.x,
            rect.position.y + rect.size.y,
        ]];

        extra_inner::generate_chunk(&mut root.inner(), rect);
    }

    #[func]
    fn move_entity(mut root: Gd<Root>, entity_key: u32, new_location: Vector2) {
        let mut root = root.bind_mut();
        let mut root = root.as_mut();
        let new_location = [new_location.x, new_location.y];
        extra_inner::move_entity_ex(&mut root.inner(), entity_key, new_location);
    }
}

// static class
#[derive(GodotClass)]
#[class(no_init)]
struct CallbackBundles;

#[godot_api]
impl CallbackBundles {
    #[func]
    fn new_generator() -> Gd<CallbackBundle> {
        let bundle = Box::new(extra_inner::Generator {});
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
