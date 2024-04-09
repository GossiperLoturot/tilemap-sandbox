use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct Tile {
    id: u32,
    x: i32,
    y: i32,
}

#[godot_api]
impl Tile {
    #[func]
    pub fn from(id: u32, x: i32, y: i32) -> Gd<Self> {
        Gd::from_init_fn(|_| Self { id, x, y })
    }

    #[func]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[func]
    pub fn x(&self) -> i32 {
        self.x
    }

    #[func]
    pub fn y(&self) -> i32 {
        self.y
    }
}

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
struct TileField {
    tiles: slab::Slab<Gd<Tile>>,
    index: ahash::HashMap<(i32, i32), u32>,
}

#[godot_api]
impl TileField {
    #[func]
    pub fn add_tile(&mut self, tile: Gd<Tile>) -> Vector2i {
        let key = (tile.bind().x, tile.bind().y);
        if !self.index.contains_key(&key) {
            let idx = self.tiles.insert(tile);
            self.index.insert(key, idx as u32);
        }
        Vector2i::new(key.0, key.1)
    }

    #[func]
    pub fn remove_tile(&mut self, key: Vector2i) {
        let key = (key.x, key.y);
        if self.index.contains_key(&key) {
            let idx = self.index.remove(&key).unwrap();
            self.tiles.remove(idx as usize);
        }
    }

    #[func]
    pub fn update_buffer(
        &self,
        multimesh: Gd<godot::engine::MultiMesh>,
        material: Gd<godot::engine::Material>,
        atlas: Gd<super::atlas::Atlas>,
    ) {
        let mut instance_buffer = vec![];
        let mut texcoord_buffer = vec![];
        let mut page_buffer = vec![];

        for (_, tile) in self.tiles.iter() {
            instance_buffer.push(1.0);
            instance_buffer.push(0.0);
            instance_buffer.push(0.0);
            instance_buffer.push(tile.bind().x as f32);

            instance_buffer.push(0.0);
            instance_buffer.push(1.0);
            instance_buffer.push(0.0);
            instance_buffer.push(tile.bind().y as f32);

            instance_buffer.push(0.0);
            instance_buffer.push(0.0);
            instance_buffer.push(1.0);
            instance_buffer.push(0.0);

            let texcoord = atlas.bind().texcoords().get(tile.bind().id() as usize);
            texcoord_buffer.push(texcoord.bind().min_x());
            texcoord_buffer.push(texcoord.bind().min_y());
            texcoord_buffer.push(texcoord.bind().max_x() - texcoord.bind().min_x());
            texcoord_buffer.push(texcoord.bind().max_y() - texcoord.bind().min_y());

            page_buffer.push(texcoord.bind().page() as f32);
        }

        let mut rendering_server = godot::engine::RenderingServer::singleton();

        let buffer = PackedFloat32Array::from(instance_buffer.as_slice());
        rendering_server.multimesh_set_buffer(multimesh.get_rid(), buffer);

        let buffer = PackedFloat32Array::from(texcoord_buffer.as_slice()).to_variant();
        rendering_server.material_set_param(material.get_rid(), "texcoord_buffer".into(), buffer);

        let buffer = PackedFloat32Array::from(page_buffer.as_slice()).to_variant();
        rendering_server.material_set_param(material.get_rid(), "page_buffer".into(), buffer);
    }
}
