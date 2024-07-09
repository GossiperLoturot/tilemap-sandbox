use crate::inner;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct NodeStore {
    pub inner: inner::NodeStore,
}
