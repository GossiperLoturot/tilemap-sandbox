use crate::inner;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Resource)]
struct BlockFieldDescEntry {
    #[export]
    #[init(default = Vector2i::new(1, 1))]
    size: Vector2i,
    #[export]
    image: Option<Gd<godot::engine::Image>>,
    #[export]
    z_along_y: bool,
    #[export]
    #[init(default = Vector2::new(1.0, 1.0))]
    rendering_size: Vector2,
    #[export]
    rendering_offset: Vector2,
    #[export]
    collision_size: Vector2,
    #[export]
    collision_offset: Vector2,
}

#[derive(GodotClass)]
#[class(init, base=Resource)]
struct BlockFieldDesc {
    #[export]
    #[init(default = 2048)]
    output_image_size: u32,
    #[export]
    #[init(default = 64)]
    max_page_size: u32,
    #[export]
    entries: Array<Gd<BlockFieldDescEntry>>,
    #[export]
    shader: Option<Gd<godot::engine::Shader>>,
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct BlockKey {
    inner: inner::BlockKey,
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct Block {
    inner: inner::Block,
}

#[godot_api]
impl Block {
    #[func]
    fn new_from(id: u32, location: Vector2i) -> Gd<Self> {
        let location = [location.x, location.y];
        let inner = inner::Block { id, location };
        Gd::from_init_fn(|_| Self { inner })
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
}

#[derive(Debug, Clone)]
struct BlockSpec {
    z_along_y: bool,
    rendering_size: inner::Vec2,
    rendering_offset: inner::Vec2,
}

#[derive(Debug, Clone)]
struct BlockChunkDown {
    material: Rid,
    multimesh: Rid,
    instance: Rid,
}

impl From<BlockChunkUp> for BlockChunkDown {
    fn from(chunk: BlockChunkUp) -> Self {
        Self {
            material: chunk.material,
            multimesh: chunk.multimesh,
            instance: chunk.instance,
        }
    }
}

#[derive(Debug, Clone)]
struct BlockChunkUp {
    serial: u32,
    material: Rid,
    multimesh: Rid,
    instance: Rid,
}

impl From<BlockChunkDown> for BlockChunkUp {
    fn from(chunk: BlockChunkDown) -> Self {
        Self {
            serial: 0,
            material: chunk.material,
            multimesh: chunk.multimesh,
            instance: chunk.instance,
        }
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct BlockField {
    pub inner: inner::BlockField,
    specs: Vec<BlockSpec>,
    texcoords: Vec<image_atlas::Texcoord32>,
    down_chunks: Vec<BlockChunkDown>,
    up_chunks: ahash::AHashMap<inner::IVec2, BlockChunkUp>,
}

#[godot_api]
impl BlockField {
    #[constant]
    const MAX_INSTANCE_SIZE: u32 = 256;

    #[constant]
    const MAX_BUFFER_SIZE: u32 = 1024;

    #[constant]
    const CHUNK_SIZE: u32 = 32;

    #[func]
    fn new_from(desc: Gd<BlockFieldDesc>, world: Gd<godot::engine::World3D>) -> Gd<Self> {
        let desc = desc.bind();

        let inner_specs = desc
            .entries
            .iter_shared()
            .map(|entry| {
                let entry = entry.bind();
                inner::BlockSpec {
                    size: [entry.size.x, entry.size.y],
                    collision_size: [entry.collision_size.x, entry.collision_size.y],
                    collision_offset: [entry.collision_offset.x, entry.collision_offset.y],
                    hint_size: [entry.rendering_size.x, entry.rendering_size.y],
                    hint_offset: [entry.rendering_offset.x, entry.rendering_offset.y],
                }
            })
            .collect::<Vec<_>>();

        let inner = inner::BlockField::new(Self::CHUNK_SIZE, inner_specs);

        let specs = desc
            .entries
            .iter_shared()
            .map(|entry| {
                let entry = entry.bind();
                BlockSpec {
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

                BlockChunkDown {
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
    fn insert(&mut self, block: Gd<Block>) -> Option<Gd<BlockKey>> {
        let block = block.bind().inner.clone();
        let key = self.inner.insert(block.clone())?;
        Some(Gd::from_init_fn(|_| BlockKey { inner: key }))
    }

    #[func]
    fn remove(&mut self, key: Gd<BlockKey>) -> Option<Gd<Block>> {
        let key = key.bind().inner.clone();
        let block = self.inner.remove(key)?;
        Some(Gd::from_init_fn(|_| Block { inner: block }))
    }

    #[func]
    fn get(&self, key: Gd<BlockKey>) -> Option<Gd<Block>> {
        let key = key.bind().inner.clone();
        let block = self.inner.get(key)?.clone();
        Some(Gd::from_init_fn(|_| Block { inner: block }))
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

            for (i, block) in chunk
                .blocks
                .iter()
                .map(|(_, block)| block)
                .take(Self::MAX_BUFFER_SIZE as usize)
                .enumerate()
            {
                let spec = &self.specs[block.id as usize];
                instance_buffer[i * 12 + 0] = spec.rendering_size[0];
                instance_buffer[i * 12 + 1] = 0.0;
                instance_buffer[i * 12 + 2] = 0.0;
                instance_buffer[i * 12 + 3] = block.location[0] as f32 + spec.rendering_offset[0];

                instance_buffer[i * 12 + 4] = 0.0;
                instance_buffer[i * 12 + 5] = spec.rendering_size[1];
                instance_buffer[i * 12 + 6] = 0.0;
                instance_buffer[i * 12 + 7] = block.location[1] as f32 + spec.rendering_offset[1];

                let z_scale = spec.rendering_size[1] * if spec.z_along_y { 1.0 } else { 0.0 };
                instance_buffer[i * 12 + 8] = 0.0;
                instance_buffer[i * 12 + 9] = 0.0;
                instance_buffer[i * 12 + 10] = z_scale;
                instance_buffer[i * 12 + 11] = 0.0;

                let texcoord = self.texcoords[block.id as usize];
                texcoord_buffer[i * 4 + 0] = texcoord.min_x;
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

    // spatial features

    #[func]
    fn has_by_point(&self, point: Vector2i) -> bool {
        let point = [point.x, point.y];
        self.inner.has_by_point(point)
    }

    #[func]
    fn get_by_point(&self, point: Vector2i) -> Option<Gd<BlockKey>> {
        let point = [point.x, point.y];
        let key = self.inner.get_by_point(point)?.clone();
        Some(Gd::from_init_fn(|_| BlockKey { inner: key }))
    }

    #[func]
    fn has_by_rect(&self, rect: Rect2i) -> bool {
        let p0 = [rect.position.x, rect.position.y];
        let p1 = [rect.position.x + rect.size.x, rect.position.y + rect.size.y];
        self.inner.has_by_rect([p0, p1])
    }

    #[func]
    fn get_by_rect(&self, rect: Rect2i) -> Array<Gd<BlockKey>> {
        let p0 = [rect.position.x, rect.position.y];
        let p1 = [rect.position.x + rect.size.x, rect.position.y + rect.size.y];
        let keys = self.inner.get_by_rect([p0, p1]).cloned();
        Array::from_iter(keys.map(|key| Gd::from_init_fn(|_| BlockKey { inner: key })))
    }

    // collision features

    #[func]
    fn has_collision_by_point(&self, point: Vector2) -> bool {
        let point = [point.x, point.y];
        self.inner.has_collision_by_point(point)
    }

    #[func]
    fn get_collision_by_point(&self, point: Vector2) -> Array<Gd<BlockKey>> {
        let point = [point.x, point.y];
        let keys = self.inner.get_collision_by_point(point).cloned();
        Array::from_iter(keys.map(|key| Gd::from_init_fn(|_| BlockKey { inner: key })))
    }

    #[func]
    fn has_collision_by_rect(&self, rect: Rect2) -> bool {
        let p0 = [rect.position.x, rect.position.y];
        let p1 = [rect.position.x + rect.size.x, rect.position.y + rect.size.y];
        self.inner.has_collision_by_rect([p0, p1])
    }

    #[func]
    fn get_collision_by_rect(&self, rect: Rect2) -> Array<Gd<BlockKey>> {
        let p0 = [rect.position.x, rect.position.y];
        let p1 = [rect.position.x + rect.size.x, rect.position.y + rect.size.y];
        let keys = self.inner.get_collision_by_rect([p0, p1]).cloned();
        Array::from_iter(keys.map(|key| Gd::from_init_fn(|_| BlockKey { inner: key })))
    }

    // hint features

    #[func]
    fn has_hint_by_point(&self, point: Vector2) -> bool {
        let point = [point.x, point.y];
        self.inner.has_hint_by_point(point)
    }

    #[func]
    fn get_hint_by_point(&self, point: Vector2) -> Array<Gd<BlockKey>> {
        let point = [point.x, point.y];
        let keys = self.inner.get_hint_by_point(point).cloned();
        Array::from_iter(keys.map(|key| Gd::from_init_fn(|_| BlockKey { inner: key })))
    }

    #[func]
    fn has_hint_by_rect(&self, rect: Rect2) -> bool {
        let p0 = [rect.position.x, rect.position.y];
        let p1 = [rect.position.x + rect.size.x, rect.position.y + rect.size.y];
        self.inner.has_hint_by_rect([p0, p1])
    }

    #[func]
    fn get_hint_by_rect(&self, rect: Rect2) -> Array<Gd<BlockKey>> {
        let p0 = [rect.position.x, rect.position.y];
        let p1 = [rect.position.x + rect.size.x, rect.position.y + rect.size.y];
        let keys = self.inner.get_hint_by_rect([p0, p1]).cloned();
        Array::from_iter(keys.map(|key| Gd::from_init_fn(|_| BlockKey { inner: key })))
    }
}
