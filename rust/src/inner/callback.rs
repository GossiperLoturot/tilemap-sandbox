#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallbackRef {
    Global,
    /// `Tile(id)` means a reference to a tile specification corresponding to `id` in the
    /// `TileField`.
    Tile(u32),
    /// `Block(id)` means a reference to a block specification corresponding to `id` in the
    /// `BlockField`.
    Block(u32),
    /// `Entity(id)` means a reference to a entity specification corresponding to `id` in the
    /// `EntityField`.
    Entity(u32),
}

/// A bundle of callbacks.
/// self is the reference for compability with dyn T.
pub trait CallbackBundle {
    fn insert(&self, r#ref: CallbackRef, builder: &mut CallbackStoreBuilder);
}

#[derive(Default, Debug)]
pub struct CallbackStoreBuilder {
    callbacks: Vec<(std::any::TypeId, CallbackRef, Box<dyn std::any::Any>)>,
}

impl CallbackStoreBuilder {
    pub fn insert<T>(&mut self, r#ref: CallbackRef, callback: T)
    where
        T: std::any::Any,
    {
        let type_id = std::any::TypeId::of::<T>();
        self.callbacks.push((type_id, r#ref, Box::new(callback)));
    }

    #[inline]
    pub fn insert_bundle<B>(&mut self, r#ref: CallbackRef, bundle: B)
    where
        B: CallbackBundle,
    {
        bundle.insert(r#ref, self);
    }

    pub fn build(self) -> CallbackStore {
        let mut store = CallbackStore::default();
        for (type_id, r#ref, callback) in self.callbacks {
            store.callbacks.push(callback);
            let index = (store.callbacks.len() - 1) as u32;

            let ref_col_0 = store.ref_cols_0.entry((type_id,)).or_default();
            ref_col_0.push(index);

            let ref_col_1 = store.ref_cols_1.entry((type_id, r#ref)).or_default();
            ref_col_1.push(index);
        }
        store
    }
}

#[derive(Default, Debug)]
pub struct CallbackStore {
    callbacks: Vec<Box<dyn std::any::Any>>,
    ref_cols_0: ahash::AHashMap<(std::any::TypeId,), Vec<u32>>,
    ref_cols_1: ahash::AHashMap<(std::any::TypeId, CallbackRef), Vec<u32>>,
}

impl CallbackStore {
    fn iter_internal<T>(&self) -> Option<impl Iterator<Item = &T>>
    where
        T: std::any::Any,
    {
        let type_id = std::any::TypeId::of::<T>();
        let ref_col = self.ref_cols_0.get(&(type_id,))?;
        let iter = ref_col.iter().map(|i| {
            let callback = self.callbacks.get(*i as usize).unwrap();
            callback.downcast_ref::<T>().unwrap()
        });
        Some(iter)
    }

    #[inline]
    pub fn iter<T>(&self) -> impl Iterator<Item = &T>
    where
        T: std::any::Any,
    {
        self.iter_internal::<T>().into_iter().flatten()
    }

    fn iter_by_ref_internal<T>(&self, r#ref: CallbackRef) -> Option<impl Iterator<Item = &T>>
    where
        T: std::any::Any,
    {
        let type_id = std::any::TypeId::of::<T>();
        let ref_col = self.ref_cols_1.get(&(type_id, r#ref))?;
        let iter = ref_col.iter().map(|i| {
            let callback = self.callbacks.get(*i as usize).unwrap();
            callback.downcast_ref::<T>().unwrap()
        });
        Some(iter)
    }

    #[inline]
    pub fn iter_by_ref<T>(&self, r#ref: CallbackRef) -> impl Iterator<Item = &T>
    where
        T: std::any::Any,
    {
        self.iter_by_ref_internal::<T>(r#ref).into_iter().flatten()
    }
}
