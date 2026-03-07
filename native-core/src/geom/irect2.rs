use core::{iter, ops::*};

use glam::*;

use super::*;

/// Creates a new IRect2 from two points.
#[inline]
pub const fn irect2(min: IVec2, max: IVec2) -> IRect2 {
    IRect2::new(min, max)
}

/// A 2-dimensional axis-aligned bounding box.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct IRect2 {
    pub min: IVec2,
    pub max: IVec2,
}

impl IRect2 {
    /// All zeroes.
    const ZERO: Self = Self {
        min: IVec2::ZERO,
        max: IVec2::ZERO,
    };

    /// Creates a new IRect2 from two points.
    #[inline]
    pub const fn new(min: IVec2, max: IVec2) -> Self {
        Self { min, max }
    }

    /// Creates a new IRect2 from a center point and extents.
    #[inline]
    pub fn from_center(center: IVec2, extents: IVec2) -> Self {
        Self {
            min: center - extents,
            max: center + extents,
        }
    }

    /// Returns the IRect2 size.
    #[inline]
    pub fn size(&self) -> IVec2 {
        self.max - self.min
    }

    /// Returns the IRect2 center point.
    #[inline]
    pub fn center(&self) -> IVec2 {
        (self.min + self.max) >> 1
    }

    /// Returns the IRect2 extents.
    #[inline]
    pub fn extents(&self) -> IVec2 {
        (self.max - self.min) >> 1
    }

    /// Returns the IRect2 volume.
    #[inline]
    pub fn volume(&self) -> i32 {
        let size = self.size();
        size.x * size.y
    }

    /// Returns the IRect2 with extended size.
    #[inline]
    pub fn extends(self, size: i32) -> Self {
        Self {
            min: self.min - IVec2::splat(size),
            max: self.max + IVec2::splat(size),
        }
    }

    /// Calculates the Euclidean division.
    #[inline]
    pub fn div_euclid(self, rhs: IRect2) -> Self {
        Self {
            min: self.min.div_euclid(rhs.min),
            max: self.max.div_euclid(rhs.max),
        }
    }

    /// Calculates the least nonnegative remainder of `self (mod rhs)`.
    #[inline]
    pub fn rem_euclid(self, rhs: IRect2) -> Self {
        Self {
            min: self.min.rem_euclid(rhs.min),
            max: self.max.rem_euclid(rhs.max),
        }
    }

    // Calculates the minimum intersection between `self` and `rhs`.
    #[inline]
    pub fn minimum(&self, rhs: IRect2) -> Self {
        Self {
            min: self.min.max(rhs.min),
            max: self.max.min(rhs.max),
        }
    }

    // Calculates the maximum union between `self` and `rhs`.
    #[inline]
    pub fn maximum(&self, rhs: IRect2) -> Self {
        Self {
            min: self.min.min(rhs.min),
            max: self.max.max(rhs.max),
        }
    }

    /// Casts into `Rect2`.
    #[inline]
    pub fn as_rect2(&self) -> Rect2 {
        Rect2 {
            min: self.min.as_vec2(),
            max: self.max.as_vec2(),
        }
    }
}

// - IRect2
impl Neg for IRect2 {
    type Output = IRect2;
    #[inline]
    fn neg(self) -> IRect2 {
        IRect2 {
            min: self.min.neg(),
            max: self.max.neg(),
        }
    }
}

// IRect2 + IVec2
impl Add<IVec2> for IRect2 {
    type Output = IRect2;
    #[inline]
    fn add(self, rhs: IVec2) -> IRect2 {
        IRect2 {
            min: self.min.add(rhs),
            max: self.max.add(rhs),
        }
    }
}

// IVec2 + IRect2
impl Add<IRect2> for IVec2 {
    type Output = IRect2;
    #[inline]
    fn add(self, rhs: IRect2) -> IRect2 {
        IRect2 {
            min: self.add(rhs.min),
            max: self.add(rhs.max),
        }
    }
}

