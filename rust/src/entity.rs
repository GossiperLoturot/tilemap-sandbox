use crate::inner;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Resource)]
struct EntityFieldDescEntry {
    #[export]
    image: Option<Gd<godot::engine::Image>>,
    #[export]
    #[init(default = Vector2::new(1.0, 1.0))]
    render_size: Vector2,
    #[export]
    render_offset: Vector2,
    #[export]
    physics_size: Vector2,
    #[export]
    physics_offset: Vector2,
    #[export]
    z_along_y: bool,
}

#[derive(GodotClass)]
#[class(init, base=Resource)]
struct EntityFieldDesc {
    #[export]
    #[init(default = 2048)]
    output_image_size: u32,
    #[export]
    #[init(default = 64)]
    max_page_size: u32,
    #[export]
    entries: Array<Gd<EntityFieldDescEntry>>,
    #[export]
    shader: Option<Gd<godot::engine::Shader>>,
}

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
struct Entity {
    inner: inner::Entity,
}

#[godot_api]
impl Entity {
    #[func]
    fn new_from(id: u32, location: Vector2) -> Gd<Self> {
        let location = (location.x, location.y);
        let inner = inner::Entity { id, location };
        Gd::from_init_fn(|_| Self { inner })
    }

    #[func]
    fn get_id(&self) -> u32 {
        self.inner.id
    }

    #[func]
    fn get_location(&self) -> Vector2 {
        let location = self.inner.location;
        Vector2::new(location.0, location.1)
    }
}

#[derive(Debug, Clone)]
struct EntitySpec {
    render_size: inner::Vec2,
    render_offset: inner::Vec2,
    physics_size: inner::Vec2,
    physics_offset: inner::Vec2,
    z_along_y: bool,
}

#[derive(Debug, Clone)]
struct EntityChunkDown {
    material: Rid,
    multimesh: Rid,
    instance: Rid,
    physics_body: Rid,
}

impl From<EntityChunkUp> for EntityChunkDown {
    fn from(chunk: EntityChunkUp) -> Self {
        Self {
            material: chunk.material,
            multimesh: chunk.multimesh,
            instance: chunk.instance,
            physics_body: chunk.physics_body,
        }
    }
}

#[derive(Debug, Clone)]
struct EntityChunkUp {
    serial: u32,
    material: Rid,
    multimesh: Rid,
    instance: Rid,
    physics_body: Rid,
}

