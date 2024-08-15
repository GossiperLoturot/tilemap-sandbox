use godot::prelude::*;

pub mod inner;

mod block;
mod entity;
mod extra;
mod tile;

struct Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Extension {}

#[derive(GodotClass)]
#[class(no_init)]
pub struct TileKey {
    pub inner: inner::TileKey,
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct Tile {
    pub inner: inner::Tile,
}

#[godot_api]
impl Tile {
    #[func]
    fn create(id: u32, location: Vector2i, variant: u8) -> Gd<Self> {
        let location = [location.x, location.y];
        let inner = inner::Tile::new(id, location, variant);
        Gd::from_object(Self { inner })
    }

    #[func]
    fn get_id(&self) -> u32 {
        self.inner.id
    }

    #[func]
    fn get_location(&self) -> Vector2i {
        let location = self.inner.location;
        Vector2i::new(location[0], location[1])
    }

    #[func]
    fn get_variant(&self) -> u8 {
        self.inner.variant
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct BlockKey {
    pub inner: inner::BlockKey,
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct Block {
    pub inner: inner::Block,
}

#[godot_api]
impl Block {
    #[func]
    fn create(id: u32, location: Vector2i, variant: u8) -> Gd<Self> {
        let location = [location.x, location.y];
        let inner = inner::Block::new(id, location, variant);
        Gd::from_object(Self { inner })
    }

    #[func]
    fn get_id(&self) -> u32 {
        self.inner.id
    }

    #[func]
    fn get_location(&self) -> Vector2i {
        let location = self.inner.location;
        Vector2i::new(location[0], location[1])
    }

    #[func]
    fn get_variant(&self) -> u8 {
        self.inner.variant
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct EntityKey {
    pub inner: inner::EntityKey,
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct Entity {
    pub inner: inner::Entity,
}

#[godot_api]
impl Entity {
    #[func]
    fn create(id: u32, location: Vector2, variant: u8) -> Gd<Self> {
        let location = [location.x, location.y];
        let inner = inner::Entity::new(id, location, variant);
        Gd::from_object(Self { inner })
    }

    #[func]
    fn get_id(&self) -> u32 {
        self.inner.id
    }

    #[func]
    fn get_location(&self) -> Vector2 {
        let location = self.inner.location;
        Vector2::new(location[0], location[1])
    }

    #[func]
    fn get_variant(&self) -> u8 {
        self.inner.variant
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct TileDescriptor {
    #[export]
    pub images: Array<Gd<godot::engine::Image>>,
    #[export]
    pub collision: bool,
}

#[godot_api]
impl TileDescriptor {
    #[func]
    fn create(images: Array<Gd<godot::engine::Image>>, collision: bool) -> Gd<TileDescriptor> {
        Gd::from_object(TileDescriptor { images, collision })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct TileFieldDescriptor {
    #[export]
    pub chunk_size: u32,
    #[export]
    pub instance_size: u32,
    #[export]
    pub output_image_size: u32,
    #[export]
    pub max_page_size: u32,
    #[export]
    pub tiles: Array<Gd<TileDescriptor>>,
    #[export]
    pub shaders: Array<Gd<godot::engine::Shader>>,
    #[export]
    pub world: Gd<godot::engine::World3D>,
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
        shaders: Array<Gd<godot::engine::Shader>>,
        world: Gd<godot::engine::World3D>,
    ) -> Gd<TileFieldDescriptor> {
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
pub struct BlockDescriptor {
    #[export]
    pub images: Array<Gd<godot::engine::Image>>,
    #[export]
    pub z_along_y: bool,
    #[export]
    pub size: Vector2i,
    #[export]
    pub collision_size: Vector2,
    #[export]
    pub collision_offset: Vector2,
    #[export]
    pub rendering_size: Vector2,
    #[export]
    pub rendering_offset: Vector2,
}

#[godot_api]
impl BlockDescriptor {
    #[func]
    fn create(
        images: Array<Gd<godot::engine::Image>>,
        z_along_y: bool,
        size: Vector2i,
        collision_size: Vector2,
        collision_offset: Vector2,
        rendering_size: Vector2,
        rendering_offset: Vector2,
    ) -> Gd<BlockDescriptor> {
        Gd::from_object(BlockDescriptor {
            images,
            z_along_y,
            size,
            collision_size,
            collision_offset,
            rendering_size,
            rendering_offset,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct BlockFieldDescriptor {
    #[export]
    pub chunk_size: u32,
    #[export]
    pub instance_size: u32,
    #[export]
    pub output_image_size: u32,
    #[export]
    pub max_page_size: u32,
    #[export]
    pub blocks: Array<Gd<BlockDescriptor>>,
    #[export]
    pub shaders: Array<Gd<godot::engine::Shader>>,
    #[export]
    pub world: Gd<godot::engine::World3D>,
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
        shaders: Array<Gd<godot::engine::Shader>>,
        world: Gd<godot::engine::World3D>,
    ) -> Gd<BlockFieldDescriptor> {
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
pub struct EntityDescriptor {
    #[export]
    pub images: Array<Gd<godot::engine::Image>>,
    #[export]
    pub z_along_y: bool,
    #[export]
    pub collision_size: Vector2,
    #[export]
    pub collision_offset: Vector2,
    #[export]
    pub rendering_size: Vector2,
    #[export]
    pub rendering_offset: Vector2,
}

#[godot_api]
impl EntityDescriptor {
    #[func]
    fn create(
        images: Array<Gd<godot::engine::Image>>,
        z_along_y: bool,
        collision_size: Vector2,
        collision_offset: Vector2,
        rendering_size: Vector2,
        rendering_offset: Vector2,
    ) -> Gd<EntityDescriptor> {
        Gd::from_object(EntityDescriptor {
            images,
            z_along_y,
            collision_size,
            collision_offset,
            rendering_size,
            rendering_offset,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct EntityFieldDescriptor {
    #[export]
    pub chunk_size: u32,
    #[export]
    pub instance_size: u32,
    #[export]
    pub output_image_size: u32,
    #[export]
    pub max_page_size: u32,
    #[export]
    pub entities: Array<Gd<EntityDescriptor>>,
    #[export]
    pub shaders: Array<Gd<godot::engine::Shader>>,
    #[export]
    pub world: Gd<godot::engine::World3D>,
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
        shaders: Array<Gd<godot::engine::Shader>>,
        world: Gd<godot::engine::World3D>,
    ) -> Gd<EntityFieldDescriptor> {
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
pub struct FlowDescriptor {
    pub value: std::rc::Rc<dyn inner::FlowBundle>,
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct FlowStoreDescriptor {
    #[export]
    pub bundles: Array<Gd<FlowDescriptor>>,
}

#[godot_api]
impl FlowStoreDescriptor {
    #[func]
    fn create(bundles: Array<Gd<FlowDescriptor>>) -> Gd<FlowStoreDescriptor> {
        Gd::from_object(FlowStoreDescriptor { bundles })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct RootDescriptor {
    #[export]
    pub tile_field: Gd<TileFieldDescriptor>,
    #[export]
    pub block_field: Gd<BlockFieldDescriptor>,
    #[export]
    pub entity_field: Gd<EntityFieldDescriptor>,
    #[export]
    pub flow_store: Gd<FlowStoreDescriptor>,
}

#[godot_api]
impl RootDescriptor {
    #[func]
    fn create(
        tile_field: Gd<TileFieldDescriptor>,
        block_field: Gd<BlockFieldDescriptor>,
        entity_field: Gd<EntityFieldDescriptor>,
        flow_store: Gd<FlowStoreDescriptor>,
    ) -> Gd<RootDescriptor> {
        Gd::from_object(RootDescriptor {
            tile_field,
            block_field,
            entity_field,
            flow_store,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct Root {
    pub inner: inner::Root,
    tile_field: tile::TileField,
    block_field: block::BlockField,
    entity_field: entity::EntityField,
}

#[godot_api]
impl Root {
    #[func]
    fn create(desc: Gd<RootDescriptor>) -> Gd<Root> {
        let desc = desc.bind();

        // inner
        let inner = {
            let tile_field = {
                let desc = desc.tile_field.bind();

                let mut tiles = vec![];
                for tile in desc.tiles.iter_shared() {
                    let tile = tile.bind();

                    tiles.push(inner::TileDescriptor {
                        collision: tile.collision,
                    });
                }

                inner::TileFieldDescriptor {
                    chunk_size: desc.chunk_size,
                    tiles,
                }
            };

            let block_field = {
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
                }

                inner::BlockFieldDescriptor {
                    chunk_size: desc.chunk_size,
                    blocks,
                }
            };

            let entity_field = {
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
                }

                inner::EntityFieldDescriptor {
                    chunk_size: desc.chunk_size,
                    entities,
                }
            };

            let flow_store = {
                let desc = desc.flow_store.bind();

                let mut bundles = vec![];
                for bundle in desc.bundles.iter_shared() {
                    let bundle = bundle.bind();

                    bundles.push(inner::FlowDescriptor {
                        value: bundle.value.clone(),
                    });
                }

                inner::FlowStoreDescriptor { bundles }
            };

            inner::Root::new(inner::RootDescriptor {
                tile_field,
                block_field,
                entity_field,
                flow_store,
            })
        };

        // tile field renderer
        let tile_field = {
            let desc = desc.tile_field.bind();

            let mut tiles = vec![];
            for tile in desc.tiles.iter_shared() {
                let tile = tile.bind();

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
            let desc = desc.block_field.bind();

            let mut blocks = vec![];
            for block in desc.blocks.iter_shared() {
                let block = block.bind();

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
            let desc = desc.entity_field.bind();

            let mut entities = vec![];
            for entity in desc.entities.iter_shared() {
                let entity = entity.bind();

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

        Gd::from_object(Root {
            inner,
            tile_field,
            block_field,
            entity_field,
        })
    }

    #[func]
    fn update_view(&mut self, min_view_rect: Rect2) {
        #[rustfmt::skip]
        let min_view_rect = [[
            min_view_rect.position.x,
            min_view_rect.position.y, ], [
            min_view_rect.position.x + min_view_rect.size.x,
            min_view_rect.position.y + min_view_rect.size.y,
        ]];

        self.tile_field.update_view(&self.inner, min_view_rect);
        self.block_field.update_view(&self.inner, min_view_rect);
        self.entity_field.update_view(&self.inner, min_view_rect);
    }
}
