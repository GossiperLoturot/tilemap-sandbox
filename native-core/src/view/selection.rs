use glam::*;
use godot::prelude::*;

use crate::dataflow;

pub struct SelectionInfo {
    pub shader: Gd<godot::classes::Shader>,
    pub world: Gd<godot::classes::World3D>,
}

pub struct Selection {
    multimesh: Rid,
    free_handles: Vec<Rid>,
    instance_buffer: Vec<f32>,
}

impl Selection {
    const BUFFER_LEN: usize = 64;

    pub fn new(info: SelectionInfo) -> Self {
        let mut rendering_server = godot::classes::RenderingServer::singleton();

        let mut free_handles = vec![];

        let mut mesh_data = VarArray::new();
        mesh_data.resize(
            godot::classes::rendering_server::ArrayType::MAX.ord() as usize,
            &Variant::nil(),
        );
        mesh_data.set(
            godot::classes::rendering_server::ArrayType::VERTEX.ord() as usize,
            &PackedVector3Array::from(&[Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 1.0), Vector3::new(1.0, 1.0, 1.0), Vector3::new(1.0, 0.0, 0.0)]).to_variant(),
        );
        mesh_data.set(
            godot::classes::rendering_server::ArrayType::TEX_UV.ord() as usize,
            &PackedVector2Array::from(&[Vector2::new(0.0, 1.0), Vector2::new(0.0, 0.0), Vector2::new(1.0, 0.0), Vector2::new(1.0, 1.0)])
            .to_variant(),
        );
        mesh_data.set(
            godot::classes::rendering_server::ArrayType::INDEX.ord() as usize,
            &PackedInt32Array::from(&[0, 1, 2, 0, 2, 3]).to_variant(),
        );

        let material = rendering_server.material_create();
        rendering_server.material_set_shader(material, info.shader.get_rid());
        free_handles.push(material);

        let mesh = rendering_server.mesh_create();
        rendering_server.mesh_add_surface_from_arrays(mesh, godot::classes::rendering_server::PrimitiveType::TRIANGLES, &mesh_data);
        rendering_server.mesh_surface_set_material(mesh, 0, material);
        free_handles.push(mesh);

        let multimesh = rendering_server.multimesh_create();
        rendering_server.multimesh_set_mesh(multimesh, mesh);
        rendering_server.multimesh_allocate_data(multimesh, Self::BUFFER_LEN as i32, godot::classes::rendering_server::MultimeshTransformFormat::TRANSFORM_3D);
        free_handles.push(multimesh);

        let instance = rendering_server.instance_create2(multimesh, info.world.get_scenario());
        free_handles.push(instance);

        Self {
            multimesh,
            free_handles,
            instance_buffer: vec![0.0; Self::BUFFER_LEN * 12],
        }
    }

    pub fn update_view(
        &mut self,
        dataflow: &dataflow::Dataflow,
        tile_keys: &[dataflow::TileKey],
        block_keys: &[dataflow::BlockKey],
        entity_keys: &[dataflow::EntityKey],
    ) {
        let mut rendering_server = godot::classes::RenderingServer::singleton();

        let mut count = 0;

        for tile_key in tile_keys {
            let tile = dataflow.get_tile(*tile_key).unwrap();

            self.instance_buffer[count * 12] = 1.0;
            self.instance_buffer[count * 12 + 1] = 0.0;
            self.instance_buffer[count * 12 + 2] = 0.0;
            self.instance_buffer[count * 12 + 3] = tile.location.x as f32;

            self.instance_buffer[count * 12 + 4] = 0.0;
            self.instance_buffer[count * 12 + 5] = 1.0;
            self.instance_buffer[count * 12 + 6] = 0.0;
            self.instance_buffer[count * 12 + 7] = tile.location.y as f32;

            self.instance_buffer[count * 12 + 8] = 0.0;
            self.instance_buffer[count * 12 + 9] = 0.0;
            self.instance_buffer[count * 12 + 10] = 0.0;
            self.instance_buffer[count * 12 + 11] = 0.0;

            count += 1;
        }

        for block_key in block_keys {
            let block = dataflow.get_block(*block_key).unwrap();
            let hint_rect = dataflow.get_block_base_hint_rect(block.id).unwrap();
            let y_sorting = dataflow.get_block_base_z_along_y(block.id).unwrap();

            let hint_offset = hint_rect[0];
            let hint_size = hint_rect[1] - hint_rect[0];

            self.instance_buffer[count * 12] = hint_size.x;
            self.instance_buffer[count * 12 + 1] = 0.0;
            self.instance_buffer[count * 12 + 2] = 0.0;
            self.instance_buffer[count * 12 + 3] = block.location.x as f32 + hint_offset.x;

            self.instance_buffer[count * 12 + 4] = 0.0;
            self.instance_buffer[count * 12 + 5] = hint_size.y;
            self.instance_buffer[count * 12 + 6] = 0.0;
            self.instance_buffer[count * 12 + 7] = block.location.y as f32 + hint_offset.y;

            let z_scale = if y_sorting { hint_size.y } else { 0.0 };
            self.instance_buffer[count * 12 + 8] = 0.0;
            self.instance_buffer[count * 12 + 9] = 0.0;
            self.instance_buffer[count * 12 + 10] = z_scale;
            self.instance_buffer[count * 12 + 11] = 0.0f32.next_up();

            count += 1;
        }

        for entity_key in entity_keys {
            let entity = dataflow.get_entity(*entity_key).unwrap();
            let hint_rect = dataflow.get_entity_base_hint_rect(entity.id).unwrap();
            let y_sorting = dataflow.get_entity_base_z_along_y(entity.id).unwrap();

            let hint_offset = hint_rect[0];
            let hint_size = hint_rect[1] - hint_rect[0];

            self.instance_buffer[count * 12] = hint_size.x;
            self.instance_buffer[count * 12 + 1] = 0.0;
            self.instance_buffer[count * 12 + 2] = 0.0;
            self.instance_buffer[count * 12 + 3] = entity.location.x + hint_offset.x;

            self.instance_buffer[count * 12 + 4] = 0.0;
            self.instance_buffer[count * 12 + 5] = hint_size.y;
            self.instance_buffer[count * 12 + 6] = 0.0;
            self.instance_buffer[count * 12 + 7] = entity.location.y + hint_offset.y;

            let z_scale = if y_sorting { hint_size.y } else { 0.0 };
            self.instance_buffer[count * 12 + 8] = 0.0;
            self.instance_buffer[count * 12 + 9] = 0.0;
            self.instance_buffer[count * 12 + 10] = z_scale;
            self.instance_buffer[count * 12 + 11] = 0.0f32.next_up();

            count += 1;
        }

        let instance_buffer = PackedFloat32Array::from(self.instance_buffer.as_slice());
        rendering_server.multimesh_set_buffer(self.multimesh, &instance_buffer);
        rendering_server.multimesh_set_visible_instances(self.multimesh, count as i32);
    }
}

impl Drop for Selection {
    fn drop(&mut self) {
        let mut rendering_server = godot::classes::RenderingServer::singleton();
        for free_handle in &self.free_handles {
            rendering_server.free_rid(*free_handle);
        }
    }
}
