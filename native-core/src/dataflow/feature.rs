/// FeatureColumn represents a category of functions that can be called.
/// Each FeatureColumn has one function with a specific signature.
/// Additionally, a single FeatureRow can implement multiple FeatureColumns,
/// each with a different type, but only one of each type is allowed.
pub trait FeatureColumn {}

/// FeatureRow has the implementation of the function to be called.
/// FeatureRow can have one of each type of FeatureColumn with different types.
/// create_row is called when registering a new row.
pub trait FeatureRow {
    fn create_row(&self, builder: &mut FeatureRowBuilder) -> Result<(), FeatureError>;
}

// empty implementation
impl FeatureRow for () {
    fn create_row(&self, _: &mut FeatureRowBuilder) -> Result<(), FeatureError> {
        Ok(())
    }
}

// internal data structure

struct Column {
    sparse: Vec<Option<u16>>,
    dense: Box<dyn std::any::Any>,
}

impl Column {
    fn new<T: 'static>(size: u16) -> Self {
        Self {
            sparse: vec![None; size as usize],
            dense: Box::new(Vec::<T>::new()),
        }
    }

    fn dense_ref<T: 'static>(&self) -> Option<&Vec<T>> {
        self.dense.downcast_ref::<Vec<T>>()
    }

    fn dense_mut<T: 'static>(&mut self) -> Option<&mut Vec<T>> {
        self.dense.downcast_mut::<Vec<T>>()
    }
}

// external data structure

pub struct ColumnRef<'a, T> {
    sparse: &'a Vec<Option<u16>>,
    dense: &'a Vec<T>,
}

impl<'a, T: 'static> ColumnRef<'a, T> {
    pub fn get(&self, id: u16) -> Result<&'a T, FeatureError> {
        let sparse_index = self
            .sparse
            .get(id as usize)
            .ok_or(FeatureError::RowNotFound)?;
        let sparse_index = sparse_index.as_ref().ok_or(FeatureError::RowNotFound)?;
        let val = self.dense.get(*sparse_index as usize).unwrap();
        Ok(val)
    }
}

/// FeatureMatrixBuilder is used to build a FeatureMatrix.
/// It is used to create a new row and add columns to it.
#[derive(Default)]
pub struct FeatureMatrixBuilder {
    row_len: usize,
    matrix: ahash::AHashMap<std::any::TypeId, Column>,
}

impl FeatureMatrixBuilder {
    pub fn insert_row(&mut self) -> FeatureRowBuilder {
        FeatureRowBuilder {
            row_len: &mut self.row_len,
            matrix: &mut self.matrix,
        }
    }

    pub fn build(self) -> FeatureMatrix {
        for (_, column) in &self.matrix {
            debug_assert_eq!(column.sparse.len(), self.row_len);
        }

        FeatureMatrix {
            matrix: self.matrix,
        }
    }
}

/// FeatureRowBuilder is used to build a new row.
/// It is used to add columns to the row.
pub struct FeatureRowBuilder<'a> {
    row_len: &'a mut usize,
    matrix: &'a mut ahash::HashMap<std::any::TypeId, Column>,
}

impl FeatureRowBuilder<'_> {
    pub fn add_column<T: FeatureColumn + 'static>(&mut self, value: T) -> Result<(), FeatureError> {
        let typ = std::any::TypeId::of::<T>();
        let column = self
            .matrix
            .entry(typ)
            .or_insert_with(|| Column::new::<T>(*self.row_len as u16));

        if column.sparse.len() != *self.row_len {
            return Err(FeatureError::RowAlreadyExists);
        }

        let dense_column = column.dense_mut().unwrap();
        dense_column.push(value);
        let dense_row = (dense_column.len() - 1) as u16;

        column.sparse.push(Some(dense_row));

        Ok(())
    }
}

impl Drop for FeatureRowBuilder<'_> {
    fn drop(&mut self) {
        *self.row_len += 1;

        for column in (*self.matrix).values_mut() {
            if column.sparse.len() == *self.row_len - 1 {
                column.sparse.push(None);
            }

            debug_assert_eq!(column.sparse.len(), *self.row_len);
        }
    }
}

/// FeatureMatrix is a collection of FeatureColumns.
/// It is used to store the data for each row.
pub struct FeatureMatrix {
    matrix: ahash::AHashMap<std::any::TypeId, Column>,
}

impl FeatureMatrix {
    pub fn new(builder: FeatureMatrixBuilder) -> Self {
        builder.build()
    }

    pub fn get_column<T: FeatureColumn + 'static>(&self) -> Result<ColumnRef<'_, T>, FeatureError> {
        let typ = std::any::TypeId::of::<T>();
        let column = self.matrix.get(&typ).ok_or(FeatureError::ColumnNotFound)?;
        let sparse = &column.sparse;
        let dense = column.dense_ref::<T>().unwrap();
        Ok(ColumnRef { sparse, dense })
    }

    pub fn get<T: FeatureColumn + 'static>(&self, id: u16) -> Result<&T, FeatureError> {
        self.get_column::<T>()?.get(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeatureError {
    ColumnNotFound,
    RowNotFound,
    RowAlreadyExists,
}

impl std::fmt::Display for FeatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeatureError::ColumnNotFound => write!(f, "Column not found"),
            FeatureError::RowNotFound => write!(f, "Row not found"),
            FeatureError::RowAlreadyExists => write!(f, "Row already exists"),
        }
    }
}

impl std::error::Error for FeatureError {}
