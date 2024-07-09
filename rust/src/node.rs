use godot::prelude::*;

use crate::inner;

#[derive(GodotClass)]
#[class(no_init)]
pub(crate) struct NodeStore {
    inner: inner::NodeStore,
}

// pass the inner reference for world
impl NodeStore {
    #[inline]
    pub(crate) fn inner_mut(&mut self) -> &mut inner::NodeStore {
        &mut self.inner
    }
}

#[godot_api]
impl NodeStore {
    #[func]
    fn new_from() -> Gd<Self> {
        let inner = Default::default();
        Gd::from_object(Self { inner })
    }
}
