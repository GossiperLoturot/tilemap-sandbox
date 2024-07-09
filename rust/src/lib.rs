use godot::prelude::*;

mod block;
mod callback;
mod entity;
mod extra;
mod inner;
mod node;
mod tile;

struct Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Extension {}

// rename `World` to `WorldServer` to avoid duplicating the class name with the built-in class in Godot.
#[derive(GodotClass)]
#[class(no_init, base=RefCounted, rename=WorldServer)]
pub struct World {
    tile_field: Gd<tile::TileField>,
    block_field: Gd<block::BlockField>,
    entity_field: Gd<entity::EntityField>,
    node_store: Gd<node::NodeStore>,
    callback_store: Gd<callback::CallbackStore>,
}

#[godot_api]
impl World {
    #[func]
    fn new_from(
        tile_field: Gd<tile::TileField>,
        block_field: Gd<block::BlockField>,
        entity_field: Gd<entity::EntityField>,
        node_store: Gd<node::NodeStore>,
        callback_store: Gd<callback::CallbackStore>,
    ) -> Gd<Self> {
        Gd::from_init_fn(|_| Self {
            tile_field,
            block_field,
            entity_field,
            node_store,
            callback_store,
        })
    }

    pub fn as_mut(&mut self) -> WorldMut {
        WorldMut {
            tile_field: self.tile_field.bind_mut(),
            block_field: self.block_field.bind_mut(),
            entity_field: self.entity_field.bind_mut(),
            node_store: self.node_store.bind_mut(),
            callback_store: self.callback_store.bind(),
        }
    }
}

pub struct WorldMut<'a> {
    tile_field: GdMut<'a, tile::TileField>,
    block_field: GdMut<'a, block::BlockField>,
    entity_field: GdMut<'a, entity::EntityField>,
    node_store: GdMut<'a, node::NodeStore>,
    callback_store: GdRef<'a, callback::CallbackStore>,
}

impl WorldMut<'_> {
    pub fn inner(&mut self) -> inner::World {
        inner::World {
            tile_field: &mut self.tile_field.inner,
            block_field: &mut self.block_field.inner,
            entity_field: &mut self.entity_field.inner,
            node_store: &mut self.node_store.inner,
            callback_store: &self.callback_store.inner,
        }
    }
}
