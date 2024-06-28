use super::*;

pub type NodeKey = (std::any::TypeId, u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NodeRelation {
    Global,
    Tile(u32),
    Block(u32),
    Entity(u32),
}

#[derive(Debug)]
struct Node<T> {
    inner: T,
    relation: NodeRelation,
    ref_slab_key: u32,
}

#[derive(Debug)]
pub struct StackBuf<const N: usize> {
    bytes: [std::mem::MaybeUninit<u8>; N],
    size: usize,
}

impl<const N: usize> StackBuf<N> {
    pub fn try_new<T>(x: T) -> Option<Self>
    where
        T: std::any::Any,
    {
        if N < std::mem::size_of::<T>() {
            return None;
        }

        let mut slf = Self {
            bytes: [std::mem::MaybeUninit::uninit(); N],
            size: std::mem::size_of::<T>(),
        };

        let src = &x as *const _ as *const _;
        let dst = slf.bytes.as_mut_ptr();
        unsafe { std::ptr::copy_nonoverlapping(src, dst, slf.size) };
        std::mem::forget(x);

        Some(slf)
    }

    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: std::any::Any,
    {
        if N < std::mem::size_of::<T>() {
            return None;
        }

        let ptr = self.bytes.as_ptr();
        Some(unsafe { &*(ptr as *const T) })
    }

    pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        T: std::any::Any,
    {
        if N < std::mem::size_of::<T>() {
            return None;
        }

        let ptr = self.bytes.as_mut_ptr();
        Some(unsafe { &mut *(ptr as *mut T) })
    }
}

type NodeColumn<T> = slab::Slab<Node<T>>;

const NODE_COLUMN_ALLOC_SIZE: usize = std::mem::size_of::<NodeColumn<()>>();

#[derive(Debug, Default)]
pub struct NodeStore {
    node_cols: ahash::AHashMap<std::any::TypeId, StackBuf<NODE_COLUMN_ALLOC_SIZE>>,
    ref_cols: ahash::AHashMap<(NodeRelation, std::any::TypeId), slab::Slab<u32>>,
}

impl NodeStore {
    pub fn insert<T: 'static>(&mut self, inner: T, relation: NodeRelation) -> Option<NodeKey> {
        let type_key = std::any::TypeId::of::<T>();

        let node_col = self
            .node_cols
            .entry(type_key)
            .or_insert_with(|| StackBuf::try_new::<NodeColumn<T>>(Default::default()).check())
            .downcast_mut::<NodeColumn<T>>()
            .check();

        let slab_key = node_col.vacant_key() as u32;

        let ref_slab_key = self
            .ref_cols
            .entry((relation, type_key))
            .or_default()
            .insert(slab_key) as u32;

        node_col.insert(Node {
            inner,
            relation,
            ref_slab_key,
        });

        Some((type_key, slab_key))
    }

    pub fn remove<T: 'static>(&mut self, node_key: NodeKey) -> Option<(NodeRelation, T)> {
        let (type_key, slab_key) = node_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let node_col = self.node_cols.get_mut(&type_key)?;

        let node = node_col
            .downcast_mut::<NodeColumn<T>>()
            .check()
            .try_remove(slab_key as usize)
            .check();

        self.ref_cols
            .get_mut(&(node.relation, type_key))
            .check()
            .try_remove(node.ref_slab_key as usize)
            .check();

        Some((node.relation, node.inner))
    }

    pub fn get<T: 'static>(&self, node_key: NodeKey) -> Option<(&NodeRelation, &T)> {
        let (type_key, slab_key) = node_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let node_col = self.node_cols.get(&type_key)?;

        let node = node_col
            .downcast_ref::<NodeColumn<T>>()
            .check()
            .get(slab_key as usize)
            .check();

        Some((&node.relation, &node.inner))
    }

    pub fn get_mut<T: 'static>(&mut self, node_key: NodeKey) -> Option<(&NodeRelation, &mut T)> {
        let (type_key, slab_key) = node_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let node_col = self.node_cols.get_mut(&type_key)?;

        let inner = node_col
            .downcast_mut::<NodeColumn<T>>()
            .check()
            .get_mut(slab_key as usize)
            .check();

        Some((&inner.relation, &mut inner.inner))
    }

    pub fn iter<T: 'static>(&self) -> Option<impl Iterator<Item = (&NodeRelation, &T)>> {
        let type_key = std::any::TypeId::of::<T>();

        let node_col = self.node_cols.get(&type_key)?;

        let iter = node_col
            .downcast_ref::<NodeColumn<T>>()
            .check()
            .iter()
            .map(|(_, node)| (&node.relation, &node.inner));

        Some(iter)
    }

    pub fn iter_mut<T: 'static>(
        &mut self,
    ) -> Option<impl Iterator<Item = (&NodeRelation, &mut T)>> {
        let type_key = std::any::TypeId::of::<T>();

        let node_col = self.node_cols.get_mut(&type_key)?;

        let iter = node_col
            .downcast_mut::<NodeColumn<T>>()
            .check()
            .iter_mut()
            .map(|(_, node)| (&node.relation, &mut node.inner));

        Some(iter)
    }

    pub fn one<T: 'static>(&self) -> Option<(&NodeRelation, &T)> {
        self.iter::<T>()?.next()
    }

    pub fn one_mut<T: 'static>(&mut self) -> Option<(&NodeRelation, &mut T)> {
        self.iter_mut::<T>()?.next()
    }

    pub fn iter_by_relation<T: 'static>(
        &self,
        relation: NodeRelation,
    ) -> Option<impl Iterator<Item = (&NodeRelation, &T)>> {
        let type_key = std::any::TypeId::of::<T>();

        let node_col = self.node_cols.get(&type_key)?;

        let node = node_col.downcast_ref::<NodeColumn<T>>().check();

        let iter = self
            .ref_cols
            .get(&(relation, type_key))?
            .iter()
            .map(|(_, slab_key)| {
                let node = node.get(*slab_key as usize).check();
                (&node.relation, &node.inner)
            });

        Some(iter)
    }

    pub fn iter_mut_by_relation<T: 'static>(
        &mut self,
        relation: NodeRelation,
    ) -> Option<impl Iterator<Item = (&NodeRelation, &mut T)>> {
        let type_key = std::any::TypeId::of::<T>();

        let node_col = self.node_cols.get_mut(&type_key)?;

        let node = node_col.downcast_mut::<NodeColumn<T>>().check();

        let iter = self
            .ref_cols
            .get(&(relation, type_key))?
            .iter()
            .map(|(_, slab_key)| {
                let node = node.get_mut(*slab_key as usize).check() as *mut Node<T>;
                let node = unsafe { &mut *node };
                (&node.relation, &mut node.inner)
            });

        Some(iter)
    }

    pub fn remove_by_relation<T: 'static>(
        &mut self,
        relation: NodeRelation,
    ) -> Option<Vec<(NodeRelation, T)>> {
        let type_key = std::any::TypeId::of::<T>();

        let node_col = self
            .node_cols
            .get_mut(&type_key)?
            .downcast_mut::<NodeColumn<T>>()
            .check();

        let mut vec = vec![];
        if let Some(ref_col) = self.ref_cols.remove(&(relation, type_key)) {
            for (_, slab_key) in ref_col {
                let node = node_col.try_remove(slab_key as usize).check();
                vec.push((node.relation, node.inner));
            }
        }

        Some(vec)
    }
}
