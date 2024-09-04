// TODO: Merge with `src/lib.rs` to simple the code by squash abstract layer.
use godot::prelude::*;

mod feature;
mod generator;

#[derive(GodotClass)]
#[class(no_init)]
struct TileFeature {
    base: <feature::Feature as crate::inner::Feature>::Tile,
}

#[godot_api]
impl TileFeature {
    #[func]
    #[inline]
    fn create() -> Gd<Self> {
        Gd::from_object(TileFeature {
            base: feature::TileFeature,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct BlockFeature {
    base: <feature::Feature as crate::inner::Feature>::Block,
}

#[godot_api]
impl BlockFeature {
    #[func]
    #[inline]
    fn create() -> Gd<Self> {
        Gd::from_object(BlockFeature {
            base: feature::BlockFeature,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct EntityFeature {
    base: <feature::Feature as crate::inner::Feature>::Entity,
}

#[godot_api]
impl EntityFeature {
    #[func]
    #[inline]
    fn create() -> Gd<Self> {
        Gd::from_object(EntityFeature {
            base: feature::EntityFeature,
        })
    }
}

// derive

#[derive(GodotClass)]
#[class(no_init)]
struct Tile {
    base: crate::Tile<feature::Feature>,
}

#[godot_api]
impl Tile {
    #[func]
    #[inline]
    fn create(id: u32, location: Vector2i) -> Gd<Self> {
        let location = [location.x, location.y];
        let tile = crate::inner::Tile {
            id,
            location,
            variant: Default::default(),
            data: Default::default(),
        };
        Gd::from_object(Tile { base: tile })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct Block {
    base: crate::Block<feature::Feature>,
}

#[godot_api]
impl Block {
    #[func]
    #[inline]
    fn create(id: u32, location: Vector2i) -> Gd<Self> {
        let location = [location.x, location.y];
        let block = crate::inner::Block {
            id,
            location,
            variant: Default::default(),
            data: Default::default(),
        };
        Gd::from_object(Block { base: block })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct Entity {
    base: crate::Entity<feature::Feature>,
}

#[godot_api]
impl Entity {
    #[func]
    #[inline]
    fn create(id: u32, location: Vector2) -> Gd<Self> {
        let location = [location.x, location.y];
        let entity = crate::inner::Entity {
            id,
            location,
            variant: Default::default(),
            data: Default::default(),
        };
        Gd::from_object(Entity { base: entity })
    }
}

// derive descriptor

#[derive(GodotClass)]
#[class(no_init)]
struct TileDescriptor {
    base: crate::TileDescriptor<feature::Feature>,
}

#[godot_api]
impl TileDescriptor {
    #[func]
    #[inline]
    fn create(
        images: Array<Gd<godot::classes::Image>>,
        collision: bool,
        feature: Gd<TileFeature>,
    ) -> Gd<Self> {
        Gd::from_object(TileDescriptor {
            base: crate::TileDescriptor {
                images,
                collision,
                feature: feature.bind().base.clone(),
            },
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct TileFieldDescriptor {
    base: crate::TileFieldDescriptor<feature::Feature>,
}

#[godot_api]
impl TileFieldDescriptor {
    #[func]
    #[inline]
    fn create(
        chunk_size: u32,
        instance_size: u32,
        output_image_size: u32,
        max_page_size: u32,
        tiles: Array<Gd<TileDescriptor>>,
        shaders: Array<Gd<godot::classes::Shader>>,
        world: Gd<godot::classes::World3D>,
    ) -> Gd<Self> {
        let tiles = tiles.iter_shared().map(|v| v.bind().base.clone()).collect();
        Gd::from_object(TileFieldDescriptor {
            base: crate::TileFieldDescriptor {
                chunk_size,
                instance_size,
                output_image_size,
                max_page_size,
                tiles,
                shaders,
                world,
            },
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct BlockDescriptor {
    base: crate::BlockDescriptor<feature::Feature>,
}

#[godot_api]
impl BlockDescriptor {
    #[func]
    #[inline]
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
            base: crate::BlockDescriptor {
                images,
                z_along_y,
                size,
                collision_size,
                collision_offset,
                rendering_size,
                rendering_offset,
                feature: feature.bind().base.clone(),
            },
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct BlockFieldDescriptor {
    base: crate::BlockFieldDescriptor<feature::Feature>,
}

#[godot_api]
impl BlockFieldDescriptor {
    #[func]
    #[inline]
    fn create(
        chunk_size: u32,
        instance_size: u32,
        output_image_size: u32,
        max_page_size: u32,
        blocks: Array<Gd<BlockDescriptor>>,
        shaders: Array<Gd<godot::classes::Shader>>,
        world: Gd<godot::classes::World3D>,
    ) -> Gd<Self> {
        let blocks = blocks
            .iter_shared()
            .map(|v| v.bind().base.clone())
            .collect();
        Gd::from_object(BlockFieldDescriptor {
            base: crate::BlockFieldDescriptor {
                chunk_size,
                instance_size,
                output_image_size,
                max_page_size,
                blocks,
                shaders,
                world,
            },
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct EntityDescriptor {
    base: crate::EntityDescriptor<feature::Feature>,
}

#[godot_api]
impl EntityDescriptor {
    #[func]
    #[inline]
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
            base: crate::EntityDescriptor {
                images,
                z_along_y,
                collision_size,
                collision_offset,
                rendering_size,
                rendering_offset,
                feature: feature.bind().base.clone(),
            },
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct EntityFieldDescriptor {
    base: crate::EntityFieldDescriptor<feature::Feature>,
}

#[godot_api]
impl EntityFieldDescriptor {
    #[func]
    #[inline]
    fn create(
        chunk_size: u32,
        instance_size: u32,
        output_image_size: u32,
        max_page_size: u32,
        entities: Array<Gd<EntityDescriptor>>,
        shaders: Array<Gd<godot::classes::Shader>>,
        world: Gd<godot::classes::World3D>,
    ) -> Gd<Self> {
        let entities = entities
            .iter_shared()
            .map(|v| v.bind().base.clone())
            .collect();
        Gd::from_object(EntityFieldDescriptor {
            base: crate::EntityFieldDescriptor {
                chunk_size,
                instance_size,
                output_image_size,
                max_page_size,
                entities,
                shaders,
                world,
            },
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct RootDescriptor {
    base: crate::RootDescriptor<feature::Feature>,
}

#[godot_api]
impl RootDescriptor {
    #[func]
    #[inline]
    fn create(
        tile_field: Gd<TileFieldDescriptor>,
        block_field: Gd<BlockFieldDescriptor>,
        entity_field: Gd<EntityFieldDescriptor>,
    ) -> Gd<Self> {
        Gd::from_object(RootDescriptor {
            base: crate::RootDescriptor {
                tile_field: tile_field.bind().base.clone(),
                block_field: block_field.bind().base.clone(),
                entity_field: entity_field.bind().base.clone(),
            },
        })
    }
}

// derive root

#[derive(GodotClass)]
#[class(no_init)]
struct Root {
    base: crate::Root<feature::Feature>,
}

#[godot_api]
impl Root {
    #[func]
    #[inline]
    fn create(desc: Gd<RootDescriptor>) -> Gd<Self> {
        let desc = &desc.bind().base;
        Gd::from_object(Root {
            base: crate::Root::create(desc),
        })
    }

    #[func]
    #[inline]
    fn forward(&mut self, min_rect: Rect2) {
        self.base.forward(min_rect);
    }

    // tile

    #[func]
    #[inline]
    fn tile_insert(&mut self, tile: Gd<Tile>) -> Gd<crate::TileKey> {
        let key = self.base.tile_insert(&tile.bind().base);
        Gd::from_object(key)
    }

    #[func]
    #[inline]
    fn tile_remove(&mut self, key: Gd<crate::TileKey>) -> Gd<Tile> {
        let tile = self.base.tile_remove(key);
        Gd::from_object(Tile { base: tile })
    }

    #[func]
    #[inline]
    fn tile_get(&self, key: Gd<crate::TileKey>) -> Gd<Tile> {
        let tile = self.base.tile_get(key);
        Gd::from_object(Tile { base: tile })
    }

    // block

    #[func]
    #[inline]
    fn block_insert(&mut self, block: Gd<Block>) -> Gd<crate::BlockKey> {
        let key = self.base.block_insert(&block.bind().base);
        Gd::from_object(key)
    }

    #[func]
    #[inline]
    fn block_remove(&mut self, key: Gd<crate::BlockKey>) -> Gd<Block> {
        let block = self.base.block_remove(key);
        Gd::from_object(Block { base: block })
    }

    #[func]
    #[inline]
    fn block_get(&self, key: Gd<crate::BlockKey>) -> Gd<Block> {
        let block = self.base.block_get(key);
        Gd::from_object(Block { base: block })
    }

    // entity

    #[func]
    #[inline]
    fn entity_insert(&mut self, entity: Gd<Entity>) -> Gd<crate::EntityKey> {
        let key = self.base.entity_insert(&entity.bind().base);
        Gd::from_object(key)
    }

    #[func]
    #[inline]
    fn entity_remove(&mut self, key: Gd<crate::EntityKey>) -> Gd<Entity> {
        let entity = self.base.entity_remove(key);
        Gd::from_object(Entity { base: entity })
    }

    #[func]
    #[inline]
    fn entity_get(&self, key: Gd<crate::EntityKey>) -> Gd<Entity> {
        let entity = self.base.entity_get(key);
        Gd::from_object(Entity { base: entity })
    }

    // view

    #[func]
    #[inline]
    fn update_view(&mut self, min_rect: Rect2) {
        self.base.update_view(min_rect);
    }

    // extra

    #[func]
    #[inline]
    fn init_generator(&mut self, generator: Gd<Generator>) {
        let resource = generator.bind().base.clone();
        self.base.base.resource_insert(resource);
    }

    #[func]
    #[inline]
    fn generate_chunk(&mut self, min_rect: Rect2) {
        #[rustfmt::skip]
        let min_rect = [[
            min_rect.position.x,
            min_rect.position.y, ], [
            min_rect.position.x + min_rect.size.x,
            min_rect.position.y + min_rect.size.y,
        ]];

        generator::Generator::generate_chunk(&mut self.base.base, min_rect);
    }
}

// extra

#[derive(GodotClass)]
#[class(no_init)]
struct GeneratorRuleDescriptor {
    base: generator::GeneratorRuleDescriptor,
}

#[godot_api]
impl GeneratorRuleDescriptor {
    #[func]
    #[inline]
    fn create_marching(prob: f32, id: u32) -> Gd<Self> {
        Gd::from_object(GeneratorRuleDescriptor {
            base: generator::GeneratorRuleDescriptor::Marching(
                generator::GeneratorRuleMarchingDescriptor { prob, id },
            ),
        })
    }

    #[func]
    #[inline]
    fn create_spawn(prob: f32, id: u32) -> Gd<Self> {
        Gd::from_object(GeneratorRuleDescriptor {
            base: generator::GeneratorRuleDescriptor::Spawn(
                generator::GeneratorRuleSpawnDescriptor { prob, id },
            ),
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct GeneratorDescriptor {
    base: generator::GeneratorDescriptor,
}

#[godot_api]
impl GeneratorDescriptor {
    #[func]
    #[inline]
    fn create(
        chunk_size: u32,
        tile_rule_descriptors: Array<Gd<GeneratorRuleDescriptor>>,
        block_rule_descriptors: Array<Gd<GeneratorRuleDescriptor>>,
        entity_rule_descriptors: Array<Gd<GeneratorRuleDescriptor>>,
    ) -> Gd<Self> {
        let mut tile_rules = vec![];
        for rule in tile_rule_descriptors.iter_shared() {
            tile_rules.push(rule.bind().base.clone())
        }

        let mut block_rules = vec![];
        for rule in block_rule_descriptors.iter_shared() {
            block_rules.push(rule.bind().base.clone())
        }

        let mut entity_rules = vec![];
        for rule in entity_rule_descriptors.iter_shared() {
            entity_rules.push(rule.bind().base.clone())
        }

        Gd::from_object(GeneratorDescriptor {
            base: generator::GeneratorDescriptor {
                chunk_size,
                tile_rules,
                block_rules,
                entity_rules,
            },
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct Generator {
    base: generator::Generator,
}

#[godot_api]
impl Generator {
    #[func]
    #[inline]
    fn create(desc: Gd<GeneratorDescriptor>) -> Gd<Self> {
        Gd::from_object(Generator {
            base: generator::Generator::new(desc.bind().base.clone()),
        })
    }
}
