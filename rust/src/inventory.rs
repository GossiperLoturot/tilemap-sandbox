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
    pub fn new(_desc: InventoryDescriptor) -> Self {
        todo!()
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
