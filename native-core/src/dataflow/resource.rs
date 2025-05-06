pub trait Resource {}

#[derive(Debug, Clone)]
pub struct ResourceCell<T> {
    value: std::rc::Rc<std::cell::RefCell<T>>,
}

impl<T> ResourceCell<T> {
    pub fn borrow(&self) -> Result<std::cell::Ref<T>, ResourceError> {
        self.value.try_borrow().map_err(|_| ResourceError::Busy)
    }

    pub fn borrow_mut(&self) -> Result<std::cell::RefMut<T>, ResourceError> {
        self.value.try_borrow_mut().map_err(|_| ResourceError::Busy)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResourceStorage {
    resources: ahash::AHashMap<std::any::TypeId, std::rc::Rc<dyn std::any::Any>>,
}

impl ResourceStorage {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert<T>(&mut self, resource: T) -> Result<(), ResourceError>
    where
        T: Resource + 'static,
    {
        let typed = std::rc::Rc::new(std::cell::RefCell::new(resource));
        let wrap = typed as std::rc::Rc<dyn std::any::Any>;

        let typ = std::any::TypeId::of::<T>();
        self.resources.insert(typ, wrap);

        Ok(())
    }

    pub fn remove<T>(&mut self) -> Result<T, ResourceError>
    where
        T: Resource + 'static,
    {
        let typ = std::any::TypeId::of::<T>();
        let wrap = self.resources.remove(&typ).ok_or(ResourceError::NotFound)?;

        let typed = wrap.downcast::<std::cell::RefCell<T>>().unwrap();
        let value = std::rc::Rc::into_inner(typed).ok_or(ResourceError::Busy)?;
        let value = std::cell::RefCell::into_inner(value);
        Ok(value)
    }

    pub fn find<T>(&self) -> Result<ResourceCell<T>, ResourceError>
    where
        T: Resource + 'static,
    {
        let typ = std::any::TypeId::of::<T>();
        let wrap_ref = self.resources.get(&typ).ok_or(ResourceError::NotFound)?;
        let wrap = wrap_ref.clone();

        let typed = wrap.downcast::<std::cell::RefCell<T>>().unwrap();
        Ok(ResourceCell { value: typed })
    }
}

// error handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceError {
    AlreadyExist,
    NotFound,
    Busy,
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyExist => write!(f, "already exist error"),
            Self::NotFound => write!(f, "not found error"),
            Self::Busy => write!(f, "busy error"),
        }
    }
}

impl std::error::Error for ResourceError {}
