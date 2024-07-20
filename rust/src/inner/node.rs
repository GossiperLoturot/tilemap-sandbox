pub type NodeKey = (std::any::TypeId, u32, u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeRef {
    Global,
    /// `Tile(key)` means a reference to a tile instance corresponding to `key` in the
    /// `TileField`.
    Tile(u32),
    /// `Block(key)` means a reference to a block instance corresponding to `key` in the
    /// `BlockField`.
    Block(u32),
    /// `Entity(key)` means a reference to a entity instance corresponding to `key` in the
    /// `EntityField`.
    Entity(u32),
}

#[derive(Debug)]
struct NodeRow<T> {
    node: T,
    r#ref: NodeRef,
    ref_row_key: u32,
}

/// NodeColumn<T>(node_col, version)
type NodeColumn<T> = (slab::Slab<NodeRow<T>>, u64);

#[derive(Debug, Default)]
pub struct NodeStore {
    node_cols: Vec<Box<dyn std::any::Any>>,
    type_map: ahash::AHashMap<std::any::TypeId, u32>,
    type_ref_map: ahash::AHashMap<(std::any::TypeId, NodeRef), (u32, slab::Slab<u32>)>,
}

impl NodeStore {
    pub fn insert<T>(&mut self, node: T, r#ref: NodeRef) -> Option<NodeKey>
    where
        T: std::any::Any,
    {
        let type_key = std::any::TypeId::of::<T>();

        let col_key = *self.type_map.entry(type_key).or_insert_with(|| {
            // initialize a new column

            if self.node_cols.len() >= u32::MAX as usize {
                panic!("capacity overflow");
            }

            self.node_cols.push(Box::new(NodeColumn::<T>::default()));
            (self.node_cols.len() - 1) as u32
        });

        let node_col = self.node_cols.get_mut(col_key as usize).unwrap();
        let ptr = node_col as *mut _ as *mut Box<NodeColumn<T>>;
        let (node_col, version) = unsafe { &mut *ptr }.as_mut();

        if node_col.vacant_key() >= u32::MAX as usize {
            panic!("capacity overflow");
        }

        let row_key = node_col.vacant_key() as u32;

        let (_, row_keys) = self
            .type_ref_map
            .entry((type_key, r#ref))
            .or_insert_with(|| (col_key, Default::default()));
        // ref_row_key is guaranteed to be less than u32::MAX.
        let ref_row_key = row_keys.insert(row_key) as u32;

        node_col.insert(NodeRow {
            node,
            r#ref,
            ref_row_key,
        });
        *version += 1;

        Some((type_key, col_key, row_key))
    }

    pub fn remove<T>(&mut self, node_key: NodeKey) -> Option<(NodeRef, T)>
    where
        T: std::any::Any,
    {
        let (type_key, col_key, row_key) = node_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let node_col = self.node_cols.get_mut(col_key as usize).unwrap();
        let ptr = node_col as *mut _ as *mut Box<NodeColumn<T>>;
        let (node_col, version) = unsafe { &mut *ptr }.as_mut();

        let node_row = node_col.try_remove(row_key as usize)?;
        *version += 1;

        let (_, row_keys) = self
            .type_ref_map
            .get_mut(&(type_key, node_row.r#ref))
            .unwrap();
        row_keys.try_remove(node_row.ref_row_key as usize).unwrap();

        Some((node_row.r#ref, node_row.node))
    }

    pub fn get<T>(&self, node_key: NodeKey) -> Option<(&NodeRef, &T)>
    where
        T: std::any::Any,
    {
        let (type_key, col_key, row_key) = node_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let node_col = self.node_cols.get(col_key as usize).unwrap();
        let ptr = node_col as *const _ as *const Box<NodeColumn<T>>;
        let (node_col, _) = unsafe { &*ptr }.as_ref();

        let node_row = node_col.get(row_key as usize)?;

        Some((&node_row.r#ref, &node_row.node))
    }

    pub fn get_mut<T>(&mut self, node_key: NodeKey) -> Option<(&NodeRef, &mut T)>
    where
        T: std::any::Any,
    {
        let (type_key, col_key, row_key) = node_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let node_col = self.node_cols.get_mut(col_key as usize).unwrap();
        let ptr = node_col as *mut _ as *mut Box<NodeColumn<T>>;
        let (node_col, _) = unsafe { &mut *ptr }.as_mut();

        let node_row = node_col.get_mut(row_key as usize)?;

        Some((&node_row.r#ref, &mut node_row.node))
    }

    fn iter_internal<T>(&self) -> Option<impl Iterator<Item = (&NodeRef, &T)>>
    where
        T: std::any::Any,
    {
        let type_key = std::any::TypeId::of::<T>();

        let col_key = *self.type_map.get(&type_key)?;

        let node_col = self.node_cols.get(col_key as usize).unwrap();
        let ptr = node_col as *const _ as *const Box<NodeColumn<T>>;
        let (node_col, _) = unsafe { &*ptr }.as_ref();

        let iter = node_col
            .iter()
            .map(|(_, node_row)| (&node_row.r#ref, &node_row.node));

        Some(iter)
    }

    fn iter_mut_internal<T>(&mut self) -> Option<impl Iterator<Item = (&NodeRef, &mut T)>>
    where
        T: std::any::Any,
    {
        let type_key = std::any::TypeId::of::<T>();

        let col_key = *self.type_map.get(&type_key)?;

        let node_col = self.node_cols.get_mut(col_key as usize).unwrap();
        let ptr = node_col as *mut _ as *mut Box<NodeColumn<T>>;
        let (node_col, _) = unsafe { &mut *ptr }.as_mut();

        let iter = node_col
            .iter_mut()
            .map(|(_, node_row)| (&node_row.r#ref, &mut node_row.node));

        Some(iter)
    }

    fn iter_by_ref_internal<T>(&self, r#ref: NodeRef) -> Option<impl Iterator<Item = &T>>
    where
        T: std::any::Any,
    {
        let type_key = std::any::TypeId::of::<T>();

        let (col_key, row_keys) = self.type_ref_map.get(&(type_key, r#ref))?;

        let node_col = self.node_cols.get(*col_key as usize).unwrap();
        let ptr = node_col as *const _ as *const Box<NodeColumn<T>>;
        let (node_col, _) = unsafe { &*ptr }.as_ref();

        let iter = row_keys.iter().map(|(_, row_key)| {
            let node_row = node_col.get(*row_key as usize).unwrap();

            &node_row.node
        });

        Some(iter)
    }

    fn iter_mut_by_ref_internal<T>(
        &mut self,
        r#ref: NodeRef,
    ) -> Option<impl Iterator<Item = &mut T>>
    where
        T: std::any::Any,
    {
        let type_key = std::any::TypeId::of::<T>();

        let (col_key, row_keys) = self.type_ref_map.get(&(type_key, r#ref))?;

        let node_col = self.node_cols.get_mut(*col_key as usize).unwrap();
        let ptr = node_col as *mut _ as *mut Box<NodeColumn<T>>;
        let (node_col, _) = unsafe { &mut *ptr }.as_mut();

        let iter = row_keys.iter().map(|(_, row_key)| {
            let node_row = node_col.get_mut(*row_key as usize).unwrap();
            let ptr = node_row as *mut NodeRow<T>;
            let node_row = unsafe { &mut *ptr };

            &mut node_row.node
        });

        Some(iter)
    }

    fn remove_by_ref_internal<T>(&mut self, r#ref: NodeRef) -> Option<Vec<T>>
    where
        T: std::any::Any,
    {
        let type_key = std::any::TypeId::of::<T>();

        let (col_key, row_keys) = self.type_ref_map.remove(&(type_key, r#ref))?;

        let node_col = self.node_cols.get_mut(col_key as usize).unwrap();
        let ptr = node_col as *mut _ as *mut Box<NodeColumn<T>>;
        let (node_col, version) = unsafe { &mut *ptr }.as_mut();

        let nodes = row_keys
            .into_iter()
            .map(|(_, row_key)| {
                let node_row = node_col.try_remove(row_key as usize).unwrap();

                node_row.node
            })
            .collect::<Vec<_>>();
        *version += 1;

        Some(nodes)
    }

    #[inline]
    pub fn iter<T>(&self) -> impl Iterator<Item = (&NodeRef, &T)>
    where
        T: std::any::Any,
    {
        self.iter_internal::<T>().into_iter().flatten()
    }

    #[inline]
    pub fn iter_mut<T>(&mut self) -> impl Iterator<Item = (&NodeRef, &mut T)>
    where
        T: std::any::Any,
    {
        self.iter_mut_internal::<T>().into_iter().flatten()
    }

    #[inline]
    pub fn one<T>(&self) -> Option<(&NodeRef, &T)>
    where
        T: std::any::Any,
    {
        self.iter::<T>().next()
    }

    #[inline]
    pub fn one_mut<T>(&mut self) -> Option<(&NodeRef, &mut T)>
    where
        T: std::any::Any,
    {
        self.iter_mut::<T>().next()
    }

    #[inline]
    pub fn iter_by_ref<T>(&self, r#ref: NodeRef) -> impl Iterator<Item = &T>
    where
        T: std::any::Any,
    {
        self.iter_by_ref_internal::<T>(r#ref).into_iter().flatten()
    }

    #[inline]
    pub fn iter_mut_by_ref<T>(&mut self, r#ref: NodeRef) -> impl Iterator<Item = &mut T>
    where
        T: std::any::Any,
    {
        self.iter_mut_by_ref_internal::<T>(r#ref)
            .into_iter()
            .flatten()
    }

    #[inline]
    pub fn remove_by_ref<T>(&mut self, r#ref: NodeRef) -> Vec<T>
    where
        T: std::any::Any,
    {
        self.remove_by_ref_internal::<T>(r#ref).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud_node() {
        let mut store = NodeStore::default();
        let key = store.insert(42, NodeRef::Global).unwrap();

        assert_eq!(store.get::<i32>(key), Some((&NodeRef::Global, &42)));
        assert_eq!(store.get_mut::<i32>(key), Some((&NodeRef::Global, &mut 42)));
        assert_eq!(store.remove::<i32>(key), Some((NodeRef::Global, 42)));

        assert_eq!(store.get::<i32>(key), None);
        assert_eq!(store.get_mut::<i32>(key), None);
        assert_eq!(store.remove::<i32>(key), None);
    }

    #[test]
    fn node_with_invalid_type() {
        let mut store = NodeStore::default();
        let key = store.insert(42, NodeRef::Global).unwrap();

        assert!(store.get::<()>(key).is_none());
        assert!(store.get_mut::<()>(key).is_none());
        assert!(store.remove::<()>(key).is_none());
    }

    #[test]
    fn iter() {
        let mut store = NodeStore::default();
        store.insert(42, NodeRef::Tile(0)).unwrap();
        store.insert(63, NodeRef::Tile(0)).unwrap();
        store.insert(42, NodeRef::Tile(1)).unwrap();
        store.insert((), NodeRef::Tile(1)).unwrap();

        let mut iter = store.iter::<i32>();
        assert_eq!(iter.next(), Some((&NodeRef::Tile(0), &42)));
        assert_eq!(iter.next(), Some((&NodeRef::Tile(0), &63)));
        assert_eq!(iter.next(), Some((&NodeRef::Tile(1), &42)));
        assert_eq!(iter.next(), None);
        drop(iter);

        let mut iter = store.iter::<()>();
        assert_eq!(iter.next(), Some((&NodeRef::Tile(1), &())));
        assert_eq!(iter.next(), None);
        drop(iter);

        let mut iter = store.iter_mut::<i32>();
        assert_eq!(iter.next(), Some((&NodeRef::Tile(0), &mut 42)));
        assert_eq!(iter.next(), Some((&NodeRef::Tile(0), &mut 63)));
        assert_eq!(iter.next(), Some((&NodeRef::Tile(1), &mut 42)));
        assert_eq!(iter.next(), None);
        drop(iter);

        let mut iter = store.iter_mut::<()>();
        assert_eq!(iter.next(), Some((&NodeRef::Tile(1), &mut ())));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_with_invalid_type() {
        let mut store = NodeStore::default();
        assert!(store.iter::<i32>().next().is_none());
        assert!(store.iter_mut::<i32>().next().is_none());
    }

    #[test]
    fn one() {
        let mut store = NodeStore::default();
        store.insert(42, NodeRef::Global).unwrap();

        assert_eq!(store.one::<i32>(), Some((&NodeRef::Global, &42)));
        assert_eq!(store.one_mut::<i32>(), Some((&NodeRef::Global, &mut 42)));

        assert_eq!(store.one::<()>(), None);
        assert_eq!(store.one_mut::<()>(), None);
    }

    #[test]
    fn iter_by_ref() {
        let mut store = NodeStore::default();
        store.insert(42, NodeRef::Tile(0)).unwrap();
        store.insert(63, NodeRef::Tile(0)).unwrap();
        store.insert(42, NodeRef::Tile(1)).unwrap();
        store.insert((), NodeRef::Tile(1)).unwrap();

        let mut iter = store.iter_by_ref::<i32>(NodeRef::Tile(0));
        assert_eq!(iter.next(), Some(&42));
        assert_eq!(iter.next(), Some(&63));
        assert_eq!(iter.next(), None);
        drop(iter);

        let mut iter = store.iter_mut_by_ref::<i32>(NodeRef::Tile(1));
        assert_eq!(iter.next(), Some(&mut 42));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_by_ref_with_invalid_type() {
        let mut store = NodeStore::default();
        store.insert(42, NodeRef::Global).unwrap();

        assert!(store.iter_by_ref::<()>(NodeRef::Global).next().is_none());
        assert!(store
            .iter_mut_by_ref::<()>(NodeRef::Global)
            .next()
            .is_none());
    }

    #[test]
    fn iter_by_ref_with_invalid_ref() {
        let mut store = NodeStore::default();
        store.insert(42, NodeRef::Global).unwrap();

        assert!(store.iter_by_ref::<i32>(NodeRef::Tile(0)).next().is_none());
        assert!(store
            .iter_mut_by_ref::<i32>(NodeRef::Tile(0))
            .next()
            .is_none());
    }

    #[test]
    fn remove_by_ref() {
        let mut store = NodeStore::default();
        store.insert(42, NodeRef::Tile(0)).unwrap();
        store.insert(63, NodeRef::Tile(0)).unwrap();
        store.insert(42, NodeRef::Tile(1)).unwrap();
        store.insert((), NodeRef::Tile(1)).unwrap();

        let vec = store.remove_by_ref::<i32>(NodeRef::Tile(0));
        assert_eq!(vec, vec![42, 63]);

        let vec = store.remove_by_ref::<()>(NodeRef::Tile(1));
        assert_eq!(vec, vec![()]);

        let mut iter = store.iter::<i32>();
        assert_eq!(iter.next(), Some((&NodeRef::Tile(1), &42)));
        assert_eq!(iter.next(), None);
        drop(iter);

        let mut iter = store.iter::<()>();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn remove_by_ref_with_invalid_type() {
        let mut store = NodeStore::default();
        store.insert(42, NodeRef::Global).unwrap();

        assert_eq!(store.remove_by_ref::<()>(NodeRef::Global), vec![]);
    }

    #[test]
    fn remove_by_ref_with_invalid_ref() {
        let mut store = NodeStore::default();
        store.insert(42, NodeRef::Global).unwrap();

        assert_eq!(store.remove_by_ref::<i32>(NodeRef::Tile(0)), vec![]);
    }
}
