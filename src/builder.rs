use crate::{
    finite::Finite,
    num::Num,
    primitives::{control::SlewRate, Control, Isometry},
};

use self::error_analysis::error_analysis;

mod error_analysis;

pub(super) struct CurveBuilder<T: Num, F> {
    /// The "true" time (in the absolute 0 to 1 range) that we're currently treating as "t_0" (in the relative -1 to 1
    /// range).
    t_0: Finite,
    /// The difference between t_0 and the start or end of the window currently being considered. Together `t_0` and
    /// `d_t` express a "view" of a portion of the curve.
    d_t: Finite,
    last_isometry: Isometry<T>,
    /// The first element of the tuple is the time since the start of the curve rescaled so that the entire duration is
    /// from to [0, 1].
    isometry_accum: F,
}

impl<T: Num, F: FnMut(Finite, &Isometry<T>)> CurveBuilder<T, F> {
    pub(super) fn new(mut isometry_accum: F) -> Self {
        isometry_accum(Finite::ZERO, &Isometry::<T>::default());
        Self {
            t_0: Finite::HALF,
            d_t: Finite::HALF,
            last_isometry: Isometry::<T>::default(),
            isometry_accum,
        }
    }

    pub(super) fn build(
        &mut self,
        start_control: &Control<T>,
        end_control: &Control<T>,
        duration_secs: Finite,
        tolerance: Finite,
    ) {
        assert!(duration_secs >= 0., "{duration_secs:?}");
        assert!(tolerance > 0., "{tolerance:?}");

        let mean_control = (start_control.clone() + end_control.clone()) / 2.;
        let control_diff = end_control.clone() - start_control.clone();

        // Rescale so time range is [-1, 1]. This makes the mean_control occur at time 0.
        let scale = duration_secs / 2.;
        let slew_rate = control_diff * (scale / 2.); // Equivalent to slew_rate * scale²
        let control_0 = mean_control * scale;

        // Call the internal function with new parameters
        let end_isometry = self.forward_kinematics(slew_rate, control_0, tolerance, 0);
        self.add_isometry(end_isometry);
    }

    /// Duration is continually re-scaled so that the range being considered is always [-1, 1]. Thus self.t_0 indicates
    /// the _midpoint_ of the time range this invocation is computing forward kinematics for (not the beginning).
    fn forward_kinematics(
        &mut self,
        slew_rate: SlewRate<T>,
        control_0: Control<T>,
        tolerance: Finite,
        depth: usize,
    ) -> Isometry<T> {
        assert!(tolerance > 0., "{tolerance:?}");
        let SlewRate {
            linear: lin_accel,
            angular: ang_accel,
        } = &slew_rate;
        let Control {
            linear: lin_0,
            angular: ang_0,
        } = control_0;

        let theta = ang_0.clone() * 2.0;
        let (x_error, y_error) = error_analysis(
            ang_accel.base_value(),
            ang_0.base_value(),
            lin_accel.base_value(),
            lin_0.base_value(),
        );

        if x_error <= tolerance && y_error <= tolerance {
            let SlewRate {
                linear: _a,
                angular: alpha,
            } = slew_rate;
            let theta_0 = ang_0 - alpha / 2.0;
            let (sin, cos) = theta_0.sin_cos();
            let x_approx = lin_0.clone() * cos * 2.0;
            let y_approx = lin_0 * sin * 2.0;
            return Isometry {
                x: x_approx,
                y: y_approx,
                theta,
            };
        }
        let left_v_0 = lin_0.clone() - lin_accel.clone() / 2.0;
        let left_omega_0 = ang_0.clone() - ang_accel.clone() / 2.0;
        let left_control = Control {
            linear: left_v_0,
            angular: left_omega_0,
        };
        let right_v_0 = lin_0 + lin_accel.clone() / 2.0;
        let right_omega_0 = ang_0 + ang_accel.clone() / 2.0;
        let right_control = Control {
            linear: right_v_0,
            angular: right_omega_0,
        };
        let current_isometry = self.end_isometry().clone();

        let next_slew_rate = slew_rate / 4.0;

        self.descend_left();
        let left = self.forward_kinematics(
            next_slew_rate.clone(),
            left_control / 2.0,
            tolerance / 2.0,
            depth + 1,
        );
        self.add_isometry(current_isometry * left.clone());
        self.switch_to_right();
        let right = self.forward_kinematics(
            next_slew_rate,
            right_control / 2.0,
            tolerance / 2.0,
            depth + 1,
        );
        self.ascend_from_right();

        let Isometry { x, y, theta: _ } = left * right;
        // This theta value may have less machine precision error than the one we got from combining the two halves.
        Isometry { x, y, theta }
    }

    fn add_isometry(&mut self, isometry: Isometry<T>) {
        (self.isometry_accum)(self.t_0 + self.d_t, &isometry);
        self.last_isometry = isometry;
    }

    fn end_isometry(&self) -> &Isometry<T> {
        &self.last_isometry
    }

    pub(super) fn into_end_isometry(self) -> Isometry<T> {
        self.last_isometry
    }

    fn descend_left(&mut self) {
        self.d_t /= 2.;
        self.t_0 -= self.d_t;
    }

    fn switch_to_right(&mut self) {
        self.t_0 += self.d_t * 2.;
    }

    fn ascend_from_right(&mut self) {
        self.t_0 -= self.d_t;
        self.d_t *= 2.;
    }
}
