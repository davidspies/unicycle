use std::fmt::Debug;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Rem, Sub, SubAssign};

use crate::finite::Finite;

pub trait Num:
    Clone
    + Debug
    + Default
    + Ord
    + PartialOrd<f64>
    + Neg<Output = Self>
    + Add<Output = Self>
    + Add<f64, Output = Self>
    + AddAssign
    + AddAssign<f64>
    + Sub<Output = Self>
    + Sub<f64, Output = Self>
    + SubAssign<f64>
    + Mul<Output = Self>
    + Mul<Finite, Output = Self>
    + Mul<f64, Output = Self>
    + Div<f64, Output = Self>
    + Rem<f64, Output = Self>
{
    fn base_value(&self) -> Finite;
    fn replace_value(self, value: Finite) -> Self;
    fn sin_cos(self) -> (Self, Self);
}
