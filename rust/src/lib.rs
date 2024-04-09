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
struct AtlasDesc {
    #[export]
    output_image_size: u32,
    #[export]
    max_page_size: u32,
    #[export]
    images: Array<Gd<godot::engine::Image>>,
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
    texture_array: Gd<godot::engine::Texture2DArray>,
    texcoords: Vec<image_atlas::Texcoord32>,
}

#[godot_api]
impl TileField {
    #[func]
    fn new_from(desc: Gd<AtlasDesc>) -> Gd<Self> {
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

        let mut texture_array = godot::engine::Texture2DArray::new_gd();
        texture_array.create_from_images(Array::from(gd_images.as_slice()));

        let texcoords = atlas
            .texcoords
            .into_iter()
            .map(|texcoord| texcoord.to_f32())
            .collect::<Vec<_>>();

        Gd::from_init_fn(|_| Self {
            texture_array,
            texcoords,
            inner: Default::default(),
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
    fn get_texture_array(&self) -> Gd<godot::engine::Texture2DArray> {
        self.texture_array.clone()
    }

    #[func]
    fn update_buffer(&self, multimesh: Rid, material: Rid) {
        let mut instance_buffer = vec![];
        let mut texcoord_buffer = vec![];
        let mut page_buffer = vec![];

        for tile in self.inner.tiles() {
            instance_buffer.push(1.0);
            instance_buffer.push(0.0);
            instance_buffer.push(0.0);
            instance_buffer.push(tile.x() as f32);

            instance_buffer.push(0.0);
            instance_buffer.push(1.0);
            instance_buffer.push(0.0);
            instance_buffer.push(tile.y() as f32);

            instance_buffer.push(0.0);
            instance_buffer.push(0.0);
            instance_buffer.push(1.0);
            instance_buffer.push(0.0);

            let texcoord = self.texcoords[tile.id() as usize];
            texcoord_buffer.push(texcoord.min_x);
            texcoord_buffer.push(texcoord.min_y);
            texcoord_buffer.push(texcoord.max_x - texcoord.min_x);
            texcoord_buffer.push(texcoord.max_y - texcoord.min_y);

            page_buffer.push(texcoord.page as f32);
        }

        let mut rendering_server = godot::engine::RenderingServer::singleton();

        let buffer = PackedFloat32Array::from(instance_buffer.as_slice());
        rendering_server.multimesh_set_buffer(multimesh, buffer);

        let buffer = PackedFloat32Array::from(texcoord_buffer.as_slice()).to_variant();
        rendering_server.material_set_param(material, "texcoord_buffer".into(), buffer);

        let buffer = PackedFloat32Array::from(page_buffer.as_slice()).to_variant();
        rendering_server.material_set_param(material, "page_buffer".into(), buffer);
    }
}
