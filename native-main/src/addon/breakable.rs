use std::rc::Rc;

use glam::*;
use native_core::dataflow::*;

use super::*;

// feature

pub trait BreakFeature<D> {
    fn r#break(&self, dataflow: &mut Dataflow, data: &D, location: Vec2);
}

// concrete feature

#[derive(Debug, Clone)]
pub struct BreakFeatureSet {
    pub item_entity_id: u16,
    pub item_id: u16,
}

impl FeatureSet for BreakFeatureSet {
    fn attach_set(&self, b: &mut FeatureSetBuilder) -> Result<(), FeatureError> {
        let slf = Rc::new(self.clone());
        // b.insert::<Rc<dyn BreakFeature<Tile>>>(slf.clone())?;
        // b.insert::<Rc<dyn BreakFeature<Block>>>(slf.clone())?;
        // b.insert::<Rc<dyn BreakFeature<Entity>>>(slf.clone())?;
        Ok(())
    }
}

// impl<D> BreakFeature<D> for BreakFeatureSet {
//     fn r#break(&self, dataflow: &mut Dataflow, _: &D, location: Vec2) {
//         dataflow
//             .insert_entity(Entity {
//                 archetype_id: self.item_entity_id,
//                 coord: location,
//                 data: Box::new(ItemEntityData {
//                     item: Item {
//                         id: self.item_id,
//                         amount: 1,
//                         data: Box::new(()),
//                         render_param: Default::default(),
//                     },
//                 }),
//                 render_state: Default::default(),
//             })
//             .unwrap();
//     }
// }

// resource

pub struct BreakableResource {
    pub default_tile_id: u16,
    pub id: u16,
}

impl Resource for BreakableResource {}

// system

// pub struct BreakableSystem;

// impl BreakableSystem {
//     pub fn break_tile(dataflow: &mut Dataflow, tile_key: TileId) -> Result<Tile, DataflowError> {
//         let resource = dataflow.find_resources::<BreakableResource>().unwrap();
//         let resource = resource.borrow().unwrap();
//
//         let tile = dataflow.get_tile(tile_key)?;
//
//         let coord = tile.coord.as_vec2() + 0.5;
//         dataflow.insert_entity(Entity {
//             archetype_id: resource.id,
//             coord,
//             data: Box::new(ParticleEntityData { lifetime: 0.333 }),
//             render_state: EntityRenderState {
//                 tick: dataflow.get_tick() as u32,
//                 ..Default::default()
//             },
//         })?;
//         let mut tile = dataflow.remove_til(tile_key)?;
//
//         if let Ok(feature) = dataflow.get_tile_feature::<Rc<dyn BreakFeature<Tile>>>(tile.archetype_id) {
//             let feature = feature.clone();
//             feature.r#break(dataflow, &tile, coord);
//         }
//
//         let ret = tile.clone();
//         tile.archetype_id = resource.default_tile_id;
//         dataflow.insert_tile(tile)?;
//         Ok(ret)
//     }
//
//     pub fn break_block(dataflow: &mut Dataflow, block_key: BlockId) -> Result<Block, DataflowError> {
//         let rng = &mut rand::thread_rng();
//
//         let resource = dataflow.find_resources::<BreakableResource>().unwrap();
//         let resource = resource.borrow().unwrap();
//
//         let block = dataflow.get_block(block_key)?;
//
//         let archetype = dataflow.get_block_archetype(block.archetype_id)?;
//         let hint_rect = archetype.hint_rect;
//         let num = ((hint_rect[1] - hint_rect[0]).element_product() / 4.0).ceil() as u32;
//         for _ in 0..num {
//             let x = rand::Rng::gen_range(rng, hint_rect[0].x..hint_rect[1].x);
//             let y = rand::Rng::gen_range(rng, hint_rect[0].y..hint_rect[1].y);
//             let location = Vec2::new(x, y);
//
//             dataflow.insert_entity(Entity {
//                 archetype_id: resource.id,
//                 coord: location,
//                 data: Box::new(ParticleEntityData { lifetime: 0.333 }),
//                 render_state: EntityRenderState {
//                     tick: dataflow.get_tick() as u32,
//                     ..Default::default()
//                 },
//             })?;
//         }
//         let block = dataflow.remove_block(block_key)?;
//
//         let location = (hint_rect[0] + hint_rect[1]) * 0.5;
//         if let Ok(feature) = dataflow.get_block_feature::<Rc<dyn BreakFeature<Block>>>(block.archetype_id) {
//             let feature = feature.clone();
//             feature.r#break(dataflow, &block, location);
//         }
//
//         Ok(block)
//     }
//
//     pub fn break_entity(
//         dataflow: &mut Dataflow,
//         entity_key: EntityId,
//     ) -> Result<Entity, DataflowError> {
//         let resource = dataflow.find_resources::<BreakableResource>().unwrap();
//         let resource = resource.borrow().unwrap();
//
//         let rect = dataflow.get_entity_hint_rect(entity_key)?;
//         let location = (rect[0] + rect[1]) * 0.5;
//
//         dataflow.insert_entity(Entity {
//             archetype_id: resource.id,
//             coord: location,
//             data: Box::new(ParticleEntityData { lifetime: 0.333 }),
//             render_state: EntityRenderState {
//                 tick: dataflow.get_tick() as u32,
//                 ..Default::default()
//             },
//         })?;
//         let entity = dataflow.remove_entity(entity_key)?;
//
//         if let Ok(feature) = dataflow.get_entity_feature::<Rc<dyn BreakFeature<Entity>>>(entity.archetype_id)
//         {
//             let feature = feature.clone();
//             feature.r#break(dataflow, &entity, location);
//         }
//
//         Ok(entity)
//     }
// }
