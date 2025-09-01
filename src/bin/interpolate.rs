use std::env;
use std::str::FromStr;

use unicycle::primitives::{Control, Isometry};
use unicycle::{Curve, Finite};

fn parse_arg<T: FromStr>(name: &str, idx: usize, args: &[String]) -> T
where
    <T as FromStr>::Err: std::fmt::Display,
{
    args.get(idx)
        .unwrap_or_else(|| panic!("missing {name}"))
        .parse::<T>()
        .unwrap_or_else(|e| panic!("invalid {name}: {e}"))
}

// CLI
// Usage:
//   cargo run --bin interpolate <start_linear> <start_angular> <end_linear> <end_angular> <duration_secs> <tolerance>
// All values are f64.
// Output: CSV with header: t,x,y,theta
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 || args.iter().any(|a| a == "-h" || a == "--help") {
        eprintln!("Usage: {} <start_linear> <start_angular> <end_linear> <end_angular> <duration_secs> <tolerance>", args[0]);
        std::process::exit(1);
    }
    assert!(
        args.len() == 7,
        "expected 6 positional arguments, got {}",
        args.len() - 1
    );
    let start_linear: f64 = parse_arg("start_linear", 1, &args);
    let start_angular: f64 = parse_arg("start_angular", 2, &args);
    let end_linear: f64 = parse_arg("end_linear", 3, &args);
    let end_angular: f64 = parse_arg("end_angular", 4, &args);
    let duration_secs: f64 = parse_arg("duration_secs", 5, &args);
    let tolerance: f64 = parse_arg("tolerance", 6, &args);

    let start_control: Control<Finite> = Control {
        linear: Finite::try_new(start_linear).unwrap(),
        angular: Finite::try_new(start_angular).unwrap(),
    };
    let end_control: Control<Finite> = Control {
        linear: Finite::try_new(end_linear).unwrap(),
        angular: Finite::try_new(end_angular).unwrap(),
    };
    let duration = Finite::try_new(duration_secs).expect("duration must be finite");
    let tol = Finite::try_new(tolerance).expect("tolerance must be finite");

    let curve: Curve = Curve::construct(start_control, end_control, duration, tol);

    println!("t,x,y,theta");
    for (t, iso) in curve.interpolation_points() {
        print_row(*t * (1.0), *iso); // t already in [0,duration]
    }
    // Ensure final point at duration exactly present (could already be the last interpolation point)
    if curve.interpolation_points().last().map(|(t, _)| *t) != Some(duration) {
        let end_iso = curve.end_isometry().clone();
        print_row(duration, end_iso);
    }
}

fn print_row(t: Finite, isom: Isometry<Finite>) {
    println!(
        "{},{},{},{}",
        t.to_f64(),
        isom.x.to_f64(),
        isom.y.to_f64(),
        isom.theta.to_f64()
    );
}
