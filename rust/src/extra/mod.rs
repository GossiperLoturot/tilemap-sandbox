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
    fn place_tile(mut root: Gd<Root>, tile: Gd<Tile>) -> Gd<TileKey> {
        let root = &mut root.bind_mut().inner;
        let tile = tile.bind().inner.clone();
        let key = extra_inner::place_tile(root, tile).unwrap();
        Gd::from_object(TileKey { inner: key })
    }

    #[func]
    fn break_tile(mut root: Gd<Root>, tile_key: Gd<TileKey>) -> Gd<Tile> {
        let tile_key = tile_key.bind().inner;
        let root = &mut root.bind_mut().inner;
        let tile = extra_inner::break_tile(root, tile_key).unwrap();
        Gd::from_object(Tile { inner: tile })
    }

    #[func]
    fn modify_tile(mut root: Gd<Root>, tile_key: Gd<TileKey>, new_tile: Gd<Tile>) -> Gd<Tile> {
        let tile_key = tile_key.bind().inner;
        let root = &mut root.bind_mut().inner;
        let new_tile = new_tile.bind().inner.clone();
        let tile = extra_inner::modify_tile(root, tile_key, new_tile).unwrap();
        Gd::from_object(Tile { inner: tile })
    }

    #[func]
    fn place_block(mut root: Gd<Root>, block: Gd<Block>) -> Gd<BlockKey> {
        let root = &mut root.bind_mut().inner;
        let block = block.bind().inner.clone();
        let key = extra_inner::place_block(root, block).unwrap();
        Gd::from_object(BlockKey { inner: key })
    }

    #[func]
    fn break_block(mut root: Gd<Root>, block_key: Gd<BlockKey>) -> Gd<Block> {
        let block_key = block_key.bind().inner;
        let root = &mut root.bind_mut().inner;
        let block = extra_inner::break_block(root, block_key).unwrap();
        Gd::from_object(Block { inner: block })
    }

    #[func]
    fn modify_block(
        mut root: Gd<Root>,
        block_key: Gd<BlockKey>,
        new_block: Gd<Block>,
    ) -> Gd<Block> {
        let block_key = block_key.bind().inner;
        let root = &mut root.bind_mut().inner;
        let new_block = new_block.bind().inner.clone();
        let block = extra_inner::modify_block(root, block_key, new_block).unwrap();
        Gd::from_object(Block { inner: block })
    }

    #[func]
    fn place_entity(mut root: Gd<Root>, entity: Gd<Entity>) -> Gd<EntityKey> {
        let root = &mut root.bind_mut().inner;
        let entity = entity.bind().inner.clone();
        let key = extra_inner::place_entity(root, entity).unwrap();
        Gd::from_object(EntityKey { inner: key })
    }

    #[func]
    fn break_entity(mut root: Gd<Root>, entity_key: Gd<EntityKey>) -> Gd<Entity> {
        let entity_key = entity_key.bind().inner;
        let root = &mut root.bind_mut().inner;
        let entity = extra_inner::break_entity(root, entity_key).unwrap();
        Gd::from_object(Entity { inner: entity })
    }

    #[func]
    fn modify_entity(
        mut root: Gd<Root>,
        entity_key: Gd<EntityKey>,
        new_entity: Gd<Entity>,
    ) -> Gd<Entity> {
        let entity_key = entity_key.bind().inner;
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
    fn new_generator() -> Gd<FlowDescriptor> {
        let value = std::rc::Rc::new(extra_inner::Generator {});
        Gd::from_object(FlowDescriptor { value })
    }
}
