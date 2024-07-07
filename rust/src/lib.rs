use godot::prelude::*;

mod block;
mod delegate;
mod entity;
mod extra;
mod inner;
mod node;
mod tile;

struct Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Extension {}
