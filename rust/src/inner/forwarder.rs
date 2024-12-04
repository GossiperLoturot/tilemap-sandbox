use crate::inner;

use super::*;

// resource

#[derive(Debug, Clone, Default)]
pub struct ForwarderResource {}

impl ForwarderResource {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn exec_rect(
        &self,
        root: &mut inner::Root,
        min_rect: [inner::Vec2; 2],
        delta_secs: f32,
    ) -> Result<(), ForwarderError> {
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
                let _ = self.tile_forward_chunk(root, [x, y], delta_secs);
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
                let _ = self.block_forward_chunk(root, [x, y], delta_secs);
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
                let _ = self.entity_forward_chunk(root, [x, y], delta_secs);
            }
        }

        Ok(())
    }

    pub fn tile_forward_chunk(
        &self,
        root: &mut inner::Root,
        chunk_location: IVec2,
        delta_secs: f32,
    ) -> Result<(), FieldError> {
        let chunk_key = root
            .tile_field
            .get_by_chunk_location(chunk_location)
            .ok_or(FieldError::NotFound)?;
        let chunk = root.tile_field.get_chunk(chunk_key)?;

        let mut local_keys = vec![];
        for (local_key, _) in &chunk.tiles {
            local_keys.push(local_key as u32);
        }

        let features = root.tile_features.clone();
        for local_key in local_keys {
            let tile_key = (chunk_key, local_key);
            let tile = root.tile_field.get(tile_key).unwrap();
            let feature = features
                .get(tile.id as usize)
                .ok_or(FieldError::InvalidId)?;
            feature.forward(root, tile_key, delta_secs);
        }
        Ok(())
    }

    pub fn block_forward_chunk(
        &self,
        root: &mut inner::Root,
        chunk_location: IVec2,
        delta_secs: f32,
    ) -> Result<(), FieldError> {
        let chunk_key = root
            .block_field
            .get_by_chunk_location(chunk_location)
            .ok_or(FieldError::NotFound)?;
        let chunk = root.block_field.get_chunk(chunk_key)?;

        let mut local_keys = vec![];
        for (local_key, _) in &chunk.blocks {
            local_keys.push(local_key as u32);
        }

        let features = root.block_features.clone();
        for local_key in local_keys {
            let block_key = (chunk_key, local_key);
            let block = root.block_field.get(block_key).unwrap();
            let feature = features
                .get(block.id as usize)
                .ok_or(FieldError::InvalidId)?;
            feature.forward(root, block_key, delta_secs);
        }
        Ok(())
    }

    pub fn entity_forward_chunk(
        &self,
        root: &mut inner::Root,
        chunk_location: IVec2,
        delta_secs: f32,
    ) -> Result<(), FieldError> {
        let chunk_key = root
            .entity_field
            .get_by_chunk_location(chunk_location)
            .ok_or(FieldError::NotFound)?;
        let chunk = root.entity_field.get_chunk(chunk_key)?;

        let mut local_keys = vec![];
        for (local_key, _) in &chunk.entities {
            local_keys.push(local_key as u32);
        }

        let features = root.entity_features.clone();
        for local_key in local_keys {
            let entity_key = (chunk_key, local_key);
            let entity = root.entity_field.get(entity_key).unwrap();
            let feature = features
                .get(entity.id as usize)
                .ok_or(FieldError::InvalidId)?;
            feature.forward(root, entity_key, delta_secs);
        }
        Ok(())
    }
}

// error handling

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForwarderError {
    NotScoped,
}

impl std::fmt::Display for ForwarderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotScoped => write!(f, "not scoped error"),
        }
    }
}
