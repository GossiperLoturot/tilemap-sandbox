use std::rc::Rc;

use glam::*;
use native_core::dataflow::*;

use super::*;

#[derive(Debug, Clone)]
pub struct ParticleEntityData {
    pub lifetime: f32,
}

impl EntityData for ParticleEntityData {}

#[derive(Debug, Clone)]
pub struct ParticleEntityFeatureSet {}

impl FeatureSet for ParticleEntityFeatureSet {
    fn attach_set(&self, b: &mut FeatureSetBuilder) -> Result<(), FeatureError> {
        let slf = Rc::new(self.clone());
        b.insert::<Rc<dyn ForwardFeature<Key = EntityKey>>>(slf.clone())?;
        Ok(())
    }
}

impl ForwardFeature for ParticleEntityFeatureSet {
    type Key = EntityKey;

    fn forward(&self, dataflow: &mut Dataflow, key: EntityKey, delta_secs: f32) {
        let mut entity = dataflow.get_entity(key).unwrap().clone();

        let Some(data) = entity.data.downcast_mut::<ParticleEntityData>() else {
            return;
        };

        data.lifetime -= delta_secs;
        if data.lifetime <= 0.0 {
            dataflow.remove_entity(key).unwrap();
            return;
        }

        dataflow.modify_entity(key, move |e| *e = entity).unwrap();
    }
}
