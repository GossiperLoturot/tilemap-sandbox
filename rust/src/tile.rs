use godot::prelude::*;

use crate::inner;

pub(crate) struct TileImageDescriptor {
    pub frames: Vec<Gd<godot::classes::Image>>,
}

pub(crate) struct TileDescriptor {
    pub images: Vec<TileImageDescriptor>,
}

pub(crate) struct TileFieldDescriptor {
    pub instance_size: u32,
    pub output_image_size: u32,
    pub max_page_size: u32,
    pub tiles: Vec<TileDescriptor>,
    pub shaders: Vec<Gd<godot::classes::Shader>>,
    pub world: Gd<godot::classes::World3D>,
}

struct TileChunkDown {
    materials: Vec<Rid>,
    multimesh: Rid,
    instance: Rid,
}

impl TileChunkDown {
    fn up(self) -> TileChunkUp {
        TileChunkUp {
            version: Default::default(),
            materials: self.materials,
            multimesh: self.multimesh,
            instance: self.instance,
        }
    }
}

struct TileChunkUp {
    version: u64,
    materials: Vec<Rid>,
    multimesh: Rid,
    instance: Rid,
}

impl TileChunkUp {
    fn down(self) -> TileChunkDown {
        TileChunkDown {
            materials: self.materials,
            multimesh: self.multimesh,
            instance: self.instance,
        }
    }
}

pub(crate) struct TileField {
    down_chunks: Vec<TileChunkDown>,
    up_chunks: ahash::AHashMap<inner::IVec2, TileChunkUp>,
    free_handles: Vec<Rid>,
    min_rect: Option<[inner::IVec2; 2]>,
}

impl TileField {
    const MAX_BUFFER_SIZE: u32 = 1024;

