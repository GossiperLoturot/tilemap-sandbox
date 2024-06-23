use godot::prelude::*;

mod behavior;
mod block;
mod entity;
mod inner;
mod node;
mod tile;
mod world;

struct Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Extension {}
