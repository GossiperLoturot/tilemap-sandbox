use godot::prelude::*;

use crate::inner;

pub(crate) struct BlockDescriptor {
    pub images: Vec<Gd<godot::engine::Image>>,
    pub z_along_y: bool,
    pub rendering_size: inner::Vec2,
    pub rendering_offset: inner::Vec2,
}

pub(crate) struct BlockFieldDescriptor {
    pub instance_size: u32,
    pub output_image_size: u32,
    pub max_page_size: u32,
    pub blocks: Vec<BlockDescriptor>,
    pub shaders: Vec<Gd<godot::engine::Shader>>,
    pub world: Gd<godot::engine::World3D>,
}

struct BlockProperty {
    z_along_y: bool,
    rendering_size: inner::Vec2,
    rendering_offset: inner::Vec2,
}

struct BlockChunkDown {
    materials: Vec<Rid>,
    multimesh: Rid,
    instance: Rid,
}

impl BlockChunkDown {
    fn up(self) -> BlockChunkUp {
        BlockChunkUp {
            version: Default::default(),
            materials: self.materials,
            multimesh: self.multimesh,
            instance: self.instance,
        }
    }
}

struct BlockChunkUp {
    version: u64,
    materials: Vec<Rid>,
    multimesh: Rid,
    instance: Rid,
}

