use glam::*;
use native_core::dataflow::*;

// feature

pub trait ForwardFeature {
    type Key;
    fn forward(&self, dataflow: &mut Dataflow, key: Self::Key, delta_secs: f32);
}

// system

pub struct ForwarderSystem;

impl ForwarderSystem {
    pub fn forward(
        dataflow: &mut Dataflow,
        min_rect: [Vec2; 2],
        delta_secs: f32,
    ) -> Result<(), std::convert::Infallible> {
        // tile
        let rect = [
            dataflow.find_tile_chunk_coord(min_rect[0]),
            dataflow.find_tile_chunk_coord(min_rect[1]),
        ];
        for y in rect[0].y..=rect[1].y {
            for x in rect[0].x..=rect[1].x {
                let chunk_location = IVec2::new(x, y);
                forward_tile_chunk(dataflow, chunk_location, delta_secs);
            }
        }

        // block
        let rect = [
            dataflow.find_block_chunk_coord(min_rect[0]),
            dataflow.find_block_chunk_coord(min_rect[1]),
        ];
        for y in rect[0].y..=rect[1].y {
            for x in rect[0].x..=rect[1].x {
                let chunk_location = IVec2::new(x, y);
                forward_block_chunk(dataflow, chunk_location, delta_secs);
            }
        }

        // entity
        let rect = [
            dataflow.find_entity_chunk_coord(min_rect[0]),
            dataflow.find_entity_chunk_coord(min_rect[1]),
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

fn forward_tile_chunk(_dataflow: &mut Dataflow, _chunk_location: IVec2, _delta_secs: f32) {
    // // TODO: implement tile forwarding
    // let Ok(tile_keys) = dataflow.get_tile_chunk_tiles(chunk_location) else {
    //     return;
    // };
    //
    // for tile_key in tile_keys.collect::<Vec<_>>() {
    //     let Ok(tile) = dataflow.get_tile(tile_key) else {
    //         continue;
    //     };
    //
    //     let Ok(feature) =
    //         dataflow.get_tile_feature::<Rc<dyn ForwardFeature<Key = EntityId>>>(tile.archetype_id)
    //     else {
    //         continue;
    //     };
    //
    //     let feature = feature.clone();
    //     feature.forward(dataflow, tile_key, delta_secs);
    // }
}

fn forward_block_chunk(_dataflow: &mut Dataflow, _chunk_location: IVec2, _delta_secs: f32) {
    // // TODO: implement block forwarding
    // let Ok(block_keys) = dataflow.get_block_ids_by_chunk_coord(chunk_location) else {
    //     return;
    // };
    //
    // for block_key in block_keys.collect::<Vec<_>>() {
    //     let Ok(block) = dataflow.get_block(block_key) else {
    //         continue;
    //     };
    //
    //     let Ok(feature) =
    //         dataflow.get_block_feature::<Rc<dyn ForwardFeature<Key = EntityId>>>(block.archetype_id)
    //     else {
    //         continue;
    //     };
    //
    //     let feature = feature.clone();
    //     feature.forward(dataflow, block_key, delta_secs);
    // }
}

fn forward_entity_chunk(_dataflow: &mut Dataflow, _chunk_location: IVec2, _delta_secs: f32) {
    // // TODO: implement entity forwarding
    // let Ok(entity_keys) = dataflow.get_entity_ids_by_chunk_coord(chunk_location) else {
    //     return;
    // };
    //
    // for entity_key in entity_keys.collect::<Vec<_>>() {
    //     let Ok(entity) = dataflow.get_entity(entity_key) else {
    //         continue;
    //     };
    //
    //     let Ok(feature) =
    //         dataflow.get_entity_feature::<Rc<dyn ForwardFeature<Key = EntityId>>>(entity.archetype_id)
    //     else {
    //         continue;
    //     };
    //
    //     let feature = feature.clone();
    //     feature.forward(dataflow, entity_key, delta_secs);
    // }
}
