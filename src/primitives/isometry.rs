use std::ops::{Mul, MulAssign};

use crate::num::Num;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Isometry<T> {
    pub x: T,
    pub y: T,
    pub theta: T,
}

impl<T: Default> Isometry<T> {
    pub fn forward_translation(x: T) -> Self {
        Self {
            x,
            ..Self::default()
        }
    }

    pub fn rotation(theta: T) -> Self {
        Self {
            theta,
            ..Self::default()
        }
    }
}

impl<T: Num> MulAssign for Isometry<T> {
    fn mul_assign(&mut self, rhs: Self) {
        let (sin, cos) = self.theta.clone().sin_cos();
        self.x += cos.clone() * rhs.x.clone() - sin.clone() * rhs.y.clone();
        self.y += sin * rhs.x + cos * rhs.y;
        self.theta += rhs.theta;
    }
}

impl<T: Num> Mul for Isometry<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result *= rhs;
        result
    }
}
