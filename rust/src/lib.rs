use godot::prelude::*;

pub mod inner;

mod block;
mod entity;
mod tile;

mod extra;

struct Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Extension {}

#[derive(GodotClass)]
#[class(no_init)]
pub struct TileKey {
    pub base: inner::TileKey,
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct BlockKey {
    pub base: inner::BlockKey,
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct EntityKey {
    pub base: inner::EntityKey,
}

#[rustfmt::skip]
pub type Tile<T> = inner::Tile<<<T as inner::Feature>::Tile as inner::TileFeature<T>>::Item>;

#[rustfmt::skip]
pub type Block<T> = inner::Block<<<T as inner::Feature>::Block as inner::BlockFeature<T>>::Item>;

#[rustfmt::skip]
pub type Entity<T> = inner::Entity<<<T as inner::Feature>::Entity as inner::EntityFeature<T>>::Item>;

// base descriptor

#[derive(Clone)]
pub struct TileDescriptor<T: inner::Feature> {
    pub images: Array<Gd<godot::classes::Image>>,
    pub collision: bool,
    pub feature: T::Tile,
}

#[derive(Clone)]
pub struct TileFieldDescriptor<T: inner::Feature> {
    pub chunk_size: u32,
    pub instance_size: u32,
    pub output_image_size: u32,
    pub max_page_size: u32,
    pub tiles: Vec<TileDescriptor<T>>,
    pub shaders: Array<Gd<godot::classes::Shader>>,
    pub world: Gd<godot::classes::World3D>,
}

#[derive(Clone)]
pub struct BlockDescriptor<T: inner::Feature> {
    pub images: Array<Gd<godot::classes::Image>>,
    pub z_along_y: bool,
    pub size: Vector2i,
    pub collision_size: Vector2,
    pub collision_offset: Vector2,
    pub rendering_size: Vector2,
    pub rendering_offset: Vector2,
    pub feature: T::Block,
}

#[derive(Clone)]
pub struct BlockFieldDescriptor<T: inner::Feature> {
    pub chunk_size: u32,
    pub instance_size: u32,
    pub output_image_size: u32,
    pub max_page_size: u32,
    pub blocks: Vec<BlockDescriptor<T>>,
    pub shaders: Array<Gd<godot::classes::Shader>>,
    pub world: Gd<godot::classes::World3D>,
}

#[derive(Clone)]
pub struct EntityDescriptor<T: inner::Feature> {
    pub images: Array<Gd<godot::classes::Image>>,
    pub z_along_y: bool,
    pub collision_size: Vector2,
    pub collision_offset: Vector2,
    pub rendering_size: Vector2,
    pub rendering_offset: Vector2,
    pub feature: T::Entity,
}

#[derive(Clone)]
pub struct EntityFieldDescriptor<T: inner::Feature> {
    pub chunk_size: u32,
    pub instance_size: u32,
    pub output_image_size: u32,
    pub max_page_size: u32,
    pub entities: Vec<EntityDescriptor<T>>,
    pub shaders: Array<Gd<godot::classes::Shader>>,
    pub world: Gd<godot::classes::World3D>,
}

#[derive(Clone)]
pub struct RootDescriptor<T: inner::Feature> {
    pub tile_field: TileFieldDescriptor<T>,
    pub block_field: BlockFieldDescriptor<T>,
    pub entity_field: EntityFieldDescriptor<T>,
}

/// base root

pub struct Root<T: inner::Feature> {
    pub base: inner::Root<T>,

    tile_field: tile::TileField,
    block_field: block::BlockField,
    entity_field: entity::EntityField,
}

impl<T: inner::Feature> Root<T> {
    pub fn create(desc: &RootDescriptor<T>) -> Root<T> {
        // base
        let base = {
            let mut tile_features = vec![];
            let tile_field = {
                let desc = &desc.tile_field;

                let mut tiles = vec![];
                for tile in &desc.tiles {
                    tiles.push(inner::TileDescriptor {
                        collision: tile.collision,
                    });

                    tile_features.push(tile.feature.clone());
                }

                inner::TileFieldDescriptor {
                    chunk_size: desc.chunk_size,
                    tiles,
                }
            };
            let tile_features = tile_features.into();

            let mut block_features = vec![];
            let block_field = {
                let desc = &desc.block_field;

                let mut blocks = vec![];
                for block in &desc.blocks {
                    blocks.push(inner::BlockDescriptor {
                        size: [block.size.x, block.size.y],
                        collision_size: [block.collision_size.x, block.collision_size.y],
                        collision_offset: [block.collision_offset.x, block.collision_offset.y],
                        hint_size: [block.rendering_size.x, block.rendering_size.y],
                        hint_offset: [block.rendering_offset.x, block.rendering_offset.y],
                    });

                    block_features.push(block.feature.clone());
                }

                inner::BlockFieldDescriptor {
                    chunk_size: desc.chunk_size,
                    blocks,
                }
            };
            let block_features = block_features.into();

            let mut entity_features = vec![];
            let entity_field = {
                let desc = &desc.entity_field;

                let mut entities = vec![];
                for entity in &desc.entities {
                    entities.push(inner::EntityDescriptor {
                        collision_size: [entity.collision_size.x, entity.collision_size.y],
                        collision_offset: [entity.collision_offset.x, entity.collision_offset.y],
                        hint_size: [entity.rendering_size.x, entity.rendering_size.y],
                        hint_offset: [entity.rendering_offset.x, entity.rendering_offset.y],
                    });

                    entity_features.push(entity.feature.clone());
                }

                inner::EntityFieldDescriptor {
                    chunk_size: desc.chunk_size,
                    entities,
                }
            };
            let entity_features = entity_features.into();

            inner::Root::new(inner::RootDescriptor {
                tile_field,
                block_field,
                entity_field,
                tile_features,
                block_features,
                entity_features,
            })
        };

        // tile field renderer
        let tile_field = {
            let desc = &desc.tile_field;

            let mut tiles = vec![];
            for tile in &desc.tiles {
                let mut images = vec![];
                for image in tile.images.iter_shared() {
                    images.push(image.clone());
                }

                tiles.push(tile::TileDescriptor { images });
            }

            let mut tile_shaders = vec![];
            for shader in desc.shaders.iter_shared() {
                tile_shaders.push(shader.clone());
            }

            tile::TileField::new(tile::TileFieldDescriptor {
                instance_size: desc.instance_size,
                output_image_size: desc.output_image_size,
                max_page_size: desc.max_page_size,
                tiles,
                shaders: tile_shaders,
                world: desc.world.clone(),
            })
        };

        // block field renderer
        let block_field = {
            let desc = &desc.block_field;

            let mut blocks = vec![];
            for block in &desc.blocks {
                let mut images = vec![];
                for image in block.images.iter_shared() {
                    images.push(image.clone());
                }

                blocks.push(block::BlockDescriptor {
                    images,
                    z_along_y: block.z_along_y,
                    rendering_size: [block.rendering_size.x, block.rendering_size.y],
                    rendering_offset: [block.rendering_offset.x, block.rendering_offset.y],
                });
            }

            let mut block_shaders = vec![];
            for shader in desc.shaders.iter_shared() {
                block_shaders.push(shader.clone());
            }

            block::BlockField::new(block::BlockFieldDescriptor {
                instance_size: desc.instance_size,
                output_image_size: desc.output_image_size,
                max_page_size: desc.max_page_size,
                blocks,
                shaders: block_shaders,
                world: desc.world.clone(),
            })
        };

        // entity field renderer
        let entity_field = {
            let desc = &desc.entity_field;

            let mut entities = vec![];
            for entity in &desc.entities {
                let mut images = vec![];
                for image in entity.images.iter_shared() {
                    images.push(image.clone());
                }

                entities.push(entity::EntityDescriptor {
                    images,
                    z_along_y: entity.z_along_y,
                    rendering_size: [entity.rendering_size.x, entity.rendering_size.y],
                    rendering_offset: [entity.rendering_offset.x, entity.rendering_offset.y],
                });
            }

            let mut entity_shaders = vec![];
            for shader in desc.shaders.iter_shared() {
                entity_shaders.push(shader.clone());
            }

            entity::EntityField::new(entity::EntityFieldDescriptor {
                instance_size: desc.instance_size,
                output_image_size: desc.output_image_size,
                max_page_size: desc.max_page_size,
                entities,
                shaders: entity_shaders,
                world: desc.world.clone(),
            })
        };

        Root {
            base,
            tile_field,
            block_field,
            entity_field,
        }
    }

    pub fn forward(&mut self, min_rect: Rect2) {
        #[rustfmt::skip]
        let min_rect = [[
            min_rect.position.x,
            min_rect.position.y, ], [
            min_rect.position.x + min_rect.size.x,
            min_rect.position.y + min_rect.size.y,
        ]];

        // tile
        let chunk_size = self.base.tile_get_chunk_size() as f32;
        #[rustfmt::skip]
        let rect = [[
            min_rect[0][0].div_euclid(chunk_size) as i32,
            min_rect[0][1].div_euclid(chunk_size) as i32, ], [
            min_rect[1][0].div_euclid(chunk_size) as i32,
            min_rect[1][1].div_euclid(chunk_size) as i32,
        ]];
        for y in rect[0][1]..=rect[1][1] {
            for x in rect[0][0]..=rect[1][0] {
                let _ = self.base.tile_forward_chunk([x, y]);
            }
        }

        // block
        let chunk_size = self.base.block_get_chunk_size() as f32;
        #[rustfmt::skip]
        let rect = [[
            min_rect[0][0].div_euclid(chunk_size) as i32,
            min_rect[0][1].div_euclid(chunk_size) as i32, ], [
            min_rect[1][0].div_euclid(chunk_size) as i32,
            min_rect[1][1].div_euclid(chunk_size) as i32,
        ]];
        for y in rect[0][1]..=rect[1][1] {
            for x in rect[0][0]..=rect[1][0] {
                let _ = self.base.block_forward_chunk([x, y]);
            }
        }

        // entity
        let chunk_size = self.base.entity_get_chunk_size() as f32;
        #[rustfmt::skip]
        let rect = [[
            min_rect[0][0].div_euclid(chunk_size) as i32,
            min_rect[0][1].div_euclid(chunk_size) as i32, ], [
            min_rect[1][0].div_euclid(chunk_size) as i32,
            min_rect[1][1].div_euclid(chunk_size) as i32,
        ]];
        for y in rect[0][1]..=rect[1][1] {
            for x in rect[0][0]..=rect[1][0] {
                let _ = self.base.entity_forward_chunk([x, y]);
            }
        }
    }

    // tile

    #[inline]
    pub fn tile_insert(&mut self, tile: &Tile<T>) -> TileKey {
        let key = self.base.tile_insert(tile.clone()).unwrap();
        TileKey { base: key }
    }

    #[inline]
    pub fn tile_remove(&mut self, key: Gd<TileKey>) -> Tile<T> {
        self.base.tile_remove(key.bind().base).unwrap()
    }

    #[inline]
    pub fn tile_get(&self, key: Gd<TileKey>) -> Tile<T> {
        self.base.tile_get(key.bind().base).unwrap().clone()
    }

    // block

    #[inline]
    pub fn block_insert(&mut self, block: &Block<T>) -> BlockKey {
        let key = self.base.block_insert(block.clone()).unwrap();
        BlockKey { base: key }
    }

    #[inline]
    pub fn block_remove(&mut self, key: Gd<BlockKey>) -> Block<T> {
        self.base.block_remove(key.bind().base).unwrap()
    }

    #[inline]
    pub fn block_get(&self, key: Gd<BlockKey>) -> Block<T> {
        self.base.block_get(key.bind().base).unwrap().clone()
    }

    // entity

    #[inline]
    pub fn entity_insert(&mut self, entity: &Entity<T>) -> EntityKey {
        let key = self.base.entity_insert(entity.clone()).unwrap();
        EntityKey { base: key }
    }

    #[inline]
    pub fn entity_remove(&mut self, key: Gd<EntityKey>) -> Entity<T> {
        self.base.entity_remove(key.bind().base).unwrap()
    }

    #[inline]
    pub fn entity_get(&self, key: Gd<EntityKey>) -> Entity<T> {
        self.base.entity_get(key.bind().base).unwrap().clone()
    }

    // view

    pub fn update_view(&mut self, min_rect: Rect2) {
        #[rustfmt::skip]
        let min_rect = [[
            min_rect.position.x,
            min_rect.position.y, ], [
            min_rect.position.x + min_rect.size.x,
            min_rect.position.y + min_rect.size.y,
        ]];

        self.tile_field.update_view(&self.base, min_rect);
        self.block_field.update_view(&self.base, min_rect);
        self.entity_field.update_view(&self.base, min_rect);
    }
}
