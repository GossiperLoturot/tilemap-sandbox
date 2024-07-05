pub type NodeKey = (std::any::TypeId, u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeRelation {
    Global,
    Tile(u32),
    Block(u32),
    Entity(u32),
}

#[derive(Debug)]
struct NodeRow<T> {
    node: T,
    relation: NodeRelation,
    ref_row_key: u32,
}

type NodeColumn<T> = slab::Slab<NodeRow<T>>;

const ALLOC_SIZE: usize = std::mem::size_of::<NodeColumn<()>>();

#[derive(Debug, Default)]
pub struct NodeStore {
    node_cols: ahash::AHashMap<std::any::TypeId, stack_any::StackAny<ALLOC_SIZE>>,
    ref_cols: ahash::AHashMap<(NodeRelation, std::any::TypeId), slab::Slab<u32>>,
}

impl NodeStore {
    pub fn insert<T>(&mut self, node: T, relation: NodeRelation) -> Option<NodeKey>
    where
        T: std::any::Any,
    {
        let type_key = std::any::TypeId::of::<T>();

        let node_col = self
            .node_cols
            .entry(type_key)
            .or_insert_with(|| stack_any::StackAny::try_new(NodeColumn::<T>::new()).unwrap())
            .downcast_mut::<NodeColumn<T>>()
            .unwrap();

        let row_key = node_col.vacant_key() as u32;

        let ref_row_key = self
            .ref_cols
            .entry((relation, type_key))
            .or_default()
            .insert(row_key) as u32;

        node_col.insert(NodeRow {
            node,
            relation,
            ref_row_key,
        });

        Some((type_key, row_key))
    }

    pub fn remove<T>(&mut self, node_key: NodeKey) -> Option<(NodeRelation, T)>
    where
        T: std::any::Any,
    {
        let (type_key, row_key) = node_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let node_col = self
            .node_cols
            .get_mut(&type_key)?
            .downcast_mut::<NodeColumn<T>>()
            .unwrap();

        let node_row = node_col.try_remove(row_key as usize)?;

        self.ref_cols
            .get_mut(&(node_row.relation, type_key))
            .unwrap()
            .try_remove(node_row.ref_row_key as usize)
            .unwrap();

        Some((node_row.relation, node_row.node))
    }

    pub fn get<T>(&self, node_key: NodeKey) -> Option<(&NodeRelation, &T)>
    where
        T: std::any::Any,
    {
        let (type_key, row_key) = node_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let node_col = self
            .node_cols
            .get(&type_key)?
            .downcast_ref::<NodeColumn<T>>()
            .unwrap();

        let node_row = node_col.get(row_key as usize)?;

        Some((&node_row.relation, &node_row.node))
    }

    pub fn get_mut<T>(&mut self, node_key: NodeKey) -> Option<(&NodeRelation, &mut T)>
    where
        T: std::any::Any,
    {
        let (type_key, row_key) = node_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let node_col = self
            .node_cols
            .get_mut(&type_key)?
            .downcast_mut::<NodeColumn<T>>()
            .unwrap();

        let node_row = node_col.get_mut(row_key as usize)?;

        Some((&node_row.relation, &mut node_row.node))
    }

    fn iter_internal<T>(&self) -> Option<impl Iterator<Item = (&NodeRelation, &T)>>
    where
        T: std::any::Any,
    {
        let type_key = std::any::TypeId::of::<T>();

        let node_col = self
            .node_cols
            .get(&type_key)?
            .downcast_ref::<NodeColumn<T>>()
            .unwrap();

        let iter = node_col
            .iter()
            .map(|(_, node_row)| (&node_row.relation, &node_row.node));

        Some(iter)
    }

    fn iter_mut_internal<T>(&mut self) -> Option<impl Iterator<Item = (&NodeRelation, &mut T)>>
    where
        T: std::any::Any,
    {
        let type_key = std::any::TypeId::of::<T>();

        let node_col = self
            .node_cols
            .get_mut(&type_key)?
            .downcast_mut::<NodeColumn<T>>()
            .unwrap();

        let iter = node_col
            .iter_mut()
            .map(|(_, node_row)| (&node_row.relation, &mut node_row.node));

        Some(iter)
    }

    fn iter_by_relation_internal<T>(
        &self,
        relation: NodeRelation,
    ) -> Option<impl Iterator<Item = &T>>
    where
        T: std::any::Any,
    {
        let type_key = std::any::TypeId::of::<T>();

        let node_col = self
            .node_cols
            .get(&type_key)?
            .downcast_ref::<NodeColumn<T>>()
            .unwrap();

        let iter = self
            .ref_cols
            .get(&(relation, type_key))?
            .iter()
            .map(|(_, row_key)| {
                let node_row = node_col.get(*row_key as usize).unwrap();
                &node_row.node
            });

        Some(iter)
    }

    fn iter_mut_by_relation_internal<T>(
        &mut self,
        relation: NodeRelation,
    ) -> Option<impl Iterator<Item = &mut T>>
    where
        T: std::any::Any,
    {
        let type_key = std::any::TypeId::of::<T>();

        let node_col = self
            .node_cols
            .get_mut(&type_key)?
            .downcast_mut::<NodeColumn<T>>()
            .unwrap();

        let iter = self
            .ref_cols
            .get(&(relation, type_key))?
            .iter()
            .map(|(_, row_key)| {
                let node_row = node_col.get_mut(*row_key as usize).unwrap() as *mut NodeRow<T>;
                let node_row = unsafe { &mut *node_row };
                &mut node_row.node
            });

        Some(iter)
    }

    fn remove_by_relation_internal<T>(&mut self, relation: NodeRelation) -> Option<Vec<T>>
    where
        T: std::any::Any,
    {
        let type_key = std::any::TypeId::of::<T>();

        let node_col = self
            .node_cols
            .get_mut(&type_key)?
            .downcast_mut::<NodeColumn<T>>()
            .unwrap();

        let mut vec = vec![];
        if let Some(ref_col) = self.ref_cols.remove(&(relation, type_key)) {
            for (_, row_key) in ref_col {
                let node_row = node_col.try_remove(row_key as usize).unwrap();
                vec.push(node_row.node);
            }
        }

        Some(vec)
    }

    #[inline]
    pub fn iter<T>(&self) -> impl Iterator<Item = (&NodeRelation, &T)>
    where
        T: std::any::Any,
    {
        self.iter_internal::<T>().into_iter().flatten()
    }

    #[inline]
    pub fn iter_mut<T>(&mut self) -> impl Iterator<Item = (&NodeRelation, &mut T)>
    where
        T: std::any::Any,
    {
        self.iter_mut_internal::<T>().into_iter().flatten()
    }

    #[inline]
    pub fn one<T>(&self) -> Option<(&NodeRelation, &T)>
    where
        T: std::any::Any,
    {
        self.iter::<T>().next()
    }

    #[inline]
    pub fn one_mut<T>(&mut self) -> Option<(&NodeRelation, &mut T)>
    where
        T: std::any::Any,
    {
        self.iter_mut::<T>().next()
    }

    #[inline]
    pub fn iter_by_relation<T>(&self, relation: NodeRelation) -> impl Iterator<Item = &T>
    where
        T: std::any::Any,
    {
        self.iter_by_relation_internal::<T>(relation)
            .into_iter()
            .flatten()
    }

    #[inline]
    pub fn iter_mut_by_relation<T>(
        &mut self,
        relation: NodeRelation,
    ) -> impl Iterator<Item = &mut T>
    where
        T: std::any::Any,
    {
        self.iter_mut_by_relation_internal::<T>(relation)
            .into_iter()
            .flatten()
    }

    #[inline]
    pub fn remove_by_relation<T>(&mut self, relation: NodeRelation) -> Vec<T>
    where
        T: std::any::Any,
    {
        self.remove_by_relation_internal::<T>(relation)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud_comp() {
        let mut store = NodeStore::default();
        let key = store.insert(42, NodeRelation::Global).unwrap();

        assert_eq!(store.get::<i32>(key), Some((&NodeRelation::Global, &42)));
        assert_eq!(
            store.get_mut::<i32>(key),
            Some((&NodeRelation::Global, &mut 42))
        );
        assert_eq!(store.remove::<i32>(key), Some((NodeRelation::Global, 42)));

        assert_eq!(store.get::<i32>(key), None);
        assert_eq!(store.get_mut::<i32>(key), None);
        assert_eq!(store.remove::<i32>(key), None);
    }

    #[test]
    fn comp_with_invalid_type() {
        let mut store = NodeStore::default();
        let key = store.insert(42, NodeRelation::Global).unwrap();

        assert!(store.get::<()>(key).is_none());
        assert!(store.get_mut::<()>(key).is_none());
        assert!(store.remove::<()>(key).is_none());
    }

    #[test]
    fn iter() {
        let mut store = NodeStore::default();
        store.insert(42, NodeRelation::Tile(0)).unwrap();
        store.insert(63, NodeRelation::Tile(0)).unwrap();
        store.insert(42, NodeRelation::Tile(1)).unwrap();
        store.insert((), NodeRelation::Tile(1)).unwrap();

        let mut iter = store.iter::<i32>();
        assert_eq!(iter.next(), Some((&NodeRelation::Tile(0), &42)));
        assert_eq!(iter.next(), Some((&NodeRelation::Tile(0), &63)));
        assert_eq!(iter.next(), Some((&NodeRelation::Tile(1), &42)));
        assert_eq!(iter.next(), None);
        drop(iter);

        let mut iter = store.iter::<()>();
        assert_eq!(iter.next(), Some((&NodeRelation::Tile(1), &())));
        assert_eq!(iter.next(), None);
        drop(iter);

        let mut iter = store.iter_mut::<i32>();
        assert_eq!(iter.next(), Some((&NodeRelation::Tile(0), &mut 42)));
        assert_eq!(iter.next(), Some((&NodeRelation::Tile(0), &mut 63)));
        assert_eq!(iter.next(), Some((&NodeRelation::Tile(1), &mut 42)));
        assert_eq!(iter.next(), None);
        drop(iter);

        let mut iter = store.iter_mut::<()>();
        assert_eq!(iter.next(), Some((&NodeRelation::Tile(1), &mut ())));
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
        store.insert(42, NodeRelation::Global).unwrap();

        assert_eq!(store.one::<i32>(), Some((&NodeRelation::Global, &42)));
        assert_eq!(
            store.one_mut::<i32>(),
            Some((&NodeRelation::Global, &mut 42))
        );

        assert_eq!(store.one::<()>(), None);
        assert_eq!(store.one_mut::<()>(), None);
    }

    #[test]
    fn iter_by_relation() {
        let mut store = NodeStore::default();
        store.insert(42, NodeRelation::Tile(0)).unwrap();
        store.insert(63, NodeRelation::Tile(0)).unwrap();
        store.insert(42, NodeRelation::Tile(1)).unwrap();
        store.insert((), NodeRelation::Tile(1)).unwrap();

        let mut iter = store.iter_by_relation::<i32>(NodeRelation::Tile(0));
        assert_eq!(iter.next(), Some(&42));
        assert_eq!(iter.next(), Some(&63));
        assert_eq!(iter.next(), None);
        drop(iter);

        let mut iter = store.iter_mut_by_relation::<i32>(NodeRelation::Tile(1));
        assert_eq!(iter.next(), Some(&mut 42));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_by_relation_with_invalid_type() {
        let mut store = NodeStore::default();
        store.insert(42, NodeRelation::Global).unwrap();

        assert!(store
            .iter_by_relation::<()>(NodeRelation::Global)
            .next()
            .is_none());
        assert!(store
            .iter_mut_by_relation::<()>(NodeRelation::Global)
            .next()
            .is_none());
    }

    #[test]
    fn iter_by_relation_with_invalid_relation() {
        let mut store = NodeStore::default();
        store.insert(42, NodeRelation::Global).unwrap();

        assert!(store
            .iter_by_relation::<i32>(NodeRelation::Tile(0))
            .next()
            .is_none());
        assert!(store
            .iter_mut_by_relation::<i32>(NodeRelation::Tile(0))
            .next()
            .is_none());
    }

    #[test]
    fn remove_by_relation() {
        let mut store = NodeStore::default();
        store.insert(42, NodeRelation::Tile(0)).unwrap();
        store.insert(63, NodeRelation::Tile(0)).unwrap();
        store.insert(42, NodeRelation::Tile(1)).unwrap();
        store.insert((), NodeRelation::Tile(1)).unwrap();

        let vec = store.remove_by_relation::<i32>(NodeRelation::Tile(0));
        assert_eq!(vec, vec![42, 63]);

        let vec = store.remove_by_relation::<()>(NodeRelation::Tile(1));
        assert_eq!(vec, vec![()]);

        let mut iter = store.iter::<i32>();
        assert_eq!(iter.next(), Some((&NodeRelation::Tile(1), &42)));
        assert_eq!(iter.next(), None);
        drop(iter);

        let mut iter = store.iter::<()>();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn remove_by_relation_with_invalid_type() {
        let mut store = NodeStore::default();
        store.insert(42, NodeRelation::Global).unwrap();

        assert_eq!(store.remove_by_relation::<()>(NodeRelation::Global), vec![]);
    }

    #[test]
    fn remove_by_relation_with_invalid_relation() {
        let mut store = NodeStore::default();
        store.insert(42, NodeRelation::Global).unwrap();

        assert_eq!(
            store.remove_by_relation::<i32>(NodeRelation::Tile(0)),
            vec![]
        );
    }
}
