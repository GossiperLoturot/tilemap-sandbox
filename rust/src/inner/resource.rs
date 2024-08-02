type ResourceRow<T> = Option<T>;

#[derive(Debug, Default)]
pub struct ResourceStore {
    typ_map: ahash::AHashMap<std::any::TypeId, Box<dyn std::any::Any>>,
}

impl ResourceStore {
    pub fn insert<T: 'static>(&mut self, value: T) -> Option<()> {
        self.typ_map
            .insert(std::any::TypeId::of::<T>(), Box::new(value));
        Some(())
    }

    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        let resource = self.typ_map.remove(&std::any::TypeId::of::<T>())?;
        let mut resource = resource.downcast::<ResourceRow<T>>().ok()?;
        resource.take()
    }

    pub fn has<T: 'static>(&self) -> bool {
        self.typ_map.contains_key(&std::any::TypeId::of::<T>())
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        let resource = self.typ_map.get(&std::any::TypeId::of::<T>())?;
        resource.downcast_ref::<T>()
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let resource = self.typ_map.get_mut(&std::any::TypeId::of::<T>())?;
        resource.downcast_mut::<T>()
    }
}
