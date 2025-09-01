use std::cmp::Ordering;
use std::f64;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, Sub, SubAssign};

use crate::num::Num;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Finite(f64);

impl PartialEq<f64> for Finite {
    fn eq(&self, other: &f64) -> bool {
        self.0 == *other
    }
}
impl PartialOrd<f64> for Finite {
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl Eq for Finite {}

impl PartialOrd for Finite {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Finite {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }

    fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        Self(self.0.clamp(min.0, max.0))
    }
}

impl Finite {
    pub const ZERO: Self = Self(0.0);
    pub const ONE: Self = Self(1.0);
    pub const HALF: Self = Self(0.5);
    pub const PI: Self = Self(f64::consts::PI);

    pub const fn try_new(value: f64) -> Option<Self> {
        if value.is_finite() {
            Some(Self(value))
        } else {
            None
        }
    }

    pub const fn to_f64(self) -> f64 {
        self.0
    }

    pub fn sin(self) -> Self {
        self.apply("sin", |x| x.sin())
    }

    pub fn abs(self) -> Self {
        self.apply("abs", |x| x.abs())
    }

    #[track_caller]
    fn apply(self, fname: &str, f: impl FnOnce(f64) -> f64) -> Self {
        let value = f(self.0);
        match Self::try_new(value) {
            Some(result) => result,
            None => panic!("{fname}({}) is not finite: {}", self.0, value),
        }
    }

    #[track_caller]
    fn apply2(self, fname: &str, other: Self, f: impl FnOnce(f64, f64) -> f64) -> Self {
        let value = f(self.0, other.0);
        match Self::try_new(value) {
            Some(result) => result,
            None => panic!("{fname}({}, {}) is not finite: {}", self.0, other.0, value),
        }
    }
}

impl Num for Finite {
    fn base_value(&self) -> Finite {
        *self
    }

    fn replace_value(self, value: Finite) -> Self {
        value
    }

    fn sin_cos(self) -> (Self, Self) {
        let (sin, cos) = self.0.sin_cos();
        match (Self::try_new(sin), Self::try_new(cos)) {
            (Some(sin), Some(cos)) => (sin, cos),
            _ => panic!(
                "sin_cos({}) is not finite: sin = {}, cos = {}",
                self.0, sin, cos
            ),
        }
    }
}

impl Neg for Finite {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.apply("neg", |x| -x)
    }
}

impl AddAssign for Finite {
    #[track_caller]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl AddAssign<f64> for Finite {
    #[track_caller]
    fn add_assign(&mut self, rhs: f64) {
        *self = *self + rhs;
    }
}

impl Add for Finite {
    type Output = Self;

    #[track_caller]
    fn add(self, rhs: Self) -> Self::Output {
        self.apply2("add", rhs, |x, y| x + y)
    }
}

impl Add<f64> for Finite {
    type Output = Self;

    #[track_caller]
    fn add(self, rhs: f64) -> Self::Output {
        match Self::try_new(rhs) {
            Some(rhs) => self.apply2("add", rhs, |x, y| x + y),
            None => panic!("{} + {}", self.0, rhs),
        }
    }
}

impl SubAssign for Finite {
    #[track_caller]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl SubAssign<f64> for Finite {
    #[track_caller]
    fn sub_assign(&mut self, rhs: f64) {
        *self = *self - rhs;
    }
}

impl Sub for Finite {
    type Output = Self;

    #[track_caller]
    fn sub(self, rhs: Self) -> Self::Output {
        self.apply2("sub", rhs, |x, y| x - y)
    }
}

impl Sub<f64> for Finite {
    type Output = Self;

    #[track_caller]
    fn sub(self, rhs: f64) -> Self::Output {
        match Self::try_new(rhs) {
            Some(rhs) => self.apply2("sub", rhs, |x, y| x - y),
            None => panic!("{} - {}", self.0, rhs),
        }
    }
}

impl MulAssign for Finite {
    #[track_caller]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl MulAssign<f64> for Finite {
    #[track_caller]
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs;
    }
}

impl Mul for Finite {
    type Output = Self;

    #[track_caller]
    fn mul(self, rhs: Self) -> Self::Output {
        self.apply2("mul", rhs, |x, y| x * y)
    }
}

impl Mul<f64> for Finite {
    type Output = Self;

    #[track_caller]
    fn mul(self, rhs: f64) -> Self::Output {
        match Self::try_new(rhs) {
            Some(rhs) => self.apply2("mul", rhs, |x, y| x * y),
            None => panic!("{} * {}", self.0, rhs),
        }
    }
}

impl DivAssign<f64> for Finite {
    #[track_caller]
    fn div_assign(&mut self, rhs: f64) {
        *self = *self / rhs;
    }
}

impl Div for Finite {
    type Output = Self;

    #[track_caller]
    fn div(self, rhs: Self) -> Self::Output {
        self.apply2("div", rhs, |x, y| x / y)
    }
}

impl Div<f64> for Finite {
    type Output = Self;

    #[track_caller]
    fn div(self, rhs: f64) -> Self::Output {
        match Self::try_new(rhs) {
            Some(rhs) => self.apply2("div", rhs, |x, y| x / y),
            None => panic!("{} / {}", self.0, rhs),
        }
    }
}

impl Rem<f64> for Finite {
    type Output = Self;

    #[track_caller]
    fn rem(self, rhs: f64) -> Self::Output {
        match Self::try_new(rhs) {
            Some(rhs) => self.apply2("rem", rhs, |x, y| x % y),
            None => panic!("{} % {}", self.0, rhs),
        }
    }
}
