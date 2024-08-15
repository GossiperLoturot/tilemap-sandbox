use crate::inner::*;

// flow definition

pub trait IResource {
    fn before(&self, root: &mut Root);
    fn after(&self, root: &mut Root);
}

pub trait ITile {
    fn place_tile(&self, root: &mut Root, tile: Tile) -> Result<TileKey, FieldError>;
    fn break_tile(&self, root: &mut Root, tile_key: TileKey) -> Result<Tile, FieldError>;
    fn modify_tile(
        &self,
        root: &mut Root,
        tile_key: TileKey,
        new_tile: Tile,
    ) -> Result<Tile, FieldError>;
}

pub trait IBlock {
    fn place_block(&self, root: &mut Root, block: Block) -> Result<BlockKey, FieldError>;
    fn break_block(&self, root: &mut Root, block_key: BlockKey) -> Result<Block, FieldError>;
    fn modify_block(
        &self,
        root: &mut Root,
        block_key: BlockKey,
        new_block: Block,
    ) -> Result<Block, FieldError>;
}

pub trait IEntity {
    fn place_entity(&self, root: &mut Root, entity: Entity) -> Result<EntityKey, FieldError>;
    fn break_entity(&self, root: &mut Root, entity_key: EntityKey) -> Result<Entity, FieldError>;
    fn modify_entity(
        &self,
        root: &mut Root,
        entity_key: EntityKey,
        new_entity: Entity,
    ) -> Result<Entity, FieldError>;
}

pub trait IForward {
    fn forward(&self, root: &mut Root, delta_secs: f32);
}

pub trait IForwardLocal {
    fn forward_local(&self, root: &mut Root, delta_secs: f32, rect: [SpaceKey; 2]);
}

pub trait IGenerate {
    fn generate_chunk(&self, root: &mut Root, rect: [Vec2; 2]);
}

// action

pub fn before(root: &mut Root) {
    for flow in root
        .flow_iter::<std::rc::Rc<dyn IResource>>()
        .cloned()
        .collect::<Vec<_>>()
    {
        flow.before(root);
    }
}

pub fn after(root: &mut Root) {
    for flow in root
        .flow_iter::<std::rc::Rc<dyn IResource>>()
        .cloned()
        .collect::<Vec<_>>()
    {
        flow.after(root);
    }
}

pub fn place_tile(root: &mut Root, tile: Tile) -> Result<TileKey, FieldError> {
    let flow = root
        .flow_one_by_ref::<std::rc::Rc<dyn ITile>>(FlowRef::Tile(tile.id))
        .cloned()
        .unwrap();
    flow.place_tile(root, tile)
}

pub fn break_tile(root: &mut Root, tile_key: TileKey) -> Result<Tile, FieldError> {
    let tile = root.tile_get(tile_key)?;
    let flow = root
        .flow_one_by_ref::<std::rc::Rc<dyn ITile>>(FlowRef::Tile(tile.id))
        .cloned()
        .unwrap();
    flow.break_tile(root, tile_key)
}

pub fn modify_tile(root: &mut Root, tile_key: TileKey, new_tile: Tile) -> Result<Tile, FieldError> {
    let old_tile = root.tile_get(tile_key)?;

    if new_tile.id != old_tile.id {
        panic!("no support for changing tile id in modify_tile");
    }

    let flow = root
        .flow_one_by_ref::<std::rc::Rc<dyn ITile>>(FlowRef::Tile(new_tile.id))
        .cloned()
        .unwrap();
    flow.modify_tile(root, tile_key, new_tile)
}

pub fn place_block(root: &mut Root, block: Block) -> Result<BlockKey, FieldError> {
    let flow = root
        .flow_one_by_ref::<std::rc::Rc<dyn IBlock>>(FlowRef::Block(block.id))
        .cloned()
        .unwrap();
    flow.place_block(root, block)
}

pub fn break_block(root: &mut Root, block_key: BlockKey) -> Result<Block, FieldError> {
    let block = root.block_get(block_key)?;
    let flow = root
        .flow_one_by_ref::<std::rc::Rc<dyn IBlock>>(FlowRef::Block(block.id))
        .cloned()
        .unwrap();
    flow.break_block(root, block_key)
}

pub fn modify_block(
    root: &mut Root,
    block_key: BlockKey,
    new_block: Block,
) -> Result<Block, FieldError> {
    let old_block = root.block_get(block_key)?;

    if new_block.id != old_block.id {
        panic!("no support for changing block id in modify_block");
    }

    let flow = root
        .flow_one_by_ref::<std::rc::Rc<dyn IBlock>>(FlowRef::Block(new_block.id))
        .cloned()
        .unwrap();
    flow.modify_block(root, block_key, new_block)
}

