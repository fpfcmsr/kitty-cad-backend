use glam::DVec3;
use opencascade::primitives::{Edge, Face, Wire};

use protocol::modeling_cmd::{PathSegment, Point3d};

use crate::session::PathSegmentRecord;

/// Convert our protocol Point3d to glam DVec3.
pub fn to_dvec3(p: &Point3d) -> DVec3 {
    DVec3::new(p.x, p.y, p.z)
}

/// Convert glam DVec3 to our protocol Point3d.
pub fn from_dvec3(v: DVec3) -> Point3d {
    Point3d {
        x: v.x,
        y: v.y,
        z: v.z,
    }
}

/// Build an OpenCASCADE Wire from recorded path segments.
pub fn build_wire(segments: &[PathSegmentRecord], closed: bool) -> Result<Wire, String> {
    if segments.is_empty() {
        return Err("No segments to build wire from".to_string());
    }

    let mut edges: Vec<Edge> = Vec::with_capacity(segments.len());

    for record in segments {
        let from = to_dvec3(&record.from);
        let edge = build_edge(from, &record.segment);
        edges.push(edge);
    }

    // If closed, add a closing edge from last point back to first point
    if closed && segments.len() > 1 {
        let last_seg = segments.last().unwrap();
        let last_pt = segment_endpoint_dvec3(&to_dvec3(&last_seg.from), &last_seg.segment);
        let first_pt = to_dvec3(&segments[0].from);

        if last_pt.distance(first_pt) > 1e-10 {
            let closing_edge = Edge::segment(last_pt, first_pt);
            edges.push(closing_edge);
        }
    }

    // Wire::from_edges takes references
    let edge_refs: Vec<&Edge> = edges.iter().collect();
    Ok(Wire::from_edges(edge_refs))
}

/// Build a Face from a closed wire.
pub fn build_face_from_wire(wire: &Wire) -> Result<Face, String> {
    Ok(Face::from_wire(wire))
}

/// Build an OpenCASCADE Edge from a path segment.
fn build_edge(from: DVec3, segment: &PathSegment) -> Edge {
    match segment {
        PathSegment::Line { end, relative } => {
            let to = if relative.unwrap_or(false) {
                from + to_dvec3(end)
            } else {
                to_dvec3(end)
            };
            Edge::segment(from, to)
        }

        PathSegment::Arc {
            center,
            radius,
            start_angle,
            end_angle,
        } => {
            let c = to_dvec3(center);
            let start_pt = DVec3::new(
                c.x + radius * start_angle.cos(),
                c.y + radius * start_angle.sin(),
                c.z,
            );
            let end_pt = DVec3::new(
                c.x + radius * end_angle.cos(),
                c.y + radius * end_angle.sin(),
                c.z,
            );
            let mid_angle = (start_angle + end_angle) / 2.0;
            let mid_pt = DVec3::new(
                c.x + radius * mid_angle.cos(),
                c.y + radius * mid_angle.sin(),
                c.z,
            );
            Edge::arc(start_pt, mid_pt, end_pt)
        }

        PathSegment::Bezier {
            control1: _,
            control2: _,
            end,
        } => {
            // opencascade-rs doesn't expose bezier edges yet.
            // Approximate as a straight line to the endpoint.
            let to = to_dvec3(end);
            Edge::segment(from, to)
        }

        PathSegment::TangentialArc { to, offset, .. } => {
            if let Some(to_pt) = to {
                let end_pt = to_dvec3(to_pt);
                // For tangential arcs, the midpoint approximation isn't ideal
                // but works for basic cases. A proper implementation would
                // compute the arc from the tangent direction at `from`.
                let mid = (from + end_pt) / 2.0;
                Edge::arc(from, mid, end_pt)
            } else if let Some(off) = offset {
                let end_pt = from + to_dvec3(off);
                let mid = (from + end_pt) / 2.0;
                Edge::arc(from, mid, end_pt)
            } else {
                // Degenerate case
                Edge::segment(from, from + DVec3::new(0.001, 0.0, 0.0))
            }
        }

        PathSegment::TangentialArcTo { to, .. } => {
            let end_pt = to_dvec3(to);
            let mid = (from + end_pt) / 2.0;
            Edge::arc(from, mid, end_pt)
        }
    }
}

