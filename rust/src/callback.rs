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
struct CallbackStoreDescriptor {
    entries: Array<Gd<CallbackBundle>>,
}

#[godot_api]
impl CallbackStoreDescriptor {
    #[func]
    fn new_from(entries: Array<Gd<CallbackBundle>>) -> Gd<Self> {
        Gd::from_object(Self { entries })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub(crate) struct CallbackStore {
    inner: inner::CallbackStore,
}

// pass the inner reference for `Root`
impl CallbackStore {
    #[inline]
    pub fn inner_ref(&self) -> &inner::CallbackStore {
        &self.inner
    }
}

#[godot_api]
impl CallbackStore {
    #[func]
    fn new_from(desc: Gd<CallbackStoreDescriptor>) -> Gd<CallbackStore> {
        let desc = desc.bind();

        let mut builder = inner::CallbackStoreBuilder::default();
        desc.entries.iter_shared().for_each(|mut entry| {
            let mut entry = entry.bind_mut();
            let bundle = entry.inner.take().unwrap();
            builder.insert_bundle(bundle);
        });

        let inner = builder.build();
        Gd::from_object(Self { inner })
    }
}
