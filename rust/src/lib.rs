use godot::prelude::*;

pub mod inner;

mod block;
mod entity;
mod tile;

struct Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Extension {}

#[derive(GodotClass)]
#[class(no_init)]
struct TileKey {
    base: inner::TileKey,
}

#[derive(GodotClass)]
#[class(no_init)]
struct Tile {
    base: inner::Tile<inner::TileData>,
}

#[godot_api]
impl Tile {
    #[func]
    fn create(id: u32, location: Vector2i) -> Gd<Self> {
        let tile = inner::Tile {
            id,
            location: [location.x, location.y],
            variant: Default::default(),
            data: Default::default(),
        };
        Gd::from_object(Tile { base: tile })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct TileFeature {
    base: inner::TileFeature,
}

#[godot_api]
impl TileFeature {
    #[func]
    fn create_empty() -> Gd<Self> {
        let feature: inner::TileFeature = inner::TileFeatureEmpty.into();
        Gd::from_object(TileFeature { base: feature })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct TileImageDescriptor {
    frames: Array<Gd<godot::classes::Image>>,
}

#[godot_api]
impl TileImageDescriptor {
    #[func]
    fn create(frames: Array<Gd<godot::classes::Image>>) -> Gd<Self> {
        Gd::from_object(TileImageDescriptor { frames })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct TileDescriptor {
    images: Array<Gd<TileImageDescriptor>>,
    collision: bool,
    feature: Gd<TileFeature>,
}

#[godot_api]
impl TileDescriptor {
    #[func]
    fn create(
        images: Array<Gd<TileImageDescriptor>>,
        collision: bool,
        feature: Gd<TileFeature>,
    ) -> Gd<Self> {
        Gd::from_object(TileDescriptor {
            images,
            collision,
            feature,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct TileFieldDescriptor {
    chunk_size: u32,
    instance_size: u32,
    output_image_size: u32,
    max_page_size: u32,
    tiles: Array<Gd<TileDescriptor>>,
    shaders: Array<Gd<godot::classes::Shader>>,
    world: Gd<godot::classes::World3D>,
}

#[godot_api]
impl TileFieldDescriptor {
    #[func]
    fn create(
        chunk_size: u32,
        instance_size: u32,
        output_image_size: u32,
        max_page_size: u32,
        tiles: Array<Gd<TileDescriptor>>,
        shaders: Array<Gd<godot::classes::Shader>>,
        world: Gd<godot::classes::World3D>,
    ) -> Gd<Self> {
        Gd::from_object(TileFieldDescriptor {
            chunk_size,
            instance_size,
            output_image_size,
            max_page_size,
            tiles,
            shaders,
            world,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct BlockKey {
    base: inner::BlockKey,
}

#[derive(GodotClass)]
#[class(no_init)]
struct Block {
    base: inner::Block<inner::BlockData>,
}

#[godot_api]
impl Block {
    #[func]
    fn create(id: u32, location: Vector2i) -> Gd<Self> {
        let block = inner::Block {
            id,
            location: [location.x, location.y],
            variant: Default::default(),
            data: Default::default(),
        };
        Gd::from_object(Block { base: block })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct BlockFeature {
    base: inner::BlockFeature,
}

#[godot_api]
impl BlockFeature {
    #[func]
    fn create_empty() -> Gd<Self> {
        let feature: inner::BlockFeature = inner::BlockFeatureEmpty.into();
        Gd::from_object(BlockFeature { base: feature })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct BlockDescriptor {
    images: Array<Gd<godot::classes::Image>>,
    z_along_y: bool,
    size: Vector2i,
    collision_size: Vector2,
    collision_offset: Vector2,
    rendering_size: Vector2,
    rendering_offset: Vector2,
    feature: Gd<BlockFeature>,
}

#[godot_api]
impl BlockDescriptor {
    #[func]
    fn create(
        images: Array<Gd<godot::classes::Image>>,
        z_along_y: bool,
        size: Vector2i,
        collision_size: Vector2,
        collision_offset: Vector2,
        rendering_size: Vector2,
        rendering_offset: Vector2,
        feature: Gd<BlockFeature>,
    ) -> Gd<Self> {
        Gd::from_object(BlockDescriptor {
            images,
            z_along_y,
            size,
            collision_size,
            collision_offset,
            rendering_size,
            rendering_offset,
            feature,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct BlockFieldDescriptor {
    chunk_size: u32,
    instance_size: u32,
    output_image_size: u32,
    max_page_size: u32,
    blocks: Array<Gd<BlockDescriptor>>,
    shaders: Array<Gd<godot::classes::Shader>>,
    world: Gd<godot::classes::World3D>,
}

#[godot_api]
impl BlockFieldDescriptor {
    #[func]
    fn create(
        chunk_size: u32,
        instance_size: u32,
        output_image_size: u32,
        max_page_size: u32,
        blocks: Array<Gd<BlockDescriptor>>,
        shaders: Array<Gd<godot::classes::Shader>>,
        world: Gd<godot::classes::World3D>,
    ) -> Gd<Self> {
        Gd::from_object(BlockFieldDescriptor {
            chunk_size,
            instance_size,
            output_image_size,
            max_page_size,
            blocks,
            shaders,
            world,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct EntityKey {
    base: inner::EntityKey,
}

#[derive(GodotClass)]
#[class(no_init)]
struct Entity {
    base: inner::Entity<inner::EntityData>,
}

#[godot_api]
impl Entity {
    #[func]
    fn create(id: u32, location: Vector2) -> Gd<Self> {
        let entity = inner::Entity {
            id,
            location: [location.x, location.y],
            variant: Default::default(),
            data: Default::default(),
        };
        Gd::from_object(Entity { base: entity })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct EntityFeature {
    base: inner::EntityFeature,
}

#[godot_api]
impl EntityFeature {
    #[func]
    fn create_empty() -> Gd<Self> {
        let feature: inner::EntityFeature = inner::EntityFeatureEmpty.into();
        Gd::from_object(EntityFeature { base: feature })
    }

    #[func]
    fn create_animal(
        min_rest_secs: f32,
        max_rest_secs: f32,
        min_distance: f32,
        max_distance: f32,
        speed: f32,
    ) -> Gd<Self> {
        let feature: inner::EntityFeature = inner::EntityFeatureAnimal {
            min_rest_secs,
            max_rest_secs,
            min_distance,
            max_distance,
            speed,
        }
        .into();
        Gd::from_object(EntityFeature { base: feature })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct EntityDescriptor {
    images: Array<Gd<godot::classes::Image>>,
    z_along_y: bool,
    collision_size: Vector2,
    collision_offset: Vector2,
    rendering_size: Vector2,
    rendering_offset: Vector2,
    feature: Gd<EntityFeature>,
}

#[godot_api]
impl EntityDescriptor {
    #[func]
    fn create(
        images: Array<Gd<godot::classes::Image>>,
        z_along_y: bool,
        collision_size: Vector2,
        collision_offset: Vector2,
        rendering_size: Vector2,
        rendering_offset: Vector2,
        feature: Gd<EntityFeature>,
    ) -> Gd<Self> {
        Gd::from_object(EntityDescriptor {
            images,
            z_along_y,
            collision_size,
            collision_offset,
            rendering_size,
            rendering_offset,
            feature,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct EntityFieldDescriptor {
    chunk_size: u32,
    instance_size: u32,
    output_image_size: u32,
    max_page_size: u32,
    entities: Array<Gd<EntityDescriptor>>,
    shaders: Array<Gd<godot::classes::Shader>>,
    world: Gd<godot::classes::World3D>,
}

#[godot_api]
impl EntityFieldDescriptor {
    #[func]
    fn create(
        chunk_size: u32,
        instance_size: u32,
        output_image_size: u32,
        max_page_size: u32,
        entities: Array<Gd<EntityDescriptor>>,
        shaders: Array<Gd<godot::classes::Shader>>,
        world: Gd<godot::classes::World3D>,
    ) -> Gd<Self> {
        Gd::from_object(EntityFieldDescriptor {
            chunk_size,
            instance_size,
            output_image_size,
            max_page_size,
            entities,
            shaders,
            world,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct GeneratorRuleDescriptor {
    base: inner::GeneratorRuleDescriptor,
}

#[godot_api]
impl GeneratorRuleDescriptor {
    #[func]
    fn create_marching(prob: f32, id: u32) -> Gd<Self> {
        let rule = inner::GeneratorRuleMarchingDescriptor { prob, id };
        let desc = inner::GeneratorRuleDescriptor::Marching(rule);
        Gd::from_object(GeneratorRuleDescriptor { base: desc })
    }

    #[func]
    fn create_spawn(prob: f32, id: u32) -> Gd<Self> {
        let rule = inner::GeneratorRuleSpawnDescriptor { prob, id };
        let desc = inner::GeneratorRuleDescriptor::Spawn(rule);
        Gd::from_object(GeneratorRuleDescriptor { base: desc })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct GeneratorDescriptor {
    chunk_size: u32,
    tile_rules: Array<Gd<GeneratorRuleDescriptor>>,
    block_rules: Array<Gd<GeneratorRuleDescriptor>>,
    entity_rules: Array<Gd<GeneratorRuleDescriptor>>,
}

#[godot_api]
impl GeneratorDescriptor {
    #[func]
    fn create(
        chunk_size: u32,
        tile_rules: Array<Gd<GeneratorRuleDescriptor>>,
        block_rules: Array<Gd<GeneratorRuleDescriptor>>,
        entity_rules: Array<Gd<GeneratorRuleDescriptor>>,
    ) -> Gd<Self> {
        Gd::from_object(GeneratorDescriptor {
            chunk_size,
            tile_rules,
            block_rules,
            entity_rules,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct RootDescriptor {
    tile_field: Gd<TileFieldDescriptor>,
    block_field: Gd<BlockFieldDescriptor>,
    entity_field: Gd<EntityFieldDescriptor>,
}

#[godot_api]
impl RootDescriptor {
    #[func]
    fn create(
        tile_field: Gd<TileFieldDescriptor>,
        block_field: Gd<BlockFieldDescriptor>,
        entity_field: Gd<EntityFieldDescriptor>,
    ) -> Gd<Self> {
        Gd::from_object(RootDescriptor {
            tile_field,
            block_field,
            entity_field,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct Root {
    base: inner::Root,
    tile_field: tile::TileField,
    block_field: block::BlockField,
    entity_field: entity::EntityField,
}

#[godot_api]
impl Root {
    #[func]
    fn create(desc: Gd<RootDescriptor>) -> Gd<Root> {
        // base
        let base = {
            let mut tile_features = vec![];
            let tile_field = {
                let desc = desc.bind();
                let desc = desc.tile_field.bind();

                let mut tiles = vec![];
                for tile in desc.tiles.iter_shared() {
                    let tile = tile.bind();

                    tiles.push(inner::TileDescriptor {
                        collision: tile.collision,
                    });

                    let feature = &tile.feature.bind().base;
                    tile_features.push(feature.clone());
                }

                inner::TileFieldDescriptor {
                    chunk_size: desc.chunk_size,
                    tiles,
                }
            };
            let tile_features = tile_features.into();

            let mut block_features = vec![];
            let block_field = {
                let desc = desc.bind();
                let desc = desc.block_field.bind();

                let mut blocks = vec![];
                for block in desc.blocks.iter_shared() {
                    let block = block.bind();

                    blocks.push(inner::BlockDescriptor {
                        size: [block.size.x, block.size.y],
                        collision_size: [block.collision_size.x, block.collision_size.y],
                        collision_offset: [block.collision_offset.x, block.collision_offset.y],
                        hint_size: [block.rendering_size.x, block.rendering_size.y],
                        hint_offset: [block.rendering_offset.x, block.rendering_offset.y],
                    });

                    let feature = &block.feature.bind().base;
                    block_features.push(feature.clone());
                }

                inner::BlockFieldDescriptor {
                    chunk_size: desc.chunk_size,
                    blocks,
                }
            };
            let block_features = block_features.into();

            let mut entity_features = vec![];
            let entity_field = {
                let desc = desc.bind();
                let desc = desc.entity_field.bind();

                let mut entities = vec![];
                for entity in desc.entities.iter_shared() {
                    let entity = entity.bind();

                    entities.push(inner::EntityDescriptor {
                        collision_size: [entity.collision_size.x, entity.collision_size.y],
                        collision_offset: [entity.collision_offset.x, entity.collision_offset.y],
                        hint_size: [entity.rendering_size.x, entity.rendering_size.y],
                        hint_offset: [entity.rendering_offset.x, entity.rendering_offset.y],
                    });

                    let feature = &entity.feature.bind().base;
                    entity_features.push(feature.clone());
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
            let desc = desc.bind();
            let desc = desc.tile_field.bind();

            let mut tiles = vec![];
            for tile in desc.tiles.iter_shared() {
                let tile = tile.bind();

                let mut images = vec![];
                for image in tile.images.iter_shared() {
                    let image = image.bind();

                    let mut frames = vec![];
                    for image in image.frames.iter_shared() {
                        frames.push(image);
                    }
                    images.push(tile::TileImageDescriptor { frames });
                }

                tiles.push(tile::TileDescriptor { images });
            }

            let mut tile_shaders = vec![];
            for shader in desc.shaders.iter_shared() {
                tile_shaders.push(shader);
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
            let desc = desc.bind();
            let desc = desc.block_field.bind();

            let mut blocks = vec![];
            for block in desc.blocks.iter_shared() {
                let block = block.bind();

                let mut images = vec![];
                for image in block.images.iter_shared() {
                    images.push(image);
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
                block_shaders.push(shader);
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
            let desc = desc.bind();
            let desc = desc.entity_field.bind();

            let mut entities = vec![];
            for entity in desc.entities.iter_shared() {
                let entity = entity.bind();

                let mut images = vec![];
                for image in entity.images.iter_shared() {
                    images.push(image);
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

        Gd::from_object(Root {
            base,
            tile_field,
            block_field,
            entity_field,
        })
    }

    // tile

    #[func]
    fn tile_insert(&mut self, tile: Gd<Tile>) -> Gd<TileKey> {
        let tile = &tile.bind().base;
        let key = self.base.tile_insert(tile.clone()).unwrap();
        Gd::from_object(TileKey { base: key })
    }

    #[func]
    fn tile_remove(&mut self, key: Gd<TileKey>) -> Gd<Tile> {
        let tile = self.base.tile_remove(key.bind().base).unwrap();
        Gd::from_object(Tile { base: tile })
    }

    #[func]
    fn tile_get(&self, key: Gd<TileKey>) -> Gd<Tile> {
        let tile = self.base.tile_get(key.bind().base).unwrap().clone();
        Gd::from_object(Tile { base: tile })
    }

    // block

    #[func]
    fn block_insert(&mut self, block: Gd<Block>) -> Gd<BlockKey> {
        let block = &block.bind().base;
        let key = self.base.block_insert(block.clone()).unwrap();
        Gd::from_object(BlockKey { base: key })
    }

    #[func]
    fn block_remove(&mut self, key: Gd<BlockKey>) -> Gd<Block> {
        let block = self.base.block_remove(key.bind().base).unwrap();
        Gd::from_object(Block { base: block })
    }

    #[func]
    fn block_get(&self, key: Gd<BlockKey>) -> Gd<Block> {
        let block = self.base.block_get(key.bind().base).unwrap().clone();
        Gd::from_object(Block { base: block })
    }

    // entity

    #[func]
    fn entity_insert(&mut self, entity: Gd<Entity>) -> Gd<EntityKey> {
        let entity = &entity.bind().base;
        let key = self.base.entity_insert(entity.clone()).unwrap();
        Gd::from_object(EntityKey { base: key })
    }

    #[func]
    fn entity_remove(&mut self, key: Gd<EntityKey>) -> Gd<Entity> {
        let entity = self.base.entity_remove(key.bind().base).unwrap();
        Gd::from_object(Entity { base: entity })
    }

    #[func]
    fn entity_get(&self, key: Gd<EntityKey>) -> Gd<Entity> {
        let entity = self.base.entity_get(key.bind().base).unwrap().clone();
        Gd::from_object(Entity { base: entity })
    }

    // extra

    #[func]
    fn init_forward(&mut self) {
        self.base.init_forward();
    }

    #[func]
    fn forward_rect(&mut self, min_rect: Rect2, delta_secs: f32) {
        #[rustfmt::skip]
        let min_rect = [[
            min_rect.position.x,
            min_rect.position.y, ], [
            min_rect.position.x + min_rect.size.x,
            min_rect.position.y + min_rect.size.y,
        ]];

        self.base.forward_rect(min_rect, delta_secs);
    }

    #[func]
    fn init_generator(&mut self, desc: Gd<GeneratorDescriptor>) {
        let desc = desc.bind();

        let mut tile_rules = vec![];
        for rule in desc.tile_rules.iter_shared() {
            let rule = &rule.bind().base;
            tile_rules.push(rule.clone());
        }

        let mut block_rules = vec![];
        for rule in desc.block_rules.iter_shared() {
            let rule = &rule.bind().base;
            block_rules.push(rule.clone());
        }

        let mut entity_rules = vec![];
        for rule in desc.entity_rules.iter_shared() {
            let rule = &rule.bind().base;
            entity_rules.push(rule.clone());
        }

        let desc = inner::GeneratorDescriptor {
            chunk_size: desc.chunk_size,
            tile_rules,
            block_rules,
            entity_rules,
        };

        self.base.init_generator(desc);
    }

    #[func]
    fn generate_rect(&mut self, min_rect: Rect2) {
        #[rustfmt::skip]
        let min_rect = [[
            min_rect.position.x,
            min_rect.position.y, ], [
            min_rect.position.x + min_rect.size.x,
            min_rect.position.y + min_rect.size.y,
        ]];

        self.base.generate_rect(min_rect);
    }

    // view

    #[func]
    fn update_view(&mut self, min_rect: Rect2) {
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
