use glam::*;

use super::*;

pub trait Intersects<T> {
    fn intersects(&self, other: &T) -> bool;
}

impl Intersects::<IVec2> for IVec2 {
    #[inline]
    fn intersects(&self, other: &IVec2) -> bool {
        self == other
    }
}

impl Intersects::<IVec2> for IRect2 {
    #[inline]
    fn intersects(&self, other: &IVec2) -> bool {
        self.min.x <= other.x && other.x <= self.max.x && self.min.y <= other.y && other.y <= self.max.y
    }
}

impl Intersects::<IRect2> for IVec2 {
    #[inline]
    fn intersects(&self, other: &IRect2) -> bool {
        Intersects::intersects(other, self)
    }
}

impl Intersects::<IRect2> for IRect2 {
    #[inline]
    fn intersects(&self, other: &IRect2) -> bool {
        let a = IVec4::new(self.min.x, self.min.y, other.min.x, other.min.y);
        let b = IVec4::new(other.max.x, other.max.y, self.max.x, self.max.y);
        a.cmple(b).all()
    }
}

impl Intersects::<Vec2> for Vec2 {
    #[inline]
    fn intersects(&self, other: &Vec2) -> bool {
        self == other
    }
}

impl Intersects::<Vec2> for Rect2 {
    #[inline]
    fn intersects(&self, other: &Vec2) -> bool {
        self.min.x <= other.x && other.x <= self.max.x && self.min.y <= other.y && other.y <= self.max.y
    }
}

impl Intersects::<Rect2> for Vec2 {
    #[inline]
    fn intersects(&self, other: &Rect2) -> bool {
        Intersects::intersects(other, self)
    }
}

impl Intersects::<Rect2> for Rect2 {
    #[inline]
    fn intersects(&self, other: &Rect2) -> bool {
        let a = Vec4::new(self.min.x, self.min.y, other.min.x, other.min.y);
        let b = Vec4::new(other.max.x, other.max.y, self.max.x, self.max.y);
        a.cmple(b).all()
    }
}
