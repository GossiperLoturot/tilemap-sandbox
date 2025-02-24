use crate::inner;

use super::*;

// resource

#[derive(Debug, Default)]
pub struct ForwarderResource {}

impl ForwarderResource {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn exec_rect(
        &self,
        root: &mut inner::Root,
        min_rect: [Vec2; 2],
        delta_secs: f32,
    ) -> Result<(), RootError> {
        // tile
        let chunk_size = root.tile_get_chunk_size() as f32;
        let chunk_size = Vec2::splat(chunk_size);
        let rect = [
            min_rect[0].div_euclid(chunk_size).as_ivec2(),
            min_rect[1].div_euclid(chunk_size).as_ivec2(),
        ];
        for y in rect[0].y..=rect[1].y {
            for x in rect[0].x..=rect[1].x {
                let chunk_location = IVec2::new(x, y);
                self.tile_forward_chunk(root, chunk_location, delta_secs);
            }
        }

        // block
        let chunk_size = root.block_get_chunk_size() as f32;
        let chunk_size = Vec2::splat(chunk_size);
        let rect = [
            min_rect[0].div_euclid(chunk_size).as_ivec2(),
            min_rect[1].div_euclid(chunk_size).as_ivec2(),
        ];
        for y in rect[0].y..=rect[1].y {
            for x in rect[0].x..=rect[1].x {
                let chunk_location = IVec2::new(x, y);
                self.block_forward_chunk(root, chunk_location, delta_secs);
            }
        }

        // entity
        let chunk_size = root.entity_get_chunk_size() as f32;
        let chunk_size = Vec2::splat(chunk_size);
        let rect = [
            min_rect[0].div_euclid(chunk_size).as_ivec2(),
            min_rect[1].div_euclid(chunk_size).as_ivec2(),
        ];
        for y in rect[0][1]..=rect[1][1] {
            for x in rect[0][0]..=rect[1][0] {
                let chunk_location = IVec2::new(x, y);
                self.entity_forward_chunk(root, chunk_location, delta_secs);
            }
        }

        Ok(())
    }

    pub fn tile_forward_chunk(
        &self,
        root: &mut inner::Root,
        chunk_location: IVec2,
        delta_secs: f32,
    ) {
        let Some(chunk_key) = root.tile_field.get_by_chunk_location(chunk_location) else {
            return;
        };
        let chunk = root.tile_field.get_chunk(chunk_key).unwrap();

        let mut local_keys = vec![];
        for (local_key, _) in &chunk.tiles {
            local_keys.push(local_key as u32);
        }

        let features = root.tile_features.clone();
        for local_key in local_keys {
            let tile_key = (chunk_key, local_key);
            let Ok(tile) = root.tile_field.get(tile_key) else {
                continue;
            };
            let feature = features.get(tile.id as usize).unwrap();
            feature.forward(root, tile_key, delta_secs);
        }
    }

    pub fn block_forward_chunk(
        &self,
        root: &mut inner::Root,
        chunk_location: IVec2,
        delta_secs: f32,
    ) {
        let Some(chunk_key) = root.block_field.get_by_chunk_location(chunk_location) else {
            return;
        };
        let chunk = root.block_field.get_chunk(chunk_key).unwrap();

        let mut local_keys = vec![];
        for (local_key, _) in &chunk.blocks {
            local_keys.push(local_key as u32);
        }

        let features = root.block_features.clone();
        for local_key in local_keys {
            let block_key = (chunk_key, local_key);
            let Ok(block) = root.block_field.get(block_key) else {
                continue;
            };
            let feature = features.get(block.id as usize).unwrap();
            feature.forward(root, block_key, delta_secs);
        }
    }

    pub fn entity_forward_chunk(
        &self,
        root: &mut inner::Root,
        chunk_location: IVec2,
        delta_secs: f32,
    ) {
        let Some(chunk_key) = root.entity_field.get_by_chunk_location(chunk_location) else {
            return;
        };
        let chunk = root.entity_field.get_chunk(chunk_key).unwrap();

        let mut local_keys = vec![];
        for (local_key, _) in &chunk.entities {
            local_keys.push(local_key as u32);
        }

        let features = root.entity_features.clone();
        for local_key in local_keys {
            let entity_key = (chunk_key, local_key);
            let Ok(entity) = root.entity_field.get(entity_key) else {
                continue;
            };
            let feature = features.get(entity.id as usize).unwrap();
            feature.forward(root, entity_key, delta_secs);
        }
    }
}
