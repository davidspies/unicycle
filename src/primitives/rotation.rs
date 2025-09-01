use std::f64::consts::{PI, TAU};
use std::mem;
use std::ops::{Add, AddAssign, Neg, Sub};

use anyhow::{ensure, Context};

use crate::finite::Finite;
use crate::num::Num;

/// Restricts T to be in the range (-π, π].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rotation<T>(T);

impl<T: Num> Rotation<T> {
    pub fn new_normalize(value: T) -> Self {
        let mut result = Self(value % TAU);
        result.wrap_once();
        result
    }

    pub fn try_new(value: T) -> anyhow::Result<Self> {
        ensure!(
            value > -PI && value <= PI,
            "rotation value exceeds (-pi, pi]: {value:?}"
        );
        Ok(Self(value))
    }

    pub fn angle_to(self, other: Self) -> Self {
        other - self
    }

    pub fn base_value(&self) -> Rotation<Finite> {
        Rotation(self.0.base_value())
    }

    pub fn sin_cos(self) -> (T, T) {
        self.0.sin_cos()
    }

    /// Returns the underlying scalar angle value in radians in the range (-pi, pi].
    pub fn value(&self) -> T {
        self.0.clone()
    }

    // Normalizes any fp64 value in the range (-3pi, 3pi] to the range (-pi, pi].
    #[track_caller]
    fn wrap_once(&mut self) {
        wrap_once(&mut self.0);
        assert!(
            self.0 > -PI && self.0 <= PI,
            "rotation value exceeds (-pi, pi]: {:?}",
            self.0
        )
    }

    #[track_caller]
    fn apply(self, fname: &str, f: impl FnOnce(T) -> T) -> Self {
        let value = f(self.0.clone());
        Self::try_new(value)
            .with_context(|| format!("{fname}({:?})", self.0))
            .unwrap()
    }

    #[track_caller]
    fn apply2<U: Num>(fname: &str, a: T, b: U, f: impl FnOnce(T, U) -> T) -> Self {
        let value = f(a.clone(), b.clone());
        Self::try_new(value)
            .with_context(|| format!("{fname}({a:?}, {b:?})"))
            .unwrap()
    }
}

fn wrap_once<T: Num>(value: &mut T) {
    if *value <= -PI {
        *value += TAU;
    } else if *value > PI {
        *value -= TAU;
    }
}

impl<T: Num> Neg for Rotation<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.apply("neg", |x| {
            let mut result = -x;
            if result.base_value() == -PI {
                result += TAU;
            }
            result
        })
    }
}

impl<T: Num + Add<U, Output = T>, U: Num> AddAssign<Rotation<U>> for Rotation<T> {
    fn add_assign(&mut self, rhs: Rotation<U>) {
        *self = mem::take(self) + rhs;
    }
}

impl<T: Num + Add<U, Output = T>, U: Num> Add<Rotation<U>> for Rotation<T> {
    type Output = Self;

    fn add(self, rhs: Rotation<U>) -> Self::Output {
        Self::apply2("add", self.0, rhs.0, |a, b| {
            let a_base = a.base_value();
            let b_base = b.base_value();
            let test_sum = a_base + b_base;
            // Avoid losing precision by ensuring no intermediate term exceeds PI in absolute value.
            // This gives us the property that a - b == 0 => a == b.
            if test_sum > PI {
                (a - PI) + (b - PI)
            } else if test_sum <= -PI {
                let result = (a + PI) + (b + PI);
                // This is necessary because of floating point precision issues.
                // Ex. a= -1 - 2 ** -52, b = 1 - pi + 2 ** -51
                // >>> a + b <= -pi
                // True
                // >>> (a + pi) + (b + pi) > pi
                // True
                if result > PI {
                    result.replace_value(Finite::PI)
                } else {
                    result
                }
            } else {
                a + b
            }
        })
    }
}

impl<T: Num + Add<U, Output = T>, U: Num> Sub<Rotation<U>> for Rotation<T> {
    type Output = Self;

    fn sub(self, rhs: Rotation<U>) -> Self::Output {
        self + (-rhs)
    }
}
