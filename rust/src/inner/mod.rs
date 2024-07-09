pub use callback::*;
pub use field::*;
pub use node::*;

mod callback;
mod field;
mod node;

pub type Vec2 = [f32; 2];
pub type IVec2 = [i32; 2];

pub struct World<'a> {
    pub tile_field: &'a mut TileField,
    pub block_field: &'a mut BlockField,
    pub entity_field: &'a mut EntityField,
    pub node_store: &'a mut NodeStore,
    pub callback_store: &'a CallbackStore,
}

// Error Handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldError {
    NotFound,
    Conflict,
    InvalidId,
}

impl std::fmt::Display for FieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldError::NotFound => write!(f, "not found error"),
            FieldError::Conflict => write!(f, "conflict error"),
            FieldError::InvalidId => write!(f, "invalid id error"),
        }
    }
}

impl std::error::Error for FieldError {}
