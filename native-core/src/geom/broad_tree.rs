use glam::*;

use super::*;

const SIZE_SM: i32 = 8;
const SIZE_MD: i32 = 32;
const SIZE_LG: i32 = 128;
const SIZE_CO: i32 = SIZE_MD / SIZE_SM;

#[derive(Debug)]
struct Cell<K, V> {
    index: ahash::AHashMap<K, u32>,
    data: slab::Slab<(K, V)>
}

impl<K, V> Default for Cell<K, V> {
    #[inline]
    fn default() -> Self {
        Self {
            index: Default::default(),
            data: Default::default(),
        }
    }
}

impl<K, V> Cell<K, V> where K: Copy + std::hash::Hash + Eq {
    #[inline]
    fn insert(&mut self, key: K, value: V) {
        let id = self.data.insert((key, value)) as u32;
        self.index.insert(key, id);
    }

    #[inline]
    fn remove(&mut self, key: K) {
        let id = self.index.remove(&key).unwrap();
        self.data.remove(id as usize);
    }

    #[inline]
    fn iter(&self) -> impl Iterator<Item = &(K, V)> + '_ {
        self.data.iter().map(|(_, v)| v)
    }
}

#[derive(Debug)]
pub struct BroadTree<K, V> {
    cells_co: ahash::AHashMap<IVec2, [Cell<K, V>; (1 + SIZE_CO * SIZE_CO) as usize]>,
    cells_lg: ahash::AHashMap<IVec2, Cell<K, V>>,
}

impl<K, V> Default for BroadTree<K, V> {
    fn default() -> Self {
        Self {
            cells_co: Default::default(),
            cells_lg: Default::default(),
        }
    }
}

impl<K, V> BroadTree<K, V> where K: Copy + std::hash::Hash + Eq, V: Clone {
    pub fn insert(&mut self, rect: IRect2, key: K, value: V) {
        let size = rect.size().max_element();

        match size {
            // small
            ..SIZE_SM => {
                let min = rect.min.div_euclid(IVec2::splat(SIZE_MD));
                let max = rect.max.div_euclid(IVec2::splat(SIZE_MD));
                GridRange::new(min, max).for_each(|coord| {
                    let [_, cells @ ..] = self.cells_co.entry(coord).or_default();
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
                    let [cell, ..] = self.cells_co.entry(coord).or_default();
                    cell.insert(key, value.clone());
                });
            }
            // large
            SIZE_MD.. => {
                let min = rect.min.div_euclid(IVec2::splat(SIZE_LG));
                let max = rect.max.div_euclid(IVec2::splat(SIZE_LG));
                GridRange::new(min, max).for_each(|coord| {
                    let cell = self.cells_lg.entry(coord).or_default();
                    cell.insert(key, value.clone());
                });
            }
        }
    }

    pub fn remove(&mut self, rect: IRect2, key: K) {
        let size = rect.size().max_element();

        match size {
            // small
            ..SIZE_SM => {
                let min = rect.min.div_euclid(IVec2::splat(SIZE_MD));
                let max = rect.max.div_euclid(IVec2::splat(SIZE_MD));
                GridRange::new(min, max).for_each(|coord| {
                    if let Some([_, cells @ ..]) = self.cells_co.get_mut(&coord) {
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
                    if let Some([cell, ..]) = self.cells_co.get_mut(&coord) {
                        cell.remove(key);
                    }
                });
            }
            // large
            SIZE_MD.. => {
                let min = rect.min.div_euclid(IVec2::splat(SIZE_LG));
                let max = rect.max.div_euclid(IVec2::splat(SIZE_LG));
                GridRange::new(min, max).for_each(|coord| {
                    if let Some(cell) = self.cells_lg.get_mut(&coord) {
                        cell.remove(key);
                    }
                });
            }
        }
    }

    pub fn find(&self, rect: IRect2) -> impl Iterator<Item = &(K, V)> + '_ {
        // small and medium
        let min = rect.min.div_euclid(IVec2::splat(SIZE_MD));
        let max = rect.max.div_euclid(IVec2::splat(SIZE_MD));
        let iter_co = GridRange::new(min, max)
            .filter_map(|coord| self.cells_co.get(&coord).map(|cells| (coord, cells)))
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
            .filter_map(|coord| self.cells_lg.get(&coord))
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
