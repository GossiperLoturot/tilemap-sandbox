use super::*;

pub trait GlobalBehavior: dyn_clone::DynClone {
    fn on_new(&self, _world: &mut World) {}
    fn on_drop(&self, _world: &mut World) {}
    fn on_update(&self, _world: &mut World) {}
}
dyn_clone::clone_trait_object!(GlobalBehavior);

pub trait TileBehavior: dyn_clone::DynClone {
    fn on_new(&self, _world: &mut World) {}
    fn on_drop(&self, _world: &mut World) {}
    fn on_place_tile(&self, _world: &mut World, _tile_key: u32) {}
    fn on_break_tile(&self, _world: &mut World, _tile_key: u32) {}
    fn on_update(&self, _world: &mut World) {}
}
dyn_clone::clone_trait_object!(TileBehavior);

pub trait BlockBehavior: dyn_clone::DynClone {
    fn on_new(&self, _world: &mut World) {}
    fn on_drop(&self, _world: &mut World) {}
    fn on_place_block(&self, _world: &mut World, _block_key: u32) {}
    fn on_break_block(&self, _world: &mut World, _block_key: u32) {}
    fn on_update(&self, _world: &mut World) {}
}
dyn_clone::clone_trait_object!(BlockBehavior);

pub trait EntityBehavior: dyn_clone::DynClone {
    fn on_new(&self, _world: &mut World) {}
    fn on_drop(&self, _world: &mut World) {}
    fn on_place_entity(&self, _world: &mut World, _entity_key: u32) {}
    fn on_break_entity(&self, _world: &mut World, _entity_key: u32) {}
    fn on_update(&self, _world: &mut World) {}
}
dyn_clone::clone_trait_object!(EntityBehavior);

#[derive(Clone)]
pub struct BehaviorRoot {
    pub global_behaviors: Vec<Box<dyn GlobalBehavior>>,
    pub tile_behaviors: Vec<Box<dyn TileBehavior>>,
    pub block_behaviors: Vec<Box<dyn BlockBehavior>>,
    pub entity_behaviors: Vec<Box<dyn EntityBehavior>>,
}

pub struct World<'a> {
    pub tile_field: &'a mut TileField,
    pub block_field: &'a mut BlockField,
    pub entity_field: &'a mut EntityField,
    pub node_store: &'a mut NodeStore,
    pub behavior_root: &'a BehaviorRoot,
}

impl World<'_> {
    pub fn install(&mut self) {
        for global_behavior in &self.behavior_root.global_behaviors {
            global_behavior.on_new(self);
        }
        for tile_behavior in &self.behavior_root.tile_behaviors {
            tile_behavior.on_new(self);
        }
        for block_behavior in &self.behavior_root.block_behaviors {
            block_behavior.on_new(self);
        }
        for entity_behavior in &self.behavior_root.entity_behaviors {
            entity_behavior.on_new(self);
        }
    }

    pub fn uninstall(&mut self) {
        for global_behavior in &self.behavior_root.global_behaviors {
            global_behavior.on_drop(self);
        }
        for tile_behavior in &self.behavior_root.tile_behaviors {
            tile_behavior.on_drop(self);
        }
        for block_behavior in &self.behavior_root.block_behaviors {
            block_behavior.on_drop(self);
        }
        for entity_behavior in &self.behavior_root.entity_behaviors {
            entity_behavior.on_drop(self);
        }
    }

    pub fn update(&mut self) {
        for global_behavior in &self.behavior_root.global_behaviors {
            global_behavior.on_update(self);
        }
        for tile_behavior in &self.behavior_root.tile_behaviors {
            tile_behavior.on_update(self);
        }
        for block_behavior in &self.behavior_root.block_behaviors {
            block_behavior.on_update(self);
        }
        for entity_behavior in &self.behavior_root.entity_behaviors {
            entity_behavior.on_update(self);
        }
    }

    pub fn place_tile(&mut self, tile: Tile) -> Result<u32, FieldError> {
        let tile_behaviors = self
            .behavior_root
            .tile_behaviors
            .get(tile.id as usize)
            .ok_or(FieldError::InvalidId)?;
        let tile_key = self.tile_field.insert(tile)?;

        tile_behaviors.on_place_tile(self, tile_key);
        Ok(tile_key)
    }

    pub fn break_tile(&mut self, tile_key: u32) -> Result<Tile, FieldError> {
        let tile = self.tile_field.remove(tile_key)?;
        let tile_behaviors = self
            .behavior_root
            .tile_behaviors
            .get(tile.id as usize)
            .ok_or(FieldError::InvalidId)?;

        tile_behaviors.on_break_tile(self, tile_key);
        Ok(tile)
    }

    pub fn place_block(&mut self, block: Block) -> Result<u32, FieldError> {
        let block_behaviors = self
            .behavior_root
            .block_behaviors
            .get(block.id as usize)
            .ok_or(FieldError::InvalidId)?;
        let block_key = self.block_field.insert(block)?;

        block_behaviors.on_place_block(self, block_key);
        Ok(block_key)
    }

    pub fn break_block(&mut self, block_key: u32) -> Result<Block, FieldError> {
        let block = self.block_field.remove(block_key)?;
        let block_behavior = self
            .behavior_root
            .block_behaviors
            .get(block.id as usize)
            .ok_or(FieldError::InvalidId)?;

        block_behavior.on_break_block(self, block_key);
        Ok(block)
    }

    pub fn place_entity(&mut self, entity: Entity) -> Result<u32, FieldError> {
        let entity_behaviors = self
            .behavior_root
            .entity_behaviors
            .get(entity.id as usize)
            .ok_or(FieldError::InvalidId)?;
        let entity_key = self.entity_field.insert(entity)?;

        entity_behaviors.on_place_entity(self, entity_key);
        Ok(entity_key)
    }

    pub fn break_entity(&mut self, entity_key: u32) -> Result<Entity, FieldError> {
        let entity = self.entity_field.remove(entity_key)?;
        let entity_behaviors = self
            .behavior_root
            .entity_behaviors
            .get(entity.id as usize)
            .ok_or(FieldError::InvalidId)?;

        entity_behaviors.on_break_entity(self, entity_key);
        Ok(entity)
    }
}
