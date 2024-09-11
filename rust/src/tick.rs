use godot::prelude::*;

use crate::inner;

pub(crate) struct TickStore {}

impl TickStore {
    const PARAMETER_NAME: &'static str = "tick";

    pub fn new() -> Self {
        let mut rendering_server = godot::classes::RenderingServer::singleton();

        rendering_server.global_shader_parameter_add(
            Self::PARAMETER_NAME.into(),
            godot::classes::rendering_server::GlobalShaderParameterType::INT,
            0.to_variant(),
        );

        Self {}
    }

    pub fn update_view(&self, root: &inner::Root) {
        let mut rendering_server = godot::classes::RenderingServer::singleton();

        rendering_server.global_shader_parameter_set(
            Self::PARAMETER_NAME.into(),
            (root.tick_get() as u32).to_variant(),
        );
    }
}
