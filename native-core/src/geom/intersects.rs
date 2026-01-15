use glam::*;

use super::*;

pub trait Intersects<T> {
    fn intersects(&self, other: &T) -> bool;
}

impl Intersects::<IVec2> for IVec2 {
    fn intersects(&self, other: &IVec2) -> bool {
        self == other
    }
}

impl Intersects::<IVec2> for IRect2 {
    fn intersects(&self, other: &IVec2) -> bool {
        self.min.x <= other.x && other.x <= self.max.x && self.min.y <= other.y && other.y <= self.max.y
    }
}

impl Intersects::<IRect2> for IVec2 {
    fn intersects(&self, other: &IRect2) -> bool {
        Intersects::intersects(other, self)
    }
}

impl Intersects::<IRect2> for IRect2 {
    fn intersects(&self, other: &IRect2) -> bool {
        self.min.x <= other.max.x && self.min.y <= other.max.y && other.min.x <= self.max.x && other.min.y <= self.max.y
    }
}

impl Intersects::<Vec2> for Vec2 {
    fn intersects(&self, other: &Vec2) -> bool {
        self == other
    }
}

impl Intersects::<Vec2> for Rect2 {
    fn intersects(&self, other: &Vec2) -> bool {
        self.min.x <= other.x && other.x <= self.max.x && self.min.y <= other.y && other.y <= self.max.y
    }
}

impl Intersects::<Rect2> for Vec2 {
    fn intersects(&self, other: &Rect2) -> bool {
        Intersects::intersects(other, self)
    }
}

impl Intersects::<Rect2> for Rect2 {
    fn intersects(&self, other: &Rect2) -> bool {
        self.min.x <= other.max.x && self.min.y <= other.max.y && other.min.x <= self.max.x && other.min.y <= self.max.y
    }
}
