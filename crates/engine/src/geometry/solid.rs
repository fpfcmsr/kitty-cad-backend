use glam::DVec3;
use opencascade::angle::Angle;
use opencascade::primitives::{Face, Shape, Solid, Wire};

/// Extrude a face along a direction by a given distance.
pub fn extrude(face: &Face, direction: DVec3, distance: f64) -> Shape {
    let dir = direction.normalize() * distance;
    let solid: Solid = face.extrude(dir);
    solid.into()
}

/// Revolve a face around an axis.
pub fn revolve(
    face: &Face,
    axis_origin: DVec3,
    axis_direction: DVec3,
    angle_degrees: f64,
) -> Shape {
    let angle = if (angle_degrees - 360.0).abs() < 1e-6 {
        None // Full revolution
    } else {
        Some(Angle::Degrees(angle_degrees))
    };
    let solid: Solid = face.revolve(axis_origin, axis_direction, angle);
    solid.into()
}

/// Loft through a series of wire profiles to create a solid.
pub fn loft(wires: Vec<Wire>) -> Shape {
    let solid: Solid = Solid::loft(wires);
    solid.into()
}
