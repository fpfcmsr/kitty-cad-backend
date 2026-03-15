use opencascade::mesh::Mesher;
use opencascade::primitives::Shape;

use protocol::responses::MeshData;

/// Tessellate an OpenCASCADE Shape into mesh data (vertices, normals, indices).
pub fn tessellate(shape: &Shape) -> MeshData {
    let mesher = Mesher::new(shape);
    let mesh = mesher.mesh();

    let mut vertices = Vec::with_capacity(mesh.vertices.len() * 3);
    let mut normals = Vec::with_capacity(mesh.normals.len() * 3);

    for v in &mesh.vertices {
        vertices.push(v.x as f32);
        vertices.push(v.y as f32);
        vertices.push(v.z as f32);
    }

    for n in &mesh.normals {
        normals.push(n.x as f32);
        normals.push(n.y as f32);
        normals.push(n.z as f32);
    }

    // Convert usize indices to u32
    let indices: Vec<u32> = mesh.indices.iter().map(|&i| i as u32).collect();

    MeshData {
        vertices,
        normals,
        indices,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::dvec3;
    use opencascade::primitives::{Edge, Face, Wire};

    #[test]
    fn test_tessellate_face() {
        let e1 = Edge::segment(dvec3(0.0, 0.0, 0.0), dvec3(1.0, 0.0, 0.0));
        let e2 = Edge::segment(dvec3(1.0, 0.0, 0.0), dvec3(1.0, 1.0, 0.0));
        let e3 = Edge::segment(dvec3(1.0, 1.0, 0.0), dvec3(0.0, 1.0, 0.0));
        let e4 = Edge::segment(dvec3(0.0, 1.0, 0.0), dvec3(0.0, 0.0, 0.0));
        let wire = Wire::from_edges([&e1, &e2, &e3, &e4]);
        let face = Face::from_wire(&wire);
        let shape: Shape = face.into();

        let mesh = tessellate(&shape);
        assert!(!mesh.vertices.is_empty(), "Mesh should have vertices");
        assert!(!mesh.indices.is_empty(), "Mesh should have indices");
        // Vertices should come in triples (x, y, z)
        assert_eq!(mesh.vertices.len() % 3, 0);
        assert_eq!(mesh.normals.len() % 3, 0);
        // Indices should come in triples (triangles)
        assert_eq!(mesh.indices.len() % 3, 0);
    }

    #[test]
    fn test_tessellate_box() {
        let e1 = Edge::segment(dvec3(0.0, 0.0, 0.0), dvec3(1.0, 0.0, 0.0));
        let e2 = Edge::segment(dvec3(1.0, 0.0, 0.0), dvec3(1.0, 1.0, 0.0));
        let e3 = Edge::segment(dvec3(1.0, 1.0, 0.0), dvec3(0.0, 1.0, 0.0));
        let e4 = Edge::segment(dvec3(0.0, 1.0, 0.0), dvec3(0.0, 0.0, 0.0));
        let wire = Wire::from_edges([&e1, &e2, &e3, &e4]);
        let face = Face::from_wire(&wire);
        let solid = face.extrude(dvec3(0.0, 0.0, 1.0));
        let shape: Shape = solid.into();

        let mesh = tessellate(&shape);
        let vertex_count = mesh.vertices.len() / 3;
        let triangle_count = mesh.indices.len() / 3;

        assert!(vertex_count >= 8, "Box should have at least 8 vertices, got {vertex_count}");
        assert!(triangle_count >= 12, "Box should have at least 12 triangles, got {triangle_count}");
    }
}
