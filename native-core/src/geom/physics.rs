use super::*;

pub struct PhysicsId(godot::builtin::Rid);

pub struct PhysicsProviderInfo {
    pub world: godot::obj::Gd<godot::classes::World2D>,
}

pub struct PhysicsProvider {
    base_shape_rid: godot::builtin::Rid,
    space_rid: godot::builtin::Rid,
}

impl PhysicsProvider {
    pub fn new(info: PhysicsProviderInfo) -> Self {
        let mut physics_server = <godot::classes::PhysicsServer2D as godot::obj::Singleton>::singleton();

        let base_shape_rid = physics_server.rectangle_shape_create();
        let extents = godot::builtin::Vector2::new(0.5, 0.5);
        physics_server.shape_set_data(base_shape_rid, &godot::builtin::Variant::from(extents));

        let space_rid = info.world.get_space();

        Self { base_shape_rid, space_rid }
    }

    pub fn insert(&self, rect: Rect2) -> PhysicsId {
        let mut physics_server = <godot::classes::PhysicsServer2D as godot::obj::Singleton>::singleton();

        let (scale, origin) = (rect.size(), rect.center());
        let transform = godot::builtin::Transform2D::from_cols(
            godot::builtin::Vector2::new(scale.x, 0.0),
            godot::builtin::Vector2::new(0.0, scale.y),
            godot::builtin::Vector2::new(origin.x, origin.y),
        );

        let body_rid = physics_server.body_create();
        physics_server.body_add_shape(body_rid, self.base_shape_rid);
        physics_server.body_set_mode(body_rid, godot::classes::physics_server_2d::BodyMode::STATIC);
        physics_server.body_set_space(body_rid, self.space_rid);
        physics_server.body_set_state(
            body_rid,
            godot::classes::physics_server_2d::BodyState::TRANSFORM,
            &godot::builtin::Variant::from(transform),
        );

        PhysicsId(body_rid)
    }

    pub fn remove(&self, id: PhysicsId) {
        let PhysicsId(body_rid) = id;
        let mut physics_server = <godot::classes::PhysicsServer2D as godot::obj::Singleton>::singleton();
        physics_server.free_rid(body_rid);
    }
}

impl Drop for PhysicsProvider {
    fn drop(&mut self) {
        let mut physics_server = <godot::classes::PhysicsServer2D as godot::obj::Singleton>::singleton();
        physics_server.free_rid(self.base_shape_rid);
    }
}
