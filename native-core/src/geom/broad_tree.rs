use glam::*;

use super::*;

type Rect = rstar::primitives::Rectangle<[i32; 2]>;
type Node<T> = rstar::primitives::GeomWithData<Rect, T>;

struct Params;

impl rstar::RTreeParams for Params {
    const MIN_SIZE: usize = 4;
    const MAX_SIZE: usize = 16;
    const REINSERTION_COUNT: usize = 2;
    type DefaultInsertionStrategy = rstar::RStarInsertionStrategy;
}

#[derive(Debug, Default)]
pub struct BroadTree<T> {
    rtree: rstar::RTree<Node<T>, Params>
}

impl<T> BroadTree<T> where T: PartialEq + Copy {
    pub fn new() -> Self {
        Self { rtree: Default::default() }
    }

    pub fn insert(&mut self, rect: IRect2, data: T) {
        let rect = Rect::from_corners(rect.min.into(), rect.max.into());
        let node = Node::new(rect, data);
        self.rtree.insert(node);
    }

    pub fn remove(&mut self, rect: IRect2, data: T) {
        let rect = Rect::from_corners(rect.min.into(), rect.max.into());
        let node = Node::new(rect, data);
        self.rtree.remove(&node);
    }

    pub fn find(&self, rect: IRect2) -> impl Iterator<Item = T> + '_ {
        let rect = Rect::from_corners(rect.min.into(), rect.max.into());
        let envelope = rstar::RTreeObject::envelope(&rect);
        self.rtree.locate_in_envelope_intersecting(&envelope).map(|obj| obj.data)
    }
}