impl BlockChunkUp {
    fn down(self) -> BlockChunkDown {
        BlockChunkDown {
            materials: self.materials,
            multimesh: self.multimesh,
            instance: self.instance,
        }
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub(crate) struct BlockField {
    props: Vec<BlockProperty>,
    texcoords: Vec<Vec<image_atlas::Texcoord32>>,
    down_chunks: Vec<BlockChunkDown>,
    up_chunks: ahash::AHashMap<inner::IVec2, BlockChunkUp>,
    free_handles: Vec<Rid>,
    min_view_rect: Option<[[i32; 2]; 2]>,
}

impl BlockField {
    const MAX_BUFFER_SIZE: u32 = 1024;

    pub fn new(desc: BlockFieldDescriptor) -> Self {
        let mut free_handles = vec![];

        let mut props = vec![];
        for block in &desc.blocks {
            props.push(BlockProperty {
                z_along_y: block.z_along_y,
                rendering_size: block.rendering_size,
                rendering_offset: block.rendering_offset,
            });
        }

        let mut variants = vec![];
        let mut entries = vec![];
        for block in &desc.blocks {
            variants.push(block.images.len() as u8);

            for image in &block.images {
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

                entries.push(image_atlas::AtlasEntry {
                    texture: image_rgba8,
                    mip: image_atlas::AtlasEntryMipOption::Clamp,
                });
            }
        }

        let atlas = image_atlas::create_atlas(&image_atlas::AtlasDescriptor {
            size: desc.output_image_size,
            max_page_count: desc.max_page_size,
            mip: image_atlas::AtlasMipOption::NoMipWithPadding(1),
            entries: &entries,
        })
        .unwrap();

        let mut images = vec![];
        for texture in &atlas.textures {
            let data = texture.mip_maps[0].to_vec();

            let image = godot::engine::Image::create_from_data(
                desc.output_image_size as i32,
                desc.output_image_size as i32,
                false,
                godot::engine::image::Format::RGBA8,
                PackedByteArray::from(data.as_slice()),
            )
            .unwrap();

            images.push(image);
        }

        let mut rendering_server = godot::engine::RenderingServer::singleton();
        let texture_array = rendering_server.texture_2d_layered_create(
            Array::from(images.as_slice()),
            godot::engine::rendering_server::TextureLayeredType::LAYERED_2D_ARRAY,
        );
        free_handles.push(texture_array);

        let mut iter = atlas.texcoords.into_iter();
        let mut texcoords = vec![];
        for variant in variants {
            let mut sub_texcoords = vec![];
            for _ in 0..variant {
                let texcoord = iter.next().unwrap();
                sub_texcoords.push(texcoord.to_f32());
            }
            texcoords.push(sub_texcoords);
        }

        let mut mesh_data = VariantArray::new();
        mesh_data.resize(
            godot::engine::rendering_server::ArrayType::MAX.ord() as usize,
            &Variant::nil(),
        );
        mesh_data.set(
            godot::engine::rendering_server::ArrayType::VERTEX.ord() as usize,
            PackedVector3Array::from(&[
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 1.0),
                Vector3::new(1.0, 1.0, 1.0),
                Vector3::new(1.0, 0.0, 0.0),
            ])
            .to_variant(),
        );
        mesh_data.set(
            godot::engine::rendering_server::ArrayType::TEX_UV.ord() as usize,
            PackedVector2Array::from(&[
                Vector2::new(0.0, 1.0),
                Vector2::new(0.0, 0.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(1.0, 1.0),
            ])
            .to_variant(),
        );
        mesh_data.set(
            godot::engine::rendering_server::ArrayType::INDEX.ord() as usize,
            PackedInt32Array::from(&[0, 1, 2, 0, 2, 3]).to_variant(),
        );

        let mut down_chunks = vec![];
        for _ in 0..desc.instance_size {
            let mut materials = vec![];
            for shader in &desc.shaders {
                let material = rendering_server.material_create();
                rendering_server.material_set_shader(material, shader.get_rid());
                rendering_server.material_set_param(
                    material,
                    "texture_array".into(),
                    texture_array.to_variant(),
                );
                free_handles.push(material);

                materials.push(material)
            }

            for i in 0..materials.len() - 1 {
                let material = materials[i];
                let next_material = materials[i + 1];
                rendering_server.material_set_next_pass(material, next_material);
            }

            let mesh = rendering_server.mesh_create();
            rendering_server.mesh_add_surface_from_arrays(
                mesh,
                godot::engine::rendering_server::PrimitiveType::TRIANGLES,
                mesh_data.clone(),
            );
            rendering_server.mesh_surface_set_material(mesh, 0, materials[0]);
            free_handles.push(mesh);

            let multimesh = rendering_server.multimesh_create();
            rendering_server.multimesh_set_mesh(multimesh, mesh);
            rendering_server.multimesh_allocate_data(
                multimesh,
                Self::MAX_BUFFER_SIZE as i32,
                godot::engine::rendering_server::MultimeshTransformFormat::TRANSFORM_3D,
            );
            free_handles.push(multimesh);

            let instance = rendering_server.instance_create2(multimesh, desc.world.get_scenario());
            rendering_server.instance_set_visible(instance, false);
            free_handles.push(instance);

            down_chunks.push(BlockChunkDown {
                materials,
                multimesh,
                instance,
            });
        }

        Self {
            props,
            texcoords,
            down_chunks,
            up_chunks: Default::default(),
            free_handles,
            min_view_rect: None,
        }
    }

    // rendering features

    pub fn update_view(&mut self, root: &inner::Root, min_view_rect: [inner::Vec2; 2]) {
        let chunk_size = root.block_get_chunk_size() as f32;

        #[rustfmt::skip]
        let min_view_rect = [[
            min_view_rect[0][0].div_euclid(chunk_size) as i32,
            min_view_rect[0][1].div_euclid(chunk_size) as i32, ], [
            min_view_rect[1][0].div_euclid(chunk_size) as i32,
            min_view_rect[1][1].div_euclid(chunk_size) as i32,
        ]];

        // remove/insert view chunk

        if Some(min_view_rect) != self.min_view_rect {
            let mut chunk_keys = vec![];
            for (chunk_key, _) in &self.up_chunks {
                let is_out_of_range_x =
                    chunk_key[0] < min_view_rect[0][0] || min_view_rect[1][0] < chunk_key[0];
                let is_out_of_range_y =
                    chunk_key[1] < min_view_rect[0][1] || min_view_rect[1][1] < chunk_key[1];
                if is_out_of_range_x || is_out_of_range_y {
                    chunk_keys.push(*chunk_key);
                }
            }

            for chunk_key in chunk_keys {
                let up_chunk = self.up_chunks.remove(&chunk_key).unwrap();

                let mut rendering_server = godot::engine::RenderingServer::singleton();
                rendering_server.instance_set_visible(up_chunk.instance, false);

                self.down_chunks.push(up_chunk.down());
            }

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
            let Some(chunk) = root.block_get_chunk(*chunk_key) else {
                continue;
            };

            if chunk.version <= up_chunk.version {
                continue;
            }

            let mut instance_buffer = [0.0; Self::MAX_BUFFER_SIZE as usize * 12];
            let mut texcoord_buffer = [0.0; Self::MAX_BUFFER_SIZE as usize * 4];
            let mut page_buffer = [0.0; Self::MAX_BUFFER_SIZE as usize];

            for (i, (_, block)) in chunk
                .blocks
                .iter()
                .take(Self::MAX_BUFFER_SIZE as usize)
                .enumerate()
            {
                let prop = &self.props[block.id as usize];
                instance_buffer[i * 12] = prop.rendering_size[0];
                instance_buffer[i * 12 + 1] = 0.0;
                instance_buffer[i * 12 + 2] = 0.0;
                instance_buffer[i * 12 + 3] = block.location[0] as f32 + prop.rendering_offset[0];

                instance_buffer[i * 12 + 4] = 0.0;
                instance_buffer[i * 12 + 5] = prop.rendering_size[1];
                instance_buffer[i * 12 + 6] = 0.0;
                instance_buffer[i * 12 + 7] = block.location[1] as f32 + prop.rendering_offset[1];

                let z_scale = prop.rendering_size[1] * if prop.z_along_y { 1.0 } else { 0.0 };
                instance_buffer[i * 12 + 8] = 0.0;
                instance_buffer[i * 12 + 9] = 0.0;
                instance_buffer[i * 12 + 10] = z_scale;
                instance_buffer[i * 12 + 11] = 0.0;

                let texcoord = &self.texcoords[block.id as usize];
                let texcoord = &texcoord[usize::min(block.variant as usize, texcoord.len() - 1)];
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

            for material in &up_chunk.materials {
                rendering_server.material_set_param(
                    *material,
                    "texcoord_buffer".into(),
                    PackedFloat32Array::from(texcoord_buffer.as_slice()).to_variant(),
                );
                rendering_server.material_set_param(
                    *material,
                    "page_buffer".into(),
                    PackedFloat32Array::from(page_buffer.as_slice()).to_variant(),
                );
            }

            up_chunk.version = chunk.version;
        }
    }
}

impl Drop for BlockField {
    fn drop(&mut self) {
        let mut rendering_server = godot::engine::RenderingServer::singleton();
        for free_handle in &self.free_handles {
            rendering_server.free_rid(*free_handle);
        }
    }
}
