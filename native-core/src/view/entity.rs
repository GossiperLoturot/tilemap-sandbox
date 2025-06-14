use glam::*;
use godot::prelude::*;

use crate::dataflow;

pub struct EntityImageDescriptor {
    pub frames: Vec<Gd<godot::classes::Image>>,
    pub step_tick: u16,
    pub is_loop: bool,
}

pub struct EntityDescriptor {
    pub images: Vec<EntityImageDescriptor>,
    pub z_along_y: bool,
    pub rendering_size: Vec2,
    pub rendering_offset: Vec2,
}

pub struct EntityFieldDescriptor {
    pub entities: Vec<EntityDescriptor>,
    pub shaders: Vec<Gd<godot::classes::Shader>>,
    pub world: Gd<godot::classes::World3D>,
}

struct EntityProperty {
    z_along_y: bool,
    rendering_size: Vec2,
    rendering_offset: Vec2,
}

struct ImageHead {
    start_texcoord_id: u32,
    end_texcoord_id: u32,
    step_tick: u16,
    is_loop: bool,
}

struct EntityChunkDown {
    materials: Vec<Rid>,
    multimesh: Rid,
    instance: Rid,
}

impl EntityChunkDown {
    fn up(self) -> EntityChunkUp {
        EntityChunkUp {
            version: Default::default(),
            materials: self.materials,
            multimesh: self.multimesh,
            instance: self.instance,
        }
    }
}

struct EntityChunkUp {
    version: u64,
    materials: Vec<Rid>,
    multimesh: Rid,
    instance: Rid,
}

impl EntityChunkUp {
    fn down(self) -> EntityChunkDown {
        EntityChunkDown {
            materials: self.materials,
            multimesh: self.multimesh,
            instance: self.instance,
        }
    }
}

pub struct EntityField {
    props: Vec<EntityProperty>,
    image_heads: Vec<Vec<ImageHead>>,
    down_chunks: Vec<EntityChunkDown>,
    up_chunks: ahash::AHashMap<IVec2, EntityChunkUp>,
    free_handles: Vec<Rid>,
    min_rect: Option<[IVec2; 2]>,
}

impl EntityField {
    const INSTANCE_SIZE: usize = 512;
    const OUTPUT_IMAGE_SIZE: usize = 1024;
    const MAX_PAGE_SIZE: usize = 8;
    const BAKE_TEXTURE_SIZE: usize = 1024;
    const MAX_BUFFER_SIZE: usize = 1024;

    pub fn new(desc: EntityFieldDescriptor) -> Self {
        let mut rendering_server = godot::classes::RenderingServer::singleton();

        let mut free_handles = vec![];

        let mut props = vec![];
        for entity in &desc.entities {
            props.push(EntityProperty {
                z_along_y: entity.z_along_y,
                rendering_size: entity.rendering_size,
                rendering_offset: entity.rendering_offset,
            });
        }

        let mut image_heads = vec![];
        let mut image_bodies = vec![];
        for entity in desc.entities {
            let mut sub_image_heads = vec![];

            for image in entity.images {
                if image_bodies.len() + image.frames.len() >= i32::MAX as usize {
                    panic!("number of frame must be less than i32::MAX");
                }

                sub_image_heads.push(ImageHead {
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
                            let rgba8 =
                                image::Rgba([color.r8(), color.g8(), color.b8(), color.a8()]);
                            image_rgba8.put_pixel(x, y, rgba8);
                        }
                    }

                    image_bodies.push(image_rgba8);
                }
            }

            image_heads.push(sub_image_heads);
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
                &PackedByteArray::from(data.as_slice()),
            )
            .unwrap();