    pub fn new(desc: TileFieldDescriptor) -> Self {
        let mut free_handles = vec![];

        let mut index_meta = vec![];
        let mut entries = vec![];
        for (i, tile) in desc.tiles.into_iter().enumerate() {
            for (j, image) in tile.images.into_iter().enumerate() {
                for (k, frame) in image.frames.into_iter().enumerate() {
                    let width = frame.get_width() as u32;
                    let height = frame.get_height() as u32;

                    let mut image_rgba8 = image::RgbaImage::new(width, height);
                    for y in 0..height {
                        for x in 0..width {
                            let color = frame.get_pixel(x as i32, y as i32);
                            let rgba8 =
                                image::Rgba([color.r8(), color.g8(), color.b8(), color.a8()]);
                            image_rgba8.put_pixel(x, y, rgba8);
                        }
                    }

                    entries.push(image_atlas::AtlasEntry {
                        texture: image_rgba8,
                        mip: image_atlas::AtlasEntryMipOption::Clamp,
                    });
                    index_meta.push((i as u32, j as u32, k as u32));
                }
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

            let image = godot::classes::Image::create_from_data(
                desc.output_image_size as i32,
                desc.output_image_size as i32,
                false,
                godot::classes::image::Format::RGBA8,
                PackedByteArray::from(data.as_slice()),
            )
            .unwrap();

            images.push(image);
        }

        let mut rendering_server = godot::classes::RenderingServer::singleton();
        let texture_array = rendering_server.texture_2d_layered_create(
            Array::from(images.as_slice()),
            godot::classes::rendering_server::TextureLayeredType::LAYERED_2D_ARRAY,
        );
        free_handles.push(texture_array);

        let mut hierarchy: Vec<Vec<Vec<u32>>> = Default::default();
        for (index, (i, j, k)) in index_meta.into_iter().enumerate() {
            while hierarchy.len() <= i as usize {
                hierarchy.push(Default::default());
            }
            let hierarchy = &mut hierarchy[i as usize];

            while hierarchy.len() <= j as usize {
                hierarchy.push(Default::default());
            }
            let hierarchy = &mut hierarchy[j as usize];

            while hierarchy.len() <= k as usize {
                hierarchy.push(Default::default());
            }
            let hierarchy = &mut hierarchy[k as usize];

            *hierarchy = index as u32;
        }
        let mut fst_index_buffer = [0.0; Self::MAX_BUFFER_SIZE as usize];
        let mut snd_index_buffer = [0.0; Self::MAX_BUFFER_SIZE as usize];
        let mut texcoord_buffer = [0.0; Self::MAX_BUFFER_SIZE as usize * 4];
        let mut page_buffer = [0.0; Self::MAX_BUFFER_SIZE as usize];
        let (mut fst_size, mut snd_size, mut thd_size) = (0, 0, 0);
        for fst in hierarchy {
            fst_index_buffer[fst_size] = snd_size as f32;
            for snd in fst {
                snd_index_buffer[snd_size] = thd_size as f32;
                for thd in snd {
                    let texcoord = atlas.texcoords[thd as usize].to_f32();
                    texcoord_buffer[thd_size * 4] = texcoord.min_x;
                    texcoord_buffer[thd_size * 4 + 1] = texcoord.min_y;
                    texcoord_buffer[thd_size * 4 + 2] = texcoord.max_x - texcoord.min_x;
                    texcoord_buffer[thd_size * 4 + 3] = texcoord.max_y - texcoord.min_y;
                    page_buffer[thd_size] = texcoord.page as f32;

                    thd_size += 1;
                }
                snd_size += 1;
            }
            fst_size += 1;
        }
        let fst_index_buffer = PackedFloat32Array::from(fst_index_buffer.as_slice());
        let snd_index_buffer = PackedFloat32Array::from(snd_index_buffer.as_slice());
        let texcoord_buffer = PackedFloat32Array::from(texcoord_buffer.as_slice());
        let page_buffer = PackedFloat32Array::from(page_buffer.as_slice());

        let mut mesh_data = VariantArray::new();
        mesh_data.resize(
            godot::classes::rendering_server::ArrayType::MAX.ord() as usize,
            &Variant::nil(),
        );
        mesh_data.set(
            godot::classes::rendering_server::ArrayType::VERTEX.ord() as usize,
            PackedVector3Array::from(&[
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(1.0, 1.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
            ])
            .to_variant(),
        );
        mesh_data.set(
            godot::classes::rendering_server::ArrayType::TEX_UV.ord() as usize,
            PackedVector2Array::from(&[
                Vector2::new(0.0, 1.0),
                Vector2::new(0.0, 0.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(1.0, 1.0),
            ])
            .to_variant(),
        );
        mesh_data.set(
            godot::classes::rendering_server::ArrayType::INDEX.ord() as usize,
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
                rendering_server.material_set_param(
                    material,
                    "fst_index_buffer".into(),
                    fst_index_buffer.to_variant(),
                );
                rendering_server.material_set_param(
                    material,
                    "snd_index_buffer".into(),
                    snd_index_buffer.to_variant(),
                );
                rendering_server.material_set_param(
                    material,
                    "texcoord_buffer".into(),
                    texcoord_buffer.to_variant(),
                );
                rendering_server.material_set_param(
                    material,
                    "page_buffer".into(),
                    page_buffer.to_variant(),
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
                godot::classes::rendering_server::PrimitiveType::TRIANGLES,
                mesh_data.clone(),
            );
            rendering_server.mesh_surface_set_material(mesh, 0, materials[0]);
            free_handles.push(mesh);

            let multimesh = rendering_server.multimesh_create();
            rendering_server.multimesh_set_mesh(multimesh, mesh);
            rendering_server.multimesh_allocate_data(
                multimesh,
                Self::MAX_BUFFER_SIZE as i32,
                godot::classes::rendering_server::MultimeshTransformFormat::TRANSFORM_3D,
            );
            free_handles.push(multimesh);

            let instance = rendering_server.instance_create2(multimesh, desc.world.get_scenario());
            rendering_server.instance_set_visible(instance, false);
            free_handles.push(instance);

            down_chunks.push(TileChunkDown {
                materials,
                multimesh,
                instance,
            });
        }

        Self {
            down_chunks,
            up_chunks: Default::default(),
            free_handles,
            min_rect: None,
        }
    }

    pub fn update_view(&mut self, root: &inner::Root, min_rect: [inner::Vec2; 2]) {
        let chunk_size = root.tile_get_chunk_size() as f32;

        #[rustfmt::skip]
        let min_rect = [[
            min_rect[0][0].div_euclid(chunk_size) as i32,
            min_rect[0][1].div_euclid(chunk_size) as i32, ], [
            min_rect[1][0].div_euclid(chunk_size) as i32,
            min_rect[1][1].div_euclid(chunk_size) as i32,
        ]];

        // remove/insert view chunk

        if Some(min_rect) != self.min_rect {
            let mut chunk_keys = vec![];
            for (chunk_key, _) in &self.up_chunks {
                let is_out_of_range_x =
                    chunk_key[0] < min_rect[0][0] || min_rect[1][0] < chunk_key[0];
                let is_out_of_range_y =
                    chunk_key[1] < min_rect[0][1] || min_rect[1][1] < chunk_key[1];
                if is_out_of_range_x || is_out_of_range_y {
                    chunk_keys.push(*chunk_key);
                }
            }

            for chunk_key in chunk_keys {
                let up_chunk = self.up_chunks.remove(&chunk_key).unwrap();

                let mut rendering_server = godot::classes::RenderingServer::singleton();
                rendering_server.instance_set_visible(up_chunk.instance, false);

                self.down_chunks.push(up_chunk.down());
            }

            for y in min_rect[0][1]..=min_rect[1][1] {
                for x in min_rect[0][0]..=min_rect[1][0] {
                    let chunk_key = [x, y];

                    if self.up_chunks.contains_key(&chunk_key) {
                        continue;
                    }

                    let Some(down_chunk) = self.down_chunks.pop() else {
                        let up = self.up_chunks.len();
                        let down = self.down_chunks.len();
                        panic!("no chunk available in pool (up:{}, down:{})", up, down);
                    };

                    let mut rendering_server = godot::classes::RenderingServer::singleton();
                    rendering_server.instance_set_visible(down_chunk.instance, true);

                    self.up_chunks.insert(chunk_key, down_chunk.up());
                }
            }

            self.min_rect = Some(min_rect);
        }

        // update view chunk

        for (chunk_key, up_chunk) in &mut self.up_chunks {
            let Ok(chunk) = root.tile_get_chunk(*chunk_key) else {
                continue;
            };

            if chunk.version <= up_chunk.version {
                continue;
            }

            let mut instance_buffer = [0.0; Self::MAX_BUFFER_SIZE as usize * 12];
            let mut extra_buffer = [0.0; Self::MAX_BUFFER_SIZE as usize * 2];

            for (i, (_, tile)) in chunk
                .tiles
                .iter()
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
                let z_offset = (hash as f32 / u16::MAX as f32) * -0.0625 - 0.0625; // -2^{-3} <= z <= -2^{-4}
                instance_buffer[i * 12 + 8] = 0.0;
                instance_buffer[i * 12 + 9] = 0.0;
                instance_buffer[i * 12 + 10] = 1.0;
                instance_buffer[i * 12 + 11] = z_offset;

                extra_buffer[i * 2] = tile.id as f32;
                extra_buffer[i * 2 + 1] = tile.variant as f32;
            }

            let mut rendering_server = godot::classes::RenderingServer::singleton();

            rendering_server.multimesh_set_buffer(
                up_chunk.multimesh,
                PackedFloat32Array::from(instance_buffer.as_slice()),
            );

            for material in &up_chunk.materials {
                rendering_server.material_set_param(
                    *material,
                    "extra_buffer".into(),
                    PackedFloat32Array::from(extra_buffer.as_slice()).to_variant(),
                );
            }

            up_chunk.version = chunk.version;
        }
    }
}

impl Drop for TileField {
    fn drop(&mut self) {
        let mut rendering_server = godot::classes::RenderingServer::singleton();
        for free_handle in &self.free_handles {
            rendering_server.free_rid(*free_handle);
        }
    }
}
