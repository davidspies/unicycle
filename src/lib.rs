use self::builder::CurveBuilder;
use self::primitives::{Control, Isometry};

pub use self::curve::Curve;
pub use self::finite::Finite;
pub use self::num::Num;

mod builder;
mod curve;
mod finite;
mod num;

pub mod primitives;

impl Curve {
    /// The curve that results from linearly interpolating between two controls over the specified duration.
    /// In other words, the resulting end isometry will be:
    ///
    /// x = ∫ cos(θ(t)) · v(t) dt over \[0, duration\]
    ///
    /// y = ∫ sin(θ(t)) · v(t) dt over \[0, duration\]
    ///
    /// θ = θ(duration)
    ///
    /// where:
    ///
    /// v(t) = start_control.linear + (end_control.linear - start_control.linear) · t
    ///
    /// ω(t) = start_control.angular + (end_control.angular - start_control.angular) · t
    ///
    /// θ(T) = ∫ ω(t) dt over \[0, T\]
    ///
    /// The final x and y values are guaranteed to be within the specified tolerance of the true value.
    ///
    /// ⚠️ Note: Smaller tolerance values will result in more interpolation points and take longer to compute. Precision
    /// bounds smaller than floating point ULP in the relevant range may result in unbounded recursion.
    pub fn construct(
        start_control: Control<Finite>,
        end_control: Control<Finite>,
        duration_secs: Finite,
        tolerance: Finite,
    ) -> Self {
        let mut interpolation_points = Vec::new();
        let mut builder = CurveBuilder::new(|t, &isometry| {
            interpolation_points.push((t * duration_secs, isometry))
        });
        builder.build(&start_control, &end_control, duration_secs, tolerance);
        Curve {
            start_control,
            end_control,
            duration: duration_secs,
            interpolation_points,
        }
    }
}

pub fn forward_kinematics<T: Num>(
    start_control: Control<T>,
    end_control: Control<T>,
    duration_secs: Finite,
    tolerance: Finite,
) -> Isometry<T> {
    let mut builder = CurveBuilder::new(|_, _| {});
    builder.build(&start_control, &end_control, duration_secs, tolerance);
    builder.into_end_isometry()
}