// IRect2 + IRect2
impl Add<IRect2> for IRect2 {
    type Output = IRect2;
    #[inline]
    fn add(self, rhs: IRect2) -> IRect2 {
        IRect2 {
            min: self.min.add(rhs.min),
            max: self.max.add(rhs.max),
        }
    }
}

// IRect2 += IVec2
impl AddAssign<IVec2> for IRect2 {
    #[inline]
    fn add_assign(&mut self, rhs: IVec2) {
        self.min.add_assign(rhs);
        self.max.add_assign(rhs);
    }
}

// IRect2 += IRect2
impl AddAssign<IRect2> for IRect2 {
    #[inline]
    fn add_assign(&mut self, rhs: IRect2) {
        self.min.add_assign(rhs.min);
        self.max.add_assign(rhs.max);
    }
}

// IRect2 - IVec2
impl Sub<IVec2> for IRect2 {
    type Output = IRect2;
    #[inline]
    fn sub(self, rhs: IVec2) -> IRect2 {
        IRect2 {
            min: self.min.sub(rhs),
            max: self.max.sub(rhs),
        }
    }
}

// IVec2 - IRect2
impl Sub<IRect2> for IVec2 {
    type Output = IRect2;
    #[inline]
    fn sub(self, rhs: IRect2) -> IRect2 {
        IRect2 {
            min: self.sub(rhs.min),
            max: self.sub(rhs.max),
        }
    }
}

// IRect2 - IRect2
impl Sub<IRect2> for IRect2 {
    type Output = IRect2;
    #[inline]
    fn sub(self, rhs: IRect2) -> IRect2 {
        IRect2 {
            min: self.min.sub(rhs.min),
            max: self.max.sub(rhs.max),
        }
    }
}

// IRect2 -= IVec2
impl SubAssign<IVec2> for IRect2 {
    #[inline]
    fn sub_assign(&mut self, rhs: IVec2) {
        self.min.sub_assign(rhs);
        self.max.sub_assign(rhs);
    }
}

// IRect2 -= IRect2
impl SubAssign<IRect2> for IRect2 {
    #[inline]
    fn sub_assign(&mut self, rhs: IRect2) {
        self.min.sub_assign(rhs.min);
        self.max.sub_assign(rhs.max);
    }
}

// IRect2 * i32
impl Mul<i32> for IRect2 {
    type Output = IRect2;
    #[inline]
    fn mul(self, rhs: i32) -> IRect2 {
        IRect2 {
            min: self.min.mul(rhs),
            max: self.max.mul(rhs),
        }
    }
}

// i32 * IRect2
impl Mul<IRect2> for i32 {
    type Output = IRect2;
    #[inline]
    fn mul(self, rhs: IRect2) -> IRect2 {
        IRect2 {
            min: self.mul(rhs.min),
            max: self.mul(rhs.max),
        }
    }
}

// IRect2 * IVec2
impl Mul<IVec2> for IRect2 {
    type Output = IRect2;
    #[inline]
    fn mul(self, rhs: IVec2) -> IRect2 {
        IRect2 {
            min: self.min.mul(rhs),
            max: self.max.mul(rhs),
        }
    }
}

// IVec2 * IRect2
impl Mul<IRect2> for IVec2 {
    type Output = IRect2;
    #[inline]
    fn mul(self, rhs: IRect2) -> IRect2 {
        IRect2 {
            min: self.mul(rhs.min),
            max: self.mul(rhs.max),
        }
    }
}

// IRect2 * IRect2
impl Mul<IRect2> for IRect2 {
    type Output = IRect2;
    #[inline]
    fn mul(self, rhs: IRect2) -> IRect2 {
        IRect2 {
            min: self.min.mul(rhs.min),
            max: self.max.mul(rhs.max),
        }
    }
}

// IRect2 *= i32
impl MulAssign<i32> for IRect2 {
    #[inline]
    fn mul_assign(&mut self, rhs: i32) {
        self.min.mul_assign(rhs);
        self.max.mul_assign(rhs);
    }
}

