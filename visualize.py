#!/usr/bin/env python3
"""Visualize interpolation points produced by the unicycle interpolate binary.

Usage:
  python visualize.py --start-linear 1.0 --start-angular 0.0 --end-linear 0.0 --end-angular 1.0 --duration 5.0 --tolerance 1e-6

This will compile (if needed) and run `cargo run --bin interpolate ...`, capture CSV output, and plot:
  - x vs y path
  - time vs theta
  - time vs linear speed (approximate using finite differences of position)

Requires matplotlib.
"""
from __future__ import annotations
import argparse
import csv
import subprocess
import sys
import math
from dataclasses import dataclass
from pathlib import Path


@dataclass
class Point:
    t: float
    x: float
    y: float
    theta: float


def run_interpolate(args) -> list[Point]:
    cmd = [
        "cargo",
        "run",
        "--quiet",
        "--bin",
        "interpolate",
        "--",
        str(args.start_linear),
        str(args.start_angular),
        str(args.end_linear),
        str(args.end_angular),
        str(args.duration),
        str(args.tolerance),
    ]
    proc = subprocess.run(cmd, capture_output=True, text=True)
    if proc.returncode != 0:
        print(proc.stderr, file=sys.stderr)
        raise SystemExit(proc.returncode)
    reader = csv.DictReader(line for line in proc.stdout.splitlines() if line.strip())
    points: list[Point] = []
    for row in reader:
        points.append(
            Point(
                t=float(row["t"]),
                x=float(row["x"]),
                y=float(row["y"]),
                theta=float(row["theta"]),
            )
        )
    return points


def plot(points: list[Point]):
    import matplotlib.pyplot as plt

    if not points:
        print("No points to visualize", file=sys.stderr)
        return
    t = [p.t for p in points]
    x = [p.x for p in points]
    y = [p.y for p in points]
    theta = [p.theta for p in points]

    # Approximate linear speed along path (finite differences)
    v = []
    for i in range(1, len(points)):
        dt = points[i].t - points[i - 1].t
        if dt <= 0:
            dt = 1e-9
        dx = points[i].x - points[i - 1].x
        dy = points[i].y - points[i - 1].y
        v.append(math.hypot(dx, dy) / dt)

    fig, axs = plt.subplots(2, 2, figsize=(10, 8))
    axs[0, 0].plot(x, y, "-o", markersize=2)
    axs[0, 0].set_title("Path (x vs y)")
    axs[0, 0].set_xlabel("x (m)")
    axs[0, 0].set_ylabel("y (m)")
    axs[0, 0].axis("equal")

    axs[0, 1].plot(t, theta)
    axs[0, 1].set_title("Theta vs Time")
    axs[0, 1].set_xlabel("t (s)")
    axs[0, 1].set_ylabel("theta (rad)")

    midpoints = [(t[i - 1] + t[i]) / 2 for i in range(1, len(t))]
    axs[1, 0].plot(midpoints, v)
    axs[1, 0].set_title("Approx Linear Speed vs Time")
    axs[1, 0].set_xlabel("t (s)")
    axs[1, 0].set_ylabel("speed (m/s)")

    axs[1, 1].axis("off")
    axs[1, 1].text(0.02, 0.98, f"Points: {len(points)}", va="top")

    fig.tight_layout()
    plt.show()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--start-linear", type=float, required=True)
    parser.add_argument("--start-angular", type=float, required=True)
    parser.add_argument("--end-linear", type=float, required=True)
    parser.add_argument("--end-angular", type=float, required=True)
    parser.add_argument("--duration", type=float, required=True)
    parser.add_argument("--tolerance", type=float, default=1e-6)
    args = parser.parse_args()

    points = run_interpolate(args)
    plot(points)


if __name__ == "__main__":
    main()
