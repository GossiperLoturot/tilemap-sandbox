use crate::inner;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct NodeStore {
    pub inner: inner::NodeStore,
}

#[godot_api]
impl NodeStore {
    #[func]
    fn new_from() -> Gd<Self> {
        let inner = Default::default();
        Gd::from_init_fn(|_| NodeStore { inner })
    }
}
