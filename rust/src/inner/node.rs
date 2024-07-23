use super::*;

pub type NodeKey = (u32, u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RefKey {
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

const CHUNK_SIZE: u32 = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SpcKeyKind {
    Global,
    Local(IVec2),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpcKey(SpcKeyKind);

impl SpcKey {
    pub const GLOBAL: SpcKey = SpcKey(SpcKeyKind::Global);
}

impl From<[i32; 2]> for SpcKey {
    fn from([x, y]: [i32; 2]) -> Self {
        SpcKey(SpcKeyKind::Local([
            x.div_euclid(CHUNK_SIZE as i32),
            y.div_euclid(CHUNK_SIZE as i32),
        ]))
    }
}

impl From<[f32; 2]> for SpcKey {
    fn from([x, y]: [f32; 2]) -> Self {
        SpcKey(SpcKeyKind::Local([
            x.div_euclid(CHUNK_SIZE as f32) as i32,
            y.div_euclid(CHUNK_SIZE as f32) as i32,
        ]))
    }
}

#[derive(Debug)]
struct NodeRow<T> {
    node: Option<T>,
    typ_row_key: u32,
    r#ref: RefKey,
    ref_row_key: u32,
    spc: SpcKey,
    spc_row_key: u32,
}

type NodeColumn<T> = slab::Slab<NodeRow<T>>;

#[derive(Debug, Default)]
pub struct NodeStore {
    node_cols: Vec<(std::any::TypeId, Box<dyn std::any::Any>)>,
    typ_map: ahash::AHashMap<std::any::TypeId, (u32, slab::Slab<NodeKey>)>,
    typ_ref_map: ahash::AHashMap<(std::any::TypeId, RefKey), (u32, slab::Slab<NodeKey>)>,
    typ_spc_map: ahash::AHashMap<(std::any::TypeId, SpcKey), (u32, slab::Slab<NodeKey>)>,
}

impl NodeStore {
    pub fn insert<T>(&mut self, r#ref: RefKey, spc: SpcKey, node: T) -> Option<NodeKey>
    where
        T: std::any::Any,
    {
        let typ = std::any::TypeId::of::<T>();

        let (col_key, row_keys) = self.typ_map.entry(typ).or_insert_with(|| {
            // initialize a new column

            let col_key = self.node_cols.len();
            if col_key >= u32::MAX as usize {
                panic!("capacity overflow");
            }

            self.node_cols
                .push((typ, Box::new(NodeColumn::<T>::default())));
            (col_key as u32, Default::default())
        });
        let col_key = *col_key;

        let (_, node_col) = self.node_cols.get_mut(col_key as usize).unwrap();

        let ptr = node_col as *mut _ as *mut Box<NodeColumn<T>>;
        let node_col = unsafe { &mut *ptr }.as_mut();

        // row_key is guaranteed to be less than u32::MAX.
        let row_key = node_col.vacant_key() as u32;

        // typ_row_key is guaranteed to be less than u32::MAX.
        let typ_row_key = row_keys.insert((col_key, row_key)) as u32;

        let (_, row_keys) = self
            .typ_ref_map
            .entry((typ, r#ref))
            .or_insert_with(|| (col_key, Default::default()));
        // ref_row_key is guaranteed to be less than u32::MAX.
        let ref_row_key = row_keys.insert((col_key, row_key)) as u32;

        let (_, row_keys) = self
            .typ_spc_map
            .entry((typ, spc))
            .or_insert_with(|| (col_key, Default::default()));
        // spc_row_key is guaranteed to be less than u32::MAX.
        let spc_row_key = row_keys.insert((col_key, row_key)) as u32;

        node_col.insert(NodeRow {
            node: Some(node),
            typ_row_key,
            r#ref,
            ref_row_key,
            spc,
            spc_row_key,
        });

        Some((col_key, row_key))
    }

    pub fn remove<T>(&mut self, node_key: NodeKey) -> Option<(RefKey, SpcKey, T)>
    where
        T: std::any::Any,
    {
        let (col_key, row_key) = node_key;

        let (typ, node_col) = self.node_cols.get_mut(col_key as usize).unwrap();

        if typ != &std::any::TypeId::of::<T>() {
            return None;
        }

        let ptr = node_col as *mut _ as *mut Box<NodeColumn<T>>;
        let node_col = unsafe { &mut *ptr }.as_mut();

        let node_row = node_col.try_remove(row_key as usize)?;
        let node = node_row.node.expect("node was popped");

        let (_, row_keys) = self.typ_map.get_mut(typ).unwrap();
        row_keys.try_remove(node_row.typ_row_key as usize).unwrap();

        let (_, row_keys) = self.typ_ref_map.get_mut(&(*typ, node_row.r#ref)).unwrap();
        row_keys.try_remove(node_row.ref_row_key as usize).unwrap();

        let (_, row_keys) = self.typ_spc_map.get_mut(&(*typ, node_row.spc)).unwrap();
        row_keys.try_remove(node_row.spc_row_key as usize).unwrap();

        Some((node_row.r#ref, node_row.spc, node))
    }

    pub fn get<T>(&self, node_key: NodeKey) -> Option<(&RefKey, &SpcKey, &T)>
    where
        T: std::any::Any,
    {
        let (col_key, row_key) = node_key;

        let (typ, node_col) = self.node_cols.get(col_key as usize)?;

        if typ != &std::any::TypeId::of::<T>() {
            return None;
        }

        let ptr = node_col as *const _ as *const Box<NodeColumn<T>>;
        let node_col = unsafe { &*ptr }.as_ref();

        let node_row = node_col.get(row_key as usize)?;
        let node = node_row.node.as_ref().expect("node was popped");

        Some((&node_row.r#ref, &node_row.spc, node))
    }

    pub fn modify<T, F>(&mut self, node_key: NodeKey, f: F) -> Option<()>
    where
        T: std::any::Any,
        F: FnOnce(&mut RefKey, &mut SpcKey, &mut T),
    {
        let (col_key, row_key) = node_key;

        let (typ, node_col) = self.node_cols.get_mut(col_key as usize)?;

        if typ != &std::any::TypeId::of::<T>() {
            return None;
        }

        let ptr = node_col as *mut _ as *mut Box<NodeColumn<T>>;
        let node_col = unsafe { &mut *ptr }.as_mut();

        let node_row = node_col.get_mut(row_key as usize)?;
        let node = node_row.node.as_mut().expect("node was popped");

        let (prev_ref, prev_spc) = (node_row.r#ref, node_row.spc);
        f(&mut node_row.r#ref, &mut node_row.spc, node);

        if prev_ref != node_row.r#ref {
            let (_, row_keys) = self.typ_ref_map.get_mut(&(*typ, prev_ref)).unwrap();
            row_keys.try_remove(node_row.ref_row_key as usize).unwrap();

            let (_, row_keys) = self
                .typ_ref_map
                .entry((*typ, node_row.r#ref))
                .or_insert((col_key, Default::default()));
            // ref_row_key is guaranteed to be less than u32::MAX.
            let ref_row_key = row_keys.insert((col_key, node_row.ref_row_key)) as u32;

            node_row.ref_row_key = ref_row_key;
        }

        if prev_spc != node_row.spc {
            let (_, row_keys) = self.typ_spc_map.get_mut(&(*typ, prev_spc)).unwrap();
            row_keys.try_remove(node_row.spc_row_key as usize).unwrap();

            let (_, row_keys) = self
                .typ_spc_map
                .entry((*typ, node_row.spc))
                .or_insert((col_key, Default::default()));
            // spc_row_key is guaranteed to be less than u32::MAX.
            let spc_row_key = row_keys.insert((col_key, node_row.spc_row_key)) as u32;

            node_row.spc_row_key = spc_row_key;
        }

        Some(())
    }

    pub fn pop<T>(&mut self, node_key: NodeKey) -> Option<(RefKey, SpcKey, T)>
    where
        T: std::any::Any,
    {
        let (col_key, row_key) = node_key;

        let (typ, node_col) = self.node_cols.get_mut(col_key as usize)?;

        if typ != &std::any::TypeId::of::<T>() {
            return None;
        }

        let ptr = node_col as *mut _ as *mut Box<NodeColumn<T>>;
        let node_col = unsafe { &mut *ptr }.as_mut();

        let node_row = node_col.get_mut(row_key as usize)?;
        let node = node_row.node.take().expect("node was already popped");

        Some((node_row.r#ref, node_row.spc, node))
    }

    pub fn push<T>(&mut self, node_key: NodeKey, r#ref: RefKey, spc: SpcKey, node: T) -> Option<()>
    where
        T: std::any::Any,
    {
        let (col_key, row_key) = node_key;

        let (typ, node_col) = self.node_cols.get_mut(col_key as usize)?;

        if typ != &std::any::TypeId::of::<T>() {
            return None;
        }

        let ptr = node_col as *mut _ as *mut Box<NodeColumn<T>>;
        let node_col = unsafe { &mut *ptr }.as_mut();

        let node_row = node_col.get_mut(row_key as usize)?;
        let old_node = std::mem::replace(&mut node_row.node, Some(node));
        if old_node.is_some() {
            panic!("node was pushed");
        }

        let (prev_ref, prev_spc) = (node_row.r#ref, node_row.spc);
        (node_row.r#ref, node_row.spc) = (r#ref, spc);

        if prev_ref != node_row.r#ref {
            let (_, row_keys) = self.typ_ref_map.get_mut(&(*typ, prev_ref)).unwrap();
            row_keys.try_remove(node_row.ref_row_key as usize).unwrap();

            let (_, row_keys) = self
                .typ_ref_map
                .entry((*typ, node_row.r#ref))
                .or_insert((col_key, Default::default()));
            // ref_row_key is guaranteed to be less than u32::MAX.
            let ref_row_key = row_keys.insert((col_key, node_row.ref_row_key)) as u32;

            node_row.ref_row_key = ref_row_key;
        }

        if prev_spc != node_row.spc {
            let (_, row_keys) = self.typ_spc_map.get_mut(&(*typ, prev_spc)).unwrap();
            row_keys.try_remove(node_row.spc_row_key as usize).unwrap();

            let (_, row_keys) = self
                .typ_spc_map
                .entry((*typ, node_row.spc))
                .or_insert((col_key, Default::default()));
            // spc_row_key is guaranteed to be less than u32::MAX.
            let spc_row_key = row_keys.insert((col_key, node_row.spc_row_key)) as u32;

            node_row.spc_row_key = spc_row_key;
        }

        Some(())
    }

    fn iter_internal<T>(&self) -> Option<impl Iterator<Item = &NodeKey>>
    where
        T: std::any::Any,
    {
        let typ = std::any::TypeId::of::<T>();
        let (_, row_keys) = self.typ_map.get(&typ)?;
        Some(row_keys.iter().map(|(_, v)| v))
    }

    fn detach_iter_internal<T>(&self) -> Option<Vec<NodeKey>>
    where
        T: std::any::Any,
    {
        let typ = std::any::TypeId::of::<T>();
        let (_, row_keys) = self.typ_map.get(&typ)?;
        let iter = row_keys.iter().map(|(_, v)| *v).collect::<Vec<_>>();
        Some(iter)
    }

    fn iter_by_ref_internal<T>(&self, r#ref: RefKey) -> Option<impl Iterator<Item = &NodeKey>>
    where
        T: std::any::Any,
    {
        let typ = std::any::TypeId::of::<T>();
        let (_, row_keys) = self.typ_ref_map.get(&(typ, r#ref))?;
        Some(row_keys.iter().map(|(_, v)| v))
    }

    fn detach_iter_by_ref_internal<T>(&self, r#ref: RefKey) -> Option<Vec<NodeKey>>
    where
        T: std::any::Any,
    {
        let typ = std::any::TypeId::of::<T>();
        let (_, row_keys) = self.typ_ref_map.get(&(typ, r#ref))?;
        let iter = row_keys.iter().map(|(_, v)| *v).collect::<Vec<_>>();
        Some(iter)
    }

    fn iter_by_rect_internal<T>(&self, rect: [Vec2; 2]) -> Option<impl Iterator<Item = &NodeKey>>
    where
        T: std::any::Any,
    {
        let typ = std::any::TypeId::of::<T>();

        let min = match SpcKey::from(rect[0]) {
            SpcKey(SpcKeyKind::Local(chunk_key)) => chunk_key,
            _ => unreachable!(),
        };
        let max = match SpcKey::from(rect[0]) {
            SpcKey(SpcKeyKind::Local(chunk_key)) => chunk_key,
            _ => unreachable!(),
        };

        let mut iters = vec![];
        for y in min[1]..max[1] {
            for x in min[0]..max[0] {
                let spc = SpcKey(SpcKeyKind::Local([x, y]));
                let (_, row_keys) = self.typ_spc_map.get(&(typ, spc))?;
                iters.push(row_keys.iter().map(|(_, v)| v));
            }
        }

        Some(iters.into_iter().flatten())
    }

    fn detach_iter_by_rect_internal<T>(&self, rect: [Vec2; 2]) -> Option<Vec<NodeKey>>
    where
        T: std::any::Any,
    {
        let typ = std::any::TypeId::of::<T>();

        let min = match SpcKey::from(rect[0]) {
            SpcKey(SpcKeyKind::Local(chunk_key)) => chunk_key,
            _ => unreachable!(),
        };
        let max = match SpcKey::from(rect[0]) {
            SpcKey(SpcKeyKind::Local(chunk_key)) => chunk_key,
            _ => unreachable!(),
        };

        let mut iters = vec![];
        for y in min[1]..max[1] {
            for x in min[0]..max[0] {
                let spc = SpcKey(SpcKeyKind::Local([x, y]));
                let (_, row_keys) = self.typ_spc_map.get(&(typ, spc))?;
                iters.push(row_keys.iter().map(|(_, v)| *v));
            }
        }

        Some(iters.into_iter().flatten().collect::<Vec<_>>())
    }

    fn remove_by_ref_internal<T>(&mut self, r#ref: RefKey) -> Option<Vec<(RefKey, SpcKey, T)>>
    where
        T: std::any::Any,
    {
        let typ = std::any::TypeId::of::<T>();
        let (col_key, row_keys) = self.typ_ref_map.remove(&(typ, r#ref))?;

        let (_, node_col) = self.node_cols.get_mut(col_key as usize).unwrap();

        let ptr = node_col as *mut _ as *mut Box<NodeColumn<T>>;
        let node_col = unsafe { &mut *ptr }.as_mut();

        let nodes = row_keys
            .into_iter()
            .map(|(_, (_, row_key))| {
                let node_row = node_col.try_remove(row_key as usize).unwrap();
                let node = node_row.node.expect("node was popped");

                let (_, row_keys) = self.typ_map.get_mut(&typ).unwrap();
                // typ_row_key is guaranteed to be less than u32::MAX.
                row_keys.try_remove(node_row.typ_row_key as usize).unwrap();

                let (_, row_keys) = self.typ_spc_map.get_mut(&(typ, node_row.spc)).unwrap();
                // spc_row_key is guaranteed to be less than u32::MAX.
                row_keys.try_remove(node_row.spc_row_key as usize).unwrap();

                (node_row.r#ref, node_row.spc, node)
            })
            .collect::<Vec<_>>();
        Some(nodes)
    }

    #[inline]
    pub fn iter<T>(&self) -> impl Iterator<Item = &NodeKey>
    where
        T: std::any::Any,
    {
        self.iter_internal::<T>().into_iter().flatten()
    }

    #[inline]
    pub fn detach_iter<T>(&self) -> Vec<NodeKey>
    where
        T: std::any::Any,
    {
        self.detach_iter_internal::<T>().unwrap_or_default()
    }

    #[inline]
    pub fn iter_by_ref<T>(&self, r#ref: RefKey) -> impl Iterator<Item = &NodeKey>
    where
        T: std::any::Any,
    {
        self.iter_by_ref_internal::<T>(r#ref).into_iter().flatten()
    }

    #[inline]
    pub fn detach_iter_by_ref<T>(&self, r#ref: RefKey) -> Vec<NodeKey>
    where
        T: std::any::Any,
    {
        self.detach_iter_by_ref_internal::<T>(r#ref)
            .unwrap_or_default()
    }

    #[inline]
    pub fn iter_by_rect<T>(&self, rect: [Vec2; 2]) -> impl Iterator<Item = &NodeKey>
    where
        T: std::any::Any,
    {
        self.iter_by_rect_internal::<T>(rect).into_iter().flatten()
    }

    #[inline]
    pub fn detach_iter_by_rect<T>(&self, rect: [Vec2; 2]) -> Vec<NodeKey>
    where
        T: std::any::Any,
    {
        self.detach_iter_by_rect_internal::<T>(rect)
            .unwrap_or_default()
    }

    #[inline]
    pub fn remove_by_ref<T>(&mut self, r#ref: RefKey) -> Vec<(RefKey, SpcKey, T)>
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
        let key = store.insert(RefKey::Global, SpcKey::GLOBAL, 42).unwrap();

        assert_eq!(
            store.get::<i32>(key),
            Some((&RefKey::Global, &SpcKey::GLOBAL, &42))
        );
        assert_eq!(
            store.remove::<i32>(key),
            Some((RefKey::Global, SpcKey::GLOBAL, 42))
        );

        assert_eq!(store.get::<i32>(key), None);
        assert_eq!(store.remove::<i32>(key), None);
    }

    #[test]
    fn node_with_invalid_type() {
        let mut store = NodeStore::default();
        let key = store.insert(RefKey::Global, SpcKey::GLOBAL, 42).unwrap();

        assert!(store.get::<()>(key).is_none());
        assert!(store.remove::<()>(key).is_none());
    }

    #[test]
    fn iter() {
        let mut store = NodeStore::default();
        let k0 = store.insert(RefKey::Tile(0), SpcKey::GLOBAL, 42).unwrap();
        let k1 = store.insert(RefKey::Tile(0), SpcKey::GLOBAL, 63).unwrap();
        let k2 = store.insert(RefKey::Tile(1), SpcKey::GLOBAL, 42).unwrap();
        let k3 = store.insert(RefKey::Tile(1), SpcKey::GLOBAL, ()).unwrap();

        let mut iter = store.iter::<i32>();
        assert_eq!(iter.next(), Some(&k0));
        assert_eq!(iter.next(), Some(&k1));
        assert_eq!(iter.next(), Some(&k2));
        assert_eq!(iter.next(), None);
        drop(iter);

        let mut iter = store.iter::<()>();
        assert_eq!(iter.next(), Some(&k3));
        assert_eq!(iter.next(), None);
        drop(iter);
    }

    #[test]
    fn iter_with_invalid_type() {
        let store = NodeStore::default();
        assert!(store.iter::<i32>().next().is_none());
    }

    #[test]
    fn iter_by_ref() {
        let mut store = NodeStore::default();
        let k0 = store.insert(RefKey::Tile(0), SpcKey::GLOBAL, 42).unwrap();
        let k1 = store.insert(RefKey::Tile(0), SpcKey::GLOBAL, 63).unwrap();
        let k2 = store.insert(RefKey::Tile(1), SpcKey::GLOBAL, 42).unwrap();
        let _k3 = store.insert(RefKey::Tile(1), SpcKey::GLOBAL, ()).unwrap();

        let mut iter = store.iter_by_ref::<i32>(RefKey::Tile(0));
        assert_eq!(iter.next(), Some(&k0));
        assert_eq!(iter.next(), Some(&k1));
        assert_eq!(iter.next(), None);
        drop(iter);

        let mut iter = store.iter_by_ref::<i32>(RefKey::Tile(1));
        assert_eq!(iter.next(), Some(&k2));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_by_ref_with_invalid_type() {
        let mut store = NodeStore::default();
        store.insert(RefKey::Global, SpcKey::GLOBAL, 42).unwrap();

        assert!(store.iter_by_ref::<()>(RefKey::Global).next().is_none());
    }

    #[test]
    fn iter_by_ref_with_invalid_ref() {
        let mut store = NodeStore::default();
        store.insert(RefKey::Global, SpcKey::GLOBAL, 42).unwrap();

        assert!(store.iter_by_ref::<i32>(RefKey::Tile(0)).next().is_none());
    }

    #[test]
    fn remove_by_ref() {
        let mut store = NodeStore::default();
        let _k1 = store.insert(RefKey::Tile(0), SpcKey::GLOBAL, 42).unwrap();
        let _k2 = store.insert(RefKey::Tile(0), SpcKey::GLOBAL, 63).unwrap();
        let k3 = store.insert(RefKey::Tile(1), SpcKey::GLOBAL, 42).unwrap();
        let _k4 = store.insert(RefKey::Tile(1), SpcKey::GLOBAL, ()).unwrap();

        let vec = store.remove_by_ref::<i32>(RefKey::Tile(0));
        assert_eq!(
            vec,
            vec![
                (RefKey::Tile(0), SpcKey::GLOBAL, 42),
                (RefKey::Tile(0), SpcKey::GLOBAL, 63)
            ]
        );

        let vec = store.remove_by_ref::<()>(RefKey::Tile(1));
        assert_eq!(vec, vec![(RefKey::Tile(1), SpcKey::GLOBAL, ())]);

        let mut iter = store.iter::<i32>();
        assert_eq!(iter.next(), Some(&k3));
        assert_eq!(iter.next(), None);
        drop(iter);

        let mut iter = store.iter::<()>();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn remove_by_ref_with_invalid_type() {
        let mut store = NodeStore::default();
        store.insert(RefKey::Global, SpcKey::GLOBAL, 42).unwrap();

        assert_eq!(store.remove_by_ref::<()>(RefKey::Global), vec![]);
    }

    #[test]
    fn remove_by_ref_with_invalid_ref() {
        let mut store = NodeStore::default();
        store.insert(RefKey::Global, SpcKey::GLOBAL, 42).unwrap();

        assert_eq!(store.remove_by_ref::<i32>(RefKey::Tile(0)), vec![]);
    }
}
