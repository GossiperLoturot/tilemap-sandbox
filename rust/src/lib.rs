use glam::*;
use godot::prelude::*;

pub mod inner;

mod block;
mod entity;
mod item;
mod register;
mod tile;

struct Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Extension {}

#[derive(GodotClass)]
#[class(no_init)]
struct PanicHook {}

#[godot_api]
impl PanicHook {
    #[func]
    fn set_hook() {
        godot_print!("Set panic hook");

        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let location_msg;
            if let Some(location) = info.location() {
                location_msg = format!("file {} at line {}", location.file(), location.line());
            } else {
                location_msg = "unknown location".into();
            }

            let payload_msg;
            if let Some(s) = info.payload().downcast_ref::<&str>() {
                payload_msg = s.to_string();
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                payload_msg = s.clone();
            } else {
                payload_msg = "unknown panic".into();
            }

            godot_error!("[RUST] {}: {}", location_msg, payload_msg);
            hook(info);
        }));
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct TileKey {
    base: inner::TileKey,
}

#[derive(GodotClass)]
#[class(no_init)]
struct Tile {
    base: inner::Tile,
}

#[godot_api]
impl Tile {
    #[func]
    fn create(id: u16, location: Vector2i) -> Gd<Self> {
        let tile = inner::Tile {
            id,
            location: IVec2::new(location.x, location.y),
            data: Default::default(),
            render_param: Default::default(),
        };
        Gd::from_object(Tile { base: tile })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct TileFeature {
    base: Box<dyn inner::TileFeature>,
}

#[godot_api]
impl TileFeature {
    #[func]
    fn create_empty() -> Gd<Self> {
        let feature = Default::default();
        Gd::from_object(TileFeature { base: feature })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct TileImageDescriptor {
    frames: Array<Gd<godot::classes::Image>>,
    step_tick: u16,
    is_loop: bool,
}

#[godot_api]
impl TileImageDescriptor {
    #[func]
    fn create(frames: Array<Gd<godot::classes::Image>>, step_tick: u16, is_loop: bool) -> Gd<Self> {
        Gd::from_object(TileImageDescriptor {
            frames,
            step_tick,
            is_loop,
        })
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
    tiles: Array<Gd<TileDescriptor>>,
    shaders: Array<Gd<godot::classes::Shader>>,
    world: Gd<godot::classes::World3D>,
}

#[godot_api]
impl TileFieldDescriptor {
    #[func]
    fn create(
        tiles: Array<Gd<TileDescriptor>>,
        shaders: Array<Gd<godot::classes::Shader>>,
        world: Gd<godot::classes::World3D>,
    ) -> Gd<Self> {
        Gd::from_object(TileFieldDescriptor {
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
    base: inner::Block,
}

#[godot_api]
impl Block {
    #[func]
    fn create(id: u16, location: Vector2i) -> Gd<Self> {
        let block = inner::Block {
            id,
            location: IVec2::new(location.x, location.y),
            data: Default::default(),
            render_param: Default::default(),
        };
        Gd::from_object(Block { base: block })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct BlockFeature {
    base: Box<dyn inner::BlockFeature>,
}

#[godot_api]
impl BlockFeature {
    #[func]
    fn create_empty() -> Gd<Self> {
        let feature = Default::default();
        Gd::from_object(BlockFeature { base: feature })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct BlockImageDescriptor {
    frames: Array<Gd<godot::classes::Image>>,
    step_tick: u16,
    is_loop: bool,
}

#[godot_api]
impl BlockImageDescriptor {
    #[func]
    fn create(frames: Array<Gd<godot::classes::Image>>, step_tick: u16, is_loop: bool) -> Gd<Self> {
        Gd::from_object(BlockImageDescriptor {
            frames,
            step_tick,
            is_loop,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct BlockDescriptor {
    images: Array<Gd<BlockImageDescriptor>>,
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
        images: Array<Gd<BlockImageDescriptor>>,
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
    blocks: Array<Gd<BlockDescriptor>>,
    shaders: Array<Gd<godot::classes::Shader>>,
    world: Gd<godot::classes::World3D>,
}

#[godot_api]
impl BlockFieldDescriptor {
    #[func]
    fn create(
        blocks: Array<Gd<BlockDescriptor>>,
        shaders: Array<Gd<godot::classes::Shader>>,
        world: Gd<godot::classes::World3D>,
    ) -> Gd<Self> {
        Gd::from_object(BlockFieldDescriptor {
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
    base: inner::Entity,
}

#[godot_api]
impl Entity {
    #[func]
    fn create(id: u16, location: Vector2) -> Gd<Self> {
        let entity = inner::Entity {
            id,
            location: Vec2::new(location.x, location.y),
            data: Default::default(),
            render_param: Default::default(),
        };
        Gd::from_object(Entity { base: entity })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct EntityFeature {
    base: Box<dyn inner::EntityFeature>,
}

#[godot_api]
impl EntityFeature {
    #[func]
    fn create_empty() -> Gd<Self> {
        let feature = Default::default();
        Gd::from_object(EntityFeature { base: feature })
    }

    #[func]
    fn create_animal(
        min_rest_secs: f32,
        max_rest_secs: f32,
        min_distance: f32,
        max_distance: f32,
        speed: f32,
        idle_variant: u8,
        walk_variant: u8,
    ) -> Gd<Self> {
        let feature = Box::new(inner::AnimalEntityFeature {
            min_rest_secs,
            max_rest_secs,
            min_distance,
            max_distance,
            speed,
            idle_variant,
            walk_variant,
        });
        Gd::from_object(EntityFeature { base: feature })
    }

    #[func]
    fn create_player() -> Gd<Self> {
        let feature = Box::new(inner::PlayerEntityFeature);
        Gd::from_object(EntityFeature { base: feature })
    }

    #[func]
    fn create_item() -> Gd<Self> {
        let feature = Box::new(inner::ItemEntityFeature);
        Gd::from_object(EntityFeature { base: feature })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct EntityImageDescriptor {
    frames: Array<Gd<godot::classes::Image>>,
    step_tick: u16,
    is_loop: bool,
}

#[godot_api]
impl EntityImageDescriptor {
    #[func]
    fn create(frames: Array<Gd<godot::classes::Image>>, step_tick: u16, is_loop: bool) -> Gd<Self> {
        Gd::from_object(EntityImageDescriptor {
            frames,
            step_tick,
            is_loop,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct EntityDescriptor {
    images: Array<Gd<EntityImageDescriptor>>,
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
        images: Array<Gd<EntityImageDescriptor>>,
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
    entities: Array<Gd<EntityDescriptor>>,
    shaders: Array<Gd<godot::classes::Shader>>,
    world: Gd<godot::classes::World3D>,
}

#[godot_api]
impl EntityFieldDescriptor {
    #[func]
    fn create(
        entities: Array<Gd<EntityDescriptor>>,
        shaders: Array<Gd<godot::classes::Shader>>,
        world: Gd<godot::classes::World3D>,
    ) -> Gd<Self> {
        Gd::from_object(EntityFieldDescriptor {
            entities,
            shaders,
            world,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct ItemFeature {
    base: Box<dyn inner::ItemFeature>,
}

#[godot_api]
impl ItemFeature {
    #[func]
    fn create_empty() -> Gd<Self> {
        let feature = Default::default();
        Gd::from_object(ItemFeature { base: feature })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct ItemDescriptor {
    name_text: String,
    desc_text: String,
    image: Gd<godot::classes::Image>,
    feature: Gd<ItemFeature>,
}

#[godot_api]
impl ItemDescriptor {
    #[func]
    fn create(
        name_text: String,
        desc_text: String,
        image: Gd<godot::classes::Image>,
        feature: Gd<ItemFeature>,
    ) -> Gd<Self> {
        Gd::from_object(ItemDescriptor {
            name_text,
            desc_text,
            image,
            feature,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct ItemStoreDescriptor {
    items: Array<Gd<ItemDescriptor>>,
}

#[godot_api]
impl ItemStoreDescriptor {
    #[func]
    fn create(items: Array<Gd<ItemDescriptor>>) -> Gd<Self> {
        Gd::from_object(ItemStoreDescriptor { items })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct GenRule {
    base: inner::GenRule,
}

#[godot_api]
impl GenRule {
    #[func]
    fn create_march_tile(prob: f32, id: u16) -> Gd<Self> {
        let gen_fn = move |root: &mut inner::Root, location: IVec2| {
            let tile = inner::Tile {
                id,
                location,
                data: Default::default(),
                render_param: Default::default(),
            };
            let _ = root.tile_insert(tile);
        };
        let gen_fn = Box::new(gen_fn);
        let rule = inner::MarchGenRule { prob, gen_fn };
        let desc = inner::GenRule::March(rule);
        Gd::from_object(GenRule { base: desc })
    }

    #[func]
    fn create_march_block(prob: f32, id: u16) -> Gd<Self> {
        let gen_fn = move |root: &mut inner::Root, location: IVec2| {
            let block = inner::Block {
                id,
                location,
                data: Default::default(),
                render_param: Default::default(),
            };
            let _ = root.block_insert(block);
        };
        let gen_fn = Box::new(gen_fn);
        let rule = inner::MarchGenRule { prob, gen_fn };
        let desc = inner::GenRule::March(rule);
        Gd::from_object(GenRule { base: desc })
    }

    #[func]
    fn create_march_entity(prob: f32, id: u16) -> Gd<Self> {
        let gen_fn = move |root: &mut inner::Root, location: IVec2| {
            let entity = inner::Entity {
                id,
                location: location.as_vec2(),
                data: Default::default(),
                render_param: Default::default(),
            };
            let _ = root.entity_insert(entity);
        };
        let gen_fn = Box::new(gen_fn);
        let rule = inner::MarchGenRule { prob, gen_fn };
        let desc = inner::GenRule::March(rule);
        Gd::from_object(GenRule { base: desc })
    }

    #[func]
    fn create_spawn_tile(prob: f32, id: u16) -> Gd<Self> {
        let gen_fn = move |root: &mut inner::Root, location: Vec2| {
            let tile = inner::Tile {
                id,
                location: location.as_ivec2(),
                data: Default::default(),
                render_param: Default::default(),
            };
            let _ = root.tile_insert(tile);
        };
        let gen_fn = Box::new(gen_fn);
        let rule = inner::SpawnGenRule { prob, gen_fn };
        let desc = inner::GenRule::Spawn(rule);
        Gd::from_object(GenRule { base: desc })
    }

    #[func]
    fn create_spawn_block(prob: f32, id: u16) -> Gd<Self> {
        let gen_fn = move |root: &mut inner::Root, location: Vec2| {
            let block = inner::Block {
                id,
                location: location.as_ivec2(),
                data: Default::default(),
                render_param: Default::default(),
            };
            let _ = root.block_insert(block);
        };
        let gen_fn = Box::new(gen_fn);
        let rule = inner::SpawnGenRule { prob, gen_fn };
        let desc = inner::GenRule::Spawn(rule);
        Gd::from_object(GenRule { base: desc })
    }

    #[func]
    fn create_spawn_entity(prob: f32, id: u16) -> Gd<Self> {
        let gen_fn = move |root: &mut inner::Root, location: Vec2| {
            let entity = inner::Entity {
                id,
                location,
                data: Default::default(),
                render_param: Default::default(),
            };
            let _ = root.entity_insert(entity);
        };
        let gen_fn = Box::new(gen_fn);
        let rule = inner::SpawnGenRule { prob, gen_fn };
        let desc = inner::GenRule::Spawn(rule);
        Gd::from_object(GenRule { base: desc })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct GenResourceDescriptor {
    gen_rules: Array<Gd<GenRule>>,
}

#[godot_api]
impl GenResourceDescriptor {
    #[func]
    fn create(gen_rules: Array<Gd<GenRule>>) -> Gd<Self> {
        Gd::from_object(GenResourceDescriptor { gen_rules })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct RootDescriptor {
    tile_field: Gd<TileFieldDescriptor>,
    block_field: Gd<BlockFieldDescriptor>,
    entity_field: Gd<EntityFieldDescriptor>,
    item_store: Gd<ItemStoreDescriptor>,
    gen_resource: Gd<GenResourceDescriptor>,
}

#[godot_api]
impl RootDescriptor {
    #[func]
    fn create(
        tile_field: Gd<TileFieldDescriptor>,
        block_field: Gd<BlockFieldDescriptor>,
        entity_field: Gd<EntityFieldDescriptor>,
        item_store: Gd<ItemStoreDescriptor>,
        gen_resource: Gd<GenResourceDescriptor>,
    ) -> Gd<Self> {
        Gd::from_object(RootDescriptor {
            tile_field,
            block_field,
            entity_field,
            item_store,
            gen_resource,
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
    item_store: item::ItemStore,
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

                inner::TileFieldDescriptor { tiles }
            };
            let tile_features = tile_features.into();

            let mut block_features = vec![];
            let block_field = {
                let desc = desc.bind();
                let desc = desc.block_field.bind();

                let mut blocks = vec![];
                for block in desc.blocks.iter_shared() {
                    let block = block.bind();

                    #[rustfmt::skip]
                    blocks.push(inner::BlockDescriptor {
                        size: IVec2::new(block.size.x, block.size.y),
                        collision_size: Vec2::new(block.collision_size.x, block.collision_size.y),
                        collision_offset: Vec2::new(block.collision_offset.x, block.collision_offset.y),
                        hint_size: Vec2::new(block.rendering_size.x, block.rendering_size.y),
                        hint_offset: Vec2::new(block.rendering_offset.x, block.rendering_offset.y),
                    });

                    let feature = &block.feature.bind().base;
                    block_features.push(feature.clone());
                }

                inner::BlockFieldDescriptor { blocks }
            };
            let block_features = block_features.into();

            let mut entity_features = vec![];
            let entity_field = {
                let desc = desc.bind();
                let desc = desc.entity_field.bind();

                let mut entities = vec![];
                for entity in desc.entities.iter_shared() {
                    let entity = entity.bind();

                    #[rustfmt::skip]
                    entities.push(inner::EntityDescriptor {
                        collision_size: Vec2::new(entity.collision_size.x, entity.collision_size.y),
                        collision_offset: Vec2::new(entity.collision_offset.x, entity.collision_offset.y),
                        hint_size: Vec2::new(entity.rendering_size.x, entity.rendering_size.y),
                        hint_offset: Vec2::new(entity.rendering_offset.x, entity.rendering_offset.y),
                    });

                    let feature = &entity.feature.bind().base;
                    entity_features.push(feature.clone());
                }

                inner::EntityFieldDescriptor { entities }
            };
            let entity_features = entity_features.into();

            let mut item_features = vec![];
            let item_store = {
                let desc = desc.bind();
                let desc = desc.item_store.bind();

                let mut items = vec![];
                for item in desc.items.iter_shared() {
                    let item = item.bind();

                    items.push(inner::ItemDescriptor {
                        name_text: item.name_text.clone(),
                    });

                    let feature = &item.feature.bind().base;
                    item_features.push(feature.clone());
                }

                inner::ItemStoreDescriptor { items }
            };
            let item_features = item_features.into();

            let gen_resource = inner::GenResourceDescriptor {
                gen_rules: Default::default(),
            };

            inner::Root::new(inner::RootDescriptor {
                tile_field,
                block_field,
                entity_field,
                item_store,

                tile_features,
                block_features,
                entity_features,
                item_features,

                gen_resource,
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

                    images.push(tile::TileImageDescriptor {
                        frames,
                        step_tick: image.step_tick,
                        is_loop: image.is_loop,
                    });
                }

                tiles.push(tile::TileDescriptor { images });
            }

            let mut tile_shaders = vec![];
            for shader in desc.shaders.iter_shared() {
                tile_shaders.push(shader);
            }

            tile::TileField::new(tile::TileFieldDescriptor {
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
                    let image = image.bind();

                    let mut frames = vec![];
                    for image in image.frames.iter_shared() {
                        frames.push(image);
                    }

                    images.push(block::BlockImageDescriptor {
                        frames,
                        step_tick: image.step_tick,
                        is_loop: image.is_loop,
                    });
                }

                blocks.push(block::BlockDescriptor {
                    images,
                    z_along_y: block.z_along_y,
                    rendering_size: Vec2::new(block.rendering_size.x, block.rendering_size.y),
                    rendering_offset: Vec2::new(block.rendering_offset.x, block.rendering_offset.y),
                });
            }

            let mut block_shaders = vec![];
            for shader in desc.shaders.iter_shared() {
                block_shaders.push(shader);
            }

            block::BlockField::new(block::BlockFieldDescriptor {
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
                    let image = image.bind();

                    let mut frames = vec![];
                    for image in image.frames.iter_shared() {
                        frames.push(image);
                    }

                    images.push(entity::EntityImageDescriptor {
                        frames,
                        step_tick: image.step_tick,
                        is_loop: image.is_loop,
                    });
                }

                entities.push(entity::EntityDescriptor {
                    images,
                    z_along_y: entity.z_along_y,
                    rendering_size: Vec2::new(entity.rendering_size.x, entity.rendering_size.y),
                    rendering_offset: Vec2::new(
                        entity.rendering_offset.x,
                        entity.rendering_offset.y,
                    ),
                });
            }

            let mut entity_shaders = vec![];
            for shader in desc.shaders.iter_shared() {
                entity_shaders.push(shader.clone());
            }

            entity::EntityField::new(entity::EntityFieldDescriptor {
                entities,
                shaders: entity_shaders,
                world: desc.world.clone(),
            })
        };

        // item store renderer
        let item_store = {
            let desc = desc.bind();
            let desc = desc.item_store.bind();

            let mut items = vec![];
            for item in desc.items.iter_shared() {
                let item = item.bind();

                items.push(item::ItemDescriptor {
                    name_text: item.name_text.clone(),
                    desc_text: item.desc_text.clone(),
                    image: item.image.clone(),
                });
            }

            item::ItemStore::new(item::ItemStoreDescriptor { items })
        };

        Gd::from_object(Root {
            base,
            tile_field,
            block_field,
            entity_field,
            item_store,
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
        let tile = self.base.tile_get(key.bind().base).unwrap();
        Gd::from_object(Tile { base: tile.clone() })
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
        let block = self.base.block_get(key.bind().base).unwrap();
        Gd::from_object(Block {
            base: block.clone(),
        })
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
        Gd::from_object(Entity {
            base: entity.clone(),
        })
    }

    #[func]
    fn entity_get(&self, key: Gd<EntityKey>) -> Gd<Entity> {
        let entity = self.base.entity_get(key.bind().base).unwrap().clone();
        Gd::from_object(Entity {
            base: entity.clone(),
        })
    }

    // item

    #[func]
    fn item_name_text(&self, id: u32) -> String {
        self.item_store.get_name_text(id).unwrap()
    }

    #[func]
    fn item_desc_text(&self, id: u32) -> String {
        self.item_store.get_desc_text(id).unwrap()
    }

    #[func]
    fn item_image(&self, id: u32) -> Gd<godot::classes::Image> {
        self.item_store.get_image(id).unwrap()
    }

    // time

    #[func]
    fn time_tick_per_secs(&self) -> u64 {
        self.base.time_tick_per_secs()
    }

    #[func]
    fn time_tick(&self) -> u64 {
        self.base.time_tick()
    }

    #[func]
    fn time_forward(&mut self, delta_secs: f32) {
        self.base.time_forward(delta_secs);
    }

    // extra

    #[func]
    fn forwarder_exec_rect(&mut self, min_rect: Rect2, delta_secs: f32) {
        let position = Vec2::new(min_rect.position.x, min_rect.position.y);
        let size = Vec2::new(min_rect.size.x, min_rect.size.y);
        let min_rect = [position, position + size];

        self.base.forwarder_exec_rect(min_rect, delta_secs).unwrap();
    }

    #[func]
    fn gen_exec_rect(&mut self, min_rect: Rect2) {
        let position = Vec2::new(min_rect.position.x, min_rect.position.y);
        let size = Vec2::new(min_rect.size.x, min_rect.size.y);
        let min_rect = [position, position + size];

        self.base.gen_exec_rect(min_rect).unwrap();
    }

    #[func]
    fn player_insert_input(&mut self, input: Vector2) {
        let input = Vec2::new(input.x, input.y);
        self.base.player_insert_input(input).unwrap();
    }

    #[func]
    fn player_get_current_location(&mut self) -> Vector2 {
        let location = self.base.player_get_current_location().unwrap();
        Vector2::new(location[0], location[1])
    }

    // view

    #[func]
    fn update_view(&mut self, min_rect: Rect2) {
        let position = Vec2::new(min_rect.position.x, min_rect.position.y);
        let size = Vec2::new(min_rect.size.x, min_rect.size.y);
        let min_rect = [position, position + size];

        self.tile_field.update_view(&self.base, min_rect);
        self.block_field.update_view(&self.base, min_rect);
        self.entity_field.update_view(&self.base, min_rect);
    }
}
