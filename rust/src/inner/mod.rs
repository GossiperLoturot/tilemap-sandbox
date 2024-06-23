pub use behavior::*;
pub use field::*;
pub use node::*;
pub use world::*;

mod behavior;
mod field;
mod node;
mod world;

pub type Vec2 = [f32; 2];
pub type IVec2 = [i32; 2];

// Error Handling

pub trait IntegrityCheck<T> {
    fn check(self) -> T;
}

impl<T> IntegrityCheck<T> for Option<T> {
    fn check(self) -> T {
        self.expect("integrity error")
    }
}

impl<T, U: std::error::Error> IntegrityCheck<T> for Result<T, U> {
    fn check(self) -> T {
        self.expect("integrity error")
    }
}

#[derive(Debug, Clone)]
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
