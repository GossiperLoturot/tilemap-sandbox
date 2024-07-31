#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlowRef {
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

/// A bundle of flows.
/// self is the reference for compability with dyn T.
pub trait FlowBundle {
    fn insert(&self, buf: &mut FlowBuffer);
}

#[derive(Default, Debug)]
pub struct FlowBuffer {
    flows: Vec<(std::any::TypeId, FlowRef, Box<dyn std::any::Any>)>,
}

impl FlowBuffer {
    pub fn register<T: 'static>(&mut self, r#ref: FlowRef, flow: T) {
        let typ = std::any::TypeId::of::<T>();

        if self.flows.len() >= u32::MAX as usize {
            panic!("capacity overflow");
        }

        self.flows.push((typ, r#ref, Box::new(flow)));
    }
}

pub struct FlowDescriptor {
    pub value: std::rc::Rc<dyn FlowBundle>,
}

pub struct FlowStoreDescriptor {
    pub bundles: Vec<FlowDescriptor>,
}

#[derive(Default, Debug)]
pub struct FlowStore {
    flows: Vec<Box<dyn std::any::Any>>,
    ref_cols_0: ahash::AHashMap<(std::any::TypeId,), Vec<u32>>,
    ref_cols_1: ahash::AHashMap<(std::any::TypeId, FlowRef), Vec<u32>>,
}

impl FlowStore {
    pub fn new(desc: FlowStoreDescriptor) -> Self {
        let mut buffer = FlowBuffer::default();

        for bundle in desc.bundles {
            bundle.value.insert(&mut buffer);
        }

        let mut store = FlowStore::default();
        for (typ, r#ref, flow) in buffer.flows {
            // key is guaranteed to be less than u32::MAX
            store.flows.push(flow);
            let key = (store.flows.len() - 1) as u32;

            let ref_col_0 = store.ref_cols_0.entry((typ,)).or_default();
            ref_col_0.push(key);

            let ref_col_1 = store.ref_cols_1.entry((typ, r#ref)).or_default();
            ref_col_1.push(key);
        }
        store
    }

    fn iter_internal<T: 'static>(&self) -> Option<impl Iterator<Item = &T>> {
        let typ = std::any::TypeId::of::<T>();

        let ref_col = self.ref_cols_0.get(&(typ,))?;

        let iter = ref_col.iter().map(|key| {
            let flow = self.flows.get(*key as usize).unwrap();
            flow.downcast_ref::<T>().unwrap()
        });

        Some(iter)
    }

    fn iter_by_ref_internal<T: 'static>(&self, r#ref: FlowRef) -> Option<impl Iterator<Item = &T>> {
        let typ = std::any::TypeId::of::<T>();

        let ref_col = self.ref_cols_1.get(&(typ, r#ref))?;

        let iter = ref_col.iter().map(|key| {
            let flow = self.flows.get(*key as usize).unwrap();
            flow.downcast_ref::<T>().unwrap()
        });

        Some(iter)
    }

    #[inline]
    pub fn iter<T: 'static>(&self) -> impl Iterator<Item = &T> {
        self.iter_internal::<T>().into_iter().flatten()
    }

    #[inline]
    pub fn one<T: 'static>(&self) -> Option<&T> {
        self.iter::<T>().next()
    }

    #[inline]
    pub fn iter_by_ref<T: 'static>(&self, r#ref: FlowRef) -> impl Iterator<Item = &T> {
        self.iter_by_ref_internal::<T>(r#ref).into_iter().flatten()
    }

    #[inline]
    pub fn one_by_ref<T: 'static>(&self, r#ref: FlowRef) -> Option<&T> {
        self.iter_by_ref::<T>(r#ref).next()
    }
}
