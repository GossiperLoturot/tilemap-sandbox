use crate::inner;

#[derive(Debug, Clone)]
pub struct ForwardResource;

impl ForwardResource {
    pub fn init(root: &mut inner::Root) {
        let slf = Self;
        root.resource_insert(slf).unwrap();
    }

    pub fn forward_rect(root: &mut inner::Root, min_rect: [inner::Vec2; 2], delta_secs: f32) {
        let slf = root.resource_remove::<Self>().unwrap();

        // tile
        let chunk_size = root.tile_get_chunk_size() as f32;
        #[rustfmt::skip]
        let rect = [[
            min_rect[0][0].div_euclid(chunk_size) as i32,
            min_rect[0][1].div_euclid(chunk_size) as i32, ], [
            min_rect[1][0].div_euclid(chunk_size) as i32,
            min_rect[1][1].div_euclid(chunk_size) as i32,
        ]];
        for y in rect[0][1]..=rect[1][1] {
            for x in rect[0][0]..=rect[1][0] {
                let _ = root.tile_forward_chunk([x, y], delta_secs);
            }
        }

        // block
        let chunk_size = root.block_get_chunk_size() as f32;
        #[rustfmt::skip]
        let rect = [[
            min_rect[0][0].div_euclid(chunk_size) as i32,
            min_rect[0][1].div_euclid(chunk_size) as i32, ], [
            min_rect[1][0].div_euclid(chunk_size) as i32,
            min_rect[1][1].div_euclid(chunk_size) as i32,
        ]];
        for y in rect[0][1]..=rect[1][1] {
            for x in rect[0][0]..=rect[1][0] {
                let _ = root.block_forward_chunk([x, y], delta_secs);
            }
        }

        // entity
        let chunk_size = root.entity_get_chunk_size() as f32;
        #[rustfmt::skip]
        let rect = [[
            min_rect[0][0].div_euclid(chunk_size) as i32,
            min_rect[0][1].div_euclid(chunk_size) as i32, ], [
            min_rect[1][0].div_euclid(chunk_size) as i32,
            min_rect[1][1].div_euclid(chunk_size) as i32,
        ]];
        for y in rect[0][1]..=rect[1][1] {
            for x in rect[0][0]..=rect[1][0] {
                let _ = root.entity_forward_chunk([x, y], delta_secs);
            }
        }

        root.resource_insert(slf).unwrap();
    }
}
