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
        let root = &mut root.bind_mut().inner;
        extra_inner::before(root);
    }

    #[func]
    fn after(mut root: Gd<Root>) {
        let root = &mut root.bind_mut().inner;
        extra_inner::after(root);
    }

    #[func]
    fn forward(mut root: Gd<Root>, delta_secs: f32) {
        let root = &mut root.bind_mut().inner;
        extra_inner::forward(root, delta_secs);
    }

    #[func]
    fn forward_local(mut root: Gd<Root>, delta_secs: f32, rect: Rect2) {
        let root = &mut root.bind_mut().inner;

        #[rustfmt::skip]
        let rect = [[
            rect.position.x,
            rect.position.y, ], [
            rect.position.x + rect.size.x,
            rect.position.y + rect.size.y,
        ]];

        extra_inner::forward_local(root, delta_secs, [rect[0].into(), rect[1].into()]);
    }

    #[func]
    fn place_tile(mut root: Gd<Root>, tile: Gd<Tile>) -> u32 {
        let root = &mut root.bind_mut().inner;
        let tile = tile.bind().inner.clone();
        extra_inner::place_tile(root, tile).unwrap()
    }

    #[func]
    fn break_tile(mut root: Gd<Root>, tile_key: u32) -> Gd<Tile> {
        let root = &mut root.bind_mut().inner;
        let tile = extra_inner::break_tile(root, tile_key).unwrap();
        Gd::from_object(Tile { inner: tile })
    }

    #[func]
    fn modify_tile(mut root: Gd<Root>, tile_key: u32, new_tile: Gd<Tile>) -> Gd<Tile> {
        let root = &mut root.bind_mut().inner;
        let new_tile = new_tile.bind().inner.clone();
        let tile = extra_inner::modify_tile(root, tile_key, new_tile).unwrap();
        Gd::from_object(Tile { inner: tile })
    }

    #[func]
    fn place_block(mut root: Gd<Root>, block: Gd<Block>) -> u32 {
        let root = &mut root.bind_mut().inner;
        let block = block.bind().inner.clone();
        extra_inner::place_block(root, block).unwrap()
    }

    #[func]
    fn break_block(mut root: Gd<Root>, block_key: u32) -> Gd<Block> {
        let root = &mut root.bind_mut().inner;
        let block = extra_inner::break_block(root, block_key).unwrap();
        Gd::from_object(Block { inner: block })
    }

    #[func]
    fn modify_block(mut root: Gd<Root>, block_key: u32, new_block: Gd<Block>) -> Gd<Block> {
        let root = &mut root.bind_mut().inner;
        let new_block = new_block.bind().inner.clone();
        let block = extra_inner::modify_block(root, block_key, new_block).unwrap();
        Gd::from_object(Block { inner: block })
    }

    #[func]
    fn place_entity(mut root: Gd<Root>, entity: Gd<Entity>) -> u32 {
        let root = &mut root.bind_mut().inner;
        let entity = entity.bind().inner.clone();
        extra_inner::place_entity(root, entity).unwrap()
    }

    #[func]
    fn break_entity(mut root: Gd<Root>, entity_key: u32) -> Gd<Entity> {
        let root = &mut root.bind_mut().inner;
        let entity = extra_inner::break_entity(root, entity_key).unwrap();
        Gd::from_object(Entity { inner: entity })
    }

    #[func]
    fn modify_entity(mut root: Gd<Root>, entity_key: u32, new_entity: Gd<Entity>) -> Gd<Entity> {
        let root = &mut root.bind_mut().inner;
        let new_entity = new_entity.bind().inner.clone();
        let entity = extra_inner::modify_entity(root, entity_key, new_entity).unwrap();
        Gd::from_object(Entity { inner: entity })
    }

    #[func]
    fn generate_chunk(mut root: Gd<Root>, rect: Rect2) {
        let root = &mut root.bind_mut().inner;

        #[rustfmt::skip]
        let rect = [[
            rect.position.x,
            rect.position.y, ], [
            rect.position.x + rect.size.x,
            rect.position.y + rect.size.y,
        ]];

        extra_inner::generate_chunk(root, rect);
    }

    #[func]
    fn move_entity(mut root: Gd<Root>, entity_key: u32, new_location: Vector2) -> bool {
        let root = &mut root.bind_mut().inner;
        let new_location = [new_location.x, new_location.y];
        extra_inner::move_entity(root, entity_key, new_location).is_ok()
    }
}

// static class
#[derive(GodotClass)]
#[class(no_init)]
struct FlowDescriptors;

#[godot_api]
impl FlowDescriptors {
    #[func]
    fn new_base_tile(tile_id: u32) -> Gd<FlowDescriptor> {
        let value = std::rc::Rc::new(extra_inner::BaseTile { tile_id });
        Gd::from_object(FlowDescriptor { value })
    }

    #[func]
    fn new_base_block(block_id: u32) -> Gd<FlowDescriptor> {
        let value = std::rc::Rc::new(extra_inner::BaseBlock { block_id });
        Gd::from_object(FlowDescriptor { value })
    }

    #[func]
    fn new_base_entity(entity_id: u32) -> Gd<FlowDescriptor> {
        let value = std::rc::Rc::new(extra_inner::BaseEntity { entity_id });
        Gd::from_object(FlowDescriptor { value })
    }

    #[func]
    fn new_animal_entity(
        entity_id: u32,
        min_rest_secs: f32,
        max_rest_secs: f32,
        min_distance: f32,
        max_distance: f32,
        speed: f32,
    ) -> Gd<FlowDescriptor> {
        let value = std::rc::Rc::new(extra_inner::AnimalEntity {
            entity_id,
            min_rest_secs,
            max_rest_secs,
            min_distance,
            max_distance,
            speed,
        });
        Gd::from_object(FlowDescriptor { value })
    }

    #[func]
    fn new_generator() -> Gd<FlowDescriptor> {
        let value = std::rc::Rc::new(extra_inner::Generator {});
        Gd::from_object(FlowDescriptor { value })
    }

    #[func]
    fn new_random_walk_forward_local() -> Gd<FlowDescriptor> {
        let value = std::rc::Rc::new(extra_inner::RandomWalkForwardLocal {});
        Gd::from_object(FlowDescriptor { value })
    }
}
