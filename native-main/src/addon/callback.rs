use native_core::*;

// resource

pub struct CallbackResource {
    pub callback: godot::builtin::Callable,
}

impl CallbackResource {
    pub fn new(callback: godot::builtin::Callable) -> Self {
        Self { callback }
    }
}

impl dataflow::Resource for CallbackResource {}
