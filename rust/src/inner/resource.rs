type ResourceRow<T> = Option<T>;

#[derive(Debug, Default)]
pub struct ResourceStore {
    typ_map: ahash::AHashMap<std::any::TypeId, Box<dyn std::any::Any>>,
}

impl ResourceStore {
    pub fn insert<T: 'static>(&mut self, value: T) -> Option<()> {
        let typ = std::any::TypeId::of::<T>();

        if self.typ_map.contains_key(&typ) {
            return None;
        }

        self.typ_map.insert(typ, Box::new(Some(value)));
        Some(())
    }

    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        let typ = std::any::TypeId::of::<T>();

        let resource = self.typ_map.remove(&typ)?;
        resource.downcast::<ResourceRow<T>>().ok()?.take()
    }

    pub fn has<T: 'static>(&self) -> bool {
        let typ = std::any::TypeId::of::<T>();

        self.typ_map.contains_key(&typ)
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        let typ = std::any::TypeId::of::<T>();

        let resource = self.typ_map.get(&typ)?;
        resource.downcast_ref::<ResourceRow<T>>()?.as_ref()
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let typ = std::any::TypeId::of::<T>();

        let resource = self.typ_map.get_mut(&typ)?;
        resource.downcast_mut::<ResourceRow<T>>()?.as_mut()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn crud() {
        let mut store = super::ResourceStore::default();
        assert_eq!(store.has::<i32>(), false);
        assert_eq!(store.get::<i32>(), None);
        assert_eq!(store.get_mut::<i32>(), None);
        assert_eq!(store.remove::<i32>(), None);

        assert_eq!(store.insert(42), Some(()));
        assert_eq!(store.insert(42), None);

        assert_eq!(store.has::<i32>(), true);
        assert_eq!(store.get::<i32>(), Some(&42));
        assert_eq!(store.get_mut::<i32>(), Some(&mut 42));

        assert_eq!(store.remove::<i32>(), Some(42));
        assert_eq!(store.remove::<i32>(), None);
    }
}
