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
        b.insert::<Rc<dyn BreakFeature<Tile>>>(slf.clone())?;
        b.insert::<Rc<dyn BreakFeature<Block>>>(slf.clone())?;
        b.insert::<Rc<dyn BreakFeature<Entity>>>(slf.clone())?;
        Ok(())
    }
}

impl<D> BreakFeature<D> for BreakFeatureSet {
    fn r#break(&self, dataflow: &mut Dataflow, _: &D, location: Vec2) {
        dataflow
            .insert_entity(Entity {
                id: self.item_entity_id,
                location,
                data: Box::new(ItemEntityData {
                    item: Item {
                        id: self.item_id,
                        amount: 1,
                        data: Box::new(()),
                        render_param: Default::default(),
                    },
                }),
                render_param: Default::default(),
            })
            .unwrap();
    }
}

// resource

pub struct BreakableResource {
    pub default_tile_id: u16,
    pub id: u16,
}

impl Resource for BreakableResource {}

// system

pub struct BreakableSystem;

impl BreakableSystem {
    pub fn break_tile(dataflow: &mut Dataflow, tile_key: TileKey) -> Result<Tile, DataflowError> {
        let resource = dataflow.find_resources::<BreakableResource>().unwrap();
        let resource = resource.borrow().unwrap();

        let tile = dataflow.get_tile(tile_key)?;

        let location = tile.location.as_vec2() + 0.5;
        dataflow.insert_entity(Entity {
            id: resource.id,
            location,
            data: Box::new(ParticleEntityData { lifetime: 0.333 }),
            render_param: EntityRenderParam {
                tick: dataflow.get_tick() as u32,
                ..Default::default()
            },
        })?;
        let mut tile = dataflow.remove_til(tile_key)?;

        if let Ok(feature) = dataflow.get_tile_feature::<Rc<dyn BreakFeature<Tile>>>(tile.id) {
            let feature = feature.clone();
            feature.r#break(dataflow, &tile, location);
        }

        let ret = tile.clone();
        tile.id = resource.default_tile_id;
        dataflow.insert_tile(tile)?;
        Ok(ret)
    }

    pub fn break_block(
        dataflow: &mut Dataflow,
        block_key: BlockKey,
    ) -> Result<Block, DataflowError> {
        let rng = &mut rand::thread_rng();

        let resource = dataflow.find_resources::<BreakableResource>().unwrap();
        let resource = resource.borrow().unwrap();

        let rect = dataflow.get_block_hint_rect(block_key)?;
        let area = (rect[1] - rect[0]).element_product();
        for _ in 0..(area / 4.0).ceil() as usize {
            let x = rand::Rng::gen_range(rng, rect[0].x..rect[1].x);
            let y = rand::Rng::gen_range(rng, rect[0].y..rect[1].y);
            let location = Vec2::new(x, y);

            dataflow.insert_entity(Entity {
                id: resource.id,
                location,
                data: Box::new(ParticleEntityData { lifetime: 0.333 }),
                render_param: EntityRenderParam {
                    tick: dataflow.get_tick() as u32,
                    ..Default::default()
                },
            })?;
        }
        let block = dataflow.remove_block(block_key)?;

        let location = (rect[0] + rect[1]) * 0.5;
        if let Ok(feature) = dataflow.get_block_feature::<Rc<dyn BreakFeature<Block>>>(block.id) {
            let feature = feature.clone();
            feature.r#break(dataflow, &block, location);
        }

        Ok(block)
    }

    pub fn break_entity(
        dataflow: &mut Dataflow,
        entity_key: EntityKey,
    ) -> Result<Entity, DataflowError> {
        let resource = dataflow.find_resources::<BreakableResource>().unwrap();
        let resource = resource.borrow().unwrap();

        let rect = dataflow.get_entity_hint_rect(entity_key)?;
        let location = (rect[0] + rect[1]) * 0.5;

        dataflow.insert_entity(Entity {
            id: resource.id,
            location,
            data: Box::new(ParticleEntityData { lifetime: 0.333 }),
            render_param: EntityRenderParam {
                tick: dataflow.get_tick() as u32,
                ..Default::default()
            },
        })?;
        let entity = dataflow.remove_entity(entity_key)?;

        if let Ok(feature) = dataflow.get_entity_feature::<Rc<dyn BreakFeature<Entity>>>(entity.id)
        {
            let feature = feature.clone();
            feature.r#break(dataflow, &entity, location);
        }

        Ok(entity)
    }
}
