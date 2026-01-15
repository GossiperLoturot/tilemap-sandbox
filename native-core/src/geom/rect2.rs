use core::{iter, ops::*};

use glam::*;

use super::*;

/// Creates a new Rect2 from two points.
#[inline]
pub const fn rect2(min: Vec2, max: Vec2) -> Rect2 {
    Rect2::new(min, max)
}

/// A 2-dimensional axis-aligned bounding box.
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct Rect2 {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect2 {
    /// All zeroes.
    const ZERO: Self = Self {
        min: Vec2::ZERO,
        max: Vec2::ZERO,
    };

    /// Creates a new Rect2 from two points.
    #[inline]
    pub const fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    /// Creates a new Rect2 from a center point and extents.
    #[inline]
    pub fn from_center(center: Vec2, extents: Vec2) -> Self {
        Self {
            min: center - extents,
            max: center + extents,
        }
    }

    /// Returns the Rect2 size.
    #[inline]
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    /// Returns the Rect2 center point.
    #[inline]
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    /// Returns the Rect2 extents.
    #[inline]
    pub fn extents(&self) -> Vec2 {
        (self.max - self.min) * 0.5
    }

    /// Returns the Rect2 volume.
    #[inline]
    pub fn volume(&self) -> f32 {
        let size = self.size();
        size.x * size.y
    }

    /// Returns the Rect2 with extended size.
    #[inline]
    pub fn extends(self, size: f32) -> Self {
        Self {
            min: self.min - Vec2::splat(size),
            max: self.max + Vec2::splat(size),
        }
    }

    /// Returns a Rect2 with the smallest integer greater than or equal to `self`'s
    /// element as element.
    #[inline]
    pub fn ceil(self) -> Self {
        Self {
            min: self.min.ceil(),
            max: self.max.ceil(),
        }
    }

    /// Returns a Rect2 with the nearest integer to `self`'s
    /// element as element.
    #[inline]
    pub fn round(self) -> Self {
        Self {
            min: self.min.round(),
            max: self.max.round(),
        }
    }

    /// Returns a Rect2 with the largest integer smaller than or equal to `self`'s
    /// element as element.
    #[inline]
    pub fn floor(self) -> Self {
        Self {
            min: self.min.floor(),
            max: self.max.floor(),
        }
    }

    /// Returns a Rect2 with `self`'s element integer part.
    #[inline]
    pub fn trunc(self) -> Self {
        Self {
            min: self.min.trunc(),
            max: self.max.trunc(),
        }
    }

    /// Returns a Rect2 with `self`'s element fractional part.
    #[inline]
    pub fn fract(self) -> Self {
        Self {
            min: self.min.fract(),
            max: self.max.fract(),
        }
    }

    /// Returns the smallest integer Rect2 that can covers `self` area.
    #[inline]
    pub fn trunc_over(self) -> Self {
        Self {
            min: self.min.floor(),
            max: self.max.ceil(),
        }
    }

    /// Returns the largest integer Rect2 that can be covered by `self` area.
    #[inline]
    pub fn trunc_under(self) -> Self {
        Self {
            min: self.min.floor(),
            max: self.max.ceil(),
        }
    }

    /// Returns a Rect2 with `self`'s element exp.
    #[inline]
    pub fn exp(self) -> Self {
        Self {
            min: self.min.exp(),
            max: self.max.exp(),
        }
    }

    /// Returns a Rect2 with `self`'s element the power of n.
    #[inline]
    pub fn powf(self, n: f32) -> Self {
        Self {
            min: self.min.powf(n),
            max: self.max.powf(n),
        }
    }

    /// Returns a Rect2 with `self`'s element recip.
    #[inline]
    pub fn recip(self) -> Self {
        Self {
            min: self.min.recip(),
            max: self.max.recip(),
        }
    }

    /// Calculates the Euclidean division.
    #[inline]
    pub fn div_euclid(self, rhs: Rect2) -> Self {
        Self {
            min: self.min.div_euclid(rhs.min),
            max: self.max.div_euclid(rhs.max),
        }
    }

    /// Calculates the least nonnegative remainder of `self (mod rhs)`.
    #[inline]
    pub fn rem_euclid(self, rhs: Rect2) -> Self {
        Self {
            min: self.min.rem_euclid(rhs.min),
            max: self.max.rem_euclid(rhs.max),
        }
    }

    // Calculates the minimum intersection between `self` and `rhs`.
    #[inline]
    pub fn minimum(&self, rhs: Rect2) -> Self {
        Self {
            min: self.min.max(rhs.min),
            max: self.max.min(rhs.max),
        }
    }

    // Calculates the maximum union between `self` and `rhs`.
    #[inline]
    pub fn maximum(&self, rhs: Rect2) -> Self {
        Self {
            min: self.min.min(rhs.min),
            max: self.max.max(rhs.max),
        }
    }

    /// Casts into `IRect2`.
    #[inline]
    pub fn as_irect2(&self) -> IRect2 {
        IRect2 {
            min: self.min.as_ivec2(),
            max: self.max.as_ivec2(),
        }
    }
}

// - Rect2
impl Neg for Rect2 {
    type Output = Rect2;
    #[inline]
    fn neg(self) -> Rect2 {
        Rect2 {
            min: self.min.neg(),
            max: self.max.neg(),
        }
    }
}

// Rect2 + Vec2
impl Add<Vec2> for Rect2 {
    type Output = Rect2;
    #[inline]
    fn add(self, rhs: Vec2) -> Rect2 {
        Rect2 {
            min: self.min.add(rhs),
            max: self.max.add(rhs),
        }
    }
}

// Vec2 + Rect2
impl Add<Rect2> for Vec2 {
    type Output = Rect2;
    #[inline]
    fn add(self, rhs: Rect2) -> Rect2 {
        Rect2 {
            min: self.add(rhs.min),
            max: self.add(rhs.max),
        }
    }
}

// Rect2 + Rect2
impl Add<Rect2> for Rect2 {
    type Output = Rect2;
    #[inline]
    fn add(self, rhs: Rect2) -> Rect2 {
        Rect2 {
            min: self.min.add(rhs.min),
            max: self.max.add(rhs.max),
        }
    }
}

// Rect2 += Vec2
impl AddAssign<Vec2> for Rect2 {
    #[inline]
    fn add_assign(&mut self, rhs: Vec2) {
        self.min.add_assign(rhs);
        self.max.add_assign(rhs);
    }
}

// Rect2 += Rect2
impl AddAssign<Rect2> for Rect2 {
    #[inline]
    fn add_assign(&mut self, rhs: Rect2) {
        self.min.add_assign(rhs.min);
        self.max.add_assign(rhs.max);
    }
}

// Rect2 - Vec2
impl Sub<Vec2> for Rect2 {
    type Output = Rect2;
    #[inline]
    fn sub(self, rhs: Vec2) -> Rect2 {
        Rect2 {
            min: self.min.sub(rhs),
            max: self.max.sub(rhs),
        }
    }
}

// Vec2 - Rect2
impl Sub<Rect2> for Vec2 {
    type Output = Rect2;
    #[inline]
    fn sub(self, rhs: Rect2) -> Rect2 {
        Rect2 {
            min: self.sub(rhs.min),
            max: self.sub(rhs.max),
        }
    }
}

// Rect2 - Rect2
impl Sub<Rect2> for Rect2 {
    type Output = Rect2;
    #[inline]
    fn sub(self, rhs: Rect2) -> Rect2 {
        Rect2 {
            min: self.min.sub(rhs.min),
            max: self.max.sub(rhs.max),
        }
    }
}

// Rect2 -= Vec2
impl SubAssign<Vec2> for Rect2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Vec2) {
        self.min.sub_assign(rhs);
        self.max.sub_assign(rhs);
    }
}

