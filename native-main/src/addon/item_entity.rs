use std::rc::Rc;

use glam::*;
use native_core::dataflow::*;

use super::*;

#[derive(Debug, Clone)]
pub struct ItemEntityData {
    pub item: Item,
}

impl EntityData for ItemEntityData {}

#[derive(Debug, Clone)]
pub struct ItemEntityFeatureSet {}

impl FeatureSet for ItemEntityFeatureSet {
    fn attach_set(&self, b: &mut FeatureSetBuilder) -> Result<(), FeatureError> {
        let slf = Rc::new(self.clone());
        // b.insert::<Rc<dyn ForwardFeature<Key = EntityId>>>(slf.clone())?;
        Ok(())
    }
}

// impl ForwardFeature for ItemEntityFeatureSet {
//     type Key = EntityId;
//
//     fn forward(&self, dataflow: &mut Dataflow, key: EntityId, delta_secs: f32) {
//         let mut entity = dataflow.get_entity(key).unwrap().clone();
//
//         let Some(data) = entity.data.downcast_mut::<ItemEntityData>() else {
//             return;
//         };
//
//         let Ok(target_location) = PlayerSystem::get_location(dataflow) else {
//             return;
//         };
//
//         if Vec2::distance(entity.coord, target_location) > 2.0 {
//             return;
//         }
//
//         entity.coord = entity.coord + (target_location - entity.coord) * delta_secs * 10.0;
//         if Vec2::distance(entity.coord, target_location) < 0.5 {
//             dataflow.remove_entity(key).unwrap();
//             let inventory_key = PlayerSystem::get_inventory_key(dataflow).unwrap();
//             dataflow
//                 .push_item_to_inventory(inventory_key, data.item.clone())
//                 .unwrap();
//             return;
//         }
//
//         dataflow.modify_entity(key, move |e| *e = entity).unwrap();
//     }
// }
