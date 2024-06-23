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
}

#[derive(Debug)]
struct NodeMeta {
    ref_slab_key: u32,
}

struct NodeColumn {
    inners: Box<dyn std::any::Any>,
    metas: slab::Slab<NodeMeta>,
}

#[derive(Default)]
pub struct NodeStore {
    node_cols: ahash::AHashMap<std::any::TypeId, NodeColumn>,
    refs: ahash::AHashMap<(NodeRelation, std::any::TypeId), slab::Slab<u32>>,
}

impl NodeStore {
    pub fn insert<T: 'static>(&mut self, inner: T, relation: NodeRelation) -> Option<NodeKey> {
        let type_key = std::any::TypeId::of::<T>();

        let node_col = self
            .node_cols
            .entry(type_key)
            .or_insert_with(|| NodeColumn {
                inners: Box::new(slab::Slab::<Node<T>>::new()),
                metas: Default::default(),
            });

        let slab_key = node_col
            .inners
            .downcast_mut::<slab::Slab<Node<T>>>()
            .check()
            .insert(Node { inner, relation }) as u32;

        let ref_slab_key = self
            .refs
            .entry((relation, type_key))
            .or_default()
            .insert(slab_key) as u32;

        node_col.metas.insert(NodeMeta { ref_slab_key });

        Some((type_key, slab_key))
    }

    pub fn remove<T: 'static>(&mut self, node_key: NodeKey) -> Option<(NodeRelation, T)> {
        let (type_key, slab_key) = node_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let node_col = self.node_cols.get_mut(&type_key)?;

        let inner = node_col
            .inners
            .downcast_mut::<slab::Slab<Node<T>>>()
            .check()
            .try_remove(slab_key as usize)
            .check();

        let meta = node_col.metas.try_remove(slab_key as usize).check();

        self.refs
            .get_mut(&(inner.relation, type_key))
            .check()
            .try_remove(meta.ref_slab_key as usize)
            .check();

        Some((inner.relation, inner.inner))
    }

    pub fn get<T: 'static>(&self, node_key: NodeKey) -> Option<(&NodeRelation, &T)> {
        let (type_key, slab_key) = node_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let node_col = self.node_cols.get(&type_key)?;

        let inner = node_col
            .inners
            .downcast_ref::<slab::Slab<Node<T>>>()
            .check()
            .get(slab_key as usize)
            .check();

        Some((&inner.relation, &inner.inner))
    }

    pub fn get_mut<T: 'static>(&mut self, node_key: NodeKey) -> Option<(&NodeRelation, &mut T)> {
        let (type_key, slab_key) = node_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let node_col = self.node_cols.get_mut(&type_key)?;

        let inner = node_col
            .inners
            .downcast_mut::<slab::Slab<Node<T>>>()
            .check()
            .get_mut(slab_key as usize)
            .check();

        Some((&inner.relation, &mut inner.inner))
    }

    pub fn iter<T: 'static>(&self) -> Option<impl Iterator<Item = (&NodeRelation, &T)>> {
        let type_key = std::any::TypeId::of::<T>();

        let node_col = self.node_cols.get(&type_key)?;

        let iter = node_col
            .inners
            .downcast_ref::<slab::Slab<Node<T>>>()
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

        let inners = node_col
            .inners
            .downcast_mut::<slab::Slab<Node<T>>>()
            .check();

        let iter = inners
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

        let inners = node_col
            .inners
            .downcast_ref::<slab::Slab<Node<T>>>()
            .check();

        let iter = self
            .refs
            .get(&(relation, type_key))?
            .iter()
            .map(|(_, slab_key)| {
                let inner = inners.get(*slab_key as usize).check();
                (&inner.relation, &inner.inner)
            });

        Some(iter)
    }

    pub fn iter_mut_by_relation<T: 'static>(
        &mut self,
        relation: NodeRelation,
    ) -> Option<impl Iterator<Item = (&NodeRelation, &mut T)>> {
        let type_key = std::any::TypeId::of::<T>();

        let node_col = self.node_cols.get_mut(&type_key)?;

        let inners = node_col
            .inners
            .downcast_mut::<slab::Slab<Node<T>>>()
            .check();

        let iter = self
            .refs
            .get(&(relation, type_key))?
            .iter()
            .map(|(_, slab_key)| {
                let inner = inners.get_mut(*slab_key as usize).check() as *mut Node<T>;
                let inner = unsafe { &mut *inner };
                (&inner.relation, &mut inner.inner)
            });

        Some(iter)
    }

    pub fn remove_by_relation<T: 'static>(
        &mut self,
        relation: NodeRelation,
    ) -> Option<Vec<(NodeRelation, T)>> {
        let type_key = std::any::TypeId::of::<T>();

        let node_col = self.node_cols.get_mut(&type_key)?;

        let inners = node_col
            .inners
            .downcast_mut::<slab::Slab<Node<T>>>()
            .check();

        let meta = &mut node_col.metas;

        let mut vec = vec![];
        if let Some(r#ref) = self.refs.remove(&(relation, type_key)) {
            for (_, slab_key) in r#ref {
                let inner = inners.try_remove(slab_key as usize).check();
                vec.push((inner.relation, inner.inner));

                meta.try_remove(slab_key as usize).check();
            }
        }

        Some(vec)
    }
}
