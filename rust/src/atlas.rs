use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Resource)]
pub struct AtlasDesc {
    #[export]
    image_size: u32,
    #[export]
    max_page_size: u32,
    #[export]
    images: Array<Gd<godot::engine::Image>>,
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct AtlasTexcoord {
    page: u32,
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

#[godot_api]
impl AtlasTexcoord {
    #[func]
    pub fn page(&self) -> u32 {
        self.page
    }

    #[func]
    pub fn min_x(&self) -> f32 {
        self.min_x
    }
    #[func]
    pub fn min_y(&self) -> f32 {
        self.min_y
    }

    #[func]
    pub fn max_x(&self) -> f32 {
        self.max_x
    }

    #[func]
    pub fn max_y(&self) -> f32 {
        self.max_y
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct Atlas {
    image_size: u32,
    page_size: u32,
    images: Array<Gd<godot::engine::Image>>,
    texcoords: Array<Gd<AtlasTexcoord>>,
}

#[godot_api]
impl Atlas {
    #[func]
    pub fn image_size(&self) -> u32 {
        self.image_size
    }

    #[func]
    pub fn page_size(&self) -> u32 {
        self.page_size
    }

    #[func]
    pub fn images(&self) -> Array<Gd<godot::engine::Image>> {
        self.images.clone()
    }

    #[func]
    pub fn texcoords(&self) -> Array<Gd<AtlasTexcoord>> {
        self.texcoords.clone()
    }

    #[func]
    pub fn from_desc(desc: Gd<AtlasDesc>) -> Gd<Self> {
        let desc = desc.bind();

        let images = desc
            .images
            .iter_shared()
            .map(|image| {
                let width = image.get_width() as u32;
                let height = image.get_height() as u32;

                let mut dst_image = image::RgbaImage::new(width, height);

                for y in 0..image.get_height() {
                    for x in 0..image.get_width() {
                        let rgba = image.get_pixel(x, y);
                        let rgba = image::Rgba([rgba.r8(), rgba.g8(), rgba.b8(), rgba.a8()]);
                        dst_image.put_pixel(x as u32, y as u32, rgba);
                    }
                }

                dst_image
            })
            .collect::<Vec<_>>();

        let atlas = image_atlas::create_atlas(&image_atlas::AtlasDescriptor {
            max_page_count: desc.max_page_size,
            size: desc.image_size,
            mip: image_atlas::AtlasMipOption::NoMip,
            entries: images
                .into_iter()
                .map(|image| image_atlas::AtlasEntry {
                    texture: image,
                    mip: image_atlas::AtlasEntryMipOption::Clamp,
                })
                .collect::<Vec<_>>()
                .as_slice(),
        })
        .unwrap();

        let images = atlas
            .textures
            .into_iter()
            .map(|mut texture| {
                let image = texture.mip_maps.pop().unwrap();

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

        let texcoords = atlas
            .texcoords
            .into_iter()
            .map(|texcoord| {
                let texcoord = texcoord.to_f32();

                Gd::from_init_fn(|_| AtlasTexcoord {
                    page: texcoord.page,
                    min_x: texcoord.min_x,
                    min_y: texcoord.min_y,
                    max_x: texcoord.max_x,
                    max_y: texcoord.max_y,
                })
            })
            .collect::<Vec<_>>();

        Gd::from_init_fn(|_| Self {
            image_size: atlas.size,
            page_size: atlas.page_count,
            images: Array::from(images.as_slice()),
            texcoords: Array::from(texcoords.as_slice()),
        })
    }
}