pub fn place_entity(root: &mut Root, entity: Entity) -> Result<EntityKey, FieldError> {
    let flow = root
        .flow_one_by_ref::<std::rc::Rc<dyn IEntity>>(FlowRef::Entity(entity.id))
        .cloned()
        .unwrap();
    flow.place_entity(root, entity)
}

pub fn break_entity(root: &mut Root, entity_key: EntityKey) -> Result<Entity, FieldError> {
    let entity = root.entity_get(entity_key)?;
    let flow = root
        .flow_one_by_ref::<std::rc::Rc<dyn IEntity>>(FlowRef::Entity(entity.id))
        .cloned()
        .unwrap();
    flow.break_entity(root, entity_key)
}

pub fn modify_entity(
    root: &mut Root,
    entity_key: EntityKey,
    new_entity: Entity,
) -> Result<Entity, FieldError> {
    let old_entity = root.entity_get(entity_key)?;

    if new_entity.id != old_entity.id {
        panic!("no support for changing entity id in modify_entity");
    }

    let flow = root
        .flow_one_by_ref::<std::rc::Rc<dyn IEntity>>(FlowRef::Entity(new_entity.id))
        .cloned()
        .unwrap();
    flow.modify_entity(root, entity_key, new_entity)
}

pub fn forward(root: &mut Root, delta_secs: f32) {
    for flow in root
        .flow_iter::<std::rc::Rc<dyn IForward>>()
        .cloned()
        .collect::<Vec<_>>()
    {
        flow.forward(root, delta_secs);
    }
}

pub fn forward_local(root: &mut Root, delta_secs: f32, rect: [SpaceKey; 2]) {
    for flow in root
        .flow_iter::<std::rc::Rc<dyn IForwardLocal>>()
        .cloned()
        .collect::<Vec<_>>()
    {
        flow.forward_local(root, delta_secs, rect);
    }
}

pub fn generate_chunk(root: &mut Root, rect: [Vec2; 2]) {
    for flow in root
        .flow_iter::<std::rc::Rc<dyn IGenerate>>()
        .cloned()
        .collect::<Vec<_>>()
    {
        flow.generate_chunk(root, rect);
    }
}

// base tile flow

#[derive(Debug, Clone)]
pub struct BaseTile {
    pub tile_id: u32,
}

impl ITile for BaseTile {
    #[inline]
    fn place_tile(&self, root: &mut Root, tile: Tile) -> Result<TileKey, FieldError> {
        root.tile_insert(tile)
    }

    #[inline]
    fn break_tile(&self, root: &mut Root, tile_key: TileKey) -> Result<Tile, FieldError> {
        root.tile_remove(tile_key)
    }

    #[inline]
    fn modify_tile(
        &self,
        root: &mut Root,
        tile_key: TileKey,
        new_tile: Tile,
    ) -> Result<Tile, FieldError> {
        root.tile_modify(tile_key, new_tile)
    }
}

impl FlowBundle for BaseTile {
    fn insert(&self, buf: &mut FlowBuffer) {
        let slf = std::rc::Rc::new(self.clone());
        buf.register::<std::rc::Rc<dyn ITile>>(FlowRef::Tile(self.tile_id), slf);
    }
}

// base block flow

#[derive(Debug, Clone)]
pub struct BaseBlock {
    pub block_id: u32,
}

impl IBlock for BaseBlock {
    #[inline]
    fn place_block(&self, root: &mut Root, block: Block) -> Result<BlockKey, FieldError> {
        root.block_insert(block)
    }

    #[inline]
    fn break_block(&self, root: &mut Root, block_key: BlockKey) -> Result<Block, FieldError> {
        root.block_remove(block_key)
    }

    #[inline]
    fn modify_block(
        &self,
        root: &mut Root,
        block_key: BlockKey,
        new_block: Block,
    ) -> Result<Block, FieldError> {
        root.block_modify(block_key, new_block)
    }
}

impl FlowBundle for BaseBlock {
    fn insert(&self, buf: &mut FlowBuffer) {
        let slf = std::rc::Rc::new(self.clone());
        buf.register::<std::rc::Rc<dyn IBlock>>(FlowRef::Block(self.block_id), slf);
    }
}

// base entity flow

#[derive(Debug, Clone)]
pub struct BaseEntity {
    pub entity_id: u32,
}

impl IEntity for BaseEntity {
    #[inline]
    fn place_entity(&self, root: &mut Root, entity: Entity) -> Result<EntityKey, FieldError> {
        root.entity_insert(entity)
    }

