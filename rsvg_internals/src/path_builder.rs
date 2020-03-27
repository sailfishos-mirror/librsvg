//! Representation of Bézier paths.

use smallvec::SmallVec;

use std::f64;
use std::f64::consts::*;
use std::slice;

use crate::float_eq_cairo::ApproxEqCairo;
use crate::util::clamp;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LargeArc(pub bool);

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Sweep {
    Negative,
    Positive,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CubicBezierCurve {
    /// The (x, y) coordinates of the first control point.
    pub pt1: (f64, f64),
    /// The (x, y) coordinates of the second control point.
    pub pt2: (f64, f64),
    /// The (x, y) coordinates of the end point of this path segment.
    pub to: (f64, f64),
}

impl CubicBezierCurve {
    fn to_cairo(self, cr: &cairo::Context) {
        let Self { pt1, pt2, to } = self;
        cr.curve_to(pt1.0, pt1.1, pt2.0, pt2.1, to.0, to.1);
    }

    fn from_coords(coords: &[f64]) -> CubicBezierCurve {
        let pt1 = (coords[0], coords[1]);
        let pt2 = (coords[2], coords[3]);
        let to = (coords[4], coords[5]);

        CubicBezierCurve { pt1, pt2, to }
    }

    fn to_packed_and_coords(&self, coords: &mut [f64]) -> PackedCommand {
        coords[0] = self.pt1.0;
        coords[1] = self.pt1.1;
        coords[2] = self.pt2.0;
        coords[3] = self.pt2.1;
        coords[4] = self.to.0;
        coords[5] = self.to.1;
        PackedCommand::CurveTo
    }
}

/// When attempting to compute the center parameterization of the arc,
/// out of range parameters may see an arc omited or treated as a line.
pub enum ArcParameterization {
    /// Center parameterization of the arc.
    CenterParameters {
        /// Center of the ellipse.
        center: (f64, f64),
        /// Radii of the ellipse (corrected).
        radii: (f64, f64),
        /// Angle of the start point.
        theta1: f64,
        /// Delta angle to the end point.
        delta_theta: f64,
    },
    /// Treat the arc as a line to the end point.
    LineTo,
    /// Omit the arc.
    Omit,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct EllipticalArc {
    /// The (x-axis, y-axis) radii for the ellipse.
    pub r: (f64, f64),
    /// The rotation angle in degrees for the ellipse's x-axis
    /// relative to the x-axis of the user coordinate system.
    pub x_axis_rotation: f64,
    /// Flag indicating whether the arc sweep should be
    /// greater than or equal to 180 degrees, or smaller than 180 degrees.
    pub large_arc: LargeArc,
    /// Flag indicating the angular direction in which the arc is drawn.
    pub sweep: Sweep,
    /// The (x, y) coordinates for the start point of this path segment.
    pub from: (f64, f64),
    /// The (x, y) coordinates for the end point of this path segment.
    pub to: (f64, f64),
}

impl EllipticalArc {
    /// Calculates a center parameterization from the endpoint parameterization.
    ///
    /// Radii may be adjusted if there is no solution.
    ///
    /// See Appendix F.6 Elliptical arc implementation notes.
    /// http://www.w3.org/TR/SVG/implnote.html#ArcImplementationNotes
    pub(crate) fn center_parameterization(self) -> ArcParameterization {
        let Self {
            r: (mut rx, mut ry),
            x_axis_rotation,
            large_arc,
            sweep,
            from: (x1, y1),
            to: (x2, y2),
        } = self;

        // If the end points are identical, omit the arc segment entirely.
        if x1.approx_eq_cairo(x2) && y1.approx_eq_cairo(y2) {
            return ArcParameterization::Omit;
        }

        // Ensure radii are non-zero.
        // Otherwise this arc is treated as a line segment joining the end points.
        //
        // A bit further down we divide by the square of the radii.
        // Check that we won't divide by zero.
        // See http://bugs.debian.org/508443
        if rx * rx < f64::EPSILON || ry * ry < f64::EPSILON {
            return ArcParameterization::LineTo;
        }

        let is_large_arc = large_arc.0;
        let is_positive_sweep = sweep == Sweep::Positive;

        let phi = x_axis_rotation * PI / 180.0;
        let (sin_phi, cos_phi) = phi.sin_cos();

        // Ensure radii are positive.
        rx = rx.abs();
        ry = ry.abs();

        // The equations simplify after a translation which places
        // the origin at the midpoint of the line joining (x1, y1) to (x2, y2),
        // followed by a rotation to line up the coordinate axes
        // with the axes of the ellipse.
        // All transformed coordinates will be written with primes.
        //
        // Compute (x1', y1').
        let mid_x = (x1 - x2) / 2.0;
        let mid_y = (y1 - y2) / 2.0;
        let x1_ = cos_phi * mid_x + sin_phi * mid_y;
        let y1_ = -sin_phi * mid_x + cos_phi * mid_y;

        // Ensure radii are large enough.
        let lambda = (x1_ / rx).powi(2) + (y1_ / ry).powi(2);
        if lambda > 1.0 {
            // If not, scale up the ellipse uniformly
            // until there is exactly one solution.
            rx *= lambda.sqrt();
            ry *= lambda.sqrt();
        }

        // Compute the transformed center (cx', cy').
        let d = (rx * y1_).powi(2) + (ry * x1_).powi(2);
        if d == 0.0 {
            return ArcParameterization::Omit;
        }
        let k = {
            let mut k = ((rx * ry).powi(2) / d - 1.0).abs().sqrt();
            if is_positive_sweep == is_large_arc {
                k = -k;
            }
            k
        };
        let cx_ = k * rx * y1_ / ry;
        let cy_ = -k * ry * x1_ / rx;

        // Compute the center (cx, cy).
        let cx = cos_phi * cx_ - sin_phi * cy_ + (x1 + x2) / 2.0;
        let cy = sin_phi * cx_ + cos_phi * cy_ + (y1 + y2) / 2.0;

        // Compute the start angle θ1.
        let ux = (x1_ - cx_) / rx;
        let uy = (y1_ - cy_) / ry;
        let u_len = (ux * ux + uy * uy).abs().sqrt();
        if u_len == 0.0 {
            return ArcParameterization::Omit;
        }
        let cos_theta1 = clamp(ux / u_len, -1.0, 1.0);
        let theta1 = {
            let mut theta1 = cos_theta1.acos();
            if uy < 0.0 {
                theta1 = -theta1;
            }
            theta1
        };

        // Compute the total delta angle Δθ.
        let vx = (-x1_ - cx_) / rx;
        let vy = (-y1_ - cy_) / ry;
        let v_len = (vx * vx + vy * vy).abs().sqrt();
        if v_len == 0.0 {
            return ArcParameterization::Omit;
        }
        let dp_uv = ux * vx + uy * vy;
        let cos_delta_theta = clamp(dp_uv / (u_len * v_len), -1.0, 1.0);
        let delta_theta = {
            let mut delta_theta = cos_delta_theta.acos();
            if ux * vy - uy * vx < 0.0 {
                delta_theta = -delta_theta;
            }
            if is_positive_sweep && delta_theta < 0.0 {
                delta_theta += PI * 2.0;
            } else if !is_positive_sweep && delta_theta > 0.0 {
                delta_theta -= PI * 2.0;
            }
            delta_theta
        };

        ArcParameterization::CenterParameters {
            center: (cx, cy),
            radii: (rx, ry),
            theta1,
            delta_theta,
        }
    }

    fn to_cairo(self, cr: &cairo::Context) {
        match self.center_parameterization() {
            ArcParameterization::CenterParameters {
                center,
                radii,
                theta1,
                delta_theta,
            } => {
                let n_segs = (delta_theta / (PI * 0.5 + 0.001)).abs().ceil() as u32;
                let d_theta = delta_theta / f64::from(n_segs);

                let mut theta = theta1;
                for _ in 0..n_segs {
                    arc_segment(center, radii, self.x_axis_rotation, theta, theta + d_theta)
                        .to_cairo(cr);
                    theta += d_theta;
                }
            }
            ArcParameterization::LineTo => {
                let (x2, y2) = self.to;
                cr.line_to(x2, y2);
            }
            ArcParameterization::Omit => {}
        }
    }

    fn from_coords(large_arc: LargeArc, sweep: Sweep, coords: &[f64]) -> EllipticalArc {
        let r = (coords[0], coords[1]);
        let x_axis_rotation = coords[2];
        let from = (coords[3], coords[4]);
        let to = (coords[5], coords[6]);

        EllipticalArc {
            r,
            x_axis_rotation,
            large_arc,
            sweep,
            from,
            to,
        }
    }

    fn to_packed_and_coords(&self, coords: &mut [f64]) -> PackedCommand {
        coords[0] = self.r.0;
        coords[1] = self.r.1;
        coords[2] = self.x_axis_rotation;
        coords[3] = self.from.0;
        coords[4] = self.from.1;
        coords[5] = self.to.0;
        coords[6] = self.to.1;

        match (self.large_arc, self.sweep) {
            (LargeArc(false), Sweep::Negative) => PackedCommand::ArcSmallNegative,
            (LargeArc(false), Sweep::Positive) => PackedCommand::ArcSmallPositive,
            (LargeArc(true), Sweep::Negative) => PackedCommand::ArcLargeNegative,
            (LargeArc(true), Sweep::Positive) => PackedCommand::ArcLargePositive,
        }
    }
}

/// Turns an arc segment into a cubic bezier curve.
///
/// Takes the center, the radii and the x-axis rotation of the ellipse,
/// the angles of the start and end points,
/// and returns cubic bezier curve parameters.
pub(crate) fn arc_segment(
    c: (f64, f64),
    r: (f64, f64),
    x_axis_rotation: f64,
    th0: f64,
    th1: f64,
) -> CubicBezierCurve {
    let (cx, cy) = c;
    let (rx, ry) = r;
    let phi = x_axis_rotation * PI / 180.0;
    let (sin_phi, cos_phi) = phi.sin_cos();
    let (sin_th0, cos_th0) = th0.sin_cos();
    let (sin_th1, cos_th1) = th1.sin_cos();

    let th_half = 0.5 * (th1 - th0);
    let t = (8.0 / 3.0) * (th_half * 0.5).sin().powi(2) / th_half.sin();
    let x1 = rx * (cos_th0 - t * sin_th0);
    let y1 = ry * (sin_th0 + t * cos_th0);
    let x3 = rx * cos_th1;
    let y3 = ry * sin_th1;
    let x2 = x3 + rx * (t * sin_th1);
    let y2 = y3 + ry * (-t * cos_th1);

    CubicBezierCurve {
        pt1: (
            cx + cos_phi * x1 - sin_phi * y1,
            cy + sin_phi * x1 + cos_phi * y1,
        ),
        pt2: (
            cx + cos_phi * x2 - sin_phi * y2,
            cy + sin_phi * x2 + cos_phi * y2,
        ),
        to: (
            cx + cos_phi * x3 - sin_phi * y3,
            cy + sin_phi * x3 + cos_phi * y3,
        ),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PathCommand {
    MoveTo(f64, f64),
    LineTo(f64, f64),
    CurveTo(CubicBezierCurve),
    Arc(EllipticalArc),
    ClosePath,
}

impl PathCommand {
    fn to_cairo(&self, cr: &cairo::Context) {
        match *self {
            PathCommand::MoveTo(x, y) => cr.move_to(x, y),
            PathCommand::LineTo(x, y) => cr.line_to(x, y),
            PathCommand::CurveTo(curve) => curve.to_cairo(cr),
            PathCommand::Arc(arc) => arc.to_cairo(cr),
            PathCommand::ClosePath => cr.close_path(),
        }
    }

    // Returns the number of coordinate values that this command will generate in a `Path`.
    fn num_coordinates(&self) -> usize {
        match *self {
            PathCommand::MoveTo(..) => 2,
            PathCommand::LineTo(..) => 2,
            PathCommand::CurveTo(_) => 6,
            PathCommand::Arc(_) => 7,
            PathCommand::ClosePath => 0,
        }
    }

    fn to_packed(&self, coords: &mut [f64]) -> PackedCommand {
        match *self {
            PathCommand::MoveTo(x, y) => {
                coords[0] = x;
                coords[1] = y;
                PackedCommand::MoveTo
            }

            PathCommand::LineTo(x, y) => {
                coords[0] = x;
                coords[1] = y;
                PackedCommand::LineTo
            }

            PathCommand::CurveTo(ref c) => c.to_packed_and_coords(coords),

            PathCommand::Arc(ref a) => a.to_packed_and_coords(coords),

            PathCommand::ClosePath => PackedCommand::ClosePath,
        }
    }

    fn from_packed(packed: &PackedCommand, coords: &[f64]) -> PathCommand {
        match *packed {
            PackedCommand::MoveTo => {
                let x = coords[0];
                let y = coords[1];
                PathCommand::MoveTo(x, y)
            }

            PackedCommand::LineTo => {
                let x = coords[0];
                let y = coords[1];
                PathCommand::LineTo(x, y)
            }

            PackedCommand::CurveTo => PathCommand::CurveTo(CubicBezierCurve::from_coords(coords)),

            PackedCommand::ClosePath => PathCommand::ClosePath,

            PackedCommand::ArcSmallNegative => PathCommand::Arc(EllipticalArc::from_coords(
                LargeArc(false),
                Sweep::Negative,
                coords,
            )),

            PackedCommand::ArcSmallPositive => PathCommand::Arc(EllipticalArc::from_coords(
                LargeArc(false),
                Sweep::Positive,
                coords,
            )),

            PackedCommand::ArcLargeNegative => PathCommand::Arc(EllipticalArc::from_coords(
                LargeArc(true),
                Sweep::Negative,
                coords,
            )),

            PackedCommand::ArcLargePositive => PathCommand::Arc(EllipticalArc::from_coords(
                LargeArc(true),
                Sweep::Positive,
                coords,
            )),
        }
    }
}

/// Constructs a path out of commands.
///
/// When you are finished constructing a path builder, turn it into
/// a `Path` with `into_path`.
#[derive(Clone)]
pub struct PathBuilder {
    path_commands: SmallVec<[PathCommand; 32]>,
}

/// An immutable path with a compact representation.
///
/// This is constructed from a `PathBuilder` once it is finished.  You
/// can get an iterator for the path's commands with the `iter`
/// function.
///
/// Most `PathCommand` variants only have a few coordinates, but `PathCommand::Arc`
/// has two extra booleans.  We separate the commands from their coordinates so
/// we can have two dense arrays: one with a compact representation of commands,
/// and another with a linear list of the coordinates for each command.
///
/// Each `PathCommand` knows how many coordinates it ought to produce, with
/// its `num_coordinates` method.
pub struct Path {
    commands: Box<[PackedCommand]>,
    coords: Box<[f64]>,
}

/// Iterator over a `Path`'s commands, from `Path::iter`.
pub struct PathIter<'a> {
    commands: slice::Iter<'a, PackedCommand>,
    coords: &'a [f64],
}

/// Packed version of a `PathCommand`, used in `Path`.
#[repr(u8)]
#[derive(Debug)]
enum PackedCommand {
    MoveTo,
    LineTo,
    CurveTo,
    ArcSmallNegative,
    ArcSmallPositive,
    ArcLargeNegative,
    ArcLargePositive,
    ClosePath,
}

impl PathBuilder {
    pub fn new() -> PathBuilder {
        PathBuilder {
            path_commands: SmallVec::new(),
        }
    }

    pub fn into_path(self) -> Path {
        let num_commands = self.path_commands.len();
        let num_coords = self
            .path_commands
            .iter()
            .map(PathCommand::num_coordinates)
            .sum();

        let mut packed_commands = Vec::with_capacity(num_commands);
        let mut coords = vec![0.0; num_coords];

        let mut coords_slice = coords.as_mut_slice();

        for c in self.path_commands {
            let n = c.num_coordinates();
            packed_commands.push(c.to_packed(&mut coords_slice[..n]));
            coords_slice = &mut coords_slice[n..];
        }

        Path {
            commands: packed_commands.into_boxed_slice(),
            coords: coords.into_boxed_slice(),
        }
    }

    pub fn move_to(&mut self, x: f64, y: f64) {
        self.path_commands.push(PathCommand::MoveTo(x, y));
    }

    pub fn line_to(&mut self, x: f64, y: f64) {
        self.path_commands.push(PathCommand::LineTo(x, y));
    }

    pub fn curve_to(&mut self, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64) {
        let curve = CubicBezierCurve {
            pt1: (x2, y2),
            pt2: (x3, y3),
            to: (x4, y4),
        };
        self.path_commands.push(PathCommand::CurveTo(curve));
    }

    pub fn arc(
        &mut self,
        x1: f64,
        y1: f64,
        rx: f64,
        ry: f64,
        x_axis_rotation: f64,
        large_arc: LargeArc,
        sweep: Sweep,
        x2: f64,
        y2: f64,
    ) {
        let arc = EllipticalArc {
            r: (rx, ry),
            x_axis_rotation,
            large_arc,
            sweep,
            from: (x1, y1),
            to: (x2, y2),
        };
        self.path_commands.push(PathCommand::Arc(arc));
    }

    pub fn close_path(&mut self) {
        self.path_commands.push(PathCommand::ClosePath);
    }
}

impl Path {
    pub fn iter(&self) -> PathIter {
        PathIter {
            commands: self.commands.iter(),
            coords: &self.coords[..],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn to_cairo(&self, cr: &cairo::Context) -> Result<(), cairo::Status> {
        assert!(!self.is_empty());

        for s in self.iter() {
            s.to_cairo(cr);
        }

        // We check the cr's status right after feeding it a new path for a few reasons:
        //
        // * Any of the individual path commands may cause the cr to enter an error state, for
        //   example, if they come with coordinates outside of Cairo's supported range.
        //
        // * The *next* call to the cr will probably be something that actually checks the status
        //   (i.e. in cairo-rs), and we don't want to panic there.

        let status = cr.status();

        if status == cairo::Status::Success {
            Ok(())
        } else {
            Err(status)
        }
    }
}

impl<'a> Iterator for PathIter<'a> {
    type Item = PathCommand;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cmd) = self.commands.next() {
            let cmd = PathCommand::from_packed(cmd, self.coords);
            let num_coords = cmd.num_coordinates();
            self.coords = &self.coords[num_coords..];
            Some(cmd)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_builder() {
        let builder = PathBuilder::new();
        let path = builder.into_path();
        assert!(path.is_empty());
        assert_eq!(path.iter().count(), 0);
    }

    #[test]
    fn all_commands() {
        let mut builder = PathBuilder::new();
        builder.move_to(42.0, 43.0);
        builder.line_to(42.0, 43.0);
        builder.curve_to(42.0, 43.0, 44.0, 45.0, 46.0, 47.0);
        builder.arc(
            42.0,
            43.0,
            44.0,
            45.0,
            46.0,
            LargeArc(true),
            Sweep::Positive,
            47.0,
            48.0,
        );
        builder.close_path();
        let path = builder.into_path();
        assert!(path.iter().eq(vec![
            PathCommand::MoveTo(42.0, 43.0),
            PathCommand::LineTo(42.0, 43.0),
            PathCommand::CurveTo(CubicBezierCurve {
                pt1: (42.0, 43.0),
                pt2: (44.0, 45.0),
                to: (46.0, 47.0),
            }),
            PathCommand::Arc(EllipticalArc {
                from: (42.0, 43.0),
                r: (44.0, 45.0),
                to: (47.0, 48.0),
                x_axis_rotation: 46.0,
                large_arc: LargeArc(true),
                sweep: Sweep::Positive,
            }),
            PathCommand::ClosePath,
        ]));
    }
}
