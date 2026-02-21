use glam::*;

use super::*;

const SIZE_SM: i32 = 8;
const SIZE_MD: i32 = 32;
const SIZE_LG: i32 = 128;
const SIZE_CO: i32 = SIZE_MD / SIZE_SM;

#[inline]
fn encode_coord(coord: IVec2) -> u64 {
    (coord.x as u32 as u64) << 32 | coord.y as u32 as u64
}

#[derive(Debug)]
struct Cell<T> {
    index: ahash::AHashMap<u64, u32>,
    data: slab::Slab<(u64, T)>
}

impl<T> Default for Cell<T> {
    fn default() -> Self {
        Self {
            index: Default::default(),
            data: Default::default(),
        }
    }
}

impl<T> Cell<T> {
    #[inline]
    fn insert(&mut self, key: u64, value: T) {
        let id = self.data.insert((key, value)) as u32;
        self.index.insert(key, id);
    }

    #[inline]
    fn remove(&mut self, key: u64) {
        let id = self.index.remove(&key).unwrap();
        self.data.remove(id as usize);
    }

    #[inline]
    fn iter(&self) -> impl Iterator<Item = &(u64, T)> + '_ {
        self.data.iter().map(|(_, v)| v)
    }
}

#[derive(Debug)]
pub struct HGrid<T> {
    cells_co: ahash::AHashMap<u64, [Cell<T>; (1 + SIZE_CO * SIZE_CO) as usize]>,
    cells_lg: ahash::AHashMap<u64, Cell<T>>,
}

impl<T> Default for HGrid<T> {
    fn default() -> Self {
        Self {
            cells_co: Default::default(),
            cells_lg: Default::default(),
        }
    }
}

impl<T> HGrid<T> where T: Clone {
    pub fn insert(&mut self, rect: IRect2, key: u64, value: T) {
        let size = rect.size().max_element();

        match size {
            // small
            ..SIZE_SM => {
                let min = rect.min.div_euclid(IVec2::splat(SIZE_MD));
                let max = rect.max.div_euclid(IVec2::splat(SIZE_MD));
                GridRange::new(min, max).for_each(|coord| {
                    let coord_ = encode_coord(coord);
                    let [_, cells @ ..] = self.cells_co.entry(coord_).or_default();
                    let min = (rect.min.div_euclid(IVec2::splat(SIZE_SM)) - coord * SIZE_CO).clamp(IVec2::ZERO, IVec2::splat(SIZE_CO - 1));
                    let max = (rect.max.div_euclid(IVec2::splat(SIZE_SM)) - coord * SIZE_CO).clamp(IVec2::ZERO, IVec2::splat(SIZE_CO - 1));
                    GridRange::new(min, max).for_each(|coord| {
                        let index = coord.x + coord.y * SIZE_CO;
                        cells[index as usize].insert(key, value.clone());
                    });
                });
            }
            // medium
            SIZE_SM..SIZE_MD => {
                let min = rect.min.div_euclid(IVec2::splat(SIZE_MD));
                let max = rect.max.div_euclid(IVec2::splat(SIZE_MD));
                GridRange::new(min, max).for_each(|coord| {
                    let coord_ = encode_coord(coord);
                    let [cell, ..] = self.cells_co.entry(coord_).or_default();
                    cell.insert(key, value.clone());
                });
            }
            // large
            SIZE_MD.. => {
                let min = rect.min.div_euclid(IVec2::splat(SIZE_LG));
                let max = rect.max.div_euclid(IVec2::splat(SIZE_LG));
                GridRange::new(min, max).for_each(|coord| {
                    let coord_ = encode_coord(coord);
                    let cell = self.cells_lg.entry(coord_).or_default();
                    cell.insert(key, value.clone());
                });
            }
        }
    }

