use crate::inner::IntegrityCheck;

pub type BehaviorKey = (std::any::TypeId, u32);

#[derive(Debug)]
struct Behavior<T, R> {
    inner: T,
    relation: R,
}

#[derive(Debug)]
struct BehaviorMeta {
    relation_0: u32,
    relation_1: u32,
}

trait AnySlab {
    fn as_any(&self) -> &dyn std::any::Any;

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    fn try_remove_any(&mut self, key: usize) -> Option<()>;
}

impl<T: 'static> AnySlab for slab::Slab<T> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn try_remove_any(&mut self, key: usize) -> Option<()> {
        self.try_remove(key).map(|_| ())
    }
}

struct BehaviorColumns {
    inners: Box<dyn AnySlab>,
    metas: slab::Slab<BehaviorMeta>,
}

#[derive(Default)]
pub struct BehaviorField<R> {
    behaviors: ahash::AHashMap<std::any::TypeId, BehaviorColumns>,
    relation_0: ahash::AHashMap<R, slab::Slab<(std::any::TypeId, u32)>>,
    relation_1: ahash::AHashMap<(R, std::any::TypeId), slab::Slab<u32>>,
}

impl<R> BehaviorField<R>
where
    R: Clone + Eq + std::hash::Hash + 'static,
{
    pub fn insert<T: 'static>(&mut self, inner: T, relation: R) -> Option<BehaviorKey> {
        let type_key = std::any::TypeId::of::<T>();

        let behaviors = self
            .behaviors
            .entry(type_key)
            .or_insert_with(|| BehaviorColumns {
                inners: Box::new(slab::Slab::<Behavior<R, T>>::new()),
                metas: Default::default(),
            });

        let slab_key = behaviors
            .inners
            .as_any_mut()
            .downcast_mut::<slab::Slab<Behavior<T, R>>>()
            .check()
            .insert(Behavior {
                inner,
                relation: relation.clone(),
            }) as u32;

        let relation_0 = self
            .relation_0
            .entry(relation.clone())
            .or_default()
            .insert((type_key, slab_key)) as u32;

        let relation_1 = self
            .relation_1
            .entry((relation.clone(), type_key))
            .or_default()
            .insert(slab_key) as u32;

        behaviors.metas.insert(BehaviorMeta {
            relation_0,
            relation_1,
        });

        Some((type_key, slab_key))
    }

    pub fn remove<T: 'static>(&mut self, behavior_key: BehaviorKey) -> Option<(T, R)> {
        let (type_key, slab_key) = behavior_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let behavior = self.behaviors.get_mut(&type_key)?;

        let inner = behavior
            .inners
            .as_any_mut()
            .downcast_mut::<slab::Slab<Behavior<T, R>>>()
            .check()
            .try_remove(slab_key as usize)
            .check();

        let meta = behavior.metas.try_remove(slab_key as usize).check();

        self.relation_0
            .get_mut(&inner.relation)
            .check()
            .try_remove(meta.relation_0 as usize)
            .check();

        Some((inner.inner, inner.relation))
    }

    pub fn get<T: 'static>(&self, behavior_key: BehaviorKey) -> Option<(&T, &R)> {
        let (type_key, slab_key) = behavior_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let behavior = self.behaviors.get(&type_key)?;

        let inner = behavior
            .inners
            .as_any()
            .downcast_ref::<slab::Slab<Behavior<T, R>>>()
            .check()
            .get(slab_key as usize)
            .check();

        Some((&inner.inner, &inner.relation))
    }

    pub fn get_mut<T: 'static>(&mut self, behavior_key: BehaviorKey) -> Option<(&mut T, &R)> {
        let (type_key, slab_key) = behavior_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let behavior = self.behaviors.get_mut(&type_key)?;

        let inner = behavior
            .inners
            .as_any_mut()
            .downcast_mut::<slab::Slab<Behavior<T, R>>>()
            .check()
            .get_mut(slab_key as usize)
            .check();

        Some((&mut inner.inner, &inner.relation))
    }

    pub fn iter<T: 'static>(&self) -> Option<impl Iterator<Item = (&T, &R)>> {
        let type_key = std::any::TypeId::of::<T>();

        let behaviors = self.behaviors.get(&type_key)?;

        let iter = behaviors
            .inners
            .as_any()
            .downcast_ref::<slab::Slab<Behavior<T, R>>>()
            .check()
            .iter()
            .map(|(_, behavior)| (&behavior.inner, &behavior.relation));

        Some(iter)
    }

    pub fn iter_mut<T: 'static>(&mut self) -> Option<impl Iterator<Item = (&mut T, &R)>> {
        let type_key = std::any::TypeId::of::<T>();

        let behaviors = self.behaviors.get_mut(&type_key)?;

        let iter = behaviors
            .inners
            .as_any_mut()
            .downcast_mut::<slab::Slab<Behavior<T, R>>>()
            .check()
            .iter_mut()
            .map(|(_, behavior)| (&mut behavior.inner, &behavior.relation));

        Some(iter)
    }

    pub fn iter_by_relation<T: 'static>(
        &self,
        relation: R,
    ) -> Option<impl Iterator<Item = (&R, &T)>> {
        let type_key = std::any::TypeId::of::<T>();

        let inners = self
            .behaviors
            .get(&type_key)?
            .inners
            .as_any()
            .downcast_ref::<slab::Slab<Behavior<T, R>>>()
            .check();

        let iter = self
            .relation_1
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
        relation: R,
    ) -> Option<impl Iterator<Item = (&R, &mut T)>> {
        let type_key = std::any::TypeId::of::<T>();

        let inners = self
            .behaviors
            .get_mut(&type_key)?
            .inners
            .as_any_mut()
            .downcast_mut::<slab::Slab<Behavior<T, R>>>()
            .check();

        let iter = self
            .relation_1
            .get(&(relation, type_key))?
            .iter()
            .map(|(_, slab_key)| {
                let inner = inners.get_mut(*slab_key as usize).check() as *mut Behavior<T, R>;
                let inner = unsafe { &mut *inner };
                (&inner.relation, &mut inner.inner)
            });

        Some(iter)
    }

    pub fn remove_by_relation(&mut self, relation: R) -> Option<()> {
        if let Some(relation_0) = self.relation_0.remove(&relation) {
            for (_, (type_key, slab_key)) in relation_0 {
                let behavior = self.behaviors.get_mut(&type_key).check();

                behavior.inners.try_remove_any(slab_key as usize)?;

                let meta = behavior.metas.try_remove(slab_key as usize).check();

                self.relation_1
                    .get_mut(&(relation.clone(), type_key))
                    .check()
                    .try_remove(meta.relation_1 as usize)
                    .check();
            }
        }

        Some(())
    }
}
