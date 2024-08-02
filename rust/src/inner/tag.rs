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

#[derive(Debug)]
struct TagRow<T> {
    tag: T,
    typ_row_key: u32,
    r#ref: RefKey,
    spc: SpaceKey,
    spc_row_key: u32,
}

type TagColumn<T> = slab::Slab<TagRow<T>>;

#[derive(Debug, Default)]
pub struct TagStore {
    tag_cols: Vec<Box<dyn std::any::Any>>,
    typ_map: ahash::AHashMap<std::any::TypeId, (u32, slab::Slab<TagKey>)>,
    typ_ref_map: ahash::AHashMap<(std::any::TypeId, RefKey), (u32, u32)>,
    typ_spc_map: ahash::AHashMap<(std::any::TypeId, SpaceKey), (u32, slab::Slab<TagKey>)>,
}

impl TagStore {
    pub fn insert<T>(&mut self, r#ref: RefKey, spc: SpaceKey, tag: T) -> Option<TagKey>
    where
        T: std::any::Any,
    {
        let typ = std::any::TypeId::of::<T>();

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
        let tag_col = tag_col.downcast_mut::<TagColumn<T>>()?;

        // row_key is guaranteed to be less than u32::MAX.
        let row_key = tag_col.vacant_key() as u32;

        // typ_row_key is guaranteed to be less than u32::MAX.
        let typ_row_key = row_keys.insert((col_key, row_key)) as u32;

        self.typ_ref_map.insert((typ, r#ref), (col_key, row_key));

        let (_, row_keys) = self
            .typ_spc_map
            .entry((typ, spc))
            .or_insert_with(|| (col_key, Default::default()));
        // spc_row_key is guaranteed to be less than u32::MAX.
        let spc_row_key = row_keys.insert((col_key, row_key)) as u32;

        tag_col.insert(TagRow {
            tag,
            typ_row_key,
            r#ref,
            spc,
            spc_row_key,
        });

        Some((col_key, row_key))
    }

    pub fn remove<T>(&mut self, tag_key: TagKey) -> Option<(RefKey, SpaceKey, T)>
    where
        T: std::any::Any,
    {
        let typ = std::any::TypeId::of::<T>();

        let (col_key, row_key) = tag_key;

        let tag_col = self.tag_cols.get_mut(col_key as usize)?;
        let tag_col = tag_col.downcast_mut::<TagColumn<T>>()?;

        let tag_row = tag_col.try_remove(row_key as usize)?;

        let (_, row_keys) = self.typ_map.get_mut(&typ).unwrap();
        row_keys.try_remove(tag_row.typ_row_key as usize).unwrap();

        self.typ_ref_map.remove(&(typ, tag_row.r#ref)).unwrap();

        let (_, row_keys) = self.typ_spc_map.get_mut(&(typ, tag_row.spc)).unwrap();
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
        let typ = std::any::TypeId::of::<T>();

        let (col_key, row_key) = tag_key;

        let tag_col = self.tag_cols.get_mut(col_key as usize)?;
        let tag_col = tag_col.downcast_mut::<TagColumn<T>>()?;

        let tag_row = tag_col.get_mut(row_key as usize)?;

        let (prev_ref, prev_spc) = (tag_row.r#ref, tag_row.spc);
        f(&mut tag_row.r#ref, &mut tag_row.spc, &mut tag_row.tag);

        if prev_ref != tag_row.r#ref {
            let tag_key = self.typ_ref_map.remove(&(typ, prev_ref)).unwrap();
            self.typ_ref_map.insert((typ, tag_row.r#ref), tag_key);
        }

        if prev_spc != tag_row.spc {
            let (_, row_keys) = self.typ_spc_map.get_mut(&(typ, prev_spc)).unwrap();
            let tag_key = row_keys.try_remove(tag_row.spc_row_key as usize).unwrap();

            let (_, row_keys) = self
                .typ_spc_map
                .entry((typ, tag_row.spc))
                .or_insert((col_key, Default::default()));
            // spc_row_key is guaranteed to be less than u32::MAX.
            let spc_row_key = row_keys.insert(tag_key) as u32;

            tag_row.spc_row_key = spc_row_key;
        }

        Some(())
    }

    pub fn get<T>(&self, tag_key: TagKey) -> Option<(&RefKey, &SpaceKey, &T)>
    where
        T: std::any::Any,
    {
        let (col_key, row_key) = tag_key;

        let tag_col = self.tag_cols.get(col_key as usize)?;
        let tag_col = tag_col.downcast_ref::<TagColumn<T>>()?;

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
        for y in min[1]..max[1] {
            for x in min[0]..max[0] {
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
    pub fn one<T>(&self) -> Option<&TagKey>
    where
        T: std::any::Any,
    {
        self.iter::<T>().next()
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

    #[inline]
    pub fn one_by_rect<T>(&self, rect: [SpaceKey; 2]) -> Option<&TagKey>
    where
        T: std::any::Any,
    {
        self.iter_by_rect::<T>(rect).next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud_tag() {
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
    fn tag_with_invalid_type() {
        let mut store = TagStore::default();
        let key = store
            .insert(RefKey::Tile(0), SpaceKey::from([0, 0]), 42)
            .unwrap();

        assert!(store.get::<()>(key).is_none());
        assert!(store.remove::<()>(key).is_none());
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
}
