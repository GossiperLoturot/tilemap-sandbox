use crate::inner::{IVec2, Vec2};

type BlockNode = rstar::primitives::GeomWithData<rstar::primitives::Rectangle<Vec2>, IVec2>;
type EntityNode = rstar::primitives::GeomWithData<rstar::primitives::Rectangle<Vec2>, u32>;

#[derive(Debug, Clone, Default)]
pub struct BlockSpec {
    pub size: Vec2,
    pub offset: Vec2,
}

#[derive(Debug, Clone)]
pub struct BlockFieldPhysics {
    specs: Vec<BlockSpec>,
    spartial_ref: rstar::RTree<BlockNode>,
    index_ref: ahash::AHashMap<IVec2, BlockNode>,
}

impl BlockFieldPhysics {
    pub fn new(specs: Vec<BlockSpec>) -> Self {
        Self {
            specs,
            spartial_ref: rstar::RTree::default(),
            index_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, block: &crate::inner::Block) -> Option<()> {
        if self.index_ref.contains_key(&block.location) {
            return None;
        }

        let spec = &self.specs[block.id as usize];

        let p0 = [
            block.location[0] as f32 + spec.offset[0],
            block.location[1] as f32 + spec.offset[1],
        ];
        let p1 = [
            block.location[0] as f32 + spec.offset[0] + spec.size[0],
            block.location[1] as f32 + spec.offset[1] + spec.size[1],
        ];

        let rect = rstar::primitives::Rectangle::from_corners(p0, p1);
        let node = rstar::primitives::GeomWithData::new(rect, block.location);
        self.spartial_ref.insert(node);

        self.index_ref.insert(block.location, node);

        Some(())
    }

    pub fn remove(&mut self, location: IVec2) -> Option<()> {
        let node = self.index_ref.get(&location)?;

        self.spartial_ref.remove(node);

        self.index_ref.remove(&location);

        Some(())
    }

    pub fn get_by_point(&self, point: Vec2) -> Option<IVec2> {
        self.spartial_ref
            .locate_at_point(&point)
            .map(|node| node.data)
    }

    pub fn get_by_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = IVec2> + '_ {
        self.spartial_ref
            .locate_in_envelope(&rstar::AABB::from_corners(rect[0], rect[1]))
            .map(|node| node.data)
    }
}

#[derive(Debug, Clone, Default)]
pub struct EntitySpec {
    pub size: Vec2,
    pub offset: Vec2,
}

#[derive(Debug, Clone)]
pub struct EntityFieldPhysics {
    specs: Vec<EntitySpec>,
    spartial_ref: rstar::RTree<EntityNode>,
    index_ref: ahash::AHashMap<u32, EntityNode>,
}

impl EntityFieldPhysics {
    pub fn new(specs: Vec<EntitySpec>) -> Self {
        Self {
            specs,
            spartial_ref: rstar::RTree::default(),
            index_ref: Default::default(),
        }
    }

    pub fn insert(&mut self, entity: &crate::inner::Entity) -> Option<()> {
        if self.index_ref.contains_key(&entity.id) {
            return None;
        }

        let spec = &self.specs[entity.id as usize];

        let p0 = [
            entity.location[0] + spec.offset[0],
            entity.location[1] + spec.offset[1],
        ];
        let p1 = [
            entity.location[0] + spec.offset[0] + spec.size[0],
            entity.location[1] + spec.offset[1] + spec.size[1],
        ];

        let rect = rstar::primitives::Rectangle::from_corners(p0, p1);
        let node = rstar::primitives::GeomWithData::new(rect, entity.id);
        self.spartial_ref.insert(node);

        self.index_ref.insert(entity.id, node);

        Some(())
    }

    pub fn remove(&mut self, id: u32) -> Option<()> {
        let node = self.index_ref.get(&id)?;

        self.spartial_ref.remove(node);

        self.index_ref.remove(&id);

        Some(())
    }

    pub fn get_by_point(&self, point: Vec2) -> Option<u32> {
        self.spartial_ref
            .locate_at_point(&point)
            .map(|node| node.data)
    }

    pub fn get_by_rect(&self, rect: [Vec2; 2]) -> impl Iterator<Item = u32> + '_ {
        self.spartial_ref
            .locate_in_envelope(&rstar::AABB::from_corners(rect[0], rect[1]))
            .map(|node| node.data)
    }
}