// Rect2 -= Rect2
impl SubAssign<Rect2> for Rect2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Rect2) {
        self.min.sub_assign(rhs.min);
        self.max.sub_assign(rhs.max);
    }
}

// Rect2 * f32
impl Mul<f32> for Rect2 {
    type Output = Rect2;
    #[inline]
    fn mul(self, rhs: f32) -> Rect2 {
        Rect2 {
            min: self.min.mul(rhs),
            max: self.max.mul(rhs),
        }
    }
}

// f32 * Rect2
impl Mul<Rect2> for f32 {
    type Output = Rect2;
    #[inline]
    fn mul(self, rhs: Rect2) -> Rect2 {
        Rect2 {
            min: self.mul(rhs.min),
            max: self.mul(rhs.max),
        }
    }
}

// Rect2 * Vec2
impl Mul<Vec2> for Rect2 {
    type Output = Rect2;
    #[inline]
    fn mul(self, rhs: Vec2) -> Rect2 {
        Rect2 {
            min: self.min.mul(rhs),
            max: self.max.mul(rhs),
        }
    }
}

// Vec2 * Rect2
impl Mul<Rect2> for Vec2 {
    type Output = Rect2;
    #[inline]
    fn mul(self, rhs: Rect2) -> Rect2 {
        Rect2 {
            min: self.mul(rhs.min),
            max: self.mul(rhs.max),
        }
    }
}

