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

/// Fillet all edges of a shape with the given radius.
pub fn fillet_all(mut shape: Shape, radius: f64) -> Shape {
    shape.fillet(radius);
    shape
}

/// Fillet a specific edge of a shape.
pub fn fillet_edge(mut shape: Shape, radius: f64, edge: &opencascade::primitives::Edge) -> Shape {
    shape.fillet_edge(radius, edge);
    shape
}

/// Shell a solid by removing specified faces and offsetting inward.
pub fn shell(shape: Shape, faces_to_remove: Vec<opencascade::primitives::Face>, thickness: f64) -> Shape {
    shape.hollow(thickness, faces_to_remove)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::query;
    use glam::dvec3;
    use opencascade::primitives::Edge;

    fn make_unit_face() -> Face {
        let e1 = Edge::segment(dvec3(0.0, 0.0, 0.0), dvec3(1.0, 0.0, 0.0));
        let e2 = Edge::segment(dvec3(1.0, 0.0, 0.0), dvec3(1.0, 1.0, 0.0));
        let e3 = Edge::segment(dvec3(1.0, 1.0, 0.0), dvec3(0.0, 1.0, 0.0));
        let e4 = Edge::segment(dvec3(0.0, 1.0, 0.0), dvec3(0.0, 0.0, 0.0));
        let wire = Wire::from_edges([&e1, &e2, &e3, &e4]);
        Face::from_wire(&wire)
    }

    #[test]
    fn test_extrude() {
        let face = make_unit_face();
        let shape = extrude(&face, DVec3::Z, 2.0);
        let vol = query::volume(&shape);
        assert!((vol - 2.0).abs() < 0.1, "Expected volume ~2.0, got {vol}");
    }

    #[test]
    fn test_revolve() {
        let face = make_unit_face();
        // Revolve 360 degrees around X axis at y=2
        let shape = revolve(&face, dvec3(0.0, 2.0, 0.0), DVec3::X, 360.0);
        // Should produce a torus-like solid
        assert!(query::face_count(&shape) > 0);
    }

    #[test]
    fn test_fillet_all() {
        let face = make_unit_face();
        let box_shape = extrude(&face, DVec3::Z, 1.0);
        let filleted = fillet_all(box_shape, 0.1);
        // Filleting should add faces (rounded edges become new faces)
        assert!(query::face_count(&filleted) > 6);
    }

    #[test]
    fn test_shell() {
        let face = make_unit_face();
        let box_shape = extrude(&face, DVec3::Z, 1.0);
        let faces: Vec<opencascade::primitives::Face> = box_shape.faces().collect();
        let to_remove = vec![faces.into_iter().last().unwrap()];
        let shelled = shell(box_shape, to_remove, 0.1);
        // Shell removes a face and creates inner walls
        assert!(query::face_count(&shelled) > 6);
    }

    #[test]
    fn test_loft() {
        // Two square profiles at different Z levels
        let w1 = Wire::rect(2.0, 2.0);
        let mut w2 = Wire::rect(1.0, 1.0);
        w2.translate(dvec3(0.0, 0.0, 3.0));

        let shape = loft(vec![w1, w2]);
        assert!(query::face_count(&shape) > 0);
        let vol = query::volume(&shape);
        assert!(vol > 0.0, "Loft should produce positive volume, got {vol}");
    }
}
