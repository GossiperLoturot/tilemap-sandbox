use godot::prelude::*;

use crate::inner;

pub(crate) struct ItemImageDescriptor {
    pub frames: Vec<Gd<godot::classes::Image>>,
    pub step_tick: u16,
    pub is_loop: bool,
}

pub(crate) struct ItemDescriptor {
    pub image: ItemImageDescriptor,
}

pub(crate) struct InventoryDescriptor {
    pub items: Vec<ItemDescriptor>,
    pub shaders: Vec<Gd<godot::classes::Shader>>,
    pub world: Gd<godot::classes::World3D>,
}

struct ImageHead {
    start_texcoord_id: u32,
    end_texcoord_id: u32,
    step_tick: u16,
    is_loop: bool,
}

pub(crate) struct Inventory {
    image_heads: Vec<ImageHead>,
    free_handles: Vec<Rid>,
}

impl Inventory {
    const OUTPUT_IMAGE_SIZE: usize = 1024;
    const MAX_PAGE_SIZE: usize = 8;
    const BAKE_TEXTURE_SIZE: usize = 1024;

    pub fn new(desc: InventoryDescriptor) -> Self {
        let mut rendering_server = godot::classes::RenderingServer::singleton();

        let mut free_handles = vec![];

        let mut image_heads = vec![];
        let mut image_bodies = vec![];
        for item in desc.items {
            let image = item.image;

            if image_bodies.len() + image.frames.len() >= i32::MAX as usize {
                panic!("number of frame must be less than i32::MAX");
            }

            image_heads.push(ImageHead {
                start_texcoord_id: image_bodies.len() as u32,
                end_texcoord_id: image_bodies.len() as u32 + image.frames.len() as u32,
                step_tick: image.step_tick,
                is_loop: image.is_loop,
            });

            for frame in image.frames {
                let width = frame.get_width() as u32;
                let height = frame.get_height() as u32;

                let mut image_rgba8 = image::RgbaImage::new(width, height);
                for y in 0..height {
                    for x in 0..width {
                        let color = frame.get_pixel(x as i32, y as i32);
                        let rgba8 = image::Rgba([color.r8(), color.g8(), color.b8(), color.a8()]);
                        image_rgba8.put_pixel(x, y, rgba8);
                    }
                }

                image_bodies.push(image_rgba8);
            }
        }

        let mut atlas_entries = vec![];
        for image_body in image_bodies {
            atlas_entries.push(image_atlas::AtlasEntry {
                texture: image_body,
                mip: image_atlas::AtlasEntryMipOption::Clamp,
            });
        }
        let atlas = image_atlas::create_atlas(&image_atlas::AtlasDescriptor {
            size: Self::OUTPUT_IMAGE_SIZE as u32,
            max_page_count: Self::MAX_PAGE_SIZE as u32,
            mip: image_atlas::AtlasMipOption::NoMipWithPadding(1),
            entries: &atlas_entries,
        })
        .unwrap();

        let mut images = vec![];
        for texture in &atlas.textures {
            let data = texture.mip_maps[0].to_vec();

            let image = godot::classes::Image::create_from_data(
                Self::OUTPUT_IMAGE_SIZE as i32,
                Self::OUTPUT_IMAGE_SIZE as i32,
                false,
                godot::classes::image::Format::RGBA8,
                PackedByteArray::from(data.as_slice()),
            )
            .unwrap();

            images.push(image);
        }

        let texture_array = rendering_server.texture_2d_layered_create(
            Array::from(images.as_slice()),
            godot::classes::rendering_server::TextureLayeredType::LAYERED_2D_ARRAY,
        );
        free_handles.push(texture_array);

        if atlas.texcoords.len() * 2 > Self::BAKE_TEXTURE_SIZE * Self::BAKE_TEXTURE_SIZE {
            panic!("number of (frame * 2) must be less than (BAKE_TEXTURE_SIZE ^ 2)");
        }
        let mut bake_data = vec![0.0; Self::BAKE_TEXTURE_SIZE * Self::BAKE_TEXTURE_SIZE * 4];
        for (i, texcoord) in atlas.texcoords.into_iter().enumerate() {
            let texcoord = texcoord.to_f32();
            bake_data[i * 8] = texcoord.min_x;
            bake_data[i * 8 + 1] = texcoord.min_y;
            bake_data[i * 8 + 2] = texcoord.max_x - texcoord.min_x;
            bake_data[i * 8 + 3] = texcoord.max_y - texcoord.min_y;

            bake_data[i * 8 + 4] = texcoord.page as f32;
            bake_data[i * 8 + 5] = 0.0;
            bake_data[i * 8 + 6] = 0.0;
            bake_data[i * 8 + 7] = 0.0;
        }
        let bake_image = godot::classes::Image::create_from_data(
            Self::BAKE_TEXTURE_SIZE as i32,
            Self::BAKE_TEXTURE_SIZE as i32,
            false,
            godot::classes::image::Format::RGBAF,
            PackedFloat32Array::from(bake_data.as_slice()).to_byte_array(),
        )
        .unwrap();
        let bake_texture = rendering_server.texture_2d_create(bake_image);
        free_handles.push(bake_texture);

        Self {
            image_heads,
            free_handles,
        }
    }

    pub fn update_view(&mut self, _root: &inner::Root) {
        todo!()
    }
}

impl Drop for Inventory {
    fn drop(&mut self) {
        let mut rendering_server = godot::classes::RenderingServer::singleton();
        for free_handle in &self.free_handles {
            rendering_server.free_rid(*free_handle);
        }
    }
}
