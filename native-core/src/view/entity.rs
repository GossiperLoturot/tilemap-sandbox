use glam::*;

use crate::dataflow;
use crate::geom::*;

pub struct EntitySpriteInfo {
    pub images: Vec<godot::obj::Gd<godot::classes::Image>>,
    pub ticks_per_image: u16,
    pub is_loop: bool,
}

pub struct EntityInfo {
    pub sprites: Vec<EntitySpriteInfo>,
    pub y_sorting: bool,
    pub rendering_rect: Rect2,
}

pub struct EntityFieldInfo {
    pub entities: Vec<EntityInfo>,
    pub shaders: Vec<godot::obj::Gd<godot::classes::Shader>>,
    pub world: godot::obj::Gd<godot::classes::World3D>,
}

struct ImageAddress {
    atlas_start_index: u32,
    atlas_end_index: u32,
    ticks_per_image: u16,
    is_loop: bool,
}

struct RenderLayout {
    y_sorting: bool,
    rendering_size: Vec2,
    rendering_offset: Vec2,
}

struct DeadPage {
    materials: Vec<godot::builtin::Rid>,
    multimesh: godot::builtin::Rid,
    instance: godot::builtin::Rid,
}

impl DeadPage {
    fn spawn(self) -> LivePage {
        LivePage {
            version: Default::default(),
            materials: self.materials,
            multimesh: self.multimesh,
            instance: self.instance,
        }
    }
}

struct LivePage {
    version: u64,
    materials: Vec<godot::builtin::Rid>,
    multimesh: godot::builtin::Rid,
    instance: godot::builtin::Rid,
}

impl LivePage {
    fn despawn(self) -> DeadPage {
        DeadPage {
            materials: self.materials,
            multimesh: self.multimesh,
            instance: self.instance,
        }
    }
}

pub struct EntityField {
    layouts: Vec<RenderLayout>,
    sprite_addrs: Vec<Vec<ImageAddress>>,
    dead_pages: Vec<DeadPage>,
    live_pages: ahash::AHashMap<IVec2, LivePage>,
    free_handles: Vec<godot::builtin::Rid>,
    rect: Option<IRect2>,
    instance_buffer: Vec<f32>,
    address_buffer: Vec<u32>,
}

impl EntityField {
    const PAGE_CAPACITY: usize = 512;
    const ATLAS_WIDTH: usize = 1024;
    const ATLAS_PAGE: usize = 8;
    const COORD_BUFFER_WIDTH: usize = 1024;
    const BUFFER_LEN: usize = 1024;

