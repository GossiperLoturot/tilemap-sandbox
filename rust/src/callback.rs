use godot::prelude::*;

use crate::inner;

#[derive(GodotClass)]
#[class(no_init)]
pub struct CallbackBundle {
    inner: Option<Box<dyn inner::CallbackBundle>>,
}

impl CallbackBundle {
    #[inline]
    pub fn new(value: Box<dyn inner::CallbackBundle>) -> Self {
        Self { inner: Some(value) }
    }

    #[inline]
    pub fn inner_ref(&self) -> &Box<dyn inner::CallbackBundle> {
        self.inner.as_ref().unwrap()
    }

    #[inline]
    pub fn inner_mut(&mut self) -> &mut Box<dyn inner::CallbackBundle> {
        self.inner.as_mut().unwrap()
    }

    #[inline]
    pub fn inner(self) -> Box<dyn inner::CallbackBundle> {
        self.inner.unwrap()
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub(crate) struct CallbackStoreBuilder {
    inner: Option<inner::CallbackStoreBuilder>,
}

#[godot_api]
impl CallbackStoreBuilder {
    #[func]
    fn new_from() -> Gd<Self> {
        let inner = Some(Default::default());
        Gd::from_object(Self { inner })
    }

    #[func]
    fn insert_bundle(&mut self, mut bundle: Gd<CallbackBundle>) {
        let slf = self.inner.as_mut().unwrap();
        let bundle = bundle.bind_mut().inner.take().unwrap();
        slf.insert_bundle(bundle);
    }

    #[func]
    fn build(&mut self) -> Gd<CallbackStore> {
        let slf = self.inner.take().unwrap();
        let store = slf.build();
        Gd::from_object(CallbackStore { inner: store })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub(crate) struct CallbackStore {
    inner: inner::CallbackStore,
}

// pass the inner reference for world
impl CallbackStore {
    #[inline]
    pub(crate) fn inner_ref(&self) -> &inner::CallbackStore {
        &self.inner
    }
}
