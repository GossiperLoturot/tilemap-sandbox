use godot::prelude::*;

use crate::inner;

#[derive(GodotClass)]
#[class(no_init)]
struct EntityDescriptor {
    #[export]
    images: Array<Gd<godot::engine::Image>>,
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
impl EntityDescriptor {
    #[func]
    fn new_from(
        images: Array<Gd<godot::engine::Image>>,
        z_along_y: bool,
        rendering_size: Vector2,
        rendering_offset: Vector2,
        collision_size: Vector2,
        collision_offset: Vector2,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            images,
            z_along_y,
            rendering_size,
            rendering_offset,
            collision_size,
            collision_offset,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct EntityFieldDescriptor {
    #[export]
    chunk_size: u32,
    #[export]
    instance_size: u32,
    #[export]
    output_image_size: u32,
    #[export]
    max_page_size: u32,
    #[export]
    entries: Array<Gd<EntityDescriptor>>,
    #[export]
    shader: Gd<godot::engine::Shader>,
    #[export]
    world: Gd<godot::engine::World3D>,
}

#[godot_api]
impl EntityFieldDescriptor {
    #[func]
    fn new_from(
        chunk_size: u32,
        instance_size: u32,
        output_image_size: u32,
        max_page_size: u32,
        entries: Array<Gd<EntityDescriptor>>,
        shader: Gd<godot::engine::Shader>,
        world: Gd<godot::engine::World3D>,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            chunk_size,
            instance_size,
            output_image_size,
            max_page_size,
            entries,
            shader,
            world,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct Entity {
    inner: inner::Entity,
}

impl Entity {
    #[inline]
    pub fn new(value: inner::Entity) -> Self {
        Self { inner: value }
    }

    #[inline]
    pub fn inner_ref(&self) -> &inner::Entity {
        &self.inner
    }

    #[inline]
    pub fn inner_mut(&mut self) -> &mut inner::Entity {
        &mut self.inner
    }

    #[inline]
    pub fn inner(self) -> inner::Entity {
        self.inner
    }
}

#[godot_api]
impl Entity {
    #[func]
    fn new_from(id: u32, location: Vector2, variant: u8) -> Gd<Self> {
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

impl EntityChunkDown {
    fn up(self) -> EntityChunkUp {
        EntityChunkUp {
            serial: Default::default(),
            material: self.material,
            multimesh: self.multimesh,
            instance: self.instance,
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

impl EntityChunkUp {
    fn down(self) -> EntityChunkDown {
        EntityChunkDown {
            material: self.material,
            multimesh: self.multimesh,
            instance: self.instance,
        }
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub(crate) struct EntityField {
    inner: inner::EntityField,
    specs: Vec<EntitySpec>,
    texcoords: Vec<Vec<image_atlas::Texcoord32>>,
    down_chunks: Vec<EntityChunkDown>,
    up_chunks: ahash::AHashMap<inner::IVec2, EntityChunkUp>,
    min_view_rect: Option<[[i32; 2]; 2]>,
}

// pass the inner reference for `Root`
impl EntityField {
    #[inline]
    pub fn inner_ref(&self) -> &inner::EntityField {
        &self.inner
    }

    #[inline]
    pub fn inner_mut(&mut self) -> &mut inner::EntityField {
        &mut self.inner
    }
}

#[godot_api]
impl EntityField {
    #[constant]
    const MAX_BUFFER_SIZE: u32 = 1024;

    #[func]
    fn new_from(desc: Gd<EntityFieldDescriptor>) -> Gd<Self> {
        let desc = desc.bind();

        let specs = desc
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

        let inner = inner::EntityField::new(desc.chunk_size, specs);

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
            .flat_map(|entry| {
                let entry = entry.bind();
                entry.images.iter_shared().collect::<Vec<_>>()
            })
            .map(|image| {
                let width = image.get_width() as u32;
                let height = image.get_height() as u32;

                let mut image_rgba8 = image::RgbaImage::new(width, height);
                for y in 0..height {
                    for x in 0..width {
                        let color = image.get_pixel(x as i32, y as i32);
                        let rgba8 = image::Rgba([color.r8(), color.g8(), color.b8(), color.a8()]);
                        image_rgba8.put_pixel(x, y, rgba8);
                    }
                }

                image_atlas::AtlasEntry {
                    texture: image_rgba8,
                    mip: image_atlas::AtlasEntryMipOption::Clamp,
                }
            })
            .collect::<Vec<_>>();

        let variants = desc
            .entries
            .iter_shared()
            .map(|entry| {
                let entry = entry.bind();
                entry.images.len() as u8
            })
            .collect::<Vec<_>>();

        let atlas = image_atlas::create_atlas(&image_atlas::AtlasDescriptor {
            size: desc.output_image_size,
            max_page_count: desc.max_page_size,
            mip: image_atlas::AtlasMipOption::NoMipWithPadding(1),
            entries: &entries,
        })
        .unwrap();

        let images = atlas
            .textures
            .into_iter()
            .map(|texture| {
                let data = texture.mip_maps[0].to_vec();

                godot::engine::Image::create_from_data(
                    desc.output_image_size as i32,
                    desc.output_image_size as i32,
                    false,
                    godot::engine::image::Format::RGBA8,
                    PackedByteArray::from(data.as_slice()),
                )
                .unwrap()
            })
            .collect::<Vec<_>>();

        let mut rendering_server = godot::engine::RenderingServer::singleton();
        let texture_array = rendering_server.texture_2d_layered_create(
            Array::from(images.as_slice()),
            godot::engine::rendering_server::TextureLayeredType::LAYERED_2D_ARRAY,
        );

        let mut iter = atlas.texcoords.into_iter();
        let texcoords = variants
            .into_iter()
            .map(|variant| {
                (0..variant)
                    .map(|_| iter.next().unwrap().to_f32())
                    .collect::<Vec<_>>()
            })
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

        let down_chunks = (0..desc.instance_size)
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

                let instance =
                    rendering_server.instance_create2(multimesh, desc.world.get_scenario());
                rendering_server.instance_set_visible(instance, false);

                EntityChunkDown {
                    material,
                    multimesh,
                    instance,
                }
            })
            .collect::<Vec<_>>();

        Gd::from_object(Self {
            inner,
            specs,
            texcoords,
            down_chunks,
            up_chunks: Default::default(),
            min_view_rect: None,
        })
    }

    #[func]
    fn get(&self, entity_key: u32) -> Gd<Entity> {
        let entity = self.inner.get(entity_key).unwrap().clone();
        Gd::from_object(Entity { inner: entity })
    }

    // rendering features

    #[func]
    fn update_view(&mut self, min_view_rect: Rect2) {
        let chunk_size = self.inner.get_chunk_size() as f32;

        #[rustfmt::skip]
        let min_view_rect = [[
            min_view_rect.position.x.div_euclid(chunk_size) as i32,
            min_view_rect.position.y.div_euclid(chunk_size) as i32, ], [
            (min_view_rect.position.x + min_view_rect.size.x).div_euclid(chunk_size) as i32,
            (min_view_rect.position.y + min_view_rect.size.y).div_euclid(chunk_size) as i32,
        ]];

        // remove/insert view chunk
        if Some(min_view_rect) != self.min_view_rect {
            let chunk_keys = self
                .up_chunks
                .iter()
                .filter_map(|(&chunk_key, _)| {
                    let is_out_of_range_x =
                        chunk_key[0] < min_view_rect[0][0] || min_view_rect[1][0] < chunk_key[0];
                    let is_out_of_range_y =
                        chunk_key[1] < min_view_rect[0][1] || min_view_rect[1][1] < chunk_key[1];
                    (is_out_of_range_x || is_out_of_range_y).then_some(chunk_key)
                })
                .collect::<Vec<_>>();

            chunk_keys.into_iter().for_each(|chunk_key| {
                let up_chunk = self.up_chunks.remove(&chunk_key).unwrap();

                let mut rendering_server = godot::engine::RenderingServer::singleton();
                rendering_server.instance_set_visible(up_chunk.instance, false);

                self.down_chunks.push(up_chunk.down());
            });

            for y in min_view_rect[0][1]..=min_view_rect[1][1] {
                for x in min_view_rect[0][0]..=min_view_rect[1][0] {
                    let chunk_key = [x, y];

                    if self.up_chunks.contains_key(&chunk_key) {
                        continue;
                    }

                    let Some(down_chunk) = self.down_chunks.pop() else {
                        let up = self.up_chunks.len();
                        let down = self.down_chunks.len();
                        panic!("no chunk available in pool (up:{}, down:{})", up, down);
                    };

                    let mut rendering_server = godot::engine::RenderingServer::singleton();
                    rendering_server.instance_set_visible(down_chunk.instance, true);

                    self.up_chunks.insert(chunk_key, down_chunk.up());
                }
            }

            self.min_view_rect = Some(min_view_rect);
        }

        // update view chunk

        for (chunk_key, up_chunk) in &mut self.up_chunks {
            let Some(chunk) = self.inner.get_chunk(*chunk_key) else {
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

                let texcoord = &self.texcoords[entity.id as usize];
                let texcoord = &texcoord[usize::min(entity.variant as usize, texcoord.len() - 1)];
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
    fn get_collision_rect(&self, entity_key: u32) -> Rect2 {
        let rect = self.inner.get_collision_rect(entity_key).unwrap();
        Rect2::from_corners(
            Vector2::new(rect[0][0], rect[0][1]),
            Vector2::new(rect[1][0], rect[1][1]),
        )
    }

    #[func]
    fn has_collision_by_point(&self, point: Vector2) -> bool {
        let point = [point.x, point.y];
        self.inner.has_collision_by_point(point)
    }

    #[func]
    fn get_collision_by_point(&self, point: Vector2) -> Array<u32> {
        let point = [point.x, point.y];
        let entity_keys = self.inner.get_collision_by_point(point);
        Array::from_iter(entity_keys)
    }

    #[func]
    fn has_collision_by_rect(&self, rect: Rect2) -> bool {
        let p0 = [rect.position.x, rect.position.y];
        let p1 = [rect.position.x + rect.size.x, rect.position.y + rect.size.y];
        self.inner.has_collision_by_rect([p0, p1])
    }

    #[func]
    fn get_collision_by_rect(&self, rect: Rect2) -> Array<u32> {
        let p0 = [rect.position.x, rect.position.y];
        let p1 = [rect.position.x + rect.size.x, rect.position.y + rect.size.y];
        let entity_keys = self.inner.get_collision_by_rect([p0, p1]);
        Array::from_iter(entity_keys)
    }

    // hint features

    #[func]
    fn get_hint_rect(&self, entity_key: u32) -> Rect2 {
        let rect = self.inner.get_hint_rect(entity_key).unwrap();
        Rect2::from_corners(
            Vector2::new(rect[0][0], rect[0][1]),
            Vector2::new(rect[1][0], rect[1][1]),
        )
    }

    #[func]
    fn has_hint_by_point(&self, point: Vector2) -> bool {
        let point = [point.x, point.y];
        self.inner.has_hint_by_point(point)
    }

    #[func]
    fn get_hint_by_point(&self, point: Vector2) -> Array<u32> {
        let point = [point.x, point.y];
        let entity_keys = self.inner.get_hint_by_point(point);
        Array::from_iter(entity_keys)
    }

    #[func]
    fn has_hint_by_rect(&self, rect: Rect2) -> bool {
        let p0 = [rect.position.x, rect.position.y];
        let p1 = [rect.position.x + rect.size.x, rect.position.y + rect.size.y];
        self.inner.has_hint_by_rect([p0, p1])
    }

    #[func]
    fn get_hint_by_rect(&self, rect: Rect2) -> Array<u32> {
        let p0 = [rect.position.x, rect.position.y];
        let p1 = [rect.position.x + rect.size.x, rect.position.y + rect.size.y];
        let entity_keys = self.inner.get_hint_by_rect([p0, p1]);
        Array::from_iter(entity_keys)
    }
}