    pub fn new(info: EntityFieldInfo) -> Self {
        let mut rendering_server = <godot::classes::RenderingServer as godot::obj::Singleton>::singleton();

        let mut free_handles = vec![];

        let mut layouts = vec![];
        for entity in &info.entities {
            layouts.push(RenderLayout {
                y_sorting: entity.y_sorting,
                rendering_size: entity.rendering_rect.size(),
                rendering_offset: entity.rendering_rect.min,
            });
        }

        let mut sprite_addrs = vec![];
        let mut images = vec![];
        for entity in info.entities {
            let mut sprite_addr = vec![];

            for image in entity.sprites {
                if images.len() + image.images.len() >= i32::MAX as usize {
                    panic!("number of frame must be less than i32::MAX");
                }

                sprite_addr.push(ImageAddress {
                    atlas_start_index: images.len() as u32,
                    atlas_end_index: images.len() as u32 + image.images.len() as u32,
                    ticks_per_image: image.ticks_per_image,
                    is_loop: image.is_loop,
                });

                for image in image.images {
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

                    images.push(image_rgba8);
                }
            }

            sprite_addrs.push(sprite_addr);
        }

        let mut atlas_inputs = vec![];
        for image in images {
            atlas_inputs.push(image_atlas::AtlasEntry {
                texture: image,
                mip: image_atlas::AtlasEntryMipOption::Clamp,
            });
        }
        let atlas_output = image_atlas::create_atlas(&image_atlas::AtlasDescriptor {
            size: Self::ATLAS_WIDTH as u32,
            max_page_count: Self::ATLAS_PAGE as u32,
            mip: image_atlas::AtlasMipOption::NoMipWithPadding(1),
            entries: &atlas_inputs,
        })
        .unwrap();

        let mut images = vec![];
        for texture in &atlas_output.textures {
            let image = &texture.mip_maps[0];

            let image = godot::classes::Image::create_from_data(
                Self::ATLAS_WIDTH as i32,
                Self::ATLAS_WIDTH as i32,
                false,
                godot::classes::image::Format::RGBA8,
                &godot::builtin::PackedByteArray::from(image.to_vec()),
            )
            .unwrap();

            images.push(image);
        }

        let texture_array = rendering_server.texture_2d_layered_create(
            &godot::builtin::Array::from(images.as_slice()),
            godot::classes::rendering_server::TextureLayeredType::LAYERED_2D_ARRAY,
        );
        free_handles.push(texture_array);

        if atlas_output.texcoords.len() * 2 > Self::COORD_BUFFER_WIDTH * Self::COORD_BUFFER_WIDTH {
            panic!("number of (image * 2) must be less than (COORD_BUFFER_WIDTH ^ 2)");
        }
        let mut coord_buffer = vec![0.0; Self::COORD_BUFFER_WIDTH * Self::COORD_BUFFER_WIDTH * 4];
        for (i, texcoord) in atlas_output.texcoords.into_iter().enumerate() {
            let coord = texcoord.to_f32();
            coord_buffer[i * 8] = coord.min_x;
            coord_buffer[i * 8 + 1] = coord.min_y;
            coord_buffer[i * 8 + 2] = coord.max_x - coord.min_x;
            coord_buffer[i * 8 + 3] = coord.max_y - coord.min_y;
            coord_buffer[i * 8 + 4] = coord.page as f32;
            coord_buffer[i * 8 + 5] = 0.0;
            coord_buffer[i * 8 + 6] = 0.0;
            coord_buffer[i * 8 + 7] = 0.0;
        }
        let coord_buffer = bytemuck::cast_slice::<_, u8>(coord_buffer.as_slice());
        let coord_image = godot::classes::Image::create_from_data(
            Self::COORD_BUFFER_WIDTH as i32,
            Self::COORD_BUFFER_WIDTH as i32,
            false,
            godot::classes::image::Format::RGBAF,
            &godot::builtin::PackedByteArray::from(coord_buffer),
        )
        .unwrap();
        let coord_texture = rendering_server.texture_2d_create(&coord_image);
        free_handles.push(coord_texture);

        let mut mesh_data = godot::builtin::VarArray::new();
        mesh_data.resize(
            godot::obj::EngineEnum::ord(godot::classes::rendering_server::ArrayType::MAX) as usize,
            &godot::builtin::Variant::nil()
        );
        mesh_data.set(
            godot::obj::EngineEnum::ord(godot::classes::rendering_server::ArrayType::VERTEX) as usize,
            &godot::meta::ToGodot::to_variant(&godot::builtin::PackedVector3Array::from(&[
                godot::builtin::Vector3::new(0.0, 0.0, 0.0),
                godot::builtin::Vector3::new(0.0, 1.0, 1.0),
                godot::builtin::Vector3::new(1.0, 1.0, 1.0),
                godot::builtin::Vector3::new(1.0, 0.0, 0.0),
            ])),
        );
        mesh_data.set(
            godot::obj::EngineEnum::ord(godot::classes::rendering_server::ArrayType::TEX_UV) as usize,
            &godot::meta::ToGodot::to_variant(&godot::builtin::PackedVector2Array::from(&[
                godot::builtin::Vector2::new(0.0, 1.0),
                godot::builtin::Vector2::new(0.0, 0.0),
                godot::builtin::Vector2::new(1.0, 0.0),
                godot::builtin::Vector2::new(1.0, 1.0),
            ])),
        );
        mesh_data.set(
            godot::obj::EngineEnum::ord(godot::classes::rendering_server::ArrayType::INDEX) as usize,
            &godot::meta::ToGodot::to_variant(&godot::builtin::PackedInt32Array::from(&[0, 1, 2, 0, 2, 3])),
        );

        let mut dead_pages = vec![];
        for _ in 0..Self::PAGE_CAPACITY {
            let mut materials = vec![];
            for shader in &info.shaders {
                let material = rendering_server.material_create();
                rendering_server.material_set_shader(material, shader.get_rid());
                rendering_server.material_set_param(material, "texture_array", &godot::meta::ToGodot::to_variant(&texture_array));
                rendering_server.material_set_param(material, "bake_texture", &godot::meta::ToGodot::to_variant(&coord_texture));
                free_handles.push(material);

                materials.push(material)
            }

            for i in 0..materials.len() - 1 {
                let material = materials[i];
                let next_material = materials[i + 1];
                rendering_server.material_set_next_pass(material, next_material);
            }

            let mesh = rendering_server.mesh_create();
            rendering_server.mesh_add_surface_from_arrays(mesh, godot::classes::rendering_server::PrimitiveType::TRIANGLES, &mesh_data);
            rendering_server.mesh_surface_set_material(mesh, 0, materials[0]);
            free_handles.push(mesh);

            let multimesh = rendering_server.multimesh_create();
            rendering_server.multimesh_set_mesh(multimesh, mesh);
            rendering_server.multimesh_allocate_data(multimesh, Self::BUFFER_LEN as i32, godot::classes::rendering_server::MultimeshTransformFormat::TRANSFORM_3D);
            free_handles.push(multimesh);

            let instance = rendering_server.instance_create2(multimesh, info.world.get_scenario());
            rendering_server.instance_set_visible(instance, false);
            free_handles.push(instance);

            dead_pages.push(DeadPage {
                materials,
                multimesh,
                instance,
            });
        }

        Self {
            layouts,
            sprite_addrs,
            dead_pages,
            live_pages: Default::default(),
            free_handles,
            rect: None,
            instance_buffer: vec![0.0; Self::BUFFER_LEN * 12],
            address_buffer: vec![0; Self::BUFFER_LEN * 4],
        }
    }

