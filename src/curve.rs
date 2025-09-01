use std::cmp::Ordering;

use crate::{
    finite::Finite,
    primitives::{Control, Isometry},
};

pub struct Curve {
    pub(super) start_control: Control<Finite>,
    pub(super) end_control: Control<Finite>,
    pub(super) duration: Finite,
    pub(super) interpolation_points: Vec<(Finite, Isometry<Finite>)>,
}

impl Curve {
    /// `time` is time since the start of the curve in seconds.
    #[track_caller]
    pub fn control_at(&self, time: Finite) -> Control<Finite> {
        assert!(time >= 0., "time {time:?} is negative");
        assert!(
            time <= self.duration,
            "time {time:?} exceeds {:?}",
            self.duration
        );

        let start_control_value = self.start_control.base_value();
        let end_control_value = self.end_control.base_value();
        let min_control = start_control_value.min(end_control_value);
        let max_control = start_control_value.max(end_control_value);

        let control = self.start_control * ((self.duration - time) / self.duration)
            + self.end_control * (time / self.duration);

        let control_value = control.base_value();
        // Avoid letting floating point errors push the control outside the bounds of the start and end controls.
        control.replace_value(control_value.max(min_control).min(max_control))
    }

    pub fn duration(&self) -> Finite {
        self.duration
    }

    pub fn start_control(&self) -> &Control<Finite> {
        &self.start_control
    }

    pub fn end_control(&self) -> &Control<Finite> {
        &self.end_control
    }

    pub fn end_isometry(&self) -> &Isometry<Finite> {
        &self.interpolation_points.last().unwrap().1
    }

    /// `time` is time since the start of the curve in seconds.
    pub fn isometry_at(&self, time: Finite) -> Isometry<Finite> {
        // Since the time points tend to be distributed approximately uniformly, interpolation search will be faster
        // than a binary search.
        let (i, j) = match interpolation_search(&self.interpolation_points, time, |&(t, _)| t) {
            Ok(i) => return self.interpolation_points[i].1,
            Err(i) => {
                assert!(
                    i > 0 && i < self.interpolation_points.len(),
                    "time {time:?} is out of bounds"
                );
                (i - 1, i)
            }
        };
        let (t_i, isom_i) = &self.interpolation_points[i];
        let (t_j, _) = self.interpolation_points[j];
        let middle_t = (*t_i + t_j) / 2.;
        let middle_theta = self.theta_at(middle_t);
        let d_theta = middle_theta - isom_i.theta;
        let a = self.lin_accel();
        let v_i = self.start_control.linear + a * *t_i;
        let v_time = self.start_control.linear + a * time;
        let dist = (v_i + v_time) * ((time - *t_i) / 2.);
        let Isometry { x, y, theta: _ } =
            *isom_i * Isometry::rotation(d_theta) * Isometry::forward_translation(dist);
        let theta_t = self.theta_at(time);
        Isometry {
            x,
            y,
            theta: theta_t,
        }
    }

    pub fn lin_accel(&self) -> Finite {
        self.end_control.linear - self.start_control.linear
    }

    pub fn ang_accel(&self) -> Finite {
        (self.end_control.angular - self.start_control.angular) / self.duration
    }

    fn theta_at(&self, t: Finite) -> Finite {
        (self.ang_accel() * t / 2. + self.start_control.angular) * t
    }

    /// These are not necessarily uniformly distributed. The distribution is based on an on-the-fly error analysis done
    /// at curve construction which puts hard bounds on the overall x and y error (theta will be accurate to within
    /// machine precision).
    pub fn interpolation_points(&self) -> &[(Finite, Isometry<Finite>)] {
        &self.interpolation_points
    }
}

pub fn interpolation_search<T>(
    arr: &[T],
    target: Finite,
    mut key: impl FnMut(&T) -> Finite,
) -> Result<usize, usize> {
    let mut left = 0;
    let mut right = arr.len() - 1;
    match target.partial_cmp(&key(&arr[left])).unwrap() {
        Ordering::Less => return Err(left),
        Ordering::Equal => return Ok(left),
        Ordering::Greater => {}
    }
    match target.partial_cmp(&key(&arr[right])).unwrap() {
        Ordering::Less => {}
        Ordering::Equal => return Ok(right),
        Ordering::Greater => return Err(right + 1),
    }
    while right - left > 1 {
        let left_key = key(&arr[left]);
        let right_key = key(&arr[right]);
        // Rather than splitting the range in half as we would with a binary search, we split it based on interpolating
        // the target value between the left and right keys.
        let pos = left
            + ((target - left_key) / (right_key - left_key) * (right - left) as f64)
                .to_f64()
                .round() as usize;
        match target.partial_cmp(&key(&arr[pos])).unwrap() {
            Ordering::Less => match target.partial_cmp(&key(&arr[pos - 1])).unwrap() {
                Ordering::Less => right = pos - 1,
                Ordering::Equal => return Ok(pos - 1),
                Ordering::Greater => return Err(pos),
            },
            Ordering::Equal => return Ok(pos),
            Ordering::Greater => match target.partial_cmp(&key(&arr[pos + 1])).unwrap() {
                Ordering::Less => return Err(pos + 1),
                Ordering::Equal => return Ok(pos + 1),
                Ordering::Greater => left = pos + 1,
            },
        }
    }
    Err(right)
}
