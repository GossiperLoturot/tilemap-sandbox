use godot::prelude::*;

mod atlas;
mod field;

struct Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Extension {}
