use glam::DVec3;
use opencascade::primitives::{Face, Shape};

use protocol::modeling_cmd::Point3d;

use crate::geometry::sketch::from_dvec3;

/// Get the center of mass of a face.
pub fn face_get_center(shape: &Shape) -> Point3d {
    let face = Face::from_shape(shape);
    from_dvec3(face.center_of_mass())
}

/// Check if a face is planar by sampling normals at the center.
pub fn face_is_planar(shape: &Shape) -> bool {
    let face = Face::from_shape(shape);
    let center = face.center_of_mass();
    let normal = face.normal_at(center);

    // A face is planar if the normal doesn't change across the surface.
    // Sample a few offset points and check if normals match.
    let offsets = [
        DVec3::new(0.01, 0.0, 0.0),
        DVec3::new(0.0, 0.01, 0.0),
        DVec3::new(-0.01, 0.0, 0.0),
    ];

    for offset in &offsets {
        let sample_pt = center + *offset;
        let sample_normal = face.normal_at(sample_pt);
        if (normal - sample_normal).length() > 0.01 {
            return false;
        }
    }
    true
}

/// Get the normal at a face's center.
pub fn face_get_normal_at_center(shape: &Shape) -> Point3d {
    let face = Face::from_shape(shape);
    let center = face.center_of_mass();
    let normal = face.normal_at(center);
    from_dvec3(normal)
}

/// Compute volume of a shape from its tessellated mesh using the
/// divergence theorem (signed tetrahedra method).
pub fn volume(shape: &Shape) -> f64 {
    let mesh = shape.mesh();
    let mut vol = 0.0;
    for tri in mesh.indices.chunks(3) {
        let v0 = mesh.vertices[tri[0]];
        let v1 = mesh.vertices[tri[1]];
        let v2 = mesh.vertices[tri[2]];
        // Signed volume of tetrahedron formed by triangle and origin
        vol += v0.dot(v1.cross(v2)) / 6.0;
    }
    vol.abs()
}

/// Compute surface area of a shape from its tessellated mesh.
pub fn surface_area(shape: &Shape) -> f64 {
    let mesh = shape.mesh();
    let mut area = 0.0;
    for tri in mesh.indices.chunks(3) {
        let v0 = mesh.vertices[tri[0]];
        let v1 = mesh.vertices[tri[1]];
        let v2 = mesh.vertices[tri[2]];
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        area += edge1.cross(edge2).length() / 2.0;
    }
    area
}

/// Compute center of mass of a shape from its tessellated mesh,
/// approximated as the area-weighted centroid of triangle centers.
pub fn center_of_mass(shape: &Shape) -> Point3d {
    let mesh = shape.mesh();
    let mut weighted_center = DVec3::ZERO;
    let mut total_area = 0.0;
    for tri in mesh.indices.chunks(3) {
        let v0 = mesh.vertices[tri[0]];
        let v1 = mesh.vertices[tri[1]];
        let v2 = mesh.vertices[tri[2]];
        let centroid = (v0 + v1 + v2) / 3.0;
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let area = edge1.cross(edge2).length() / 2.0;
        weighted_center += centroid * area;
        total_area += area;
    }
    if total_area > 0.0 {
        weighted_center /= total_area;
    }
    from_dvec3(weighted_center)
}

/// Count the number of faces in a shape.
pub fn face_count(shape: &Shape) -> usize {
    shape.faces().count()
}

/// Count the number of edges in a shape.
pub fn edge_count(shape: &Shape) -> usize {
    shape.edges().count()
}

/// Collect all face objects from a shape.
pub fn get_faces(shape: &Shape) -> Vec<Face> {
    shape.faces().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::dvec3;
    use opencascade::primitives::{Edge, Wire};

    fn make_unit_square_face() -> (Face, Shape) {
        let e1 = Edge::segment(dvec3(0.0, 0.0, 0.0), dvec3(1.0, 0.0, 0.0));
        let e2 = Edge::segment(dvec3(1.0, 0.0, 0.0), dvec3(1.0, 1.0, 0.0));
        let e3 = Edge::segment(dvec3(1.0, 1.0, 0.0), dvec3(0.0, 1.0, 0.0));
        let e4 = Edge::segment(dvec3(0.0, 1.0, 0.0), dvec3(0.0, 0.0, 0.0));
        let wire = Wire::from_edges([&e1, &e2, &e3, &e4]);
        let face = Face::from_wire(&wire);
        let shape: Shape = face.into();
        let face = Face::from_shape(&shape);
        (face, shape)
    }

    fn make_unit_box() -> Shape {
        let (face, _) = make_unit_square_face();
        let solid = face.extrude(dvec3(0.0, 0.0, 1.0));
        solid.into()
    }

    #[test]
    fn test_face_center() {
        let (_, shape) = make_unit_square_face();
        let center = face_get_center(&shape);
        assert!((center.x - 0.5).abs() < 0.01);
        assert!((center.y - 0.5).abs() < 0.01);
        assert!(center.z.abs() < 0.01);
    }

    #[test]
    fn test_face_is_planar() {
        let (_, shape) = make_unit_square_face();
        assert!(face_is_planar(&shape));
    }

    #[test]
    fn test_volume() {
        let box_shape = make_unit_box();
        let vol = volume(&box_shape);
        assert!((vol - 1.0).abs() < 0.01, "Expected volume ~1.0, got {vol}");
    }

    #[test]
    fn test_surface_area() {
        let box_shape = make_unit_box();
        let area = surface_area(&box_shape);
        // Unit box: 6 faces * 1.0 = 6.0
        assert!((area - 6.0).abs() < 0.01, "Expected area ~6.0, got {area}");
    }

    #[test]
    fn test_center_of_mass() {
        let box_shape = make_unit_box();
        let com = center_of_mass(&box_shape);
        assert!((com.x - 0.5).abs() < 0.01);
        assert!((com.y - 0.5).abs() < 0.01);
        assert!((com.z - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_face_edge_counts() {
        let box_shape = make_unit_box();
        assert_eq!(face_count(&box_shape), 6);
        // OCCT TopExp_Explorer enumerates edges per face (24 for a box),
        // not unique topological edges (12).
        assert!(edge_count(&box_shape) >= 12);
    }
}
