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
    fn insert(&self, builder: &mut CallbackStoreBuilder);
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

        if self.callbacks.len() >= u32::MAX as usize {
            panic!("capacity overflow");
        }

        self.callbacks.push((type_id, r#ref, Box::new(callback)));
    }

    #[inline]
    pub fn insert_bundle(&mut self, bundle: Box<dyn CallbackBundle>) {
        bundle.insert(self);
    }

    pub fn build(self) -> CallbackStore {
        let mut store = CallbackStore::default();
        for (type_id, r#ref, callback) in self.callbacks {
            // key is guaranteed to be less than u32::MAX
            store.callbacks.push(callback);
            let key = (store.callbacks.len() - 1) as u32;

            let ref_col_0 = store.ref_cols_0.entry((type_id,)).or_default();
            ref_col_0.push(key);

            let ref_col_1 = store.ref_cols_1.entry((type_id, r#ref)).or_default();
            ref_col_1.push(key);
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

        let iter = ref_col.iter().map(|key| {
            let callback = self.callbacks.get(*key as usize).unwrap();
            let ptr = callback as *const _ as *const Box<T>;
            let callback = unsafe { &*ptr }.as_ref();

            callback
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

        let iter = ref_col.iter().map(|key| {
            let callback = self.callbacks.get(*key as usize).unwrap();
            let ptr = callback as *const _ as *const Box<T>;
            let callback = unsafe { &*ptr }.as_ref();

            callback
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_callback() {
        let mut builder = CallbackStoreBuilder::default();

        builder.insert::<i32>(CallbackRef::Global, 1);
        builder.insert::<i32>(CallbackRef::Tile(0), 2);
        builder.insert::<i32>(CallbackRef::Tile(0), 3);
        builder.insert::<i32>(CallbackRef::Tile(1), 4);

        builder.insert::<i64>(CallbackRef::Global, 11);
        builder.insert::<i64>(CallbackRef::Tile(0), 12);
        builder.insert::<i64>(CallbackRef::Tile(0), 13);
        builder.insert::<i64>(CallbackRef::Tile(1), 14);

        let store = builder.build();

        let mut vec = store.iter::<i32>().cloned().collect::<Vec<_>>();
        vec.sort();
        assert_eq!(vec, vec![1, 2, 3, 4]);

        let mut vec = store.iter::<i64>().cloned().collect::<Vec<_>>();
        vec.sort();
        assert_eq!(vec, vec![11, 12, 13, 14]);

        let mut vec = store
            .iter_by_ref::<i32>(CallbackRef::Tile(0))
            .cloned()
            .collect::<Vec<_>>();
        vec.sort();
        assert_eq!(vec, vec![2, 3]);

        let mut vec = store
            .iter_by_ref::<i64>(CallbackRef::Tile(0))
            .cloned()
            .collect::<Vec<_>>();
        vec.sort();
        assert_eq!(vec, vec![12, 13]);
    }
}
