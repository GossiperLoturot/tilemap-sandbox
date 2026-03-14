use glam::*;

use super::*;

const BLOCK_SIZE: i32 = 8;
const CHUNK_SIZE: i32 = 32;
const DIV_SIZE: i32 = CHUNK_SIZE / BLOCK_SIZE;
const BLOCK_LEN: usize = (1 + DIV_SIZE * DIV_SIZE) as usize;

#[inline]
fn encode_coord(coord: IVec2) -> u64 {
    (coord.x as u32 as u64) << 32 | coord.y as u32 as u64
}

#[derive(Debug)]
struct Cell<T> {
    keys: Vec<u64>,
    values: Vec<T>
}

impl<T> Default for Cell<T> {
    fn default() -> Self {
        Self {
            keys: Default::default(),
            values: Default::default(),
        }
    }
}

impl<T> Cell<T> {
    #[inline]
    fn insert(&mut self, key: u64, value: T) {
        self.keys.push(key);
        self.values.push(value);
    }

    #[inline]
    fn remove(&mut self, key: u64) {
        if let Some(i) = self.keys.iter().position(|k| *k == key) {
            self.keys.swap_remove(i);
            self.values.swap_remove(i);
        }
    }

    #[inline]
    fn iter(&self) -> impl Iterator<Item = (&u64, &T)> {
        Iterator::zip(self.keys.iter(), self.values.iter())
    }
}

#[derive(Debug)]
pub struct HGrid<T> {
    cells: ahash::AHashMap<u64, [Cell<T>; BLOCK_LEN]>,
}

impl<T> Default for HGrid<T> {
    fn default() -> Self {
        Self {
            cells: Default::default(),
        }
    }
}

impl<T> HGrid<T> where T: Clone {
    pub fn insert(&mut self, rect: IRect2, key: u64, value: T) {
        let size = rect.size().max_element();

        match size {
            ..BLOCK_SIZE => {
                let min = rect.min.div_euclid(IVec2::splat(CHUNK_SIZE));
                let max = rect.max.div_euclid(IVec2::splat(CHUNK_SIZE));
                for y in min.y..=max.y {
                    for x in min.x..=max.x {
                        let coord = IVec2::new(x, y);
                        let coord_ = encode_coord(coord);
                        let [_, cells @ ..] = self.cells.entry(coord_).or_default();
                        let min = (rect.min.div_euclid(IVec2::splat(BLOCK_SIZE)) - coord * DIV_SIZE).clamp(IVec2::ZERO, IVec2::splat(DIV_SIZE - 1));
                        let max = (rect.max.div_euclid(IVec2::splat(BLOCK_SIZE)) - coord * DIV_SIZE).clamp(IVec2::ZERO, IVec2::splat(DIV_SIZE - 1));
                        for v in min.y..=max.y {
                            for u in min.x..=max.x {
                                cells[(u + v * DIV_SIZE) as usize].insert(key, value.clone());
                            }
                        }
                    }
                }
            }
            BLOCK_SIZE.. => {
                let min = rect.min.div_euclid(IVec2::splat(CHUNK_SIZE));
                let max = rect.max.div_euclid(IVec2::splat(CHUNK_SIZE));
                for y in min.y..=max.y {
                    for x in min.x..=max.x {
                        let coord = IVec2::new(x, y);
                        let coord_ = encode_coord(coord);
                        let [cell, ..] = self.cells.entry(coord_).or_default();
                        cell.insert(key, value.clone());
                    }
                }
            }
        }
    }

    pub fn remove(&mut self, rect: IRect2, key: u64) {
        let size = rect.size().max_element();

        match size {
            ..BLOCK_SIZE => {
                let min = rect.min.div_euclid(IVec2::splat(CHUNK_SIZE));
                let max = rect.max.div_euclid(IVec2::splat(CHUNK_SIZE));
                for y in min.y..=max.y {
                    for x in min.x..=max.x {
                        let coord = IVec2::new(x, y);
                        let coord_ = encode_coord(coord);
                        if let Some([_, cells @ ..]) = self.cells.get_mut(&coord_) {
                            let min = (rect.min.div_euclid(IVec2::splat(BLOCK_SIZE)) - coord * DIV_SIZE).clamp(IVec2::ZERO, IVec2::splat(DIV_SIZE - 1));
                            let max = (rect.max.div_euclid(IVec2::splat(BLOCK_SIZE)) - coord * DIV_SIZE).clamp(IVec2::ZERO, IVec2::splat(DIV_SIZE - 1));
                            for v in min.y..=max.y {
                                for u in min.x..=max.x {
                                    cells[(u + v * DIV_SIZE) as usize].remove(key);
                                }
                            }
                        }
                    }
                }
            }
            BLOCK_SIZE.. => {
                let min = rect.min.div_euclid(IVec2::splat(CHUNK_SIZE));
                let max = rect.max.div_euclid(IVec2::splat(CHUNK_SIZE));
                for y in min.y..=max.y {
                    for x in min.x..=max.x {
                        let coord = IVec2::new(x, y);
                        let coord_ = encode_coord(coord);
                        if let Some([cell, ..]) = self.cells.get_mut(&coord_) {
                            cell.remove(key);
                        }
                    }
                }
            }
        }
    }

    #[inline]
    pub fn check_move(&self, rect: IRect2, new_rect: IRect2) -> bool {
        assert_eq!(rect.size(), new_rect.size(), "Rect size must be same.");

        let size = rect.size().max_element();
        let chunk_size = match size { ..BLOCK_SIZE => BLOCK_SIZE, BLOCK_SIZE.. => CHUNK_SIZE };

        let min = rect.min.div_euclid(IVec2::splat(chunk_size));
        let max = rect.max.div_euclid(IVec2::splat(chunk_size));
        let new_min = new_rect.min.div_euclid(IVec2::splat(chunk_size));
        let new_max = new_rect.max.div_euclid(IVec2::splat(chunk_size));

        min != new_min || max != new_max
    }

    pub fn find(&self, rect: IRect2) -> impl Iterator<Item = (&u64, &T)> {
        let min = rect.min.div_euclid(IVec2::splat(CHUNK_SIZE));
        let max = rect.max.div_euclid(IVec2::splat(CHUNK_SIZE));
        (min.y..=max.y).flat_map(move |y| {
            (min.x..=max.x).filter_map(move |x| {
                let coord = IVec2::new(x, y);
                let coord_ = encode_coord(coord);
                self.cells.get(&coord_).map(|cells| (coord, cells))
            })
        })
            .flat_map(move |(coord, [cell, cells @ ..])| {
                let min = (rect.min.div_euclid(IVec2::splat(BLOCK_SIZE)) - coord * DIV_SIZE).clamp(IVec2::ZERO, IVec2::splat(DIV_SIZE - 1));
                let max = (rect.max.div_euclid(IVec2::splat(BLOCK_SIZE)) - coord * DIV_SIZE).clamp(IVec2::ZERO, IVec2::splat(DIV_SIZE - 1));
                let cells_iter = (min.y..=max.y).flat_map(move |v| {
                    (min.x..=max.x).flat_map(move |u| {
                        cells[(u + v * DIV_SIZE) as usize].iter()
                    })
                });
                Iterator::chain(cell.iter(), cells_iter)
            })
    }
}