// IRect2 *= IVec2
impl MulAssign<IVec2> for IRect2 {
    #[inline]
    fn mul_assign(&mut self, rhs: IVec2) {
        self.min.mul_assign(rhs);
        self.max.mul_assign(rhs);
    }
}

// IRect2 *= IRect2
impl MulAssign<IRect2> for IRect2 {
    #[inline]
    fn mul_assign(&mut self, rhs: IRect2) {
        self.min.mul_assign(rhs.min);
        self.max.mul_assign(rhs.max);
    }
}

// IRect2 / i32
impl Div<i32> for IRect2 {
    type Output = IRect2;
    #[inline]
    fn div(self, rhs: i32) -> IRect2 {
        IRect2 {
            min: self.min.div(rhs),
            max: self.max.div(rhs),
        }
    }
}

// i32 / IRect2
impl Div<IRect2> for i32 {
    type Output = IRect2;
    #[inline]
    fn div(self, rhs: IRect2) -> IRect2 {
        IRect2 {
            min: self.div(rhs.min),
            max: self.div(rhs.max),
        }
    }
}

// IRect2 / IVec2
impl Div<IVec2> for IRect2 {
    type Output = IRect2;
    #[inline]
    fn div(self, rhs: IVec2) -> IRect2 {
        IRect2 {
            min: self.min.div(rhs),
            max: self.max.div(rhs),
        }
    }
}

// IVec2 / IRect2
impl Div<IRect2> for IVec2 {
    type Output = IRect2;
    #[inline]
    fn div(self, rhs: IRect2) -> IRect2 {
        IRect2 {
            min: self.div(rhs.min),
            max: self.div(rhs.max),
        }
    }
}

// IRect2 / IRect2
impl Div<IRect2> for IRect2 {
    type Output = IRect2;
    #[inline]
    fn div(self, rhs: IRect2) -> IRect2 {
        IRect2 {
            min: self.min.div(rhs.min),
            max: self.max.div(rhs.max),
        }
    }
}

// IRect2 /= i32
impl DivAssign<i32> for IRect2 {
    #[inline]
    fn div_assign(&mut self, rhs: i32) {
        self.min.div_assign(rhs);
        self.max.div_assign(rhs);
    }
}

// IRect2 /= IVec2
impl DivAssign<IVec2> for IRect2 {
    #[inline]
    fn div_assign(&mut self, rhs: IVec2) {
        self.min.div_assign(rhs);
        self.max.div_assign(rhs);
    }
}

// IRect2 /= IRect2
impl DivAssign<IRect2> for IRect2 {
    #[inline]
    fn div_assign(&mut self, rhs: IRect2) {
        self.min.div_assign(rhs.min);
        self.max.div_assign(rhs.max);
    }
}

// IRect2 % i32
impl Rem<i32> for IRect2 {
    type Output = IRect2;
    #[inline]
    fn rem(self, rhs: i32) -> IRect2 {
        IRect2 {
            min: self.min.rem(rhs),
            max: self.max.rem(rhs),
        }
    }
}

// i32 % IRect2
impl Rem<IRect2> for i32 {
    type Output = IRect2;
    #[inline]
    fn rem(self, rhs: IRect2) -> IRect2 {
        IRect2 {
            min: self.rem(rhs.min),
            max: self.rem(rhs.max),
        }
    }
}

// IRect2 % IVec2
impl Rem<IVec2> for IRect2 {
    type Output = IRect2;
    #[inline]
    fn rem(self, rhs: IVec2) -> IRect2 {
        IRect2 {
            min: self.min.rem(rhs),
            max: self.max.rem(rhs),
        }
    }
}

// IVec2 % IRect2
impl Rem<IRect2> for IVec2 {
    type Output = IRect2;
    #[inline]
    fn rem(self, rhs: IRect2) -> IRect2 {
        IRect2 {
            min: self.rem(rhs.min),
            max: self.rem(rhs.max),
        }
    }
}