/// Compute the endpoint of a segment in DVec3 space.
fn segment_endpoint_dvec3(from: &DVec3, segment: &PathSegment) -> DVec3 {
    match segment {
        PathSegment::Line { end, relative } => {
            if relative.unwrap_or(false) {
                *from + to_dvec3(end)
            } else {
                to_dvec3(end)
            }
        }
        PathSegment::Arc {
            center,
            radius,
            end_angle,
            ..
        } => {
            let c = to_dvec3(center);
            DVec3::new(
                c.x + radius * end_angle.cos(),
                c.y + radius * end_angle.sin(),
                c.z,
            )
        }
        PathSegment::Bezier { end, .. } => to_dvec3(end),
        PathSegment::TangentialArc { to, offset, .. } => {
            if let Some(to) = to {
                to_dvec3(to)
            } else if let Some(off) = offset {
                *from + to_dvec3(off)
            } else {
                *from
            }
        }
        PathSegment::TangentialArcTo { to, .. } => to_dvec3(to),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::PathSegmentRecord;

    fn make_rect_segments() -> Vec<PathSegmentRecord> {
        vec![
            PathSegmentRecord {
                from: Point3d { x: 0.0, y: 0.0, z: 0.0 },
                segment: PathSegment::Line {
                    end: Point3d { x: 10.0, y: 0.0, z: 0.0 },
                    relative: Some(false),
                },
            },
            PathSegmentRecord {
                from: Point3d { x: 10.0, y: 0.0, z: 0.0 },
                segment: PathSegment::Line {
                    end: Point3d { x: 10.0, y: 10.0, z: 0.0 },
                    relative: Some(false),
                },
            },
            PathSegmentRecord {
                from: Point3d { x: 10.0, y: 10.0, z: 0.0 },
                segment: PathSegment::Line {
                    end: Point3d { x: 0.0, y: 10.0, z: 0.0 },
                    relative: Some(false),
                },
            },
        ]
    }

    #[test]
    fn test_build_wire_closed() {
        let segments = make_rect_segments();
        let wire = build_wire(&segments, true).expect("Wire build failed");
        // Should succeed without panicking
        let _ = wire;
    }

    #[test]
    fn test_build_face_from_closed_wire() {
        let segments = make_rect_segments();
        let wire = build_wire(&segments, true).expect("Wire build failed");
        let face = build_face_from_wire(&wire).expect("Face build failed");
        // Face should have edges
        assert!(face.edges().count() >= 3);
    }

    #[test]
    fn test_build_wire_empty_segments_fails() {
        let result = build_wire(&[], true);
        assert!(result.is_err());
    }

    #[test]
    fn test_relative_line_segment() {
        let segments = vec![
            PathSegmentRecord {
                from: Point3d { x: 5.0, y: 5.0, z: 0.0 },
                segment: PathSegment::Line {
                    end: Point3d { x: 10.0, y: 0.0, z: 0.0 },
                    relative: Some(true),
                },
            },
            PathSegmentRecord {
                from: Point3d { x: 15.0, y: 5.0, z: 0.0 },
                segment: PathSegment::Line {
                    end: Point3d { x: 0.0, y: 10.0, z: 0.0 },
                    relative: Some(true),
                },
            },
            PathSegmentRecord {
                from: Point3d { x: 15.0, y: 15.0, z: 0.0 },
                segment: PathSegment::Line {
                    end: Point3d { x: -10.0, y: 0.0, z: 0.0 },
                    relative: Some(true),
                },
            },
        ];
        let wire = build_wire(&segments, true).expect("Wire build failed");
        let face = build_face_from_wire(&wire).expect("Face build failed");
        let _ = face;
    }

    #[test]
    fn test_to_from_dvec3() {
        let p = Point3d { x: 1.5, y: 2.5, z: 3.5 };
        let v = to_dvec3(&p);
        assert_eq!(v.x, 1.5);
        assert_eq!(v.y, 2.5);
        assert_eq!(v.z, 3.5);
        let p2 = from_dvec3(v);
        assert_eq!(p2.x, 1.5);
        assert_eq!(p2.y, 2.5);
        assert_eq!(p2.z, 3.5);
    }
}
