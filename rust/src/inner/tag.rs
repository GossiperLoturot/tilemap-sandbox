use super::*;

pub type TagKey = (u32, u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RefKey {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpaceKey(IVec2);

impl SpaceKey {
    const CHUNK_SIZE: u32 = 32;
}

impl From<IVec2> for SpaceKey {
    fn from([x, y]: [i32; 2]) -> Self {
        SpaceKey([
            x.div_euclid(Self::CHUNK_SIZE as i32),
            y.div_euclid(Self::CHUNK_SIZE as i32),
        ])
    }
}

impl From<Vec2> for SpaceKey {
    fn from([x, y]: [f32; 2]) -> Self {
        SpaceKey([
            x.div_euclid(Self::CHUNK_SIZE as f32) as i32,
            y.div_euclid(Self::CHUNK_SIZE as f32) as i32,
        ])
    }
}

trait TagColumnAny {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn remove_any(&mut self, row_key: u32) -> Option<TagRow<()>>;
    fn modify_ref(&mut self, row_key: u32, r#ref: RefKey, ref_row_key: u32) -> Option<()>;
    fn modify_spc(&mut self, row_key: u32, spc: SpaceKey, spc_row_key: u32) -> Option<()>;
    fn get_any(&mut self, row_key: u32) -> Option<TagRow<()>>;
}

#[derive(Debug)]
struct TagRow<T> {
    tag: T,
    typ: std::any::TypeId,
    typ_row_key: u32,
    r#ref: RefKey,
    ref_row_key: u32,
    spc: SpaceKey,
    spc_row_key: u32,
}

type TagColumn<T> = slab::Slab<TagRow<T>>;

impl<T: 'static> TagColumnAny for TagColumn<T> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn remove_any(&mut self, row_key: u32) -> Option<TagRow<()>> {
        let row = self.try_remove(row_key as usize)?;
        Some(TagRow {
            tag: (),
            typ: row.typ,
            typ_row_key: row.typ_row_key,
            r#ref: row.r#ref,
            ref_row_key: row.ref_row_key,
            spc: row.spc,
            spc_row_key: row.spc_row_key,
        })
    }

    fn modify_ref(&mut self, row_key: u32, r#ref: RefKey, ref_row_key: u32) -> Option<()> {
        let row = self.get_mut(row_key as usize)?;
        row.r#ref = r#ref;
        row.ref_row_key = ref_row_key;
        Some(())
    }

    fn modify_spc(&mut self, row_key: u32, spc: SpaceKey, spc_row_key: u32) -> Option<()> {
        let row = self.get_mut(row_key as usize)?;
        row.spc = spc;
        row.spc_row_key = spc_row_key;
        Some(())
    }

    fn get_any(&mut self, row_key: u32) -> Option<TagRow<()>> {
        let row = self.get(row_key as usize)?;
        Some(TagRow {
            tag: (),
            typ: row.typ,
            typ_row_key: row.typ_row_key,
            r#ref: row.r#ref,
            ref_row_key: row.ref_row_key,
            spc: row.spc,
            spc_row_key: row.spc_row_key,
        })
    }
}

#[derive(Default)]
pub struct TagStore {
    tag_cols: Vec<Box<dyn TagColumnAny>>,
    typ_map: ahash::AHashMap<std::any::TypeId, (u32, slab::Slab<TagKey>)>,
    typ_ref_map: ahash::AHashMap<(std::any::TypeId, RefKey), TagKey>,
    typ_spc_map: ahash::AHashMap<(std::any::TypeId, SpaceKey), (u32, slab::Slab<TagKey>)>,
    ref_map: ahash::AHashMap<RefKey, slab::Slab<TagKey>>,
}

impl TagStore {
    pub fn insert<T>(&mut self, r#ref: RefKey, spc: SpaceKey, tag: T) -> Option<TagKey>
    where
        T: std::any::Any,
    {
        let typ = std::any::TypeId::of::<T>();

        // prevent multiple tag at same ref
        if self.typ_ref_map.contains_key(&(typ, r#ref)) {
            return None;
        }

        let (col_key, row_keys) = self.typ_map.entry(typ).or_insert_with(|| {
            // initialize a new column

            let col_key = self.tag_cols.len();
            if col_key >= u32::MAX as usize {
                panic!("capacity overflow");
            }

            self.tag_cols.push(Box::new(TagColumn::<T>::default()));
            (col_key as u32, Default::default())
        });
        let col_key = *col_key;

        let tag_col = self.tag_cols.get_mut(col_key as usize)?;
        let tag_col = tag_col.as_any_mut().downcast_mut::<TagColumn<T>>()?;

        // row_key is guaranteed to be less than u32::MAX.
        let row_key = tag_col.vacant_key() as u32;

        // typ_row_key is guaranteed to be less than u32::MAX.
        let typ_row_key = row_keys.insert((col_key, row_key)) as u32;

        let row_keys = self.ref_map.entry(r#ref).or_default();
        // ref_row_key is guaranteed to be less than u32::MAX.
        let ref_row_key = row_keys.insert((col_key, row_key)) as u32;

        self.typ_ref_map.insert((typ, r#ref), (col_key, row_key));

        let (_, row_keys) = self
            .typ_spc_map
            .entry((typ, spc))
            .or_insert_with(|| (col_key, Default::default()));
        // spc_row_key is guaranteed to be less than u32::MAX.
        let spc_row_key = row_keys.insert((col_key, row_key)) as u32;

        tag_col.insert(TagRow {
            tag,
            typ,
            typ_row_key,
            r#ref,
            ref_row_key,
            spc,
            spc_row_key,
        });

        Some((col_key, row_key))
    }

    pub fn remove<T>(&mut self, tag_key: TagKey) -> Option<(RefKey, SpaceKey, T)>
    where
        T: std::any::Any,
    {
        let (col_key, row_key) = tag_key;

        let tag_col = self.tag_cols.get_mut(col_key as usize)?;
        let tag_col = tag_col.as_any_mut().downcast_mut::<TagColumn<T>>()?;

        let tag_row = tag_col.try_remove(row_key as usize)?;

        let (_, row_keys) = self.typ_map.get_mut(&tag_row.typ).unwrap();
        row_keys.try_remove(tag_row.typ_row_key as usize).unwrap();

        let row_keys = self.ref_map.get_mut(&tag_row.r#ref).unwrap();
        row_keys.try_remove(tag_row.ref_row_key as usize).unwrap();

        self.typ_ref_map
            .remove(&(tag_row.typ, tag_row.r#ref))
            .unwrap();

        let (_, row_keys) = self
            .typ_spc_map
            .get_mut(&(tag_row.typ, tag_row.spc))
            .unwrap();
        row_keys.try_remove(tag_row.spc_row_key as usize).unwrap();

        Some((tag_row.r#ref, tag_row.spc, tag_row.tag))
    }

    pub fn modify<T>(
        &mut self,
        tag_key: TagKey,
        f: impl FnOnce(&mut RefKey, &mut SpaceKey, &mut T),
    ) -> Option<()>
    where
        T: std::any::Any,
    {
        let (col_key, row_key) = tag_key;

        let tag_col = self.tag_cols.get_mut(col_key as usize)?;
        let tag_col = tag_col.as_any_mut().downcast_mut::<TagColumn<T>>()?;

        let tag_row = tag_col.get_mut(row_key as usize)?;

        let (prev_ref, prev_spc) = (tag_row.r#ref, tag_row.spc);
        f(&mut tag_row.r#ref, &mut tag_row.spc, &mut tag_row.tag);

        if prev_ref != tag_row.r#ref {
            let row_keys = self.ref_map.get_mut(&prev_ref).unwrap();
            let tag_key = row_keys.try_remove(tag_row.ref_row_key as usize).unwrap();

            let row_keys = self.ref_map.entry(tag_row.r#ref).or_default();
            // ref_row_key is guaranteed to be less than u32::MAX.
            let ref_row_key = row_keys.insert(tag_key) as u32;

            tag_row.ref_row_key = ref_row_key;

            let tag_key = self.typ_ref_map.remove(&(tag_row.typ, prev_ref)).unwrap();
            self.typ_ref_map
                .insert((tag_row.typ, tag_row.r#ref), tag_key);
        }

        if prev_spc != tag_row.spc {
            let (_, row_keys) = self.typ_spc_map.get_mut(&(tag_row.typ, prev_spc)).unwrap();
            let tag_key = row_keys.try_remove(tag_row.spc_row_key as usize).unwrap();

            let (_, row_keys) = self
                .typ_spc_map
                .entry((tag_row.typ, tag_row.spc))
                .or_insert_with(|| (col_key, Default::default()));
            // spc_row_key is guaranteed to be less than u32::MAX.
            let spc_row_key = row_keys.insert(tag_key) as u32;

            tag_row.spc_row_key = spc_row_key;
        }

        Some(())
    }

    pub fn remove_by_ref(&mut self, r#ref: RefKey) -> Option<()> {
        let tag_keys = self.ref_map.remove(&r#ref)?;

        for (_, tag_key) in tag_keys {
            let (col_key, row_key) = tag_key;

            let tag_col = self.tag_cols.get_mut(col_key as usize).unwrap();
            let tag_row = tag_col.remove_any(row_key).unwrap();

            let (_, row_keys) = self.typ_map.get_mut(&tag_row.typ).unwrap();
            row_keys.try_remove(tag_row.typ_row_key as usize).unwrap();

            self.typ_ref_map
                .remove(&(tag_row.typ, tag_row.r#ref))
                .unwrap();

            let (_, row_keys) = self
                .typ_spc_map
                .get_mut(&(tag_row.typ, tag_row.spc))
                .unwrap();
            row_keys.try_remove(tag_row.spc_row_key as usize).unwrap();
        }

        Some(())
    }

    pub fn modify_ref_by_ref(&mut self, r#ref: RefKey, new_ref: RefKey) -> Option<()> {
        let tag_keys = self.ref_map.remove(&r#ref)?;

        for (_, tag_key) in tag_keys {
            let (col_key, row_key) = tag_key;

            let tag_col = self.tag_cols.get_mut(col_key as usize).unwrap();
            let tag_row = tag_col.get_any(row_key).unwrap();

            let row_keys = self.ref_map.entry(new_ref).or_default();
            // ref_row_key is guaranteed to be less than u32::MAX.
            let ref_row_key = row_keys.insert(tag_key) as u32;

            let tag_key = self
                .typ_ref_map
                .remove(&(tag_row.typ, tag_row.r#ref))
                .unwrap();
            self.typ_ref_map.insert((tag_row.typ, new_ref), tag_key);

            tag_col.modify_ref(row_key, new_ref, ref_row_key).unwrap();
        }

        Some(())
    }

    pub fn modify_spc_by_ref(&mut self, r#ref: RefKey, new_spc: SpaceKey) -> Option<()> {
        let tag_keys = self.ref_map.get(&r#ref)?;

        for (_, tag_key) in tag_keys {
            let (col_key, row_key) = *tag_key;

            let tag_col = self.tag_cols.get_mut(col_key as usize).unwrap();
            let tag_row = tag_col.get_any(row_key).unwrap();

            let (_, row_keys) = self
                .typ_spc_map
                .get_mut(&(tag_row.typ, tag_row.spc))
                .unwrap();
            let tag_key = row_keys.try_remove(tag_row.spc_row_key as usize).unwrap();

            let (_, row_keys) = self
                .typ_spc_map
                .entry((tag_row.typ, new_spc))
                .or_insert_with(|| (col_key, Default::default()));
            // spc_row_key is guaranteed to be less than u32::MAX.
            let spc_row_key = row_keys.insert(tag_key) as u32;

            tag_col.modify_spc(row_key, new_spc, spc_row_key).unwrap();
        }

        Some(())
    }

    pub fn get<T>(&self, tag_key: TagKey) -> Option<(&RefKey, &SpaceKey, &T)>
    where
        T: std::any::Any,
    {
        let (col_key, row_key) = tag_key;

        let tag_col = self.tag_cols.get(col_key as usize)?;
        let tag_col = tag_col.as_any().downcast_ref::<TagColumn<T>>()?;

        let tag_row = tag_col.get(row_key as usize)?;

        Some((&tag_row.r#ref, &tag_row.spc, &tag_row.tag))
    }

    fn iter_internal<T>(&self) -> Option<impl Iterator<Item = &TagKey>>
    where
        T: std::any::Any,
    {
        let typ = std::any::TypeId::of::<T>();
        let (_, row_keys) = self.typ_map.get(&typ)?;
        Some(row_keys.iter().map(|(_, v)| v))
    }

    fn one_by_ref_internal<T>(&self, r#ref: RefKey) -> Option<&TagKey>
    where
        T: std::any::Any,
    {
        let typ = std::any::TypeId::of::<T>();
        self.typ_ref_map.get(&(typ, r#ref))
    }

    fn iter_by_rect_internal<T>(&self, rect: [SpaceKey; 2]) -> Option<impl Iterator<Item = &TagKey>>
    where
        T: std::any::Any,
    {
        let typ = std::any::TypeId::of::<T>();

        let (min, max) = (rect[0].0, rect[1].0);

        let mut iters = vec![];
        for y in min[1]..=max[1] {
            for x in min[0]..=max[0] {
                let spc = SpaceKey([x, y]);
                let (_, row_keys) = self.typ_spc_map.get(&(typ, spc))?;
                iters.push(row_keys.iter().map(|(_, v)| v));
            }
        }

        Some(iters.into_iter().flatten())
    }

    #[inline]
    pub fn iter<T>(&self) -> impl Iterator<Item = &TagKey>
    where
        T: std::any::Any,
    {
        self.iter_internal::<T>().into_iter().flatten()
    }

    #[inline]
    pub fn detach_iter<T>(&self) -> Vec<TagKey>
    where
        T: std::any::Any,
    {
        self.iter::<T>().copied().collect()
    }

    #[inline]
    pub fn one_by_ref<T>(&self, r#ref: RefKey) -> Option<&TagKey>
    where
        T: std::any::Any,
    {
        self.one_by_ref_internal::<T>(r#ref)
    }

    #[inline]
    pub fn iter_by_rect<T>(&self, rect: [SpaceKey; 2]) -> impl Iterator<Item = &TagKey>
    where
        T: std::any::Any,
    {
        self.iter_by_rect_internal::<T>(rect).into_iter().flatten()
    }

    #[inline]
    pub fn detach_iter_by_rect<T>(&self, rect: [SpaceKey; 2]) -> Vec<TagKey>
    where
        T: std::any::Any,
    {
        self.iter_by_rect::<T>(rect).copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud() {
        let mut store = TagStore::default();
        let key = store
            .insert(RefKey::Tile(0), SpaceKey::from([0, 0]), 42)
            .unwrap();

        assert_eq!(
            store.get::<i32>(key),
            Some((&RefKey::Tile(0), &SpaceKey::from([0, 0]), &42))
        );
        assert_eq!(
            store.remove::<i32>(key),
            Some((RefKey::Tile(0), SpaceKey::from([0, 0]), 42))
        );

        assert_eq!(store.get::<i32>(key), None);
        assert_eq!(store.remove::<i32>(key), None);
    }

    #[test]
    fn insert_with_invalid_type() {
        let mut store = TagStore::default();
        let key = store
            .insert(RefKey::Tile(0), SpaceKey::from([0, 0]), 42)
            .unwrap();

        assert!(store.get::<()>(key).is_none());
        assert!(store.remove::<()>(key).is_none());
    }

    #[test]
    fn insert_with_duplication() {
        let mut store = TagStore::default();
        store
            .insert(RefKey::Tile(0), SpaceKey::from([0, 0]), 42)
            .unwrap();

        assert!(store
            .insert(RefKey::Tile(0), SpaceKey::from([0, 0]), 42)
            .is_none());
    }

    #[test]
    fn modify() {
        let mut store = TagStore::default();
        let key_0 = store
            .insert(RefKey::Tile(0), SpaceKey::from([0, 0]), 42)
            .unwrap();
        let key_1 = store
            .insert(RefKey::Tile(1), SpaceKey::from([0, 0]), ())
            .unwrap();

        store.modify::<i32>(key_0, |_, _, tag| *tag = 43).unwrap();
        assert_eq!(
            store.get::<i32>(key_0),
            Some((&RefKey::Tile(0), &SpaceKey::from([0, 0]), &43))
        );

        store
            .modify::<i32>(key_0, |r#ref, _, _| *r#ref = RefKey::Tile(10))
            .unwrap();
        assert_eq!(
            store.get::<i32>(key_0),
            Some((&RefKey::Tile(10), &SpaceKey::from([0, 0]), &43))
        );
        assert_eq!(store.one_by_ref::<i32>(RefKey::Tile(0)), None);
        assert_eq!(store.one_by_ref::<i32>(RefKey::Tile(10)), Some(&key_0));

        store
            .modify::<()>(key_1, |_, spc, _| *spc = SpaceKey::from([1000, 0]))
            .unwrap();
        assert_eq!(
            store.get::<()>(key_1),
            Some((&RefKey::Tile(1), &SpaceKey::from([1000, 0]), &()))
        );
        assert_eq!(
            store.detach_iter_by_rect::<()>([SpaceKey::from([0, 0]), SpaceKey::from([0, 0])]),
            vec![]
        );
        assert_eq!(
            store.detach_iter_by_rect::<()>([SpaceKey::from([1000, 0]), SpaceKey::from([1000, 0])]),
            vec![key_1]
        );
    }

    #[test]
    fn modify_invalid() {
        let mut store = TagStore::default();
        let key_0 = store
            .insert(RefKey::Tile(0), SpaceKey::from([0, 0]), 42)
            .unwrap();
        let key_1 = store
            .insert(RefKey::Tile(1), SpaceKey::from([0, 0]), 63)
            .unwrap();

        store.remove::<i32>(key_1).unwrap();

        assert!(store.modify::<()>(key_0, |_, _, _| {}).is_none());
        assert!(store.modify::<i32>(key_1, |_, _, _| {}).is_none());
    }

    #[test]
    fn remove_by_ref() {
        let mut store = TagStore::default();
        let key_0 = store
            .insert(RefKey::Tile(0), SpaceKey::from([0, 0]), 42)
            .unwrap();
        let key_1 = store
            .insert(RefKey::Tile(0), SpaceKey::from([0, 0]), ())
            .unwrap();

        store.remove_by_ref(RefKey::Tile(0)).unwrap();

        assert_eq!(store.get::<i32>(key_0), None);
        assert_eq!(store.get::<()>(key_1), None);
        assert_eq!(store.one_by_ref::<i32>(RefKey::Tile(0)), None);
        assert_eq!(store.one_by_ref::<()>(RefKey::Tile(0)), None);
    }

    #[test]
    fn modify_by_ref() {
        let mut store = TagStore::default();
        let key_0 = store
            .insert(RefKey::Tile(0), SpaceKey::from([0, 0]), 42)
            .unwrap();
        let key_1 = store
            .insert(RefKey::Tile(0), SpaceKey::from([0, 0]), ())
            .unwrap();

        store
            .modify_ref_by_ref(RefKey::Tile(0), RefKey::Tile(1))
            .unwrap();
        assert_eq!(store.one_by_ref::<i32>(RefKey::Tile(0)), None);
        assert_eq!(store.one_by_ref::<()>(RefKey::Tile(0)), None);
        assert_eq!(store.one_by_ref::<i32>(RefKey::Tile(1)), Some(&key_0));
        assert_eq!(store.one_by_ref::<()>(RefKey::Tile(1)), Some(&key_1));

        store
            .modify_spc_by_ref(RefKey::Tile(1), SpaceKey::from([1000, 0]))
            .unwrap();
        assert_eq!(
            store
                .iter_by_rect::<i32>([SpaceKey::from([0, 0]), SpaceKey::from([0, 0])])
                .next(),
            None
        );
        assert_eq!(
            store
                .iter_by_rect::<()>([SpaceKey::from([0, 0]), SpaceKey::from([0, 0])])
                .next(),
            None
        );
        assert_eq!(
            store
                .iter_by_rect::<i32>([SpaceKey::from([1000, 0]), SpaceKey::from([1000, 0])])
                .next(),
            Some(&key_0)
        );
        assert_eq!(
            store
                .iter_by_rect::<()>([SpaceKey::from([1000, 0]), SpaceKey::from([1000, 0])])
                .next(),
            Some(&key_1)
        );
    }

    #[test]
    fn iter() {
        let mut store = TagStore::default();
        let k0 = store
            .insert(RefKey::Tile(0), SpaceKey::from([0, 0]), 42)
            .unwrap();
        let k1 = store
            .insert(RefKey::Tile(1), SpaceKey::from([0, 0]), 42)
            .unwrap();
        let k2 = store
            .insert(RefKey::Tile(1), SpaceKey::from([0, 0]), ())
            .unwrap();

        let mut iter = store.iter::<i32>();
        assert_eq!(iter.next(), Some(&k0));
        assert_eq!(iter.next(), Some(&k1));
        assert_eq!(iter.next(), None);
        drop(iter);

        let mut iter = store.iter::<()>();
        assert_eq!(iter.next(), Some(&k2));
        assert_eq!(iter.next(), None);
        drop(iter);
    }

    #[test]
    fn iter_with_invalid_type() {
        let store = TagStore::default();
        assert!(store.iter::<i32>().next().is_none());
    }

    #[test]
    fn detach_iter() {
        let mut store = TagStore::default();
        let k0 = store
            .insert(RefKey::Tile(0), SpaceKey::from([0, 0]), 42)
            .unwrap();
        let k1 = store
            .insert(RefKey::Tile(1), SpaceKey::from([0, 0]), 42)
            .unwrap();
        let k2 = store
            .insert(RefKey::Tile(1), SpaceKey::from([0, 0]), ())
            .unwrap();

        assert_eq!(store.detach_iter::<i32>(), vec![k0, k1]);
        assert_eq!(store.detach_iter::<()>(), vec![k2]);
    }

    #[test]
    fn one_by_ref() {
        let mut store = TagStore::default();
        let k0 = store
            .insert(RefKey::Tile(0), SpaceKey::from([0, 0]), 42)
            .unwrap();
        let k1 = store
            .insert(RefKey::Tile(1), SpaceKey::from([0, 0]), 42)
            .unwrap();
        let _k2 = store
            .insert(RefKey::Tile(1), SpaceKey::from([0, 0]), ())
            .unwrap();

        let one = store.one_by_ref::<i32>(RefKey::Tile(0));
        assert_eq!(one, Some(&k0));

        let one = store.one_by_ref::<i32>(RefKey::Tile(1));
        assert_eq!(one, Some(&k1));
    }

    #[test]
    fn one_by_ref_with_invalid_type() {
        let mut store = TagStore::default();
        store
            .insert(RefKey::Tile(0), SpaceKey::from([0, 0]), 42)
            .unwrap();

        assert_eq!(store.one_by_ref::<()>(RefKey::Tile(0)), None);
    }

    #[test]
    fn one_by_ref_with_invalid_ref() {
        let mut store = TagStore::default();
        store
            .insert(RefKey::Tile(0), SpaceKey::from([0, 0]), 42)
            .unwrap();

        assert_eq!(store.one_by_ref::<i32>(RefKey::Tile(1)), None);
    }

    #[test]
    fn iter_by_rect() {
        let mut store = TagStore::default();
        let k0 = store
            .insert(RefKey::Tile(0), SpaceKey::from([0, 0]), 42)
            .unwrap();
        let k1 = store
            .insert(RefKey::Tile(1), SpaceKey::from([0, 0]), 42)
            .unwrap();
        let k2 = store
            .insert(RefKey::Tile(1), SpaceKey::from([0, 0]), ())
            .unwrap();

        let mut iter = store.iter_by_rect::<i32>([SpaceKey::from([0, 0]), SpaceKey::from([0, 0])]);
        assert_eq!(iter.next(), Some(&k0));
        assert_eq!(iter.next(), Some(&k1));
        assert_eq!(iter.next(), None);
        drop(iter);

        let mut iter = store.iter_by_rect::<()>([SpaceKey::from([0, 0]), SpaceKey::from([0, 0])]);
        assert_eq!(iter.next(), Some(&k2));
        assert_eq!(iter.next(), None);
        drop(iter);
    }

    #[test]
    fn iter_by_rect_with_invalid_type() {
        let store = TagStore::default();
        assert!(store
            .iter_by_rect::<i32>([SpaceKey::from([0, 0]), SpaceKey::from([0, 0])])
            .next()
            .is_none());
    }

    #[test]
    fn detach_iter_by_rect() {
        let mut store = TagStore::default();
        let k0 = store
            .insert(RefKey::Tile(0), SpaceKey::from([0, 0]), 42)
            .unwrap();
        let k1 = store
            .insert(RefKey::Tile(1), SpaceKey::from([0, 0]), 42)
            .unwrap();
        let k2 = store
            .insert(RefKey::Tile(1), SpaceKey::from([0, 0]), ())
            .unwrap();

        assert_eq!(
            store.detach_iter_by_rect::<i32>([SpaceKey::from([0, 0]), SpaceKey::from([0, 0])]),
            vec![k0, k1]
        );
        assert_eq!(
            store.detach_iter_by_rect::<()>([SpaceKey::from([0, 0]), SpaceKey::from([0, 0])]),
            vec![k2]
        );
    }
}
