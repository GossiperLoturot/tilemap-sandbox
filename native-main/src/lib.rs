use godot::prelude::*;

mod addon;
mod context;
mod panic_hook;

struct Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Extension {}
