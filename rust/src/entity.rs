use crate::inner;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct EntityFieldDescEntry {
    #[export]
    image: Gd<godot::engine::Image>,
    #[export]
    z_along_y: bool,
    #[export]
    rendering_size: Vector2,
    #[export]
    rendering_offset: Vector2,
    #[export]
    collision_size: Vector2,
    #[export]
    collision_offset: Vector2,
}

#[godot_api]
impl EntityFieldDescEntry {
    #[func]
    fn new_from(
        image: Gd<godot::engine::Image>,
        z_along_y: bool,
        rendering_size: Vector2,
        rendering_offset: Vector2,
        collision_size: Vector2,
        collision_offset: Vector2,
    ) -> Gd<Self> {
        Gd::from_init_fn(|_| Self {
            image,
            z_along_y,
            rendering_size,
            rendering_offset,
            collision_size,
            collision_offset,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct EntityFieldDesc {
    #[export]
    output_image_size: u32,
    #[export]
    max_page_size: u32,
    #[export]
    entries: Array<Gd<EntityFieldDescEntry>>,
    #[export]
    shader: Gd<godot::engine::Shader>,
}

#[godot_api]
impl EntityFieldDesc {
    #[func]
    fn new_from(
        output_image_size: u32,
        max_page_size: u32,
        entries: Array<Gd<EntityFieldDescEntry>>,
        shader: Gd<godot::engine::Shader>,
    ) -> Gd<Self> {
        Gd::from_init_fn(|_| Self {
            output_image_size,
            max_page_size,
            entries,
            shader,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct EntityKey {
    pub inner: u32,
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct Entity {
    pub inner: inner::Entity,
}

#[godot_api]
impl Entity {
    #[func]
    fn new_from(id: u32, location: Vector2) -> Gd<Self> {
        let location = [location.x, location.y];
        let inner = inner::Entity::new(id, location);
        Gd::from_init_fn(|_| Self { inner })
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
}

#[derive(Debug, Clone)]
struct EntitySpec {
    z_along_y: bool,
    rendering_size: inner::Vec2,
    rendering_offset: inner::Vec2,
}

#[derive(Debug, Clone)]
struct EntityChunkDown {
    material: Rid,
    multimesh: Rid,
    instance: Rid,
}

impl From<EntityChunkUp> for EntityChunkDown {
    fn from(chunk: EntityChunkUp) -> Self {
        Self {
            material: chunk.material,
            multimesh: chunk.multimesh,
            instance: chunk.instance,
        }
    }
}

#[derive(Debug, Clone)]
struct EntityChunkUp {
    serial: u64,
    material: Rid,
    multimesh: Rid,
    instance: Rid,
}

impl From<EntityChunkDown> for EntityChunkUp {
    fn from(chunk: EntityChunkDown) -> Self {
        Self {
            serial: Default::default(),
            material: chunk.material,
            multimesh: chunk.multimesh,
            instance: chunk.instance,
        }
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct EntityField {
    pub inner: inner::EntityField,
    specs: Vec<EntitySpec>,
    texcoords: Vec<image_atlas::Texcoord32>,
    down_chunks: Vec<EntityChunkDown>,
    up_chunks: ahash::AHashMap<inner::IVec2, EntityChunkUp>,
}

#[godot_api]
impl EntityField {
    #[constant]
    const MAX_INSTANCE_SIZE: u32 = 256;

    #[constant]
    const MAX_BUFFER_SIZE: u32 = 1024;

    #[constant]
    const CHUNK_SIZE: u32 = 32;

    #[func]
    fn new_from(desc: Gd<EntityFieldDesc>, world: Gd<godot::engine::World3D>) -> Gd<Self> {
        let desc = desc.bind();

        let inner_specs = desc
            .entries
            .iter_shared()
            .map(|entry| {
                let entry = entry.bind();
                inner::EntitySpec::new(
                    [entry.collision_size.x, entry.collision_size.y],
                    [entry.collision_offset.x, entry.collision_offset.y],
                    [entry.rendering_size.x, entry.rendering_size.y],
                    [entry.rendering_offset.x, entry.rendering_offset.y],
                )
            })
            .collect::<Vec<_>>();

        let inner = inner::EntityField::new(Self::CHUNK_SIZE, inner_specs);

        let specs = desc
            .entries
            .iter_shared()
            .map(|entry| {
                let entry = entry.bind();
                EntitySpec {
                    z_along_y: entry.z_along_y,
                    rendering_size: [entry.rendering_size.x, entry.rendering_size.y],
                    rendering_offset: [entry.rendering_offset.x, entry.rendering_offset.y],
                }
            })
            .collect::<Vec<_>>();

        let entries = desc
            .entries
            .iter_shared()
            .map(|entry| {
                let gd_image = &entry.bind().image;

                let width = gd_image.get_width() as u32;
                let height = gd_image.get_height() as u32;

                let mut image = image::RgbaImage::new(width, height);
                for y in 0..height {
                    for x in 0..width {
                        let rgba = gd_image.get_pixel(x as i32, y as i32);
                        let rgba = image::Rgba([rgba.r8(), rgba.g8(), rgba.b8(), rgba.a8()]);
                        image.put_pixel(x, y, rgba);
                    }
                }

                image
            })
            .map(|image| image_atlas::AtlasEntry {
                texture: image,
                mip: image_atlas::AtlasEntryMipOption::Clamp,
            })
            .collect::<Vec<_>>();

        let atlas = image_atlas::create_atlas(&image_atlas::AtlasDescriptor {
            size: desc.output_image_size,
            max_page_count: desc.max_page_size,
            mip: image_atlas::AtlasMipOption::NoMipWithPadding(1),
            entries: &entries,
        })
        .unwrap();

        let gd_images = atlas
            .textures
            .into_iter()
            .map(|texture| texture.mip_maps.into_iter().next().unwrap())
            .map(|image| {
                godot::engine::Image::create_from_data(
                    image.width() as i32,
                    image.height() as i32,
                    false,
                    godot::engine::image::Format::RGBA8,
                    PackedByteArray::from(image.to_vec().as_slice()),
                )
                .unwrap()
            })
            .collect::<Vec<_>>();

        let mut rendering_server = godot::engine::RenderingServer::singleton();
        let texture_array = rendering_server.texture_2d_layered_create(
            Array::from(gd_images.as_slice()),
            godot::engine::rendering_server::TextureLayeredType::LAYERED_2D_ARRAY,
        );

        let texcoords = atlas
            .texcoords
            .into_iter()
            .map(|texcoord| texcoord.to_f32())
            .collect::<Vec<_>>();

        let shader = desc.shader.get_rid();
        let mesh_data = {
            let mut data = VariantArray::new();
            data.resize(
                godot::engine::rendering_server::ArrayType::MAX.ord() as usize,
                &Variant::nil(),
            );
            data.set(
                godot::engine::rendering_server::ArrayType::VERTEX.ord() as usize,
                PackedVector3Array::from(&[
                    Vector3::new(0.0, 0.0, 0.0),
                    Vector3::new(0.0, 1.0, 1.0),
                    Vector3::new(1.0, 1.0, 1.0),
                    Vector3::new(1.0, 0.0, 0.0),
                ])
                .to_variant(),
            );
            data.set(
                godot::engine::rendering_server::ArrayType::TEX_UV.ord() as usize,
                PackedVector2Array::from(&[
                    Vector2::new(0.0, 1.0),
                    Vector2::new(0.0, 0.0),
                    Vector2::new(1.0, 0.0),
                    Vector2::new(1.0, 1.0),
                ])
                .to_variant(),
            );
            data.set(
                godot::engine::rendering_server::ArrayType::INDEX.ord() as usize,
                PackedInt32Array::from(&[0, 1, 2, 0, 2, 3]).to_variant(),
            );
            data
        };

        let down_chunks = (0..Self::MAX_INSTANCE_SIZE)
            .map(|_| {
                let material = rendering_server.material_create();
                rendering_server.material_set_shader(material, shader);
                rendering_server.material_set_param(
                    material,
                    "texture_array".into(),
                    texture_array.to_variant(),
                );

                let mesh = rendering_server.mesh_create();
                rendering_server.mesh_add_surface_from_arrays(
                    mesh,
                    godot::engine::rendering_server::PrimitiveType::TRIANGLES,
                    mesh_data.clone(),
                );
                rendering_server.mesh_surface_set_material(mesh, 0, material);

                let multimesh = rendering_server.multimesh_create();
                rendering_server.multimesh_set_mesh(multimesh, mesh);
                rendering_server.multimesh_allocate_data(
                    multimesh,
                    Self::MAX_BUFFER_SIZE as i32,
                    godot::engine::rendering_server::MultimeshTransformFormat::TRANSFORM_3D,
                );

                let instance = rendering_server.instance_create2(multimesh, world.get_scenario());
                rendering_server.instance_set_visible(instance, false);

                EntityChunkDown {
                    material,
                    multimesh,
                    instance,
                }
            })
            .collect::<Vec<_>>();

        Gd::from_init_fn(|_| Self {
            inner,
            specs,
            texcoords,
            down_chunks,
            up_chunks: Default::default(),
        })
    }

    #[func]
    fn insert(&mut self, entity: Gd<Entity>) -> Option<Gd<EntityKey>> {
        let entity = entity.bind().inner.clone();
        let key = self.inner.insert(entity).ok()?;
        Some(Gd::from_init_fn(|_| EntityKey { inner: key }))
    }

    #[func]
    fn remove(&mut self, key: Gd<EntityKey>) -> Option<Gd<Entity>> {
        let key = key.bind().inner;
        let entity = self.inner.remove(key).ok()?;
        Some(Gd::from_init_fn(|_| Entity { inner: entity }))
    }

    #[func]
    fn modify(&mut self, key: Gd<EntityKey>, new_entity: Gd<Entity>) -> Option<Gd<Entity>> {
        let key = key.bind().inner;
        let new_entity = new_entity.bind().inner.clone();
        let entity = self.inner.modify(key, new_entity).ok()?;
        Some(Gd::from_init_fn(|_| Entity { inner: entity }))
    }

    #[func]
    fn get(&self, key: Gd<EntityKey>) -> Option<Gd<Entity>> {
        let key = key.bind().inner;
        let entity = self.inner.get(key).ok()?.clone();
        Some(Gd::from_init_fn(|_| Entity { inner: entity }))
    }

    // rendering features

    #[func]
    fn insert_view(&mut self, key: Vector2i) -> bool {
        let key = [key.x, key.y];
        if self.up_chunks.contains_key(&key) {
            return false;
        }

        let Some(chunk) = self.down_chunks.pop() else {
            return false;
        };

        let mut rendering_server = godot::engine::RenderingServer::singleton();
        rendering_server.instance_set_visible(chunk.instance, true);

        self.up_chunks.insert(key, chunk.into());

        true
    }

    #[func]
    fn remove_view(&mut self, key: Vector2i) -> bool {
        let key = [key.x, key.y];
        let Some(chunk) = self.up_chunks.remove(&key) else {
            return false;
        };

        let mut rendering_server = godot::engine::RenderingServer::singleton();
        rendering_server.instance_set_visible(chunk.instance, false);

        self.down_chunks.push(chunk.into());

        true
    }

    #[func]
    fn update_view(&mut self) {
        for (key, up_chunk) in &mut self.up_chunks {
            let Some(chunk) = self.inner.get_chunk(*key) else {
                continue;
            };

            if chunk.serial <= up_chunk.serial {
                continue;
            }

            let mut instance_buffer = [0.0; Self::MAX_BUFFER_SIZE as usize * 12];
            let mut texcoord_buffer = [0.0; Self::MAX_BUFFER_SIZE as usize * 4];
            let mut page_buffer = [0.0; Self::MAX_BUFFER_SIZE as usize];

            for (i, entity) in chunk
                .entities
                .iter()
                .map(|(_, entity)| entity)
                .take(Self::MAX_BUFFER_SIZE as usize)
                .enumerate()
            {
                let spec = &self.specs[entity.id as usize];
                instance_buffer[i * 12] = spec.rendering_size[0];
                instance_buffer[i * 12 + 1] = 0.0;
                instance_buffer[i * 12 + 2] = 0.0;
                instance_buffer[i * 12 + 3] = entity.location[0] + spec.rendering_offset[0];

                instance_buffer[i * 12 + 4] = 0.0;
                instance_buffer[i * 12 + 5] = spec.rendering_size[1];
                instance_buffer[i * 12 + 6] = 0.0;
                instance_buffer[i * 12 + 7] = entity.location[1] + spec.rendering_offset[1];

                let z_scale = spec.rendering_size[1] * if spec.z_along_y { 1.0 } else { 0.0 };
                instance_buffer[i * 12 + 8] = 0.0;
                instance_buffer[i * 12 + 9] = 0.0;
                instance_buffer[i * 12 + 10] = z_scale;
                instance_buffer[i * 12 + 11] = 0.0;

                let texcoord = self.texcoords[entity.id as usize];
                texcoord_buffer[i * 4] = texcoord.min_x;
                texcoord_buffer[i * 4 + 1] = texcoord.min_y;
                texcoord_buffer[i * 4 + 2] = texcoord.max_x - texcoord.min_x;
                texcoord_buffer[i * 4 + 3] = texcoord.max_y - texcoord.min_y;

                page_buffer[i] = texcoord.page as f32;
            }

            let mut rendering_server = godot::engine::RenderingServer::singleton();
            rendering_server.multimesh_set_buffer(
                up_chunk.multimesh,
                PackedFloat32Array::from(instance_buffer.as_slice()),
            );
            rendering_server.material_set_param(
                up_chunk.material,
                "texcoord_buffer".into(),
                PackedFloat32Array::from(texcoord_buffer.as_slice()).to_variant(),
            );
            rendering_server.material_set_param(
                up_chunk.material,
                "page_buffer".into(),
                PackedFloat32Array::from(page_buffer.as_slice()).to_variant(),
            );

            up_chunk.serial = chunk.serial;
        }
    }

    // collision features

    #[func]
    fn has_collision_by_point(&self, point: Vector2) -> bool {
        let point = [point.x, point.y];
        self.inner.has_collision_by_point(point)
    }

    #[func]
    fn get_collision_by_point(&self, point: Vector2) -> Array<Gd<EntityKey>> {
        let point = [point.x, point.y];
        let keys = self.inner.get_collision_by_point(point);
        Array::from_iter(keys.map(|key| Gd::from_init_fn(|_| EntityKey { inner: key })))
    }

    #[func]
    fn has_collision_by_rect(&self, rect: Rect2) -> bool {
        let p0 = [rect.position.x, rect.position.y];
        let p1 = [rect.position.x + rect.size.x, rect.position.y + rect.size.y];
        self.inner.has_collision_by_rect([p0, p1])
    }

    #[func]
    fn get_collision_by_rect(&self, rect: Rect2) -> Array<Gd<EntityKey>> {
        let p0 = [rect.position.x, rect.position.y];
        let p1 = [rect.position.x + rect.size.x, rect.position.y + rect.size.y];
        let keys = self.inner.get_collision_by_rect([p0, p1]);
        Array::from_iter(keys.map(|key| Gd::from_init_fn(|_| EntityKey { inner: key })))
    }

    // hint features

    #[func]
    fn has_hint_by_point(&self, point: Vector2) -> bool {
        let point = [point.x, point.y];
        self.inner.has_hint_by_point(point)
    }

    #[func]
    fn get_hint_by_point(&self, point: Vector2) -> Array<Gd<EntityKey>> {
        let point = [point.x, point.y];
        let keys = self.inner.get_hint_by_point(point);
        Array::from_iter(keys.map(|key| Gd::from_init_fn(|_| EntityKey { inner: key })))
    }

    #[func]
    fn has_hint_by_rect(&self, rect: Rect2) -> bool {
        let p0 = [rect.position.x, rect.position.y];
        let p1 = [rect.position.x + rect.size.x, rect.position.y + rect.size.y];
        self.inner.has_hint_by_rect([p0, p1])
    }

    #[func]
    fn get_hint_by_rect(&self, rect: Rect2) -> Array<Gd<EntityKey>> {
        let p0 = [rect.position.x, rect.position.y];
        let p1 = [rect.position.x + rect.size.x, rect.position.y + rect.size.y];
        let keys = self.inner.get_hint_by_rect([p0, p1]);
        Array::from_iter(keys.map(|key| Gd::from_init_fn(|_| EntityKey { inner: key })))
    }
}