    pub fn remove(&mut self, rect: IRect2, key: u64) {
        let size = rect.size().max_element();

        match size {
            // small
            ..SIZE_SM => {
                let min = rect.min.div_euclid(IVec2::splat(SIZE_MD));
                let max = rect.max.div_euclid(IVec2::splat(SIZE_MD));
                GridRange::new(min, max).for_each(|coord| {
                    let coord_ = encode_coord(coord);
                    if let Some([_, cells @ ..]) = self.cells_co.get_mut(&coord_) {
                        let min = (rect.min.div_euclid(IVec2::splat(SIZE_SM)) - coord * SIZE_CO).clamp(IVec2::ZERO, IVec2::splat(SIZE_CO - 1));
                        let max = (rect.max.div_euclid(IVec2::splat(SIZE_SM)) - coord * SIZE_CO).clamp(IVec2::ZERO, IVec2::splat(SIZE_CO - 1));
                        GridRange::new(min, max).for_each(|coord| {
                            let index = coord.x + coord.y * SIZE_CO;
                            cells[index as usize].remove(key);
                        });
                    }
                });
            }
            // medium
            SIZE_SM..SIZE_MD => {
                let min = rect.min.div_euclid(IVec2::splat(SIZE_MD));
                let max = rect.max.div_euclid(IVec2::splat(SIZE_MD));
                GridRange::new(min, max).for_each(|coord| {
                    let coord_ = encode_coord(coord);
                    if let Some([cell, ..]) = self.cells_co.get_mut(&coord_) {
                        cell.remove(key);
                    }
                });
            }
            // large
            SIZE_MD.. => {
                let min = rect.min.div_euclid(IVec2::splat(SIZE_LG));
                let max = rect.max.div_euclid(IVec2::splat(SIZE_LG));
                GridRange::new(min, max).for_each(|coord| {
                    let coord_ = encode_coord(coord);
                    if let Some(cell) = self.cells_lg.get_mut(&coord_) {
                        cell.remove(key);
                    }
                });
            }
        }
    }

    #[inline]
    pub fn check_move(&self, rect: IRect2, new_rect: IRect2) -> bool {
        assert_eq!(rect.size(), new_rect.size(), "Rect size must be same.");

        let size = rect.size().max_element();
        let chunk_size = match size {
            // small
            ..SIZE_SM => SIZE_SM,
            // medium
            SIZE_SM..SIZE_MD => SIZE_MD,
            // large
            SIZE_MD.. => SIZE_LG,
        };

        let min = rect.min.div_euclid(IVec2::splat(chunk_size));
        let max = rect.max.div_euclid(IVec2::splat(chunk_size));
        let new_min = new_rect.min.div_euclid(IVec2::splat(chunk_size));
        let new_max = new_rect.max.div_euclid(IVec2::splat(chunk_size));

        min != new_min || max != new_max
    }

    pub fn find(&self, rect: IRect2) -> impl Iterator<Item = &(u64, T)> + '_ {
        // small and medium
        let min = rect.min.div_euclid(IVec2::splat(SIZE_MD));
        let max = rect.max.div_euclid(IVec2::splat(SIZE_MD));
        let iter_co = GridRange::new(min, max)
            .filter_map(|coord| {
                let coord_ = encode_coord(coord);
                self.cells_co.get(&coord_).map(|cells| (coord, cells))
            })
            .flat_map(move |(coord, [cell, cells @ ..])| {
                let min = (rect.min.div_euclid(IVec2::splat(SIZE_SM)) - coord * SIZE_CO).clamp(IVec2::ZERO, IVec2::splat(SIZE_CO - 1));
                let max = (rect.max.div_euclid(IVec2::splat(SIZE_SM)) - coord * SIZE_CO).clamp(IVec2::ZERO, IVec2::splat(SIZE_CO - 1));
                GridRange::new(min, max)
                    .flat_map(|coord| {
                        let index = coord.x + coord.y * SIZE_CO;
                        cells[index as usize].iter()
                    })
                    .chain(cell.iter())
            });
        // large
        let min = rect.min.div_euclid(IVec2::splat(SIZE_LG));
        let max = rect.max.div_euclid(IVec2::splat(SIZE_LG));
        let iter_lg = GridRange::new(min, max)
            .filter_map(|coord| {
                let coord_ = encode_coord(coord);
                self.cells_lg.get(&coord_)
            })
            .flat_map(|cell| cell.iter());

        iter_co.chain(iter_lg)
    }
}

pub struct GridRange {
    current: IVec2,
    min_x: i32,
    max_x: i32,
    max_y: i32,
}

impl GridRange {
    #[inline]
    pub fn new(min: IVec2, max: IVec2) -> Self {
        if min.x > max.x || min.y > max.y {
            panic!("min must be less than or equal to max");
        }
        
        Self {
            current: min,
            min_x: min.x,
            max_x: max.x,
            max_y: max.y,
        }
    }
}

impl Iterator for GridRange {
    type Item = IVec2;

    #[inline] 
    fn next(&mut self) -> Option<Self::Item> {
        if self.current.y > self.max_y {
            return None;
        }
        let result = self.current;

        self.current.x += 1;
        if self.current.x > self.max_x {
            self.current.x = self.min_x;
            self.current.y += 1;
        }

        Some(result)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.current.y > self.max_y {
            return (0, Some(0));
        }
        
        let num_rows = (self.max_y - self.current.y) as usize;
        let width = (self.max_x - self.min_x + 1) as usize;
        let num_cols = (self.max_x - self.current.x + 1) as usize;
        
        let count = num_rows * width + num_cols;
        (count, Some(count))
    }
}
