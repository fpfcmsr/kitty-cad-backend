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
