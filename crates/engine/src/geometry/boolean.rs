use opencascade::primitives::Shape;

pub fn union(a: &Shape, b: &Shape) -> Shape {
    let result = a.union(b);
    result.into()
}

pub fn subtract(a: &Shape, b: &Shape) -> Shape {
    let result = a.subtract(b);
    result.into()
}

/// Intersection is not directly exposed by opencascade-rs.
/// Stub implementation: returns the first shape unchanged.
pub fn intersect(a: &Shape, _b: &Shape) -> Shape {
    // TODO: Implement via opencascade-sys FFI (BRepAlgoAPI_Common)
    // For now, log a warning and return shape a unchanged.
    tracing::warn!("Boolean intersect not yet implemented, returning first shape unchanged");
    // We need to return a Shape but can't clone it.
    // Use the subtract-subtract identity: A ∩ B = A - (A - B)
    // This gives us the intersection using available operations.
    let a_minus_b = a.subtract(_b);
    let result = a.subtract(&a_minus_b);
    result.into()
}
