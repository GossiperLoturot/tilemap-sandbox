use glam::*;

use super::*;

#[derive(Debug, Default)]
pub struct BroadTree<T> {
    grid_s: ahash::AHashMap<IVec2, ahash::AHashSet<T>>,
    grid_m: ahash::AHashMap<IVec2, ahash::AHashSet<T>>,
    grid_l: ahash::AHashMap<IVec2, ahash::AHashSet<T>>,
}

impl<T> BroadTree<T> where T: Copy + std::hash::Hash + Eq {
    const S_SIZE: u32 = 8;
    const M_SIZE: u32 = 32;
    const L_SIZE: u32 = 128;

    pub fn new() -> Self {
        Self {
            grid_s: Default::default(),
            grid_m: Default::default(),
            grid_l: Default::default(),
        }
    }

    pub fn insert(&mut self, rect: IRect2, data: T) {
        let size = rect.size().max_element() as u32;

        // small
        if size <= Self::S_SIZE {
            let min = rect.min.div_euclid(IVec2::splat(Self::S_SIZE as i32));
            let max = rect.max.div_euclid(IVec2::splat(Self::S_SIZE as i32));
            for y in min.y..=max.y {
                for x in min.x..=max.x {
                    let coord = IVec2::new(x, y);
                    let grid = self.grid_s.entry(coord).or_default();
                    grid.insert(data);
                }
            }
        // medium
        } else if size <= Self::M_SIZE {
            let min = rect.min.div_euclid(IVec2::splat(Self::M_SIZE as i32));
            let max = rect.max.div_euclid(IVec2::splat(Self::M_SIZE as i32));
            for y in min.y..=max.y {
                for x in min.x..=max.x {
                    let coord = IVec2::new(x, y);
                    let grid = self.grid_m.entry(coord).or_default();
                    grid.insert(data);
                }
            }
        // large
        } else {
            let min = rect.min.div_euclid(IVec2::splat(Self::L_SIZE as i32));
            let max = rect.max.div_euclid(IVec2::splat(Self::L_SIZE as i32));
            for y in min.y..=max.y {
                for x in min.x..=max.x {
                    let coord = IVec2::new(x, y);
                    let grid = self.grid_l.entry(coord).or_default();
                    grid.insert(data);
                }
            }
        }
    }

    pub fn remove(&mut self, rect: IRect2, data: T) {
        let size = rect.size().max_element() as u32;

        // small
        if size <= Self::S_SIZE {
            let min = rect.min.div_euclid(IVec2::splat(Self::S_SIZE as i32));
            let max = rect.max.div_euclid(IVec2::splat(Self::S_SIZE as i32));
            for y in min.y..=max.y {
                for x in min.x..=max.x {
                    let coord = IVec2::new(x, y);
                    self.grid_s.get_mut(&coord).map(|grid| grid.remove(&data));
                }
            }
        // medium
        } else if size <= Self::M_SIZE {
            let min = rect.min.div_euclid(IVec2::splat(Self::M_SIZE as i32));
            let max = rect.max.div_euclid(IVec2::splat(Self::M_SIZE as i32));
            for y in min.y..=max.y {
                for x in min.x..=max.x {
                    let coord = IVec2::new(x, y);
                    self.grid_m.get_mut(&coord).map(|grid| grid.remove(&data));
                }
            }
        // large
        } else {
            let min = rect.min.div_euclid(IVec2::splat(Self::L_SIZE as i32));
            let max = rect.max.div_euclid(IVec2::splat(Self::L_SIZE as i32));
            for y in min.y..=max.y {
                for x in min.x..=max.x {
                    let coord = IVec2::new(x, y);
                    self.grid_l.get_mut(&coord).map(|grid| grid.remove(&data));
                }
            }
        }
    }

    pub fn find(&self, rect: IRect2) -> impl Iterator<Item = T> + '_ {
        // small
        let min = rect.min.div_euclid(IVec2::splat(Self::S_SIZE as i32));
        let max = rect.max.div_euclid(IVec2::splat(Self::S_SIZE as i32));
        let iter_s = (min.y..=max.y).flat_map(move |y| {
            (min.x..=max.x).flat_map(move |x| {
                let coord = IVec2::new(x, y);
                self.grid_s.get(&coord).into_iter().flat_map(|grid| grid.iter().copied())
            })
        });
        // medium
        let min = rect.min.div_euclid(IVec2::splat(Self::M_SIZE as i32));
        let max = rect.max.div_euclid(IVec2::splat(Self::M_SIZE as i32));
        let iter_m = (min.y..=max.y).flat_map(move |y| {
            (min.x..=max.x).flat_map(move |x| {
                let coord = IVec2::new(x, y);
                self.grid_m.get(&coord).into_iter().flat_map(|grid| grid.iter().copied())
            })
        });
        // large
        let min = rect.min.div_euclid(IVec2::splat(Self::L_SIZE as i32));
        let max = rect.max.div_euclid(IVec2::splat(Self::L_SIZE as i32));
        let iter_l = (min.y..=max.y).flat_map(move |y| {
            (min.x..=max.x).flat_map(move |x| {
                let coord = IVec2::new(x, y);
                self.grid_l.get(&coord).into_iter().flat_map(|grid| grid.iter().copied())
            })
        });

        iter_s.chain(iter_m).chain(iter_l)
    }
}
