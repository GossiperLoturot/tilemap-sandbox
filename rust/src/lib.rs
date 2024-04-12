use godot::prelude::*;

struct Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Extension {}

mod inner {
    #[derive(Debug, Clone, Default)]
    pub struct Tile {
        id: u32,
        x: i32,
        y: i32,
    }

    impl Tile {
        pub fn new(id: u32, x: i32, y: i32) -> Self {
            Self { id, x, y }
        }

        pub fn id(&self) -> u32 {
            self.id
        }

        pub fn x(&self) -> i32 {
            self.x
        }

        pub fn y(&self) -> i32 {
            self.y
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct TileField {
        tiles: slab::Slab<Tile>,
        index: ahash::AHashMap<(i32, i32), u32>,
    }

    impl TileField {
        pub fn add_tile(&mut self, tile: Tile) -> (i32, i32) {
            if !self.index.contains_key(&(tile.x, tile.y)) {
                let idx = self.tiles.insert(tile.clone()) as u32;
                self.index.insert((tile.x, tile.y), idx);
            }
            (tile.x, tile.y)
        }

        pub fn remove_tile(&mut self, key: (i32, i32)) {
            if self.index.contains_key(&key) {
                let idx = self.index.remove(&key).unwrap();
                self.tiles.remove(idx as usize);
            }
        }

        pub fn get_tile(&self, key: &(i32, i32)) -> Option<&Tile> {
            self.index.get(key).map(|idx| &self.tiles[*idx as usize])
        }

        pub fn tiles(&self) -> impl Iterator<Item = &Tile> {
            self.tiles.iter().map(|(_, tile)| tile)
        }
    }
}

#[derive(GodotClass)]
#[class(init, base=Resource)]
struct TileFieldDesc {
    #[export]
    output_image_size: u32,
    #[export]
    max_page_size: u32,
    #[export]
    images: Array<Gd<godot::engine::Image>>,
    #[export]
    shader: Gd<godot::engine::Shader>,
}

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
struct Tile {
    inner: inner::Tile,
}

#[godot_api]
impl Tile {
    #[func]
    fn new_from(id: u32, x: i32, y: i32) -> Gd<Self> {
        let inner = inner::Tile::new(id, x, y);
        Gd::from_init_fn(|_| Self { inner })
    }

    #[func]
    fn get_id(&self) -> u32 {
        self.inner.id()
    }

    #[func]
    fn get_x(&self) -> i32 {
        self.inner.x()
    }

    #[func]
    fn get_y(&self) -> i32 {
        self.inner.y()
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct TileField {
    inner: inner::TileField,
    texcoords: Vec<image_atlas::Texcoord32>,
    material: Rid,
    multimesh: Rid,
    instance: Rid,
}

#[godot_api]
impl TileField {
    const MAX_INSTANCE_COUNT: usize = 1024;

    #[func]
    fn new_from(desc: Gd<TileFieldDesc>, world: Gd<godot::engine::World3D>) -> Gd<Self> {
        let desc = desc.bind();

        let entries = desc
            .images
            .iter_shared()
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
        let gd_mesh_data = {
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
                    Vector2::new(0.0, 0.0),
                    Vector2::new(0.0, 1.0),
                    Vector2::new(1.0, 1.0),
                    Vector2::new(1.0, 0.0),
                ])
                .to_variant(),
            );
            data.set(
                godot::engine::rendering_server::ArrayType::INDEX.ord() as usize,
                PackedInt32Array::from(&[0, 1, 2, 0, 2, 3]).to_variant(),
            );
            data
        };

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
            gd_mesh_data,
        );
        rendering_server.mesh_surface_set_material(mesh, 0, material);

        let multimesh = rendering_server.multimesh_create();
        rendering_server.multimesh_set_mesh(multimesh, mesh);
        rendering_server.multimesh_allocate_data(
            multimesh,
            Self::MAX_INSTANCE_COUNT as i32,
            godot::engine::rendering_server::MultimeshTransformFormat::TRANSFORM_3D,
        );

        let instance = rendering_server.instance_create2(multimesh, world.get_scenario());
        rendering_server.instance_set_visible(instance, false);

        Gd::from_init_fn(|_| Self {
            inner: Default::default(),
            texcoords,
            material,
            multimesh,
            instance,
        })
    }

    #[func]
    fn add_tile(&mut self, tile: Gd<Tile>) -> Vector2i {
        let tile = tile.bind().inner.clone();
        let key = self.inner.add_tile(tile);
        Vector2i::new(key.0, key.1)
    }

    #[func]
    fn remove_tile(&mut self, key: Vector2i) {
        let key = (key.x, key.y);
        self.inner.remove_tile(key);
    }

    #[func]
    fn get_tile(&self, key: Vector2i) -> Option<Gd<Tile>> {
        let key = (key.x, key.y);
        self.inner
            .get_tile(&key)
            .cloned()
            .map(|inner| Gd::from_init_fn(|_| Tile { inner }))
    }

    #[func]
    fn spawn(&mut self) {
        let mut instance_buffer = vec![0.0; Self::MAX_INSTANCE_COUNT * 12];
        let mut texcoord_buffer = vec![0.0; Self::MAX_INSTANCE_COUNT * 4];
        let mut page_buffer = vec![0.0; Self::MAX_INSTANCE_COUNT];

        for (i, tile) in self
            .inner
            .tiles()
            .take(Self::MAX_INSTANCE_COUNT)
            .enumerate()
        {
            instance_buffer[i * 12 + 0] = 1.0;
            instance_buffer[i * 12 + 1] = 0.0;
            instance_buffer[i * 12 + 2] = 0.0;
            instance_buffer[i * 12 + 3] = tile.x() as f32;

            instance_buffer[i * 12 + 4] = 0.0;
            instance_buffer[i * 12 + 5] = 1.0;
            instance_buffer[i * 12 + 6] = 0.0;
            instance_buffer[i * 12 + 7] = tile.y() as f32;

            instance_buffer[i * 12 + 8] = 0.0;
            instance_buffer[i * 12 + 9] = 0.0;
            instance_buffer[i * 12 + 10] = 1.0;
            instance_buffer[i * 12 + 11] = 0.0;

            let texcoord = self.texcoords[tile.id() as usize];
            texcoord_buffer[i * 4 + 0] = texcoord.min_x;
            texcoord_buffer[i * 4 + 1] = texcoord.min_y;
            texcoord_buffer[i * 4 + 2] = texcoord.max_x - texcoord.min_x;
            texcoord_buffer[i * 4 + 3] = texcoord.max_y - texcoord.min_y;

            page_buffer[i] = texcoord.page as f32;
        }

        let mut rendering_server = godot::engine::RenderingServer::singleton();
        rendering_server.multimesh_set_buffer(
            self.multimesh,
            PackedFloat32Array::from(instance_buffer.as_slice()),
        );
        rendering_server.material_set_param(
            self.material,
            "texcoord_buffer".into(),
            PackedFloat32Array::from(texcoord_buffer.as_slice()).to_variant(),
        );
        rendering_server.material_set_param(
            self.material,
            "page_buffer".into(),
            PackedFloat32Array::from(page_buffer.as_slice()).to_variant(),
        );
        rendering_server.instance_set_visible(self.instance, true);
    }

    #[func]
    fn kill(&mut self) {
        let mut rendering_server = godot::engine::RenderingServer::singleton();
        rendering_server.instance_set_visible(self.instance, false);
    }
}
