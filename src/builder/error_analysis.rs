use std::{f64::consts::FRAC_PI_2, mem};

use crate::{finite::Finite, num::Num};

pub(super) fn error_analysis(
    alpha: Finite,
    omega_0: Finite,
    a: Finite,
    v_0: Finite,
) -> (Finite, Finite) {
    if a == 0. && v_0 == 0. {
        return (Finite::ZERO, Finite::ZERO);
    }
    let theta = |t: Finite| (alpha / 2. * (t - 1.0) + omega_0) * (t + 1.0);
    let (sin_min, sin_max, cos_min, cos_max) = trig_quadratic_extrema(-omega_0, alpha, theta);
    let cos_diff = cos_max - cos_min;
    let sin_diff = sin_max - sin_min;

    let abs_v0 = v_0.abs();
    let abs_accel = a.abs();
    if abs_v0 < abs_accel {
        let dist = (v_0 * v_0) / abs_accel + abs_accel;
        return (cos_diff * dist, sin_diff * dist);
    }
    let dist = abs_v0 * 2.;

    let theta_0 = theta(Finite::ZERO);
    let theta_c = |t: Finite| (theta_0 + theta(t)) / 2.;
    let theta_r = |t: Finite| (theta(t) - theta_0) / 2.;
    let (sin, cos) = theta_0.sin_cos();
    let mx = omega_0 * (-sin);
    let my = omega_0 * cos;
    let t_eq = -(a / (v_0 * 3.));
    let bx = mx * t_eq;
    let by = my * t_eq;

    // Find extrema of theta_d and theta_m
    let (theta_r_min, theta_r_max) = quadratic_extrema(-omega_0, alpha, theta_r);
    let (sin_min, sin_max, cos_min, cos_max) = trig_quadratic_extrema(-omega_0, alpha, theta_c);

    let mut bx_lower = bx;
    let mut bx_upper = bx;
    let mut by_lower = by;
    let mut by_upper = by;

    for theta_r_ext in [theta_r_min, theta_r_max] {
        let sinx_minus_x = sin_x_minus_x(theta_r_ext);
        for sin_theta_m_ext in [sin_min, sin_max] {
            let f = |t: Finite| (theta_r(t) + sinx_minus_x) * -2. * sin_theta_m_ext - mx * t;
            let t_crit_num = -mx - omega_0 * sin_theta_m_ext;
            let t_crit_denom = alpha * sin_theta_m_ext;
            let (f_lower, f_upper) = quadratic_extrema(t_crit_num, t_crit_denom, f);
            bx_lower = bx_lower.min(f_lower);
            bx_upper = bx_upper.max(f_upper);
        }
        for cos_theta_m_ext in [cos_min, cos_max] {
            let f = |t: Finite| (theta_r(t) + sinx_minus_x) * 2. * cos_theta_m_ext - my * t;
            let t_crit_num = my - omega_0 * cos_theta_m_ext;
            let t_crit_denom = alpha * cos_theta_m_ext;
            let (f_lower, f_upper) = quadratic_extrema(t_crit_num, t_crit_denom, f);
            by_lower = by_lower.min(f_lower);
            by_upper = by_upper.max(f_upper);
        }
    }

    let x_error = (bx_upper - bx_lower).min(cos_diff) * dist;
    let y_error = (by_upper - by_lower).min(sin_diff) * dist;
    (x_error, y_error)
}

fn quadratic_extrema(
    crit_num: Finite,
    crit_denom: Finite,
    quadratic: impl Fn(Finite) -> Finite,
) -> (Finite, Finite) {
    let mut f_min = quadratic(-Finite::ONE);
    let mut f_max = quadratic(Finite::ONE);
    if f_max < f_min {
        mem::swap(&mut f_min, &mut f_max);
    }
    if crit_num.abs() < crit_denom.abs() {
        let t_crit = crit_num / crit_denom;
        let f_crit = quadratic(t_crit);
        f_min = f_min.min(f_crit);
        f_max = f_max.max(f_crit);
    }
    (f_min, f_max)
}

/// Returns (sin_min, sin_max, cos_min, cos_max)
fn trig_quadratic_extrema(
    crit_num: Finite,
    crit_denom: Finite,
    quadratic: impl Fn(Finite) -> Finite,
) -> (Finite, Finite, Finite, Finite) {
    let (f_min, f_max) = quadratic_extrema(crit_num, crit_denom, quadratic);
    // Technically, if we know we're using double-precision arithmetic we can support a much larger range, but we won't
    // be encountering such large values in practice so it doesn't matter.
    assert!(
        f_min >= -(2.0f64.powi(24)) && f_max <= 2.0f64.powi(24),
        "Bounds outside supported range {:?}",
        (f_min, f_max)
    );

    let (mut sin_min, mut cos_min) = f_min.sin_cos();
    let (mut sin_max, mut cos_max) = f_max.sin_cos();
    if sin_max < sin_min {
        mem::swap(&mut sin_min, &mut sin_max);
    }
    if cos_max < cos_min {
        mem::swap(&mut cos_min, &mut cos_max);
    }

    // Check multiples of pi/2 between f_min and f_max
    let start = (f_min / FRAC_PI_2).to_f64().ceil() as i32;
    let end = (f_max / FRAC_PI_2).to_f64().floor() as i32;

    for k in start..=end.min(start + 3) {
        match k.rem_euclid(4) {
            0 => cos_max = Finite::ONE,
            1 => sin_max = Finite::ONE,
            2 => cos_min = -Finite::ONE,
            3 => sin_min = -Finite::ONE,
            _ => unreachable!(),
        }
    }

    (sin_min, sin_max, cos_min, cos_max)
}

fn sin_x_minus_x(x: Finite) -> Finite {
    if x.abs() < 0.1 {
        let xsqr = x * x;
        // 7th degree Taylor series expansion of y = sin(x) - x
        //
        // The error is within x^9 / 9! which is a little better than evaluating sin(x) - x
        // directly in f64 arithmetic where x is close to 0.1
        // (The naive expression retains 43 bits of precision, but the error is ~ y * 1/2**45)
        -x * (xsqr / 6. * (Finite::ONE - xsqr / 20. * (Finite::ONE - xsqr / 42.)))
    } else {
        x.sin() - x
    }
}