    pub fn update_view(&mut self, dataflow: &dataflow::Dataflow, rect: Rect2) {
        let mut rendering_server = <godot::classes::RenderingServer as godot::obj::Singleton>::singleton();

        let rect = IRect2::new(
            dataflow.find_entity_page_coord(rect.min),
            dataflow.find_entity_page_coord(rect.max),
        );

        // remove / insert view page

        if Some(rect) != self.rect {
            let mut page_coords = vec![];
            for (page_coord, _) in &self.live_pages {
                if !Intersects::intersects(page_coord, &rect) {
                    page_coords.push(*page_coord);
                }
            }
            for page_coord in page_coords {
                let live_page = self.live_pages.remove(&page_coord).unwrap();

                rendering_server.instance_set_visible(live_page.instance, false);

                self.dead_pages.push(live_page.despawn());
            }

            for y in rect[0].y..=rect[1].y {
                for x in rect[0].x..=rect[1].x {
                    let page_coord = IVec2::new(x, y);

                    if self.live_pages.contains_key(&page_coord) {
                        continue;
                    }

                    let Some(dead_page) = self.dead_pages.pop() else {
                        let live_count = self.live_pages.len();
                        let dead_count = self.dead_pages.len();
                        panic!("no page available in pool (live:{}, dead:{})", live_count, dead_count);
                    };

                    rendering_server.instance_set_visible(dead_page.instance, true);

                    self.live_pages.insert(page_coord, dead_page.spawn());
                }
            }

            self.rect = Some(rect);
        }

        // update view page

        for (page_coord, live_page) in &mut self.live_pages {
            let Ok(page) = dataflow.get_entity_page(*page_coord) else {
                continue;
            };

            for material in &live_page.materials {
                let tick = dataflow.get_tick() as i32;
                rendering_server.material_set_param(*material, "tick", &godot::meta::ToGodot::to_variant(&tick));
            }

            if page.version <= live_page.version {
                continue;
            }

            let mut count = 0;
            for (i, entity) in page.entities.iter().take(Self::BUFFER_LEN).enumerate() {
                let layout = &self.layouts[entity.archetype_id as usize];

                self.instance_buffer[i * 12] = layout.rendering_size.x;
                self.instance_buffer[i * 12 + 1] = 0.0;
                self.instance_buffer[i * 12 + 2] = 0.0;
                self.instance_buffer[i * 12 + 3] = entity.coord.x + layout.rendering_offset.x;

                self.instance_buffer[i * 12 + 4] = 0.0;
                self.instance_buffer[i * 12 + 5] = layout.rendering_size.y;
                self.instance_buffer[i * 12 + 6] = 0.0;
                self.instance_buffer[i * 12 + 7] = entity.coord.y + layout.rendering_offset.y;

                let z_scale = if layout.y_sorting { layout.rendering_size.y } else { 0.0 };
                self.instance_buffer[i * 12 + 8] = 0.0;
                self.instance_buffer[i * 12 + 9] = 0.0;
                self.instance_buffer[i * 12 + 10] = z_scale;
                self.instance_buffer[i * 12 + 11] = 0.0;

                let image_addr = &self.sprite_addrs[entity.archetype_id as usize][entity.variant as usize];
                self.address_buffer[i * 4] = image_addr.atlas_start_index;
                self.address_buffer[i * 4 + 1] = image_addr.atlas_end_index;
                self.address_buffer[i * 4 + 2] = image_addr.ticks_per_image as u32 | ((image_addr.is_loop as u32) << 16);
                self.address_buffer[i * 4 + 3] = entity.tick;

                count += 1;
            }

            let instance_buffer = godot::builtin::PackedFloat32Array::from(self.instance_buffer.as_slice());
            rendering_server.multimesh_set_buffer(live_page.multimesh, &instance_buffer);
            rendering_server.multimesh_set_visible_instances(live_page.multimesh, count);

            let address_buffer = bytemuck::cast_slice::<_, i32>(self.address_buffer.as_slice());
            let address_buffer = godot::builtin::PackedInt32Array::from(address_buffer);
            for material in &live_page.materials {
                rendering_server.material_set_param(*material, "head_buffer", &godot::meta::ToGodot::to_variant(&address_buffer));
            }

            live_page.version = page.version;
        }
    }
}

impl Drop for EntityField {
    fn drop(&mut self) {
        let mut rendering_server = <godot::classes::RenderingServer as godot::obj::Singleton>::singleton();
        for free_handle in &self.free_handles {
            rendering_server.free_rid(*free_handle);
        }
    }
}
