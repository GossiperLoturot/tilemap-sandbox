type ResourceRow<T> = Option<T>;

#[derive(Debug, Default)]
pub struct ResourceStore {
    typ_map: ahash::AHashMap<std::any::TypeId, Box<dyn std::any::Any>>,
}

impl ResourceStore {
    pub fn insert<T: 'static>(&mut self, value: T) -> Result<(), ResourceError> {
        let typ = std::any::TypeId::of::<T>();

        if self.typ_map.contains_key(&typ) {
            return Err(ResourceError::Conflict);
        }

        self.typ_map.insert(typ, Box::new(Some(value)));
        Ok(())
    }

    pub fn remove<T: 'static>(&mut self) -> Result<T, ResourceError> {
        let typ = std::any::TypeId::of::<T>();

        let resource = self.typ_map.remove(&typ).ok_or(ResourceError::NotFound)?;
        let resource = resource
            .downcast::<ResourceRow<T>>()
            .unwrap()
            .take()
            .unwrap();
        Ok(resource)
    }

    pub fn has<T: 'static>(&self) -> bool {
        let typ = std::any::TypeId::of::<T>();

        self.typ_map.contains_key(&typ)
    }

    pub fn get<T: 'static>(&self) -> Result<&T, ResourceError> {
        let typ = std::any::TypeId::of::<T>();

        let resource = self.typ_map.get(&typ).ok_or(ResourceError::NotFound)?;
        let resource = resource
            .downcast_ref::<ResourceRow<T>>()
            .unwrap()
            .as_ref()
            .unwrap();
        Ok(resource)
    }

    pub fn get_mut<T: 'static>(&mut self) -> Result<&mut T, ResourceError> {
        let typ = std::any::TypeId::of::<T>();

        let resource = self.typ_map.get_mut(&typ).ok_or(ResourceError::NotFound)?;
        let reosurce = resource
            .downcast_mut::<ResourceRow<T>>()
            .unwrap()
            .as_mut()
            .unwrap();
        Ok(reosurce)
    }
}

// error handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceError {
    NotFound,
    Conflict,
    InvalidId,
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "not found error"),
            Self::Conflict => write!(f, "conflict error"),
            Self::InvalidId => write!(f, "invalid id error"),
        }
    }
}

impl std::error::Error for ResourceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud() {
        let mut store = super::ResourceStore::default();
        assert_eq!(store.has::<i32>(), false);
        assert_eq!(store.get::<i32>(), Err(ResourceError::NotFound));
        assert_eq!(store.get_mut::<i32>(), Err(ResourceError::NotFound));
        assert_eq!(store.remove::<i32>(), Err(ResourceError::NotFound));

        assert_eq!(store.insert(42), Ok(()));
        assert_eq!(store.insert(42), Err(ResourceError::Conflict));

        assert_eq!(store.has::<i32>(), true);
        assert_eq!(store.get::<i32>(), Ok(&42));
        assert_eq!(store.get_mut::<i32>(), Ok(&mut 42));

        assert_eq!(store.remove::<i32>(), Ok(42));
        assert_eq!(store.remove::<i32>(), Err(ResourceError::NotFound));
    }
}
