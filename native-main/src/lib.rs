use godot::prelude::*;

mod addon;
mod context;
mod panic_hook;

struct Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Extension {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            let mut engine = godot::classes::Engine::singleton();

            // Register the Context singleton
            engine.register_singleton("Context", &context::Context::new_alloc())
        }
    }

    fn on_level_deinit(level: InitLevel) {
        if level == InitLevel::Scene {
            let mut engine = godot::classes::Engine::singleton();

            // Unregister the Context singleton
            if let Some(context) = engine.get_singleton("Context") {
                engine.unregister_singleton("Context");
                context.free();
            } else {
                godot_error!("Context singleton not found");
            }
        }
    }
}
