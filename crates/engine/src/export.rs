use opencascade::primitives::Shape;
use tempfile::TempDir;

/// Export a shape to STEP format, returning the file contents as bytes.
pub fn export_step(shape: &Shape) -> Result<Vec<u8>, String> {
    let tmp_dir = TempDir::new().map_err(|e| format!("Failed to create temp dir: {e}"))?;
    let path = tmp_dir.path().join("export.step");

    shape
        .write_step(&path)
        .map_err(|e| format!("STEP export failed: {e:?}"))?;

    std::fs::read(&path).map_err(|e| format!("Failed to read exported STEP file: {e}"))
}

/// Export a shape to STL format, returning the file contents as bytes.
pub fn export_stl(shape: &Shape) -> Result<Vec<u8>, String> {
    let tmp_dir = TempDir::new().map_err(|e| format!("Failed to create temp dir: {e}"))?;
    let path = tmp_dir.path().join("export.stl");

    shape
        .write_stl(&path)
        .map_err(|e| format!("STL export failed: {e:?}"))?;

    std::fs::read(&path).map_err(|e| format!("Failed to read exported STL file: {e}"))
}

/// Import a STEP file and return the resulting shape.
pub fn import_step(data: &[u8]) -> Result<Shape, String> {
    let tmp_dir = TempDir::new().map_err(|e| format!("Failed to create temp dir: {e}"))?;
    let path = tmp_dir.path().join("import.step");

    std::fs::write(&path, data).map_err(|e| format!("Failed to write STEP data: {e}"))?;

    Shape::read_step(&path).map_err(|e| format!("STEP import failed: {e:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::dvec3;
    use opencascade::primitives::{Edge, Face, Wire};

    fn make_test_box() -> Shape {
        let e1 = Edge::segment(dvec3(0.0, 0.0, 0.0), dvec3(10.0, 0.0, 0.0));
        let e2 = Edge::segment(dvec3(10.0, 0.0, 0.0), dvec3(10.0, 10.0, 0.0));
        let e3 = Edge::segment(dvec3(10.0, 10.0, 0.0), dvec3(0.0, 10.0, 0.0));
        let e4 = Edge::segment(dvec3(0.0, 10.0, 0.0), dvec3(0.0, 0.0, 0.0));
        let wire = Wire::from_edges([&e1, &e2, &e3, &e4]);
        let face = Face::from_wire(&wire);
        let solid = face.extrude(dvec3(0.0, 0.0, 5.0));
        solid.into()
    }

    #[test]
    fn test_step_roundtrip() {
        let shape = make_test_box();
        let step_data = export_step(&shape).expect("STEP export failed");
        assert!(!step_data.is_empty());
        assert!(
            String::from_utf8_lossy(&step_data).contains("ISO-10303-21"),
            "STEP file should contain ISO header"
        );

        let imported = import_step(&step_data).expect("STEP import failed");
        // Verify the imported shape has faces
        assert!(imported.faces().count() > 0);
    }

    #[test]
    fn test_stl_export() {
        let shape = make_test_box();
        let stl_data = export_stl(&shape).expect("STL export failed");
        assert!(!stl_data.is_empty());
    }
}
