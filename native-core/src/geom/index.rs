use glam::*;

use super::*;

pub type Id = u64;

const BLOCK_SIZE: i32 = 32;
const BLOCK_LEN: usize = (BLOCK_SIZE * BLOCK_SIZE) as usize;

#[inline]
fn encode_coord(coord: IVec2) -> u64 {
    (coord.x as u32 as u64) << 32 | coord.y as u32 as u64
}

#[derive(Debug, Default)]
pub struct SpatialIndex {
    pages: ahash::AHashMap<u64, [Option<Id>; BLOCK_LEN]>,
}

impl SpatialIndex {
    pub fn insert(&mut self, rect: IRect2, value: Id) {
        let min = rect.min.div_euclid(IVec2::splat(BLOCK_SIZE));
        let max = rect.max.div_euclid(IVec2::splat(BLOCK_SIZE));

        for y in min.y..=max.y {
            for x in min.x..=max.x {
                let coord = IVec2::new(x, y);
                let pages = self.pages.entry(encode_coord(coord)).or_insert_with(|| [None; BLOCK_LEN]);
                let min = (rect.min - coord * BLOCK_SIZE).clamp(IVec2::ZERO, IVec2::splat(BLOCK_SIZE - 1));
                let max = (rect.max - coord * BLOCK_SIZE).clamp(IVec2::ZERO, IVec2::splat(BLOCK_SIZE - 1));
                for v in min.y..=max.y {
                    for u in min.x..=max.x {
                        pages[(u + v * BLOCK_SIZE) as usize] = Some(value);
                    }
                }
            }
        }
    }

    pub fn remove(&mut self, rect: IRect2) {
        let min = rect.min.div_euclid(IVec2::splat(BLOCK_SIZE));
        let max = rect.max.div_euclid(IVec2::splat(BLOCK_SIZE));

        for y in min.y..=max.y {
            for x in min.x..=max.x {
                let coord = IVec2::new(x, y);
                if let Some(pages) = self.pages.get_mut(&encode_coord(coord)) {
                    let min = (rect.min - coord * BLOCK_SIZE).clamp(IVec2::ZERO, IVec2::splat(BLOCK_SIZE - 1));
                    let max = (rect.max - coord * BLOCK_SIZE).clamp(IVec2::ZERO, IVec2::splat(BLOCK_SIZE - 1));
                    for v in min.y..=max.y {
                        for u in min.x..=max.x {
                            pages[(u + v * BLOCK_SIZE) as usize] = None;
                        }
                    }
                }
            }
        }
    }

    pub fn find_point(&self, point: IVec2) -> Option<&Id> {
        let coord = point.div_euclid(IVec2::splat(BLOCK_SIZE));
        let page = self.pages.get(&encode_coord(coord))?;
        let coord = (point - coord * BLOCK_SIZE).clamp(IVec2::ZERO, IVec2::splat(BLOCK_SIZE - 1));
        page[(coord.x + coord.y * BLOCK_SIZE) as usize].as_ref()
    }

    pub fn find_rect(&self, rect: IRect2) -> impl Iterator<Item = &Id> {
        let min = rect.min.div_euclid(IVec2::splat(BLOCK_SIZE));
        let max = rect.max.div_euclid(IVec2::splat(BLOCK_SIZE));

        (min.y..=max.y).flat_map(move |y| {
            (min.x..=max.x).filter_map(move |x| {
                let coord = IVec2::new(x, y);
                self.pages.get(&encode_coord(coord)).map(|pages| (coord, pages))
            })
        })
            .flat_map(move |(coord, pages)| {
                let min = (rect.min - coord * BLOCK_SIZE).clamp(IVec2::ZERO, IVec2::splat(BLOCK_SIZE - 1));
                let max = (rect.max - coord * BLOCK_SIZE).clamp(IVec2::ZERO, IVec2::splat(BLOCK_SIZE - 1));
                (min.y..=max.y).flat_map(move |v| {
                    (min.x..=max.x).flat_map(move |u| {
                        pages[(u + v * BLOCK_SIZE) as usize].iter()
                    })
                })
            })
    }
}
