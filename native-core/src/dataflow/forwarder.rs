use crate::dataflow;

use super::*;

pub struct ForwarderSystem;

impl ForwarderSystem {
    pub fn forward(
        dataflow: &mut dataflow::Dataflow,
        min_rect: [Vec2; 2],
        delta_secs: f32,
    ) -> Result<(), std::convert::Infallible> {
        // tile
        let chunk_size = dataflow.get_tile_chunk_size() as f32;
        let chunk_size = Vec2::splat(chunk_size);
        let rect = [
            min_rect[0].div_euclid(chunk_size).as_ivec2(),
            min_rect[1].div_euclid(chunk_size).as_ivec2(),
        ];
        for y in rect[0].y..=rect[1].y {
            for x in rect[0].x..=rect[1].x {
                let chunk_location = IVec2::new(x, y);
                forward_tile_chunk(dataflow, chunk_location, delta_secs);
            }
        }

        // block
        let chunk_size = dataflow.get_block_chunk_size() as f32;
        let chunk_size = Vec2::splat(chunk_size);
        let rect = [
            min_rect[0].div_euclid(chunk_size).as_ivec2(),
            min_rect[1].div_euclid(chunk_size).as_ivec2(),
        ];
        for y in rect[0].y..=rect[1].y {
            for x in rect[0].x..=rect[1].x {
                let chunk_location = IVec2::new(x, y);
                forward_block_chunk(dataflow, chunk_location, delta_secs);
            }
        }

        // entity
        let chunk_size = dataflow.get_entity_chunk_size() as f32;
        let chunk_size = Vec2::splat(chunk_size);
        let rect = [
            min_rect[0].div_euclid(chunk_size).as_ivec2(),
            min_rect[1].div_euclid(chunk_size).as_ivec2(),
        ];
        for y in rect[0].y..=rect[1].y {
            for x in rect[0].x..=rect[1].x {
                let chunk_location = IVec2::new(x, y);
                forward_entity_chunk(dataflow, chunk_location, delta_secs);
            }
        }

        Ok(())
    }
}

fn forward_tile_chunk(dataflow: &mut dataflow::Dataflow, chunk_location: IVec2, delta_secs: f32) {
    let Some(chunk_key) = dataflow.tile_field.get_by_chunk_location(chunk_location) else {
        return;
    };
    let chunk = dataflow.tile_field.get_chunk(chunk_key).unwrap();

    let mut local_keys = vec![];
    for (local_key, _) in &chunk.tiles {
        local_keys.push(local_key as u32);
    }

    let features = dataflow.tile_features.clone();
    for local_key in local_keys {
        let tile_key = (chunk_key, local_key);
        let Ok(tile) = dataflow.tile_field.get(tile_key) else {
            continue;
        };
        let feature = features.get(tile.id as usize).unwrap();
        feature.forward(dataflow, tile_key, delta_secs);
    }
}

fn forward_block_chunk(dataflow: &mut dataflow::Dataflow, chunk_location: IVec2, delta_secs: f32) {
    let Some(chunk_key) = dataflow.block_field.get_by_chunk_location(chunk_location) else {
        return;
    };
    let chunk = dataflow.block_field.get_chunk(chunk_key).unwrap();

    let mut local_keys = vec![];
    for (local_key, _) in &chunk.blocks {
        local_keys.push(local_key as u32);
    }

    let features = dataflow.block_features.clone();
    for local_key in local_keys {
        let block_key = (chunk_key, local_key);
        let Ok(block) = dataflow.block_field.get(block_key) else {
            continue;
        };
        let feature = features.get(block.id as usize).unwrap();
        feature.forward(dataflow, block_key, delta_secs);
    }
}

fn forward_entity_chunk(dataflow: &mut dataflow::Dataflow, chunk_location: IVec2, delta_secs: f32) {
    let Some(chunk_key) = dataflow.entity_field.get_by_chunk_location(chunk_location) else {
        return;
    };
    let chunk = dataflow.entity_field.get_chunk(chunk_key).unwrap();

    let mut local_keys = vec![];
    for (local_key, _) in &chunk.entities {
        local_keys.push(local_key as u32);
    }

    let features = dataflow.entity_features.clone();
    for local_key in local_keys {
        let entity_key = (chunk_key, local_key);
        let Ok(entity) = dataflow.entity_field.get(entity_key) else {
            continue;
        };
        let feature = features.get(entity.id as usize).unwrap();
        feature.forward(dataflow, entity_key, delta_secs);
    }
}
