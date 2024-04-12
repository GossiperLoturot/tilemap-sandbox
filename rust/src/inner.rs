pub type IVec2 = (i32, i32);

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

#[derive(Debug, Clone, Default)]
pub struct TileField {
    chunks: ahash::AHashMap<IVec2, TileChunk>,
    spatial_ref: ahash::AHashMap<IVec2, (IVec2, u32)>,
}

impl TileField {
    pub const CHUNK_SIZE: u32 = 32;

    pub fn add_tile(&mut self, tile: Tile) -> Option<IVec2> {
        let location = tile.location;
        if self.spatial_ref.contains_key(&location) {
            return None;
        }

        let chunk_key = {
            let x = location.0.div_euclid(Self::CHUNK_SIZE as i32);
            let y = location.1.div_euclid(Self::CHUNK_SIZE as i32);
            (x, y)
        };
        let chunk = self.chunks.entry(chunk_key).or_default();
        chunk.serial += 1;
        let tile_key = chunk.tiles.insert(tile) as u32;

        self.spatial_ref.insert(location, (chunk_key, tile_key));

        Some(location)
    }

    pub fn remove_tile(&mut self, location: IVec2) -> Option<Tile> {
        let (chunk_key, tile_key) = self.spatial_ref.remove(&location)?;

        let chunk = self.chunks.get_mut(&chunk_key).unwrap();
        chunk.serial += 1;
        let tile = chunk.tiles.remove(tile_key as usize);

        Some(tile)
    }

    pub fn get_tile(&self, location: IVec2) -> Option<&Tile> {
        let (chunk_key, tile_key) = self.spatial_ref.get(&location)?;

        let tile = &self.chunks[chunk_key].tiles[*tile_key as usize];
        Some(tile)
    }

    pub fn serial_by_chunk(&self, chunk_key: IVec2) -> u32 {
        self.chunks
            .get(&chunk_key)
            .into_iter()
            .map(|chunk| chunk.serial)
            .next()
            .unwrap_or_default()
    }

    pub fn tiles_by_chunk(&self, chunk_key: IVec2) -> impl Iterator<Item = &Tile> {
        self.chunks
            .get(&chunk_key)
            .into_iter()
            .flat_map(|chunk| chunk.tiles.iter())
            .map(|(_, tile)| tile)
    }
}