    #[inline]
    fn break_entity(&self, root: &mut Root, entity_key: EntityKey) -> Result<Entity, FieldError> {
        root.entity_remove(entity_key)
    }

    #[inline]
    fn modify_entity(
        &self,
        root: &mut Root,
        entity_key: EntityKey,
        new_entity: Entity,
    ) -> Result<Entity, FieldError> {
        root.entity_modify(entity_key, new_entity)
    }
}

impl FlowBundle for BaseEntity {
    fn insert(&self, buf: &mut FlowBuffer) {
        let slf = std::rc::Rc::new(self.clone());
        buf.register::<std::rc::Rc<dyn IEntity>>(FlowRef::Entity(self.entity_id), slf);
    }
}

// generator flow

#[derive(Debug, Clone)]
pub struct GeneratorResource {
    pub prev_rect: Option<[IVec2; 2]>,
    pub visited_chunk: ahash::AHashSet<IVec2>,
}

#[derive(Debug, Clone)]
pub struct Generator {}

impl IResource for Generator {
    fn before(&self, root: &mut Root) {
        let resource = GeneratorResource {
            prev_rect: None,
            visited_chunk: Default::default(),
        };
        root.resource_insert(resource).unwrap();
    }

    fn after(&self, root: &mut Root) {
        root.resource_remove::<GeneratorResource>();
    }
}

impl IGenerate for Generator {
    fn generate_chunk(&self, root: &mut Root, rect: [Vec2; 2]) {
        const CHUNK_SIZE: u32 = 32;

        #[rustfmt::skip]
        let rect = [[
            rect[0][0].div_euclid(CHUNK_SIZE as f32) as i32,
            rect[0][1].div_euclid(CHUNK_SIZE as f32) as i32, ], [
            rect[1][0].div_euclid(CHUNK_SIZE as f32) as i32,
            rect[1][1].div_euclid(CHUNK_SIZE as f32) as i32,
        ]];

        let tag = root.resource_get::<GeneratorResource>().unwrap();
        if Some(rect) != tag.prev_rect {
            for y in rect[0][1]..=rect[1][1] {
                for x in rect[0][0]..=rect[1][0] {
                    let chunk_key = [x, y];

                    let tag = root.resource_get::<GeneratorResource>().unwrap();
                    if tag.visited_chunk.contains(&chunk_key) {
                        continue;
                    }

                    let tag = root.resource_get_mut::<GeneratorResource>().unwrap();
                    tag.visited_chunk.insert(chunk_key);

                    for v in 0..CHUNK_SIZE {
                        for u in 0..CHUNK_SIZE {
                            let id = rand::Rng::gen_range(&mut rand::thread_rng(), 0..=1);
                            let location = [
                                x * CHUNK_SIZE as i32 + u as i32,
                                y * CHUNK_SIZE as i32 + v as i32,
                            ];
                            let _ = place_tile(root, Tile::new(id, location, 0));
                        }
                    }

                    for _ in 0..64 {
                        let id = rand::Rng::gen_range(&mut rand::thread_rng(), 0..=3);
                        let u = rand::Rng::gen_range(&mut rand::thread_rng(), 0..CHUNK_SIZE);
                        let v = rand::Rng::gen_range(&mut rand::thread_rng(), 0..CHUNK_SIZE);
                        let location = [
                            x * CHUNK_SIZE as i32 + u as i32,
                            y * CHUNK_SIZE as i32 + v as i32,
                        ];
                        let _ = place_block(root, Block::new(id, location, 0));
                    }

                    for _ in 0..64 {
                        let id = rand::Rng::gen_range(&mut rand::thread_rng(), 1..=5);
                        let u =
                            rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..CHUNK_SIZE as f32);
                        let v =
                            rand::Rng::gen_range(&mut rand::thread_rng(), 0.0..CHUNK_SIZE as f32);
                        let location = [
                            x as f32 * CHUNK_SIZE as f32 + u,
                            y as f32 * CHUNK_SIZE as f32 + v,
                        ];
                        let _ = place_entity(root, Entity::new(id, location, 0));
                    }
                }
            }

            let tag = root.resource_get_mut::<GeneratorResource>().unwrap();
            tag.prev_rect = Some(rect);
        }
    }
}

impl FlowBundle for Generator {
    fn insert(&self, buf: &mut FlowBuffer) {
        let slf = std::rc::Rc::new(self.clone());
        buf.register::<std::rc::Rc<dyn IResource>>(FlowRef::Global, slf.clone());
        buf.register::<std::rc::Rc<dyn IGenerate>>(FlowRef::Global, slf);
    }
}