// Rect2 * Rect2
impl Mul<Rect2> for Rect2 {
    type Output = Rect2;
    #[inline]
    fn mul(self, rhs: Rect2) -> Rect2 {
        Rect2 {
            min: self.min.mul(rhs.min),
            max: self.max.mul(rhs.max),
        }
    }
}

// Rect2 *= f32
impl MulAssign<f32> for Rect2 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        self.min.mul_assign(rhs);
        self.max.mul_assign(rhs);
    }
}

// Rect2 *= Vec2
impl MulAssign<Vec2> for Rect2 {
    #[inline]
    fn mul_assign(&mut self, rhs: Vec2) {
        self.min.mul_assign(rhs);
        self.max.mul_assign(rhs);
    }
}

// Rect2 *= Rect2
impl MulAssign<Rect2> for Rect2 {
    #[inline]
    fn mul_assign(&mut self, rhs: Rect2) {
        self.min.mul_assign(rhs.min);
        self.max.mul_assign(rhs.max);
    }
}

// Rect2 / f32
impl Div<f32> for Rect2 {
    type Output = Rect2;
    #[inline]
    fn div(self, rhs: f32) -> Rect2 {
        Rect2 {
            min: self.min.div(rhs),
            max: self.max.div(rhs),
        }
    }
}

// f32 / Rect2
impl Div<Rect2> for f32 {
    type Output = Rect2;
    #[inline]
    fn div(self, rhs: Rect2) -> Rect2 {
        Rect2 {
            min: self.div(rhs.min),
            max: self.div(rhs.max),
        }
    }
}

// Rect2 / Vec2
impl Div<Vec2> for Rect2 {
    type Output = Rect2;
    #[inline]
    fn div(self, rhs: Vec2) -> Rect2 {
        Rect2 {
            min: self.min.div(rhs),
            max: self.max.div(rhs),
        }
    }
}

// Vec2 / Rect2
impl Div<Rect2> for Vec2 {
    type Output = Rect2;
    #[inline]
    fn div(self, rhs: Rect2) -> Rect2 {
        Rect2 {
            min: self.div(rhs.min),
            max: self.div(rhs.max),
        }
    }
}

// Rect2 / Rect2
impl Div<Rect2> for Rect2 {
    type Output = Rect2;
    #[inline]
    fn div(self, rhs: Rect2) -> Rect2 {
        Rect2 {
            min: self.min.div(rhs.min),
            max: self.max.div(rhs.max),
        }
    }
}

// Rect2 /= f32
impl DivAssign<f32> for Rect2 {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        self.min.div_assign(rhs);
        self.max.div_assign(rhs);
    }
}

// Rect2 /= Vec2
impl DivAssign<Vec2> for Rect2 {
    #[inline]
    fn div_assign(&mut self, rhs: Vec2) {
        self.min.div_assign(rhs);
        self.max.div_assign(rhs);
    }
}

// Rect2 /= Rect2
impl DivAssign<Rect2> for Rect2 {
    #[inline]
    fn div_assign(&mut self, rhs: Rect2) {
        self.min.div_assign(rhs.min);
        self.max.div_assign(rhs.max);
    }
}

// Rect2 % f32
impl Rem<f32> for Rect2 {
    type Output = Rect2;
    #[inline]
    fn rem(self, rhs: f32) -> Rect2 {
        Rect2 {
            min: self.min.rem(rhs),
            max: self.max.rem(rhs),
        }
    }
}

// f32 % Rect2
impl Rem<Rect2> for f32 {
    type Output = Rect2;
    #[inline]
    fn rem(self, rhs: Rect2) -> Rect2 {
        Rect2 {
            min: self.rem(rhs.min),
            max: self.rem(rhs.max),
        }
    }
}

