use super::*;

#[derive(Default, Debug)]
pub struct DelegateStore {
    delegates: Vec<Box<dyn std::any::Any>>,
    ref_cols_0: ahash::AHashMap<(std::any::TypeId,), Vec<u32>>,
    ref_cols_1: ahash::AHashMap<(std::any::TypeId, u32), Vec<u32>>,
    ref_cols_2: ahash::AHashMap<(std::any::TypeId, u32, u32), Vec<u32>>,
}

impl DelegateStore {
    pub const GLOBAL_LAYER: u32 = 0;
    pub const TILE_LAYER: u32 = 1;
    pub const BLOCK_LAYER: u32 = 2;
    pub const ENTITY_LAYER: u32 = 3;

    pub fn insert<T>(&mut self, layer: u32, id: u32, delegate: T)
    where
        T: std::any::Any,
    {
        let type_id = std::any::TypeId::of::<T>();

        self.delegates.push(Box::new(delegate));
        let index = (self.delegates.len() - 1) as u32;

        let ref_col_0 = self.ref_cols_0.entry((type_id,)).or_default();
        ref_col_0.push(index);

        let ref_col_1 = self.ref_cols_1.entry((type_id, layer)).or_default();
        ref_col_1.push(index);

        let ref_col_2 = self.ref_cols_2.entry((type_id, layer, id)).or_default();
        ref_col_2.push(index);
    }

    fn iter_internal_0<T>(&self) -> Option<impl Iterator<Item = &T>>
    where
        T: std::any::Any,
    {
        let type_id = std::any::TypeId::of::<T>();
        let ref_col = self.ref_cols_0.get(&(type_id,))?;
        let iter = ref_col.iter().map(|i| {
            let delegate = self.delegates.get(*i as usize).unwrap();
            delegate.downcast_ref::<T>().unwrap()
        });
        Some(iter)
    }

    fn iter_internal_1<T>(&self, layer: u32) -> Option<impl Iterator<Item = &T>>
    where
        T: std::any::Any,
    {
        let type_id = std::any::TypeId::of::<T>();
        let ref_col = self.ref_cols_1.get(&(type_id, layer))?;
        let iter = ref_col.iter().map(|i| {
            let delegate = self.delegates.get(*i as usize).unwrap();
            delegate.downcast_ref::<T>().unwrap()
        });
        Some(iter)
    }

    fn iter_internal_2<T>(&self, layer: u32, id: u32) -> Option<impl Iterator<Item = &T>>
    where
        T: std::any::Any,
    {
        let type_id = std::any::TypeId::of::<T>();
        let ref_col = self.ref_cols_2.get(&(type_id, layer, id))?;
        let iter = ref_col.iter().map(|i| {
            let delegate = self.delegates.get(*i as usize).unwrap();
            delegate.downcast_ref::<T>().unwrap()
        });
        Some(iter)
    }

    #[inline]
    pub fn iter_0<T>(&self) -> impl Iterator<Item = &T>
    where
        T: std::any::Any,
    {
        self.iter_internal_0::<T>().into_iter().flatten()
    }

    #[inline]
    pub fn iter_1<T>(&self, layer: u32) -> impl Iterator<Item = &T>
    where
        T: std::any::Any,
    {
        self.iter_internal_1::<T>(layer).into_iter().flatten()
    }

    #[inline]
    pub fn iter_2<T>(&self, layer: u32, id: u32) -> impl Iterator<Item = &T>
    where
        T: std::any::Any,
    {
        self.iter_internal_2::<T>(layer, id).into_iter().flatten()
    }
}

pub struct World<'a> {
    pub tile_field: &'a mut TileField,
    pub block_field: &'a mut BlockField,
    pub entity_field: &'a mut EntityField,
    pub node_store: &'a mut NodeStore,
    pub delegate_store: &'a DelegateStore,
}
