use godot::prelude::*;

use crate::inner;

#[derive(GodotClass)]
#[class(no_init)]
struct TileFieldDescEntry {
    #[export]
    images: Array<Gd<godot::engine::Image>>,
    #[export]
    collision: bool,
}

#[godot_api]
impl TileFieldDescEntry {
    #[func]
    fn new_from(images: Array<Gd<godot::engine::Image>>, collision: bool) -> Gd<Self> {
        Gd::from_object(Self { images, collision })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct TileFieldDesc {
    #[export]
    output_image_size: u32,
    #[export]
    max_page_size: u32,
    #[export]
    mip_block_size: u32,
    #[export]
    entries: Array<Gd<TileFieldDescEntry>>,
    #[export]
    shader: Gd<godot::engine::Shader>,
}

#[godot_api]
impl TileFieldDesc {
    #[func]
    fn new_from(
        output_image_size: u32,
        max_page_size: u32,
        mip_block_size: u32,
        entries: Array<Gd<TileFieldDescEntry>>,
        shader: Gd<godot::engine::Shader>,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            output_image_size,
            max_page_size,
            mip_block_size,
            entries,
            shader,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct Tile {
    inner: inner::Tile,
}

impl Tile {
    #[inline]
    pub fn new(value: inner::Tile) -> Self {
        Self { inner: value }
    }

    #[inline]
    pub fn inner_ref(&self) -> &inner::Tile {
        &self.inner
    }

    #[inline]
    pub fn inner_mut(&mut self) -> &mut inner::Tile {
        &mut self.inner
    }

    #[inline]
    pub fn inner(self) -> inner::Tile {
        self.inner
    }
}

#[godot_api]
impl Tile {
    #[func]
    fn new_from(id: u32, location: Vector2i, variant: u8) -> Gd<Self> {
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

#[derive(Debug, Clone)]
struct TileChunkDown {
    material: Rid,
    multimesh: Rid,
    instance: Rid,
}

impl From<TileChunkUp> for TileChunkDown {
    fn from(chunk: TileChunkUp) -> Self {
        Self {
            material: chunk.material,
            multimesh: chunk.multimesh,
            instance: chunk.instance,
        }
    }
}

#[derive(Debug, Clone)]
struct TileChunkUp {
    serial: u64,
    material: Rid,
    multimesh: Rid,
    instance: Rid,
}

impl From<TileChunkDown> for TileChunkUp {
    fn from(chunk: TileChunkDown) -> Self {
        Self {
            serial: Default::default(),
            material: chunk.material,
            multimesh: chunk.multimesh,
            instance: chunk.instance,
        }
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub(crate) struct TileField {
    inner: inner::TileField,
    texcoords: Vec<Vec<image_atlas::Texcoord32>>,
    down_chunks: Vec<TileChunkDown>,
    up_chunks: ahash::AHashMap<inner::IVec2, TileChunkUp>,
}

// pass the inner reference for world
impl TileField {
    #[inline]
    pub(crate) fn inner_mut(&mut self) -> &mut inner::TileField {
        &mut self.inner
    }
}

#[godot_api]
impl TileField {
    #[constant]
    const MAX_INSTANCE_SIZE: u32 = 256;

    #[constant]
    const MAX_BUFFER_SIZE: u32 = 1024;

    #[constant]
    const CHUNK_SIZE: u32 = 32;

    #[func]
    fn new_from(desc: Gd<TileFieldDesc>, world: Gd<godot::engine::World3D>) -> Gd<Self> {
        let desc = desc.bind();

        let inner_specs = desc
            .entries
            .iter_shared()
            .map(|entry| {
                let entry = entry.bind();
                inner::TileSpec::new(entry.collision)
            })
            .collect::<Vec<_>>();

        let inner = inner::TileField::new(Self::CHUNK_SIZE, inner_specs);

        let entries = desc
            .entries
            .iter_shared()
            .flat_map(|entry| {
                let gd_images = &entry.bind().images;
                gd_images.iter_shared().collect::<Vec<_>>()
            })
            .map(|gd_image| {
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

                image_atlas::AtlasEntry {
                    texture: image,
                    mip: image_atlas::AtlasEntryMipOption::Clamp,
                }
            })
            .collect::<Vec<_>>();

        let variants = desc
            .entries
            .iter_shared()
            .map(|entry| {
                let gd_images = &entry.bind().images;
                gd_images.len() as u8
            })
            .collect::<Vec<_>>();

        let atlas = image_atlas::create_atlas(&image_atlas::AtlasDescriptor {
            size: desc.output_image_size,
            max_page_count: desc.max_page_size,
            mip: image_atlas::AtlasMipOption::MipWithBlock(
                image_atlas::AtlasMipFilter::Linear,
                desc.mip_block_size,
            ),
            entries: &entries,
        })
        .unwrap();

        let pad_mip_level = (desc.output_image_size / desc.mip_block_size).ilog2();
        let pad_pixel_size = (0..pad_mip_level).map(|i| 1 << i).sum::<usize>();
        let gd_images = atlas
            .textures
            .into_iter()
            .map(|texture| {
                let mut data = texture
                    .mip_maps
                    .into_iter()
                    .fold(vec![], |mut data, image| {
                        data.append(&mut image.to_vec());
                        data
                    });
                data.append(&mut vec![0; pad_pixel_size * 4]);
                data
            })
            .map(|data| {
                godot::engine::Image::create_from_data(
                    desc.output_image_size as i32,
                    desc.output_image_size as i32,
                    true,
                    godot::engine::image::Format::RGBA8,
                    PackedByteArray::from(data.as_slice()),
                )
                .unwrap()
            })
            .collect::<Vec<_>>();

        let mut rendering_server = godot::engine::RenderingServer::singleton();
        let texture_array = rendering_server.texture_2d_layered_create(
            Array::from(gd_images.as_slice()),
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
                    Vector3::new(0.0, 1.0, 0.0),
                    Vector3::new(1.0, 1.0, 0.0),
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

                TileChunkDown {
                    material,
                    multimesh,
                    instance,
                }
            })
            .collect::<Vec<_>>();

        Gd::from_object(Self {
            inner,
            texcoords,
            down_chunks,
            up_chunks: Default::default(),
        })
    }

    #[func]
    fn insert(&mut self, tile: Gd<Tile>) -> u32 {
        let tile = tile.bind().inner.clone();
        self.inner.insert(tile).unwrap()
    }

    #[func]
    fn remove(&mut self, key: u32) -> Gd<Tile> {
        let tile = self.inner.remove(key).unwrap();
        Gd::from_object(Tile { inner: tile })
    }

    #[func]
    fn modify(&mut self, key: u32, new_tile: Gd<Tile>) -> Gd<Tile> {
        let new_tile = new_tile.bind().inner.clone();
        let tile = self.inner.modify(key, new_tile).unwrap();
        Gd::from_object(Tile { inner: tile })
    }

    #[func]
    fn get(&self, key: u32) -> Gd<Tile> {
        let tile = self.inner.get(key).unwrap().clone();
        Gd::from_object(Tile { inner: tile })
    }

    // rendering features

    #[func]
    fn insert_view(&mut self, key: Vector2i) {
        let key = [key.x, key.y];

        if self.up_chunks.contains_key(&key) {
            panic!("chunk already exists ({}, {})", key[0], key[1]);
        }

        let Some(chunk) = self.down_chunks.pop() else {
            let up = self.up_chunks.len();
            let down = self.down_chunks.len();
            panic!("no chunk available in pool (up:{}, down:{})", up, down);
        };

        let mut rendering_server = godot::engine::RenderingServer::singleton();
        rendering_server.instance_set_visible(chunk.instance, true);

        self.up_chunks.insert(key, chunk.into());
    }

    #[func]
    fn remove_view(&mut self, key: Vector2i) {
        let key = [key.x, key.y];

        let Some(chunk) = self.up_chunks.remove(&key) else {
            panic!("chunk is not found ({}, {})", key[0], key[1]);
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

            for (i, tile) in chunk
                .tiles
                .iter()
                .map(|(_, tile)| tile)
                .take(Self::MAX_BUFFER_SIZE as usize)
                .enumerate()
            {
                instance_buffer[i * 12] = 2.0;
                instance_buffer[i * 12 + 1] = 0.0;
                instance_buffer[i * 12 + 2] = 0.0;
                instance_buffer[i * 12 + 3] = tile.location[0] as f32 - 0.5;

                instance_buffer[i * 12 + 4] = 0.0;
                instance_buffer[i * 12 + 5] = 2.0;
                instance_buffer[i * 12 + 6] = 0.0;
                instance_buffer[i * 12 + 7] = tile.location[1] as f32 - 0.5;

                let mut hasher = ahash::AHasher::default();
                std::hash::Hasher::write_i32(&mut hasher, tile.location[0]);
                std::hash::Hasher::write_i32(&mut hasher, tile.location[1]);
                let hash = std::hash::Hasher::finish(&hasher) as u16;
                let z_offset = (hash as f32 / u16::MAX as f32) * -0.0625; // -2^{-4} <= z <= 0
                instance_buffer[i * 12 + 8] = 0.0;
                instance_buffer[i * 12 + 9] = 0.0;
                instance_buffer[i * 12 + 10] = 1.0;
                instance_buffer[i * 12 + 11] = z_offset;

                let texcoord = self.texcoords[tile.id as usize][tile.variant as usize];
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

    // spatial features

    #[func]
    fn has_by_point(&self, point: Vector2i) -> bool {
        let point = [point.x, point.y];
        self.inner.has_by_point(point)
    }

    #[func]
    fn get_by_point(&self, point: Vector2i) -> u32 {
        let point = [point.x, point.y];
        self.inner.get_by_point(point).unwrap()
    }

    // collision features

    #[func]
    fn has_collision_by_point(&self, point: Vector2) -> bool {
        let point = [point.x, point.y];
        self.inner.has_collision_by_point(point)
    }

    #[func]
    fn get_collision_by_point(&self, point: Vector2) -> Array<u32> {
        let point = [point.x, point.y];
        let keys = self.inner.get_collision_by_point(point);
        Array::from_iter(keys)
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
        let keys = self.inner.get_collision_by_rect([p0, p1]);
        Array::from_iter(keys)
    }
}
