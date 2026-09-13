# Unicycle Forward Kinematics with linear linear and angular velocities

If for some inexplicable reason you want to model unicycle motion where your linear and angular velocities are linear (affine) functions, this library provides an efficient way to achieve that. You plug in your coordinate tolerance and it performs the integral to within that tolerance.

The technique is adaptive midpoint quadrature with a novel rigorous "error analysis" fit to this particular problem of unicycle kinematics.

## Why do you care about linear and angular velocities being linear functions?

It's an easy way to constrain the linear and angular velocity _and_ acceleration at the same time.

## Why not use adaptive midpoint quadrature with a typical error bound approximation?

The usual approaches can fail catastrophically in rare situations. If you expect to construct a lot of such curves, you'll inevitably hit a pathological case sooner or later and won't know it. You can additionally bound the error by setting a maximum interval size, but then you're not really relying on adaptive quadrature anymore and don't get the speed improvement it's supposed to deliver in the nominal case.

## Why not get an exact analytic result with Fresnel sin/Fresnel cos (which is what Wolfram Alpha gives when I plug in the integral)?

The libraries I could find implementing Fresnel sin and Fresnel cos don't take a tolerance parameter and evaluate 20-degree polynomials under the hood to get f64-ULP precision. This turns out to _not_ actually be particularly precise when you attempt to deal with numerical stability issues caused by catastrophic cancellation and small denominators in the overall expression Wolfram Alpha gives. Getting this "closed form" to be stable still involves arbitrary decisions for handling various places zeros can pop up and now you're at risk of both precision _and_ performance issues.

## Okay, but seriously _somebody_ out there must have already written a plug-and-play unicycle forward kinematics function with no caveats that "just works", right?

Yeah, you'd think that.

## Are you going to explain your error analysis technique? The code is basically undocumented

At some point, I probably should. For now, here's Claude Fable 5.1's explanation, which seems pretty good. It also caught an error I should deal with at some point that means that actually the error in the worst case is potentially larger by up to a factor of sqrt(2) (It says 1.2 but after more investigation I found it can get as bad as sqrt(2)). If you're worried about it, you can plug in a tolerance that's a factor of sqrt(2) lower:

### Claude's description

The error analysis lives in src/builder/error_analysis.rs and decides, for each window of the curve, whether the straight-segment approximation in forward_kinematics is accurate enough or whether the window must be split in half. Here is how it works.

What is being approximated

Each window is rescaled to time τ in [-1, 1], with v(τ) = v0 + aτ and ω(τ) = ω0 + ατ. Heading relative to the window start is θ(τ) = ω0(τ+1) + α(τ²−1)/2, and the true displacement is x = ∫cos θ(τ) v(τ) dτ, y = ∫sin θ(τ) v(τ) dτ.

The approximation used when the bound passes is a midpoint rule. Total signed distance ∫v dτ = 2v0 is exact, and it is all placed along the heading at the window midpoint, θ0 = θ(0) = ω0 − α/2. So the x error is ∫(cos θ(τ) − cos θ0) v(τ) dτ, and the function returns an interval width that provably contains that error. If both widths are at or below the tolerance, the segment is accepted. Otherwise the window is halved, each half gets half the tolerance, and the two halves are composed. The heading is always exact (2ω0), so only position error accumulates, and composition only rotates it.

The coarse bound

First it finds the range of θ over the window. θ is a quadratic, so quadratic_extrema evaluates it at τ = ±1 and at the single critical point τ = −ω0/α if that lies inside the window. The critical point is passed as a numerator and denominator so a zero α means "no interior critical point" instead of a division by zero. trig_quadratic_extrema then turns that θ range into ranges for sin and cos, checking any multiples of π/2 that fall inside the θ range, since those are where sin and cos hit ±1. Only four consecutive multiples need checking.

Since θ0 is itself in the θ range, |cos θ(τ) − cos θ0| ≤ cos_max − cos_min for every τ. Multiplying by the total absolute path length gives a valid bound. When |v0| < |a| the velocity changes sign inside the window, and the path length is v0²/|a| + |a|, which is the closed form of ∫|v0 + aτ| dτ. This coarse bound is all that is used in that case, because the finer argument below needs v to keep one sign.

The fine bound

When v does not change sign, the path length is 2|v0| and a tighter argument applies. Using the sum-to-product identity, cos θ(τ) − cos θ0 = −2 sin θc(τ) sin θr(τ), where θc = (θ(τ)+θ0)/2 and θr = (θ(τ)−θ0)/2. The bound is built from three pieces:

- The linear term is integrated exactly. mx = −ω0 sin θ0 is the derivative of cos θ(τ) at the midpoint. Subtracting mx·τ from the integrand leaves a residual h(τ) that is second order in τ. The removed term integrates to 2a·mx/3, and dividing by the path length gives bx, which seeds the interval so it always contains the exact linear contribution.
- sin θr is sandwiched. sin x − x is monotone decreasing, so over the range of θr it lies between its values at θr_min and θr_max. That means sin θr(τ) is bounded by θr(τ) plus a constant, and sin_x_minus_x uses a Taylor series near zero to avoid the cancellation that would otherwise cost precision in that constant.
- sin θc is replaced by its extrema. For each τ the residual is a product of two intervals, so its extremes occur at the four corners. For each corner, f(τ) = (θr(τ) + c)(−2s) − mx·τ is again a quadratic in τ, so quadratic_extrema finds its range over the window with the critical point at τ = (−mx − sω0)/(αs).

The union of these ranges, together with bx, is [L, U]. Since v has one sign, ∫h·v lies in [L, U] times 2v0, and after subtracting the linear contribution the error lies in an interval of width (U − L)·2|v0| that contains zero. The y bound is the same with cos θc and my = ω0 cos θ0. Finally each fine width is capped by the coarse width, since either one is valid.

The payoff is convergence rate. The coarse bound shrinks by about 4x per halving, while the fine bound shrinks by about 8x, because the residual after removing the tangent line is quadratic in the window size.

One caveat

Each half gets half the tolerance, but the right half's error is rotated by the left half's heading before it is added. A rotation can mix an x error and a y error into one axis, so a per-axis bound of tol/2 on each half can in principle produce a combined per-axis error up to about 1.2 times the tolerance. Treating the tolerance as a Euclidean bound, or splitting it slightly unevenly, would close that gap.