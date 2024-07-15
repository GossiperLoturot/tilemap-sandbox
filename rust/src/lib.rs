use godot::prelude::*;

pub use block::*;
pub use callback::*;
pub use entity::*;
// pub use node::*;
pub use tile::*;

pub mod inner;

mod block;
mod callback;
mod entity;
mod node;
mod tile;

mod extra;

struct Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Extension {}

#[derive(GodotClass)]
#[class(no_init)]
pub struct Root {
    #[export]
    tile_field: Gd<tile::TileField>,
    #[export]
    block_field: Gd<block::BlockField>,
    #[export]
    entity_field: Gd<entity::EntityField>,
    #[export]
    node_store: Gd<node::NodeStore>,
    #[export]
    callback_store: Gd<callback::CallbackStore>,
}

#[godot_api]
impl Root {
    #[func]
    fn new_from(
        tile_field: Gd<tile::TileField>,
        block_field: Gd<block::BlockField>,
        entity_field: Gd<entity::EntityField>,
        node_store: Gd<node::NodeStore>,
        callback_store: Gd<callback::CallbackStore>,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            tile_field,
            block_field,
            entity_field,
            node_store,
            callback_store,
        })
    }

    #[inline]
    pub fn as_ref(&mut self) -> RootRef {
        RootRef {
            tile_field: self.tile_field.bind(),
            block_field: self.block_field.bind(),
            entity_field: self.entity_field.bind(),
            node_store: self.node_store.bind(),
            callback_store: self.callback_store.bind(),
        }
    }

    #[inline]
    pub fn as_mut(&mut self) -> RootMut {
        RootMut {
            tile_field: self.tile_field.bind_mut(),
            block_field: self.block_field.bind_mut(),
            entity_field: self.entity_field.bind_mut(),
            node_store: self.node_store.bind_mut(),
            callback_store: self.callback_store.bind(),
        }
    }
}

pub struct RootRef<'a> {
    tile_field: GdRef<'a, tile::TileField>,
    block_field: GdRef<'a, block::BlockField>,
    entity_field: GdRef<'a, entity::EntityField>,
    node_store: GdRef<'a, node::NodeStore>,
    callback_store: GdRef<'a, callback::CallbackStore>,
}

impl RootRef<'_> {
    #[inline]
    pub fn inner(&mut self) -> inner::RootRef {
        inner::RootRef {
            tile_field: self.tile_field.inner_ref(),
            block_field: self.block_field.inner_ref(),
            entity_field: self.entity_field.inner_ref(),
            node_store: self.node_store.inner_ref(),
            callback_store: self.callback_store.inner_ref(),
        }
    }
}

pub struct RootMut<'a> {
    tile_field: GdMut<'a, tile::TileField>,
    block_field: GdMut<'a, block::BlockField>,
    entity_field: GdMut<'a, entity::EntityField>,
    node_store: GdMut<'a, node::NodeStore>,
    callback_store: GdRef<'a, callback::CallbackStore>,
}

impl RootMut<'_> {
    #[inline]
    pub fn inner(&mut self) -> inner::RootMut {
        inner::RootMut {
            tile_field: self.tile_field.inner_mut(),
            block_field: self.block_field.inner_mut(),
            entity_field: self.entity_field.inner_mut(),
            node_store: self.node_store.inner_mut(),
            callback_store: self.callback_store.inner_ref(),
        }
    }
}
