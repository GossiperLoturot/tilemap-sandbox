use godot::prelude::*;

mod block;
mod inner;
mod tile;

struct Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Extension {}
