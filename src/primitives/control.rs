use std::ops::{Add, AddAssign, Div, Mul, Sub};

use crate::{finite::Finite, num::Num};

pub type Control<T> = LinearAndAngular<T>;
pub type SlewRate<T> = LinearAndAngular<T>;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LinearAndAngular<T> {
    pub linear: T,
    pub angular: T,
}

impl<T: Num> LinearAndAngular<T> {
    pub fn base_value(&self) -> LinearAndAngular<Finite> {
        LinearAndAngular {
            linear: self.linear.base_value(),
            angular: self.angular.base_value(),
        }
    }

    pub fn max(self, other: Self) -> Self {
        Self {
            linear: self.linear.max(other.linear),
            angular: self.angular.max(other.angular),
        }
    }

    pub fn min(self, other: Self) -> Self {
        Self {
            linear: self.linear.min(other.linear),
            angular: self.angular.min(other.angular),
        }
    }

    pub fn replace_value(self, value: LinearAndAngular<Finite>) -> Self {
        Self {
            linear: self.linear.replace_value(value.linear),
            angular: self.angular.replace_value(value.angular),
        }
    }
}

impl<T: AddAssign> AddAssign for LinearAndAngular<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.linear += rhs.linear;
        self.angular += rhs.angular;
    }
}

impl<T: AddAssign> Add for LinearAndAngular<T> {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        self += rhs;
        self
    }
}

impl<T: Sub<Output = T>> Sub for LinearAndAngular<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            linear: self.linear - rhs.linear,
            angular: self.angular - rhs.angular,
        }
    }
}

impl<T: Mul<Rhs, Output = T>, Rhs: Clone> Mul<Rhs> for LinearAndAngular<T> {
    type Output = Self;

    fn mul(self, rhs: Rhs) -> Self {
        Self {
            linear: self.linear * rhs.clone(),
            angular: self.angular * rhs,
        }
    }
}

impl<T: Div<Rhs, Output = T>, Rhs: Clone> Div<Rhs> for LinearAndAngular<T> {
    type Output = Self;

    fn div(self, rhs: Rhs) -> Self {
        Self {
            linear: self.linear / rhs.clone(),
            angular: self.angular / rhs,
        }
    }
}
