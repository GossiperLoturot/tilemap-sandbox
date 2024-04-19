pub type Vec2 = [f32; 2];
pub type IVec2 = [i32; 2];

#[derive(Debug, Clone, Default)]
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
    spatial_ref: ahash::AHashMap<IVec2, (IVec2, u32)>,
}

impl TileField {
    pub fn new(chunk_size: u32) -> Self {
        Self {
            chunk_size,
            chunks: Default::default(),
            spatial_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, tile: Tile) -> Option<IVec2> {
        let location = tile.location;
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

        self.spatial_ref.insert(location, (chunk_key, tile_key));

        Some(location)
    }

    pub fn remove(&mut self, location: IVec2) -> Option<Tile> {
        let (chunk_key, tile_key) = *self.spatial_ref.get(&location)?;

        let chunk = self.chunks.get_mut(&chunk_key).unwrap();
        chunk.serial += 1;
        let tile = chunk.tiles.remove(tile_key as usize);

        self.spatial_ref.remove(&location);

        Some(tile)
    }

    pub fn get(&self, location: IVec2) -> Option<&Tile> {
        let (chunk_key, tile_key) = *self.spatial_ref.get(&location)?;

        let tile = &self.chunks[&chunk_key].tiles[tile_key as usize];
        Some(tile)
    }

    pub fn get_chunk(&self, chunk_key: IVec2) -> Option<&TileChunk> {
        self.chunks.get(&chunk_key)
    }
}

#[derive(Debug, Clone, Default)]
pub struct BlockSpec {
    pub size: IVec2,
}

#[derive(Debug, Clone, Default)]
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
    spatial_ref: ahash::AHashMap<IVec2, (IVec2, u32)>,
}

impl BlockField {
    pub fn new(chunk_size: u32, specs: Vec<BlockSpec>) -> Self {
        Self {
            chunk_size,
            specs,
            chunks: Default::default(),
            spatial_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, block: Block) -> Option<IVec2> {
        let size = self.specs[block.id as usize].size;

        let location = block.location;
        for x in 0..size[0] {
            for y in 0..size[1] {
                let x = location[0] + x;
                let y = location[1] + y;
                if self.spatial_ref.contains_key(&[x, y]) {
                    return None;
                }
            }
        }

        let chunk_key = {
            let x = location[0].div_euclid(self.chunk_size as i32);
            let y = location[1].div_euclid(self.chunk_size as i32);
            [x, y]
        };
        let chunk = self.chunks.entry(chunk_key).or_default();
        chunk.serial += 1;
        let block_key = chunk.blocks.insert(block) as u32;

        for x in 0..size[0] {
            for y in 0..size[1] {
                let x = location[0] + x;
                let y = location[1] + y;
                self.spatial_ref.insert([x, y], (chunk_key, block_key));
            }
        }

        Some(location)
    }

    pub fn remove(&mut self, location: IVec2) -> Option<Block> {
        let (chunk_key, block_key) = *self.spatial_ref.get(&location)?;

        let chunk = self.chunks.get_mut(&chunk_key).unwrap();
        chunk.serial += 1;
        let block = chunk.blocks.remove(block_key as usize);

        let size = self.specs[block.id as usize].size;
        for x in 0..size[0] {
            for y in 0..size[1] {
                let x = block.location[0] + x;
                let y = block.location[1] + y;
                self.spatial_ref.remove(&[x, y]);
            }
        }

        Some(block)
    }

    pub fn get(&self, location: IVec2) -> Option<&Block> {
        let (chunk_key, block_key) = *self.spatial_ref.get(&location)?;

        let block = &self.chunks[&chunk_key].blocks[block_key as usize];
        Some(block)
    }

    pub fn get_chunk(&self, chunk_key: IVec2) -> Option<&BlockChunk> {
        self.chunks.get(&chunk_key)
    }
}

#[derive(Debug, Clone, Default)]
pub struct EntitySpec {
    pub size: Vec2,
}

#[derive(Debug, Clone, Default)]
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
    chunks: ahash::AHashMap<IVec2, EntityChunk>,
    index_ref: slab::Slab<(IVec2, u32)>,
}

impl EntityField {
    pub fn new(chunk_size: u32) -> Self {
        Self {
            chunk_size,
            chunks: Default::default(),
            index_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, entity: Entity) -> Option<u32> {
        let location = entity.location;

        let chunk_key = {
            let x = location[0].div_euclid(self.chunk_size as f32) as i32;
            let y = location[1].div_euclid(self.chunk_size as f32) as i32;
            [x, y]
        };
        let chunk = self.chunks.entry(chunk_key).or_default();
        chunk.serial += 1;
        let block_key = chunk.entities.insert(entity) as u32;

        let index = self.index_ref.insert((chunk_key, block_key)) as u32;

        Some(index)
    }

    pub fn remove(&mut self, index: u32) -> Option<Entity> {
        let (chunk_key, entity_key) = self.index_ref[index as usize];

        let chunk = self.chunks.get_mut(&chunk_key).unwrap();
        chunk.serial += 1;
        let entity = chunk.entities.remove(entity_key as usize);

        Some(entity)
    }

    pub fn get(&self, index: u32) -> Option<&Entity> {
        let (chunk_key, entity_key) = *self.index_ref.get(index as usize)?;

        let entity = &self.chunks[&chunk_key].entities[entity_key as usize];
        Some(entity)
    }

    pub fn get_chunk(&self, chunk_key: IVec2) -> Option<&EntityChunk> {
        self.chunks.get(&chunk_key)
    }
}