// IRect2 % IRect2
impl Rem<IRect2> for IRect2 {
    type Output = IRect2;
    #[inline]
    fn rem(self, rhs: IRect2) -> IRect2 {
        IRect2 {
            min: self.min.rem(rhs.min),
            max: self.max.rem(rhs.max),
        }
    }
}

// IRect2 %= i32
impl RemAssign<i32> for IRect2 {
    #[inline]
    fn rem_assign(&mut self, rhs: i32) {
        self.min.rem_assign(rhs);
        self.max.rem_assign(rhs);
    }
}

// IRect2 %= IVec2
impl RemAssign<IVec2> for IRect2 {
    fn rem_assign(&mut self, rhs: IVec2) {
        self.min.rem_assign(rhs);
        self.max.rem_assign(rhs);
    }
}

// IRect2 %= IRect2
impl RemAssign<IRect2> for IRect2 {
    #[inline]
    fn rem_assign(&mut self, rhs: IRect2) {
        self.min.rem_assign(rhs.min);
        self.max.rem_assign(rhs.max);
    }
}

impl AsRef<[IVec2; 2]> for IRect2 {
    #[inline]
    fn as_ref(&self) -> &[IVec2; 2] {
        unsafe { &*(self as *const IRect2 as *const [IVec2; 2]) }
    }
}

impl AsMut<[IVec2; 2]> for IRect2 {
    #[inline]
    fn as_mut(&mut self) -> &mut [IVec2; 2] {
        unsafe { &mut *(self as *mut IRect2 as *mut [IVec2; 2]) }
    }
}

impl iter::Sum for IRect2 {
    #[inline]
    fn sum<I: Iterator<Item = IRect2>>(iter: I) -> IRect2 {
        iter.fold(IRect2::ZERO, IRect2::add)
    }
}

impl<'a> iter::Sum<&'a IRect2> for IRect2 {
    #[inline]
    fn sum<I: Iterator<Item = &'a IRect2>>(iter: I) -> IRect2 {
        iter.fold(IRect2::ZERO, |a, &b| IRect2::add(a, b))
    }
}

impl iter::Product for IRect2 {
    #[inline]
    fn product<I: Iterator<Item = IRect2>>(iter: I) -> IRect2 {
        iter.fold(IRect2::ZERO, IRect2::mul)
    }
}

impl<'a> iter::Product<&'a IRect2> for IRect2 {
    #[inline]
    fn product<I: Iterator<Item = &'a IRect2>>(iter: I) -> IRect2 {
        iter.fold(IRect2::ZERO, |a, &b| IRect2::mul(a, b))
    }
}

impl Index<usize> for IRect2 {
    type Output = IVec2;
    #[inline]
    fn index(&self, index: usize) -> &IVec2 {
        match index {
            0 => &self.min,
            1 => &self.max,
            _ => panic!("index out of rect"),
        }
    }
}

impl IndexMut<usize> for IRect2 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut IVec2 {
        match index {
            0 => &mut self.min,
            1 => &mut self.max,
            _ => panic!("index out of rect"),
        }
    }
}

impl From<[IVec2; 2]> for IRect2 {
    #[inline]
    fn from(value: [IVec2; 2]) -> IRect2 {
        IRect2 {
            min: value[0],
            max: value[1],
        }
    }
}

impl From<IRect2> for [IVec2; 2] {
    #[inline]
    fn from(value: IRect2) -> [IVec2; 2] {
        [value.min, value.max]
    }
}

impl From<(IVec2, IVec2)> for IRect2 {
    #[inline]
    fn from(value: (IVec2, IVec2)) -> IRect2 {
        IRect2 {
            min: value.0,
            max: value.1,
        }
    }
}

impl From<IRect2> for (IVec2, IVec2) {
    #[inline]
    fn from(value: IRect2) -> (IVec2, IVec2) {
        (value.min, value.max)
    }
}
