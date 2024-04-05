use godot::prelude::*;

struct Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Extension {}

#[derive(GodotClass)]
#[class(init, base=Resource)]
struct TileSetDescItem {
    #[export]
    name: GString,
    #[export]
    image: Gd<godot::engine::Image>,
}

#[derive(GodotClass)]
#[class(init, base=Resource)]
struct TileSetDesc {
    #[export]
    image_size: u32,
    #[export]
    max_page_size: u32,
    #[export]
    items: Array<Gd<TileSetDescItem>>,
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct TileSetCoord {
    #[export]
    page: u32,
    #[export]
    min_x: f32,
    #[export]
    min_y: f32,
    #[export]
    max_x: f32,
    #[export]
    max_y: f32,
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
struct TileSet {
    #[export]
    image_size: u32,
    #[export]
    page_size: u32,
    #[export]
    images: Array<Gd<godot::engine::Image>>,
    #[export]
    texcoords: Array<Gd<TileSetCoord>>,
}

#[godot_api]
impl TileSet {
    #[func]
    fn from_desc(desc: Gd<TileSetDesc>) -> Gd<Self> {
        let desc = desc.bind();

        let images = desc
            .items
            .iter_shared()
            .map(|item| {
                let src_image = &item.bind().image;
                let width = src_image.get_width() as u32;
                let height = src_image.get_height() as u32;

                let mut dst_image = image::RgbaImage::new(width, height);

                for y in 0..src_image.get_height() {
                    for x in 0..src_image.get_width() {
                        let rgba = src_image.get_pixel(x, y);
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

                Gd::from_init_fn(|_| TileSetCoord {
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