// Rect2 % Vec2
impl Rem<Vec2> for Rect2 {
    type Output = Rect2;
    #[inline]
    fn rem(self, rhs: Vec2) -> Rect2 {
        Rect2 {
            min: self.min.rem(rhs),
            max: self.max.rem(rhs),
        }
    }
}

// Vec2 % Rect2
impl Rem<Rect2> for Vec2 {
    type Output = Rect2;
    #[inline]
    fn rem(self, rhs: Rect2) -> Rect2 {
        Rect2 {
            min: self.rem(rhs.min),
            max: self.rem(rhs.max),
        }
    }
}

// Rect2 % Rect2
impl Rem<Rect2> for Rect2 {
    type Output = Rect2;
    #[inline]
    fn rem(self, rhs: Rect2) -> Rect2 {
        Rect2 {
            min: self.min.rem(rhs.min),
            max: self.max.rem(rhs.max),
        }
    }
}

// Rect2 %= f32
impl RemAssign<f32> for Rect2 {
    fn rem_assign(&mut self, rhs: f32) {
        self.min.rem_assign(rhs);
        self.max.rem_assign(rhs);
    }
}

// Rect2 %= Vec2
impl RemAssign<Vec2> for Rect2 {
    fn rem_assign(&mut self, rhs: Vec2) {
        self.min.rem_assign(rhs);
        self.max.rem_assign(rhs);
    }
}

// Rect2 %= Rect2
impl RemAssign<Rect2> for Rect2 {
    #[inline]
    fn rem_assign(&mut self, rhs: Rect2) {
        self.min.rem_assign(rhs.min);
        self.max.rem_assign(rhs.max);
    }
}

impl AsRef<[Vec2; 2]> for Rect2 {
    #[inline]
    fn as_ref(&self) -> &[Vec2; 2] {
        unsafe { &*(self as *const Rect2 as *const [Vec2; 2]) }
    }
}

impl AsMut<[Vec2; 2]> for Rect2 {
    #[inline]
    fn as_mut(&mut self) -> &mut [Vec2; 2] {
        unsafe { &mut *(self as *mut Rect2 as *mut [Vec2; 2]) }
    }
}

impl iter::Sum for Rect2 {
    #[inline]
    fn sum<I: Iterator<Item = Rect2>>(iter: I) -> Rect2 {
        iter.fold(Rect2::ZERO, Rect2::add)
    }
}

impl<'a> iter::Sum<&'a Rect2> for Rect2 {
    #[inline]
    fn sum<I: Iterator<Item = &'a Rect2>>(iter: I) -> Rect2 {
        iter.fold(Rect2::ZERO, |a, &b| Rect2::add(a, b))
    }
}

impl iter::Product for Rect2 {
    #[inline]
    fn product<I: Iterator<Item = Rect2>>(iter: I) -> Rect2 {
        iter.fold(Rect2::ZERO, Rect2::mul)
    }
}

impl<'a> iter::Product<&'a Rect2> for Rect2 {
    #[inline]
    fn product<I: Iterator<Item = &'a Rect2>>(iter: I) -> Rect2 {
        iter.fold(Rect2::ZERO, |a, &b| Rect2::mul(a, b))
    }
}

impl Index<usize> for Rect2 {
    type Output = Vec2;
    #[inline]
    fn index(&self, index: usize) -> &Vec2 {
        match index {
            0 => &self.min,
            1 => &self.max,
            _ => panic!("index out of rect"),
        }
    }
}

impl IndexMut<usize> for Rect2 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Vec2 {
        match index {
            0 => &mut self.min,
            1 => &mut self.max,
            _ => panic!("index out of rect"),
        }
    }
}

impl From<[Vec2; 2]> for Rect2 {
    #[inline]
    fn from(value: [Vec2; 2]) -> Rect2 {
        Rect2 {
            min: value[0],
            max: value[1],
        }
    }
}

impl From<Rect2> for [Vec2; 2] {
    #[inline]
    fn from(value: Rect2) -> [Vec2; 2] {
        [value.min, value.max]
    }
}

impl From<(Vec2, Vec2)> for Rect2 {
    #[inline]
    fn from(value: (Vec2, Vec2)) -> Rect2 {
        Rect2 {
            min: value.0,
            max: value.1,
        }
    }
}

impl From<Rect2> for (Vec2, Vec2) {
    #[inline]
    fn from(value: Rect2) -> (Vec2, Vec2) {
        (value.min, value.max)
    }
}
