use godot::prelude::*;

mod atlas;

struct Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Extension {}
