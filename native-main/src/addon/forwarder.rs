use glam::*;
use native_core::dataflow::*;

pub struct ForwarderSystem;

impl ForwarderSystem {
    pub fn forward(
        dataflow: &mut Dataflow,
        min_rect: [Vec2; 2],
        delta_secs: f32,
    ) -> Result<(), std::convert::Infallible> {
        // tile
        let rect = [
            dataflow.get_tile_chunk_location(min_rect[0]),
            dataflow.get_tile_chunk_location(min_rect[1]),
        ];
        for y in rect[0].y..=rect[1].y {
            for x in rect[0].x..=rect[1].x {
                let chunk_location = IVec2::new(x, y);
                forward_tile_chunk(dataflow, chunk_location, delta_secs);
            }
        }

        // block
        let rect = [
            dataflow.get_block_chunk_location(min_rect[0]),
            dataflow.get_block_chunk_location(min_rect[1]),
        ];
        for y in rect[0].y..=rect[1].y {
            for x in rect[0].x..=rect[1].x {
                let chunk_location = IVec2::new(x, y);
                forward_block_chunk(dataflow, chunk_location, delta_secs);
            }
        }

        // entity
        let rect = [
            dataflow.get_entity_chunk_location(min_rect[0]),
            dataflow.get_entity_chunk_location(min_rect[1]),
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

fn forward_tile_chunk(dataflow: &mut Dataflow, chunk_location: IVec2, delta_secs: f32) {
    let Ok(tile_keys) = dataflow.get_tile_keys_by_chunk_location(chunk_location) else {
        return;
    };

    for tile_key in tile_keys.collect::<Vec<_>>() {
        let Ok(tile) = dataflow.get_tile(tile_key) else {
            continue;
        };

        let feature = dataflow.get_tile_feature(tile.id).unwrap();
        feature.forward(dataflow, tile_key, delta_secs);
    }
}

fn forward_block_chunk(dataflow: &mut Dataflow, chunk_location: IVec2, delta_secs: f32) {
    let Ok(block_keys) = dataflow.get_block_keys_by_chunk_location(chunk_location) else {
        return;
    };

    for block_key in block_keys.collect::<Vec<_>>() {
        let Ok(block) = dataflow.get_block(block_key) else {
            continue;
        };

        let feature = dataflow.get_block_feature(block.id).unwrap();
        feature.forward(dataflow, block_key, delta_secs);
    }
}

fn forward_entity_chunk(dataflow: &mut Dataflow, chunk_location: IVec2, delta_secs: f32) {
    let Ok(entity_keys) = dataflow.get_entity_keys_by_chunk_location(chunk_location) else {
        return;
    };

    for entity_key in entity_keys.collect::<Vec<_>>() {
        let Ok(entity) = dataflow.get_entity(entity_key) else {
            continue;
        };

        let feature = dataflow.get_entity_feature(entity.id).unwrap();
        feature.forward(dataflow, entity_key, delta_secs);
    }
}
