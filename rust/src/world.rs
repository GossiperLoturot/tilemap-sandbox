use crate::{block, entity, inner, node, tile};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct GlobalBehavior {
    pub inner: Box<dyn inner::GlobalBehavior>,
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct TileBehavior {
    pub inner: Box<dyn inner::TileBehavior>,
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct BlockBehavior {
    pub inner: Box<dyn inner::BlockBehavior>,
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct EntityBehavior {
    pub inner: Box<dyn inner::EntityBehavior>,
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct BehaviorRoot {
    inner: inner::BehaviorRoot,
}

#[godot_api]
impl BehaviorRoot {
    #[func]
    fn new_from(
        global_behaviors: Array<Gd<GlobalBehavior>>,
        tile_behaviors: Array<Gd<TileBehavior>>,
        block_behaviors: Array<Gd<BlockBehavior>>,
        entity_behaviors: Array<Gd<EntityBehavior>>,
    ) -> Gd<Self> {
        let global_behaviors = global_behaviors
            .iter_shared()
            .map(|behavior| behavior.bind().inner.clone())
            .collect::<Vec<_>>();
        let tile_behaviors = tile_behaviors
            .iter_shared()
            .map(|behavior| behavior.bind().inner.clone())
            .collect::<Vec<_>>();
        let block_behaviors = block_behaviors
            .iter_shared()
            .map(|behavior| behavior.bind().inner.clone())
            .collect::<Vec<_>>();
        let entity_behaviors = entity_behaviors
            .iter_shared()
            .map(|behavior| behavior.bind().inner.clone())
            .collect::<Vec<_>>();
        let inner = inner::BehaviorRoot {
            global_behaviors,
            tile_behaviors,
            block_behaviors,
            entity_behaviors,
        };
        Gd::from_init_fn(|_| Self { inner })
    }
}

// Rename World to WorldServer to avoid duplicating the class name with the built-in class in Godot.
#[derive(GodotClass)]
#[class(no_init, base=RefCounted, rename=WorldServer)]
struct World {
    tile_field: Gd<tile::TileField>,
    block_field: Gd<block::BlockField>,
    entity_field: Gd<entity::EntityField>,
    node_store: Gd<node::NodeStore>,
    behavior_root: Gd<BehaviorRoot>,
}

#[godot_api]
impl World {
    #[func]
    fn new_from(
        mut tile_field: Gd<tile::TileField>,
        mut block_field: Gd<block::BlockField>,
        mut entity_field: Gd<entity::EntityField>,
        mut node_store: Gd<node::NodeStore>,
        behavior_root: Gd<BehaviorRoot>,
    ) -> Gd<Self> {
        {
            let mut world = inner::World {
                tile_field: &mut tile_field.bind_mut().inner,
                block_field: &mut block_field.bind_mut().inner,
                entity_field: &mut entity_field.bind_mut().inner,
                node_store: &mut node_store.bind_mut().inner,
                behavior_root: &behavior_root.bind().inner,
            };

            world.install();
        }

        Gd::from_init_fn(|_| Self {
            tile_field,
            block_field,
            entity_field,
            node_store,
            behavior_root,
        })
    }

    #[func]
    fn update(&mut self) {
        let mut world = inner::World {
            tile_field: &mut self.tile_field.bind_mut().inner,
            block_field: &mut self.block_field.bind_mut().inner,
            entity_field: &mut self.entity_field.bind_mut().inner,
            node_store: &mut self.node_store.bind_mut().inner,
            behavior_root: &self.behavior_root.bind().inner,
        };

        world.update();
    }

    #[func]
    fn place_tile(&mut self, tile: Gd<tile::Tile>) -> Option<Gd<tile::TileKey>> {
        let mut world = inner::World {
            tile_field: &mut self.tile_field.bind_mut().inner,
            block_field: &mut self.block_field.bind_mut().inner,
            entity_field: &mut self.entity_field.bind_mut().inner,
            node_store: &mut self.node_store.bind_mut().inner,
            behavior_root: &self.behavior_root.bind().inner,
        };

        let tile = tile.bind().inner.clone();
        let tile_key = world.place_tile(tile).ok()?;

        let tile_key = tile::TileKey { inner: tile_key };
        Some(Gd::from_init_fn(|_| tile_key))
    }

    #[func]
    fn break_tile(&mut self, tile_key: Gd<tile::TileKey>) -> Option<Gd<tile::Tile>> {
        let mut world = inner::World {
            tile_field: &mut self.tile_field.bind_mut().inner,
            block_field: &mut self.block_field.bind_mut().inner,
            entity_field: &mut self.entity_field.bind_mut().inner,
            node_store: &mut self.node_store.bind_mut().inner,
            behavior_root: &self.behavior_root.bind().inner,
        };

        let tile_key = tile_key.bind().inner;
        let tile = world.break_tile(tile_key).ok()?;

        let tile = tile::Tile { inner: tile };
        Some(Gd::from_init_fn(|_| tile))
    }

    #[func]
    fn place_block(&mut self, block: Gd<block::Block>) -> Option<Gd<block::BlockKey>> {
        let mut world = inner::World {
            tile_field: &mut self.tile_field.bind_mut().inner,
            block_field: &mut self.block_field.bind_mut().inner,
            entity_field: &mut self.entity_field.bind_mut().inner,
            node_store: &mut self.node_store.bind_mut().inner,
            behavior_root: &self.behavior_root.bind().inner,
        };

        let block = block.bind().inner.clone();
        let block_key = world.place_block(block).ok()?;

        let block_key = block::BlockKey { inner: block_key };
        Some(Gd::from_init_fn(|_| block_key))
    }

    #[func]
    fn break_block(&mut self, block_key: Gd<block::BlockKey>) -> Option<Gd<block::Block>> {
        let mut world = inner::World {
            tile_field: &mut self.tile_field.bind_mut().inner,
            block_field: &mut self.block_field.bind_mut().inner,
            entity_field: &mut self.entity_field.bind_mut().inner,
            node_store: &mut self.node_store.bind_mut().inner,
            behavior_root: &self.behavior_root.bind().inner,
        };

        let block_key = block_key.bind().inner;
        let block = world.break_block(block_key).ok()?;

        let block = block::Block { inner: block };
        Some(Gd::from_init_fn(|_| block))
    }

    #[func]
    fn place_entity(&mut self, entity: Gd<entity::Entity>) -> Option<Gd<entity::EntityKey>> {
        let mut world = inner::World {
            tile_field: &mut self.tile_field.bind_mut().inner,
            block_field: &mut self.block_field.bind_mut().inner,
            entity_field: &mut self.entity_field.bind_mut().inner,
            node_store: &mut self.node_store.bind_mut().inner,
            behavior_root: &self.behavior_root.bind().inner,
        };

        let entity = entity.bind().inner.clone();
        let entity_key = world.place_entity(entity).ok()?;

        let entity_key = entity::EntityKey { inner: entity_key };
        Some(Gd::from_init_fn(|_| entity_key))
    }

    #[func]
    fn break_entity(&mut self, entity_key: Gd<entity::EntityKey>) -> Option<Gd<entity::Entity>> {
        let mut world = inner::World {
            tile_field: &mut self.tile_field.bind_mut().inner,
            block_field: &mut self.block_field.bind_mut().inner,
            entity_field: &mut self.entity_field.bind_mut().inner,
            node_store: &mut self.node_store.bind_mut().inner,
            behavior_root: &self.behavior_root.bind().inner,
        };

        let entity_key = entity_key.bind().inner;
        let entity = world.break_entity(entity_key).ok()?;

        let entity = entity::Entity { inner: entity };
        Some(Gd::from_init_fn(|_| entity))
    }
}

impl Drop for World {
    fn drop(&mut self) {
        let mut world = inner::World {
            tile_field: &mut self.tile_field.bind_mut().inner,
            block_field: &mut self.block_field.bind_mut().inner,
            entity_field: &mut self.entity_field.bind_mut().inner,
            node_store: &mut self.node_store.bind_mut().inner,
            behavior_root: &self.behavior_root.bind().inner,
        };

        world.uninstall();
    }
}