            images.push(image);
        }

        let texture_array = rendering_server.texture_2d_layered_create(
            &Array::from(images.as_slice()),
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
            &PackedFloat32Array::from(bake_data.as_slice()).to_byte_array(),
        )
        .unwrap();
        let bake_texture = rendering_server.texture_2d_create(&bake_image);
        free_handles.push(bake_texture);

        let mut mesh_data = VariantArray::new();
        mesh_data.resize(
            godot::classes::rendering_server::ArrayType::MAX.ord() as usize,
            &Variant::nil(),
        );
        mesh_data.set(
            godot::classes::rendering_server::ArrayType::VERTEX.ord() as usize,
            &PackedVector3Array::from(&[
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 1.0),
                Vector3::new(1.0, 1.0, 1.0),
                Vector3::new(1.0, 0.0, 0.0),
            ])
            .to_variant(),
        );
        mesh_data.set(
            godot::classes::rendering_server::ArrayType::TEX_UV.ord() as usize,
            &PackedVector2Array::from(&[
                Vector2::new(0.0, 1.0),
                Vector2::new(0.0, 0.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(1.0, 1.0),
            ])
            .to_variant(),
        );
        mesh_data.set(
            godot::classes::rendering_server::ArrayType::INDEX.ord() as usize,
            &PackedInt32Array::from(&[0, 1, 2, 0, 2, 3]).to_variant(),
        );

        let mut down_chunks = vec![];
        for _ in 0..Self::INSTANCE_SIZE {
            let mut materials = vec![];
            for shader in &desc.shaders {
                let material = rendering_server.material_create();
                rendering_server.material_set_shader(material, shader.get_rid());
                rendering_server.material_set_param(
                    material,
                    "texture_array",
                    &texture_array.to_variant(),
                );
                rendering_server.material_set_param(
                    material,
                    "bake_texture",
                    &bake_texture.to_variant(),
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
                &mesh_data,
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

            down_chunks.push(EntityChunkDown {
                materials,
                multimesh,
                instance,
            });
        }

        Self {
            props,
            image_heads,
            down_chunks,
            up_chunks: Default::default(),
            free_handles,
            min_rect: None,
        }
    }

    pub fn update_view(&mut self, dataflow: &dataflow::Dataflow, min_rect: [Vec2; 2]) {
        let mut rendering_server = godot::classes::RenderingServer::singleton();

        let min_rect = [
            dataflow.get_entity_chunk_location(min_rect[0]),
            dataflow.get_entity_chunk_location(min_rect[1]),
        ];

        // remove/insert view chunk
        if Some(min_rect) != self.min_rect {
            let mut chunk_locations = vec![];
            for (chunk_location, _) in &self.up_chunks {
                let is_out_of_range_x =
                    chunk_location.x < min_rect[0].x || min_rect[1].x < chunk_location.x;
                let is_out_of_range_y =
                    chunk_location.y < min_rect[0].y || min_rect[1].y < chunk_location.y;
                if is_out_of_range_x || is_out_of_range_y {
                    chunk_locations.push(*chunk_location);
                }
            }
            for chunk_location in chunk_locations {
                let up_chunk = self.up_chunks.remove(&chunk_location).unwrap();

                rendering_server.instance_set_visible(up_chunk.instance, false);

                self.down_chunks.push(up_chunk.down());
            }

            for y in min_rect[0].y..=min_rect[1].y {
                for x in min_rect[0].x..=min_rect[1].x {
                    let chunk_location = IVec2::new(x, y);

                    if self.up_chunks.contains_key(&chunk_location) {
                        continue;
                    }

                    let Some(down_chunk) = self.down_chunks.pop() else {
                        let up = self.up_chunks.len();
                        let down = self.down_chunks.len();
                        panic!("no chunk available in pool (up:{}, down:{})", up, down);
                    };

                    rendering_server.instance_set_visible(down_chunk.instance, true);

                    self.up_chunks.insert(chunk_location, down_chunk.up());
                }
            }

            self.min_rect = Some(min_rect);
        }

        // update view chunk

        for (chunk_location, up_chunk) in &mut self.up_chunks {
            let Ok(version) = dataflow.get_entity_version_by_chunk_location(*chunk_location) else {
                continue;
            };

            for material in &up_chunk.materials {
                rendering_server.material_set_param(
                    *material,
                    "tick",
                    &(dataflow.get_tick() as i32).to_variant(),
                );
            }

            if version <= up_chunk.version {
                continue;
            }

            let mut instance_buffer = [0.0; Self::MAX_BUFFER_SIZE * 12];
            let mut head_buffer = [0; Self::MAX_BUFFER_SIZE * 4];

            let entity_keys = dataflow
                .get_entity_keys_by_chunk_location(*chunk_location)
                .unwrap();
            for (i, entity_key) in entity_keys.take(Self::MAX_BUFFER_SIZE).enumerate() {
                let entity = dataflow.get_entity(entity_key).unwrap();
                let prop = &self.props[entity.id as usize];

                instance_buffer[i * 12] = prop.rendering_size.x;
                instance_buffer[i * 12 + 1] = 0.0;
                instance_buffer[i * 12 + 2] = 0.0;
                instance_buffer[i * 12 + 3] = entity.location.x + prop.rendering_offset.x;

                instance_buffer[i * 12 + 4] = 0.0;
                instance_buffer[i * 12 + 5] = prop.rendering_size.y;
                instance_buffer[i * 12 + 6] = 0.0;
                instance_buffer[i * 12 + 7] = entity.location.y + prop.rendering_offset.y;

                let z_scale = if prop.z_along_y {
                    prop.rendering_size.y
                } else {
                    0.0
                };
                instance_buffer[i * 12 + 8] = 0.0;
                instance_buffer[i * 12 + 9] = 0.0;
                instance_buffer[i * 12 + 10] = z_scale;
                instance_buffer[i * 12 + 11] = 0.0;

                let image_head =
                    &self.image_heads[entity.id as usize][entity.render_param.variant as usize];
                head_buffer[i * 4] = image_head.start_texcoord_id;
                head_buffer[i * 4 + 1] = image_head.end_texcoord_id;
                head_buffer[i * 4 + 2] =
                    image_head.step_tick as u32 | ((image_head.is_loop as u32) << 16);
                head_buffer[i * 4 + 3] = entity.render_param.tick;
            }

            let head_buffer: &[i32] = unsafe { std::mem::transmute(head_buffer.as_slice()) };

            rendering_server.multimesh_set_buffer(
                up_chunk.multimesh,
                &PackedFloat32Array::from(instance_buffer.as_slice()),
            );

            for material in &up_chunk.materials {
                rendering_server.material_set_param(
                    *material,
                    "head_buffer",
                    &PackedInt32Array::from(head_buffer).to_variant(),
                );
            }

            up_chunk.version = version;
        }
    }
}

impl Drop for EntityField {
    fn drop(&mut self) {
        let mut rendering_server = godot::classes::RenderingServer::singleton();
        for free_handle in &self.free_handles {
            rendering_server.free_rid(*free_handle);
        }
    }
}
