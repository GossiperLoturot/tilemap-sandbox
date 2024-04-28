use rand::Rng;

pub type Vec2 = [f32; 2];
pub type IVec2 = [i32; 2];

type RectNode<T, U> = rstar::primitives::GeomWithData<rstar::primitives::Rectangle<T>, U>;

#[derive(Debug, Clone)]
pub struct TileKey {
    chunk_key: IVec2,
    tile_key: u32,
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub id: u32,
    pub location: IVec2,
}

#[derive(Debug, Clone, Default)]
pub struct TileChunk {
    pub serial: u32,
    pub tiles: slab::Slab<Tile>,
}

#[derive(Debug, Clone)]
pub struct TileField {
    chunk_size: u32,
    chunks: ahash::AHashMap<IVec2, TileChunk>,
    spatial_ref: ahash::AHashMap<IVec2, TileKey>,
}

impl TileField {
    pub fn new(chunk_size: u32) -> Self {
        Self {
            chunk_size,
            chunks: Default::default(),
            spatial_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, tile: Tile) -> Option<TileKey> {
        let location = tile.location;

        // check by spatial features
        if self.spatial_ref.contains_key(&location) {
            return None;
        }

        let chunk_key = {
            let x = location[0].div_euclid(self.chunk_size as i32);
            let y = location[1].div_euclid(self.chunk_size as i32);
            [x, y]
        };
        let chunk = self.chunks.entry(chunk_key).or_default();
        chunk.serial += 1;
        let tile_key = chunk.tiles.insert(tile) as u32;
        let key = TileKey {
            chunk_key,
            tile_key,
        };

        // spatial features
        self.spatial_ref.insert(location, key.clone());

        Some(key)
    }

    pub fn remove(&mut self, key: TileKey) -> Option<Tile> {
        let chunk = self.chunks.get_mut(&key.chunk_key)?;
        chunk.serial += 1;
        let tile = chunk.tiles.try_remove(key.tile_key as usize)?;

        self.spatial_ref.remove(&tile.location);

        Some(tile)
    }

    pub fn get(&self, key: TileKey) -> Option<&Tile> {
        let chunk = &self.chunks.get(&key.chunk_key)?;
        chunk.tiles.get(key.tile_key as usize)
    }

    pub fn get_chunk(&self, chunk_key: IVec2) -> Option<&TileChunk> {
        self.chunks.get(&chunk_key)
    }

    // spatial features

    pub fn has_by_point(&self, point: IVec2) -> bool {
        self.spatial_ref.contains_key(&point)
    }

    pub fn get_by_point(&self, point: IVec2) -> Option<&TileKey> {
        self.spatial_ref.get(&point)
    }
}

#[derive(Debug, Clone)]
pub struct BlockSpec {
    pub size: IVec2,
    pub collision_size: Vec2,
    pub collision_offset: Vec2,
    pub hint_size: Vec2,
    pub hint_offset: Vec2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockKey {
    chunk_key: IVec2,
    block_key: u32,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub id: u32,
    pub location: IVec2,
}

#[derive(Debug, Clone, Default)]
pub struct BlockChunk {
    pub serial: u32,
    pub blocks: slab::Slab<Block>,
}

#[derive(Debug, Clone)]
pub struct BlockField {
    chunk_size: u32,
    specs: Vec<BlockSpec>,
    chunks: ahash::AHashMap<IVec2, BlockChunk>,
    spatial_ref: rstar::RTree<RectNode<IVec2, BlockKey>>,
    collision_ref: rstar::RTree<RectNode<Vec2, BlockKey>>,
    hint_ref: rstar::RTree<RectNode<Vec2, BlockKey>>,
}

impl BlockField {
    pub fn new(chunk_size: u32, specs: Vec<BlockSpec>) -> Self {
        // validate specs
        specs.iter().for_each(|spec| {
            assert!(spec.size[0] > 0);
            assert!(spec.size[1] > 0);
            assert!(spec.collision_size[0] >= 0.0);
            assert!(spec.collision_size[1] >= 0.0);
            assert!(spec.hint_size[0] >= 0.0);
            assert!(spec.hint_size[1] >= 0.0);
        });

        Self {
            chunk_size,
            specs,
            chunks: Default::default(),
            spatial_ref: Default::default(),
            collision_ref: Default::default(),
            hint_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, block: Block) -> Option<BlockKey> {
        let location = block.location;
        let spec = &self.specs[block.id as usize];

        // check by spatial features
        if self.has_by_rect([
            [location[0], location[1]],
            [
                location[0] + spec.size[0] - 1,
                location[1] + spec.size[1] - 1,
            ],
        ]) {
            return None;
        }

        let chunk_key = {
            let x = location[0].div_euclid(self.chunk_size as i32);
            let y = location[1].div_euclid(self.chunk_size as i32);
            [x, y]
        };
        let chunk = self.chunks.entry(chunk_key).or_default();
        let block_key = chunk.blocks.insert(block) as u32;
        chunk.serial += 1;
        let key = BlockKey {
            chunk_key,
            block_key,
        };

        // spatial features
        let rect = rstar::primitives::Rectangle::from_corners(
            [location[0], location[1]],
            [
                location[0] + spec.size[0] - 1,
                location[1] + spec.size[1] - 1,
            ],
        );
        let node = rstar::primitives::GeomWithData::new(rect, key.clone());
        self.spatial_ref.insert(node);

        // collision features
        let rect = rstar::primitives::Rectangle::from_corners(
            [
                location[0] as f32 + spec.collision_offset[0],
                location[1] as f32 + spec.collision_offset[1],
            ],
            [
                location[0] as f32 + spec.collision_offset[0] + spec.collision_size[0],
                location[1] as f32 + spec.collision_offset[1] + spec.collision_size[1],
            ],
        );
        let node = rstar::primitives::GeomWithData::new(rect, key.clone());
        self.collision_ref.insert(node);

        // hint features
        let rect = rstar::primitives::Rectangle::from_corners(
            [
                location[0] as f32 + spec.hint_offset[0],
                location[1] as f32 + spec.hint_offset[1],
            ],
            [
                location[0] as f32 + spec.hint_offset[0] + spec.hint_size[0],
                location[1] as f32 + spec.hint_offset[1] + spec.hint_size[1],
            ],
        );
        let node = rstar::primitives::GeomWithData::new(rect, key.clone());
        self.hint_ref.insert(node);

        Some(key)
    }

    pub fn remove(&mut self, key: BlockKey) -> Option<Block> {
        let chunk = self.chunks.get_mut(&key.chunk_key)?;
        let block = chunk.blocks.try_remove(key.block_key as usize)?;
        chunk.serial += 1;

        let spec = &self.specs[block.id as usize];
        let location = block.location;

        // spatial features
        let rect = rstar::primitives::Rectangle::from_corners(
            [location[0], location[1]],
            [
                location[0] + spec.size[0] - 1,
                location[1] + spec.size[1] - 1,
            ],
        );
        let node = &rstar::primitives::GeomWithData::new(rect, key.clone());
        self.spatial_ref.remove(node);

        // collision features
        let rect = rstar::primitives::Rectangle::from_corners(
            [
                location[0] as f32 + spec.collision_offset[0],
                location[1] as f32 + spec.collision_offset[1],
            ],
            [
                location[0] as f32 + spec.collision_offset[0] + spec.collision_size[0],
                location[1] as f32 + spec.collision_offset[1] + spec.collision_size[1],
            ],
        );
        let node = &rstar::primitives::GeomWithData::new(rect, key.clone());
        self.collision_ref.remove(node);

        // hint features
        let rect = rstar::primitives::Rectangle::from_corners(
            [
                location[0] as f32 + spec.hint_offset[0],
                location[1] as f32 + spec.hint_offset[1],
            ],
            [
                location[0] as f32 + spec.hint_offset[0] + spec.hint_size[0],
                location[1] as f32 + spec.hint_offset[1] + spec.hint_size[1],
            ],
        );
        let node = &rstar::primitives::GeomWithData::new(rect, key.clone());
        self.hint_ref.remove(node);

        Some(block)
    }

    pub fn get(&self, key: BlockKey) -> Option<&Block> {
        let chunk = self.chunks.get(&key.chunk_key)?;
        chunk.blocks.get(key.block_key as usize)
    }

    pub fn get_chunk(&self, chunk_key: IVec2) -> Option<&BlockChunk> {
        self.chunks.get(&chunk_key)
    }

    // spatial features

    pub fn has_by_point(&self, point: IVec2) -> bool {
        self.spatial_ref.locate_at_point(&point).is_some()
    }

    pub fn get_by_point(&self, point: IVec2) -> Option<&BlockKey> {
        let node = self.spatial_ref.locate_at_point(&point)?;
        Some(&node.data)
    }

    pub fn has_by_rect(&self, rect: [IVec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.spatial_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    pub fn get_by_rect(&self, rect: [IVec2; 2]) -> impl Iterator<Item = &BlockKey> {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.spatial_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| &node.data)
    }

    // collision features

    pub fn has_collision_by_point(&self, point: Vec2) -> bool {
        self.collision_ref.locate_at_point(&point).is_some()
    }

    pub fn get_collision_by_point(&self, point: Vec2) -> impl Iterator<Item = &BlockKey> {
        self.collision_ref
            .locate_all_at_point(&point)
            .map(|node| &node.data)
    }

    pub fn has_collision_by_rect(&self, rect: [Vec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    pub fn get_collision_by_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = &BlockKey> {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| &node.data)
    }

    // hint features

    pub fn has_hint_by_point(&self, point: Vec2) -> bool {
        self.hint_ref.locate_at_point(&point).is_some()
    }

    pub fn get_hint_by_point(&self, point: Vec2) -> impl Iterator<Item = &BlockKey> {
        self.hint_ref
            .locate_all_at_point(&point)
            .map(|node| &node.data)
    }

    pub fn has_hint_by_rect(&self, rect: [Vec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    pub fn get_hint_by_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = &BlockKey> {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| &node.data)
    }
}

#[derive(Debug, Clone)]
pub struct EntitySpec {
    pub collision_size: Vec2,
    pub collision_offset: Vec2,
    pub hint_size: Vec2,
    pub hint_offset: Vec2,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityKey {
    chunk_key: IVec2,
    entity_key: u32,
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub id: u32,
    pub location: Vec2,
}

#[derive(Debug, Clone, Default)]
pub struct EntityChunk {
    pub serial: u32,
    pub entities: slab::Slab<Entity>,
}

#[derive(Debug, Clone)]
pub struct EntityField {
    chunk_size: u32,
    specs: Vec<EntitySpec>,
    chunks: ahash::AHashMap<IVec2, EntityChunk>,
    collision_ref: rstar::RTree<RectNode<Vec2, EntityKey>>,
    hint_ref: rstar::RTree<RectNode<Vec2, EntityKey>>,
}

impl EntityField {
    pub fn new(chunk_size: u32, specs: Vec<EntitySpec>) -> Self {
        // validate specs
        specs.iter().for_each(|spec| {
            assert!(spec.collision_size[0] >= 0.0);
            assert!(spec.collision_size[1] >= 0.0);
            assert!(spec.hint_size[0] >= 0.0);
            assert!(spec.hint_size[1] >= 0.0);
        });

        Self {
            chunk_size,
            specs,
            chunks: Default::default(),
            collision_ref: Default::default(),
            hint_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, entity: Entity) -> EntityKey {
        let spec = &self.specs[entity.id as usize];
        let location = entity.location;

        let chunk_key = {
            let x = location[0].div_euclid(self.chunk_size as f32) as i32;
            let y = location[1].div_euclid(self.chunk_size as f32) as i32;
            [x, y]
        };
        let chunk = self.chunks.entry(chunk_key).or_default();
        let entity_key = chunk.entities.insert(entity) as u32;
        chunk.serial += 1;
        let key = EntityKey {
            chunk_key,
            entity_key,
        };

        // collision features
        let rect = rstar::primitives::Rectangle::from_corners(
            [
                location[0] + spec.collision_offset[0],
                location[1] + spec.collision_offset[1],
            ],
            [
                location[0] + spec.collision_offset[0] + spec.collision_size[0],
                location[1] + spec.collision_offset[1] + spec.collision_size[1],
            ],
        );
        let node = rstar::primitives::GeomWithData::new(rect, key.clone());
        self.collision_ref.insert(node);

        // hint features
        let rect = rstar::primitives::Rectangle::from_corners(
            [
                location[0] + spec.hint_offset[0],
                location[1] + spec.hint_offset[1],
            ],
            [
                location[0] + spec.hint_offset[0] + spec.hint_size[0],
                location[1] + spec.hint_offset[1] + spec.hint_size[1],
            ],
        );
        let node = rstar::primitives::GeomWithData::new(rect, key.clone());
        self.hint_ref.insert(node);

        key
    }

    pub fn remove(&mut self, key: EntityKey) -> Option<Entity> {
        let chunk = self.chunks.get_mut(&key.chunk_key)?;
        let entity = chunk.entities.try_remove(key.entity_key as usize)?;
        chunk.serial += 1;

        let spec = &self.specs[entity.id as usize];
        let location = entity.location;

        // collision features
        let rect = rstar::primitives::Rectangle::from_corners(
            [
                location[0] + spec.collision_offset[0],
                location[1] + spec.collision_offset[1],
            ],
            [
                location[0] + spec.collision_offset[0] + spec.collision_size[0],
                location[1] + spec.collision_offset[1] + spec.collision_size[1],
            ],
        );
        let node = &rstar::primitives::GeomWithData::new(rect, key.clone());
        self.collision_ref.remove(node);

        // hint features
        let rect = rstar::primitives::Rectangle::from_corners(
            [
                location[0] + spec.hint_offset[0],
                location[1] + spec.hint_offset[1],
            ],
            [
                location[0] + spec.hint_offset[0] + spec.hint_size[0],
                location[1] + spec.hint_offset[1] + spec.hint_size[1],
            ],
        );
        let node = &rstar::primitives::GeomWithData::new(rect, key.clone());
        self.hint_ref.remove(node);

        Some(entity)
    }

    pub fn get(&self, key: EntityKey) -> Option<&Entity> {
        let chunk = self.chunks.get(&key.chunk_key)?;
        chunk.entities.get(key.entity_key as usize)
    }

    pub fn get_chunk(&self, chunk_key: IVec2) -> Option<&EntityChunk> {
        self.chunks.get(&chunk_key)
    }

    // collision features

    pub fn has_collision_by_point(&self, point: Vec2) -> bool {
        self.collision_ref.locate_at_point(&point).is_some()
    }

    pub fn get_collision_by_point(&self, point: Vec2) -> impl Iterator<Item = &EntityKey> {
        self.collision_ref
            .locate_all_at_point(&point)
            .map(|node| &node.data)
    }

    pub fn has_collision_by_rect(&self, rect: [Vec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    pub fn get_collision_by_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = &EntityKey> {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.collision_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| &node.data)
    }

    // hint features

    pub fn has_hint_by_point(&self, point: Vec2) -> bool {
        self.hint_ref.locate_at_point(&point).is_some()
    }

    pub fn get_hint_by_point(&self, point: Vec2) -> impl Iterator<Item = &EntityKey> {
        self.hint_ref
            .locate_all_at_point(&point)
            .map(|node| &node.data)
    }

    pub fn has_hint_by_rect(&self, rect: [Vec2; 2]) -> bool {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .next()
            .is_some()
    }

    pub fn get_hint_by_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = &EntityKey> {
        let rect = rstar::AABB::from_corners(rect[0], rect[1]);
        self.hint_ref
            .locate_in_envelope_intersecting(&rect)
            .map(|node| &node.data)
    }
}

#[derive(Debug, Clone)]
pub enum AgentData {
    Empty,
    Herbivore {
        scan_secs: f32,
        scan_distance: f32,
        speed: f32,
        next_scan: Option<f32>,
        next_location: Option<Vec2>,
        elapsed: f32,
    },
}

#[derive(Debug, Clone)]
pub struct Agent {
    entity_key: EntityKey,
    data: AgentData,
}

#[derive(Debug, Clone)]
pub struct AgentSystem {
    agents: slab::Slab<Agent>,
    inverse_ref: ahash::AHashMap<EntityKey, u32>,
}

impl AgentSystem {
    pub fn new() -> Self {
        Self {
            agents: Default::default(),
            inverse_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, entity_key: EntityKey, data: AgentData) -> Option<()> {
        if self.inverse_ref.contains_key(&entity_key) {
            return None;
        }

        let agent = Agent {
            entity_key: entity_key.clone(),
            data,
        };
        let key = self.agents.insert(agent) as u32;
        self.inverse_ref.insert(entity_key, key);

        Some(())
    }

    pub fn remove(&mut self, entity_key: EntityKey) -> Option<AgentData> {
        let key = self.inverse_ref.remove(&entity_key)?;
        let agent = self.agents.remove(key as usize);
        Some(agent.data)
    }

    pub fn update(&mut self, entity_field: &mut EntityField, delta_secs: f32) {
        let mut rng = rand::thread_rng();

        for (_, agent) in &mut self.agents {
            match &mut agent.data {
                AgentData::Empty => {}
                AgentData::Herbivore {
                    scan_secs,
                    scan_distance,
                    speed,
                    next_scan,
                    next_location,
                    elapsed,
                } => {
                    let mut entity = entity_field.remove(agent.entity_key.clone()).unwrap();

                    if let Some(next_scan) = next_scan {
                        if *elapsed >= *next_scan {
                            *next_scan = *elapsed + *scan_secs;
                            let distance: f32 =
                                rng.sample(rand::distributions::Uniform::new(0.0, *scan_distance));
                            let direction: f32 =
                                rng.sample(rand::distributions::Uniform::new(0.0, 6.283185307179));
                            *next_location = Some([
                                entity.location[0] + direction.cos() * distance,
                                entity.location[1] + direction.sin() * distance,
                            ]);
                        }
                    } else {
                        *next_scan = Some(*elapsed + *scan_secs);
                        *next_location = None;
                    }

                    if let Some(next_location) = next_location {
                        let diff = [
                            next_location[0] - entity.location[0],
                            next_location[1] - entity.location[1],
                        ];
                        let direction = diff[1].atan2(diff[0]);
                        let distance = (diff[0].powi(2) + diff[1].powi(2)).sqrt();

                        let delta_distance = distance.min(delta_secs * *speed);
                        entity.location = [
                            entity.location[0] + direction.cos() * delta_distance,
                            entity.location[1] + direction.sin() * delta_distance,
                        ];
                    }

                    *elapsed += delta_secs;

                    agent.entity_key = entity_field.insert(entity);
                }
            }
        }
    }
}
