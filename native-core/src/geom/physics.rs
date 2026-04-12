use glam::*;

use super::*;

#[cfg(test)]
mod test {
    #[test]
    fn use_physics_provider() {
        let provider = <godot::classes::PhysicsServer2D as godot::obj::Singleton>::singleton();
        println!("{:?}", provider);
    }
}