impl From<EntityChunkDown> for EntityChunkUp {
    fn from(chunk: EntityChunkDown) -> Self {
        Self {
            serial: 0,
            material: chunk.material,
            multimesh: chunk.multimesh,
            instance: chunk.instance,
            physics_body: chunk.physics_body,
        }
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct EntityField {
    inner: inner::EntityField,
    specs: Vec<EntitySpec>,
    texcoords: Vec<image_atlas::Texcoord32>,
    down_chunks: Vec<EntityChunkDown>,
    up_chunks: ahash::AHashMap<inner::IVec2, EntityChunkUp>,
    physics_shape: Rid,
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

        let inner = inner::EntityField::new(Self::CHUNK_SIZE);

        let specs = desc
            .entries
            .iter_shared()
            .map(|entry| {
                let entry = entry.bind();
                EntitySpec {
                    render_size: (entry.render_size.x, entry.render_size.y),
                    render_offset: (entry.render_offset.x, entry.render_offset.y),
                    physics_size: (entry.physics_size.x, entry.physics_size.y),
                    physics_offset: (entry.physics_offset.x, entry.physics_offset.y),
                    z_along_y: entry.z_along_y,
                }
            })
            .collect::<Vec<_>>();

        let entries = desc
            .entries
            .iter_shared()
            .map(|entry| {
                let gd_image = &entry.bind().image;
                let gd_image = gd_image.as_ref().unwrap();

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

        let shader = desc.shader.as_ref().unwrap().get_rid();
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

        let mut physics_server = godot::engine::PhysicsServer3D::singleton();
        let physics_shape = physics_server.box_shape_create();
        physics_server.shape_set_data(physics_shape, Vector3::new(0.5, 0.5, 0.5).to_variant());

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

                let physics_body = physics_server.body_create();
                physics_server.body_set_mode(
                    physics_body,
                    godot::engine::physics_server_3d::BodyMode::STATIC,
                );
                physics_server.body_set_space(physics_body, world.get_space());
                physics_server.body_set_state(
                    physics_body,
                    godot::engine::physics_server_3d::BodyState::TRANSFORM,
                    Transform3D::IDENTITY.to_variant(),
                );
                physics_server.body_set_ray_pickable(physics_body, true);

                EntityChunkDown {
                    material,
                    multimesh,
                    instance,
                    physics_body,
                }
            })
            .collect::<Vec<_>>();

        Gd::from_init_fn(|_| Self {
            inner,
            specs,
            texcoords,
            down_chunks,
            up_chunks: Default::default(),
            physics_shape,
        })
    }

    #[func]
    fn insert(&mut self, entity: Gd<Entity>) {
        let entity = entity.bind().inner.clone();
        self.inner.insert(entity);
    }

    #[func]
    fn remove(&mut self, key: u32) {
        self.inner.remove(key);
    }

    #[func]
    fn get(&self, key: u32) -> Option<Gd<Entity>> {
        self.inner
            .get(key)
            .cloned()
            .map(|inner| Gd::from_init_fn(|_| Entity { inner }))
    }

    #[func]
    fn insert_view(&mut self, key: Vector2i) {
        let key = (key.x, key.y);
        if self.up_chunks.contains_key(&key) {
            return;
        }

        let Some(chunk) = self.down_chunks.pop() else {
            return;
        };

        let mut rendering_server = godot::engine::RenderingServer::singleton();
        rendering_server.instance_set_visible(chunk.instance, true);

        self.up_chunks.insert(key, chunk.into());
    }

    #[func]
    fn remove_view(&mut self, key: Vector2i) {
        let key = (key.x, key.y);
        let Some(chunk) = self.up_chunks.remove(&key) else {
            return;
        };

        let mut rendering_server = godot::engine::RenderingServer::singleton();
        rendering_server.instance_set_visible(chunk.instance, false);

        self.down_chunks.push(chunk.into());
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
            let mut physics_buffer = [0.0; Self::MAX_BUFFER_SIZE as usize * 4];

            for (i, entity) in chunk
                .entities
                .iter()
                .map(|(_, entity)| entity)
                .take(Self::MAX_BUFFER_SIZE as usize)
                .enumerate()
            {
                let spec = &self.specs[entity.id as usize];
                instance_buffer[i * 12 + 0] = spec.render_size.0;
                instance_buffer[i * 12 + 1] = 0.0;
                instance_buffer[i * 12 + 2] = 0.0;
                instance_buffer[i * 12 + 3] = entity.location.0 + spec.render_offset.0;

                instance_buffer[i * 12 + 4] = 0.0;
                instance_buffer[i * 12 + 5] = spec.render_size.1;
                instance_buffer[i * 12 + 6] = 0.0;
                instance_buffer[i * 12 + 7] = entity.location.1 + spec.render_offset.1;

                let z_scale = spec.render_size.1 * if spec.z_along_y { 1.0 } else { 0.0 };
                instance_buffer[i * 12 + 8] = 0.0;
                instance_buffer[i * 12 + 9] = 0.0;
                instance_buffer[i * 12 + 10] = z_scale;
                instance_buffer[i * 12 + 11] = 0.0;

                let texcoord = self.texcoords[entity.id as usize];
                texcoord_buffer[i * 4 + 0] = texcoord.min_x;
                texcoord_buffer[i * 4 + 1] = texcoord.min_y;
                texcoord_buffer[i * 4 + 2] = texcoord.max_x - texcoord.min_x;
                texcoord_buffer[i * 4 + 3] = texcoord.max_y - texcoord.min_y;

                page_buffer[i] = texcoord.page as f32;

                physics_buffer[i * 4 + 0] = spec.physics_size.0;
                physics_buffer[i * 4 + 1] = spec.physics_size.1;
                physics_buffer[i * 4 + 2] = entity.location.0 + spec.physics_offset.0;
                physics_buffer[i * 4 + 3] = entity.location.1 + spec.physics_offset.1;
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

            let mut physics_server = godot::engine::PhysicsServer3D::singleton();
            let size = usize::min(chunk.entities.len(), Self::MAX_BUFFER_SIZE as usize);
            for i in 0..size {
                let u = physics_buffer[i * 4 + 0];
                let v = physics_buffer[i * 4 + 1];
                let x = physics_buffer[i * 4 + 2];
                let y = physics_buffer[i * 4 + 3];

                if u * v == 0.0 {
                    continue;
                }

                let transform = Transform3D::IDENTITY
                    .scaled(Vector3::new(u, v, 1.0))
                    .translated(Vector3::new(x + u * 0.5, y + v * 0.5, 0.5));

                physics_server
                    .body_add_shape_ex(up_chunk.physics_body, self.physics_shape)
                    .transform(transform)
                    .done();
            }

            up_chunk.serial = chunk.serial;
        }
    }
}
