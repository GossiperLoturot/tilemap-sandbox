use godot::prelude::*;

use crate::{inner, world};

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct Behavior;

#[godot_api]
impl Behavior {
    #[func]
    fn new_unit_global() -> Gd<world::GlobalBehavior> {
        let inner = Box::new(());
        Gd::from_init_fn(|_| world::GlobalBehavior { inner })
    }

    #[func]
    fn new_unit_tile() -> Gd<world::TileBehavior> {
        let inner = Box::new(());
        Gd::from_init_fn(|_| world::TileBehavior { inner })
    }

    #[func]
    fn new_unit_block() -> Gd<world::BlockBehavior> {
        let inner = Box::new(());
        Gd::from_init_fn(|_| world::BlockBehavior { inner })
    }

    #[func]
    fn new_unit_entity() -> Gd<world::EntityBehavior> {
        let inner = Box::new(());
        Gd::from_init_fn(|_| world::EntityBehavior { inner })
    }

    #[func]
    fn new_time() -> Gd<world::GlobalBehavior> {
        let inner = Box::new(inner::TimeBehavior);
        Gd::from_init_fn(|_| world::GlobalBehavior { inner })
    }

    #[func]
    fn new_random_walk(
        min_rest_secs: f32,
        max_rest_secs: f32,
        min_distance: f32,
        max_distance: f32,
        speed: f32,
    ) -> Gd<world::EntityBehavior> {
        let inner = Box::new(inner::RandomWalkBehavior {
            min_rest_secs,
            max_rest_secs,
            min_distance,
            max_distance,
            speed,
        });
        Gd::from_init_fn(|_| world::EntityBehavior { inner })
    }
}
