/// FeatureRow has the implementation of the function to be called.
/// FeatureRow can have one of each type of FeatureColumn with different types.
/// create_row is called when registering a new row.
pub trait FeatureRow {
    fn create_row(&self, builder: &mut FeatureRowBuilder) -> Result<(), FeatureError>;
}

// empty implementation
impl FeatureRow for () {
    fn create_row(&self, _builder: &mut FeatureRowBuilder) -> Result<(), FeatureError> {
        Ok(())
    }
}

/// FeatureMatrixBuilder is used to build a FeatureMatrix.
/// It is used to create a new row and add columns to it.
#[derive(Default)]
pub struct FeatureMatrixBuilder {
    row_len: u16,
    matrix: ahash::AHashMap<(std::any::TypeId, u16), Box<dyn std::any::Any>>,
}

impl FeatureMatrixBuilder {
    pub fn insert_row(&mut self) -> FeatureRowBuilder {
        FeatureRowBuilder {
            row_len: &mut self.row_len,
            matrix: &mut self.matrix,
        }
    }

    pub fn build(self) -> FeatureMatrix {
        FeatureMatrix {
            matrix: self.matrix,
        }
    }
}

/// FeatureRowBuilder is used to build a new row.
/// It is used to add columns to the row.
pub struct FeatureRowBuilder<'a> {
    row_len: &'a mut u16,
    matrix: &'a mut ahash::AHashMap<(std::any::TypeId, u16), Box<dyn std::any::Any>>,
}

impl FeatureRowBuilder<'_> {
    pub fn add_column<T: 'static>(&mut self, value: T) -> Result<(), FeatureError> {
        let key = (std::any::TypeId::of::<T>(), *self.row_len);

        if self.matrix.contains_key(&key) {
            return Err(FeatureError::AlreadyExists);
        }

        self.matrix.insert(key, Box::new(value));
        Ok(())
    }
}

impl Drop for FeatureRowBuilder<'_> {
    fn drop(&mut self) {
        *self.row_len += 1;
    }
}

/// FeatureMatrix is a collection of FeatureColumns.
/// It is used to store the data for each row.
pub struct FeatureMatrix {
    matrix: ahash::AHashMap<(std::any::TypeId, u16), Box<dyn std::any::Any>>,
}

impl FeatureMatrix {
    pub fn new(builder: FeatureMatrixBuilder) -> Self {
        builder.build()
    }

    pub fn get<T: 'static>(&self, id: u16) -> Result<&T, FeatureError> {
        let key = (std::any::TypeId::of::<T>(), id);
        let any = self.matrix.get(&key).ok_or(FeatureError::NotFound)?;
        let value = any.downcast_ref::<T>().unwrap();
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeatureError {
    NotFound,
    AlreadyExists,
}

impl std::fmt::Display for FeatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeatureError::NotFound => write!(f, "feature not found"),
            FeatureError::AlreadyExists => write!(f, "feature already exists"),
        }
    }
}

impl std::error::Error for FeatureError {}
