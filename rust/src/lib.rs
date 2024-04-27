use godot::prelude::*;

mod block;
mod entity;
mod inner;
mod tile;

struct Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Extension {}
