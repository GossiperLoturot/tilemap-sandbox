use crate::inner;

use super::*;

// resource

#[derive(Debug, Clone)]
pub struct ForwarderResource;

impl ForwarderResource {
    pub fn init(root: &mut inner::Root) -> Result<(), ForwarderError> {
        let slf = Self;
        root.resource_insert(slf)?;
        Ok(())
    }

    pub fn exec_rect(
        root: &mut inner::Root,
        min_rect: [inner::Vec2; 2],
        delta_secs: f32,
    ) -> Result<(), ForwarderError> {
        let slf = root.resource_remove::<Self>()?;

        // tile
        let chunk_size = root.tile_get_chunk_size() as f32;
        #[rustfmt::skip]
        let rect = [[
            min_rect[0][0].div_euclid(chunk_size) as i32,
            min_rect[0][1].div_euclid(chunk_size) as i32, ], [
            min_rect[1][0].div_euclid(chunk_size) as i32,
            min_rect[1][1].div_euclid(chunk_size) as i32,
        ]];
        for y in rect[0][1]..=rect[1][1] {
            for x in rect[0][0]..=rect[1][0] {
                let _ = root.tile_forward_chunk([x, y], delta_secs);
            }
        }

        // block
        let chunk_size = root.block_get_chunk_size() as f32;
        #[rustfmt::skip]
        let rect = [[
            min_rect[0][0].div_euclid(chunk_size) as i32,
            min_rect[0][1].div_euclid(chunk_size) as i32, ], [
            min_rect[1][0].div_euclid(chunk_size) as i32,
            min_rect[1][1].div_euclid(chunk_size) as i32,
        ]];
        for y in rect[0][1]..=rect[1][1] {
            for x in rect[0][0]..=rect[1][0] {
                let _ = root.block_forward_chunk([x, y], delta_secs);
            }
        }

        // entity
        let chunk_size = root.entity_get_chunk_size() as f32;
        #[rustfmt::skip]
        let rect = [[
            min_rect[0][0].div_euclid(chunk_size) as i32,
            min_rect[0][1].div_euclid(chunk_size) as i32, ], [
            min_rect[1][0].div_euclid(chunk_size) as i32,
            min_rect[1][1].div_euclid(chunk_size) as i32,
        ]];
        for y in rect[0][1]..=rect[1][1] {
            for x in rect[0][0]..=rect[1][0] {
                let _ = root.entity_forward_chunk([x, y], delta_secs);
            }
        }

        root.resource_insert(slf).unwrap();
        Ok(())
    }
}

// error handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForwarderError {
    Resource(ResourceError),
}

impl std::fmt::Display for ForwarderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resource(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for ForwarderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Resource(e) => Some(e),
        }
    }
}

impl From<ResourceError> for ForwarderError {
    fn from(value: ResourceError) -> Self {
        Self::Resource(value)
    }
}
