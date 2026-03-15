use uuid::Uuid;

use protocol::modeling_cmd::{ModelingCmd, Point3d};
use protocol::responses::*;

use crate::session::{Entity, EntityType, PathBuilder, Session, SketchMode};

/// Dispatches a modeling command against the session and returns the response.
pub fn dispatch(session: &mut Session, cmd: ModelingCmd) -> Result<OkModelingCmdResponse, String> {
    match cmd {
        // -- Sketch operations --
        ModelingCmd::StartPath {} => {
            let path_id = Uuid::new_v4();
            session.paths.insert(
                path_id,
                PathBuilder {
                    id: path_id,
                    pen_position: None,
                    segments: vec![],
                    closed: false,
                },
            );
            session.entities.insert(
                path_id,
                Entity {
                    id: path_id,
                    entity_type: EntityType::Path,
                    parent_id: None,
                    children: vec![],
                    visible: true,
                },
            );
            Ok(OkModelingCmdResponse::StartPath {
                data: StartPathData { path_id },
            })
        }

        ModelingCmd::MovePathPen { path, to } => {
            let builder = session
                .paths
                .get_mut(&path)
                .ok_or_else(|| format!("Path {path} not found"))?;
            builder.pen_position = Some(to);
            Ok(OkModelingCmdResponse::Empty {})
        }

        ModelingCmd::ExtendPath { path, segment } => {
            let builder = session
                .paths
                .get_mut(&path)
                .ok_or_else(|| format!("Path {path} not found"))?;
            let from = builder
                .pen_position
                .clone()
                .ok_or("Pen position not set; call MovePathPen first")?;

            // Update pen position based on segment endpoint
            let new_pos = segment_endpoint(&from, &segment);
            builder.segments.push(crate::session::PathSegmentRecord {
                from,
                segment,
            });
            builder.pen_position = Some(new_pos);
            Ok(OkModelingCmdResponse::Empty {})
        }

        ModelingCmd::ClosePath { path_id } => {
            let builder = session
                .paths
                .get_mut(&path_id)
                .ok_or_else(|| format!("Path {path_id} not found"))?;
            builder.closed = true;

            // Create a face entity from the closed path
            let face_id = Uuid::new_v4();
            session.entities.insert(
                face_id,
                Entity {
                    id: face_id,
                    entity_type: EntityType::Face,
                    parent_id: Some(path_id),
                    children: vec![],
                    visible: true,
                },
            );
            // Register face as child of path
            if let Some(path_entity) = session.entities.get_mut(&path_id) {
                path_entity.children.push(face_id);
            }

            Ok(OkModelingCmdResponse::ClosePath {
                data: ClosePathData { face_id },
            })
        }

        // -- Sketch mode --
        ModelingCmd::EnableSketchMode {
            entity_id,
            planar_normal,
            ..
        } => {
            let normal = planar_normal.unwrap_or(Point3d {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            });
            session.sketch_mode = Some(SketchMode {
                entity_id,
                plane_origin: Point3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                plane_normal: normal,
                plane_x_axis: Point3d {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
                plane_y_axis: Point3d {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
            });
            Ok(OkModelingCmdResponse::Empty {})
        }

        ModelingCmd::SketchModeDisable {} => {
            session.sketch_mode = None;
            Ok(OkModelingCmdResponse::Empty {})
        }

        // -- 3D Solid operations (stub until OpenCASCADE integration) --
        ModelingCmd::Extrude {
            target, distance, ..
        } => {
            let solid_id = Uuid::new_v4();
            let cap_face = Uuid::new_v4();
            let side_face = Uuid::new_v4();
            let edge_id = Uuid::new_v4();

            session.entities.insert(
                solid_id,
                Entity {
                    id: solid_id,
                    entity_type: EntityType::Solid,
                    parent_id: Some(target),
                    children: vec![cap_face, side_face, edge_id],
                    visible: true,
                },
            );
            for &face_or_edge in &[cap_face, side_face, edge_id] {
                let etype = if face_or_edge == edge_id {
                    EntityType::Edge
                } else {
                    EntityType::Face
                };
                session.entities.insert(
                    face_or_edge,
                    Entity {
                        id: face_or_edge,
                        entity_type: etype,
                        parent_id: Some(solid_id),
                        children: vec![],
                        visible: true,
                    },
                );
            }

            tracing::info!(%solid_id, %target, distance, "Extrude (stub)");

            Ok(OkModelingCmdResponse::Extrude {
                data: ExtrudeData {
                    solid_id,
                    face_ids: vec![cap_face, side_face],
                    edge_ids: vec![edge_id],
                },
            })
        }

        ModelingCmd::Revolve {
            target, angle, ..
        } => {
            let solid_id = Uuid::new_v4();
            tracing::info!(%solid_id, %target, angle, "Revolve (stub)");
            session.entities.insert(
                solid_id,
                Entity {
                    id: solid_id,
                    entity_type: EntityType::Solid,
                    parent_id: Some(target),
                    children: vec![],
                    visible: true,
                },
            );
            Ok(OkModelingCmdResponse::Revolve {
                data: RevolveData {
                    solid_id,
                    face_ids: vec![],
                    edge_ids: vec![],
                },
            })
        }

        ModelingCmd::Solid3dFilletEdge { object_id, .. } => {
            tracing::info!(%object_id, "Solid3dFilletEdge (stub)");
            Ok(OkModelingCmdResponse::Empty {})
        }

        // -- Booleans (stub) --
        ModelingCmd::BooleanUnion { .. }
        | ModelingCmd::BooleanSubtract { .. }
        | ModelingCmd::BooleanIntersect { .. } => {
            tracing::info!("Boolean operation (stub)");
            Ok(OkModelingCmdResponse::Empty {})
        }

        // -- Camera --
        ModelingCmd::DefaultCameraLookAt {
            center,
            eye,
            up,
            vantage,
            ..
        } => {
            if let Some(c) = center {
                session.camera.center = c;
            }
            if let Some(e) = eye.or(vantage) {
                session.camera.position = e;
            }
            if let Some(u) = up {
                session.camera.up = u;
            }
            Ok(OkModelingCmdResponse::Empty {})
        }

        ModelingCmd::DefaultCameraGetSettings {} => {
            let cam = &session.camera;
            Ok(OkModelingCmdResponse::DefaultCameraGetSettings {
                data: CameraSettings {
                    pos: cam.position.clone(),
                    center: cam.center.clone(),
                    up: cam.up.clone(),
                    fov_y: Some(cam.fov_y),
                    ortho: cam.ortho,
                    ortho_scale: Some(cam.ortho_scale),
                },
            })
        }

        ModelingCmd::DefaultCameraZoom { .. }
        | ModelingCmd::ZoomToFit { .. }
        | ModelingCmd::ViewIsometric { .. } => Ok(OkModelingCmdResponse::Empty {}),

        // -- Selection --
        ModelingCmd::SelectAdd { entities } => {
            for id in &entities {
                session.selection.insert(*id);
            }
            Ok(OkModelingCmdResponse::SelectAdd {
                data: SelectAddData {},
            })
        }
        ModelingCmd::SelectRemove { entities } => {
            for id in &entities {
                session.selection.remove(id);
            }
            Ok(OkModelingCmdResponse::Empty {})
        }
        ModelingCmd::SelectClear {} => {
            session.selection.clear();
            Ok(OkModelingCmdResponse::Empty {})
        }
        ModelingCmd::HighlightSetEntity { sequence, .. } => {
            Ok(OkModelingCmdResponse::HighlightSetEntity {
                data: HighlightSetEntityData {
                    entity_id: None,
                    sequence,
                },
            })
        }

        // -- Scene --
        ModelingCmd::SceneClearAll {} => {
            session.clear();
            Ok(OkModelingCmdResponse::Empty {})
        }

        ModelingCmd::SetSceneUnits { unit } => {
            session.units = unit;
            Ok(OkModelingCmdResponse::Empty {})
        }

        ModelingCmd::SetBackgroundColor { .. } | ModelingCmd::EdgeLinesVisible { .. } => {
            Ok(OkModelingCmdResponse::Empty {})
        }

        // -- Planes --
        ModelingCmd::MakePlane {
            origin,
            x_axis,
            y_axis,
            size: _,
            ..
        } => {
            let plane_id = Uuid::new_v4();
            // Compute z_axis as cross product of x_axis and y_axis
            let z_axis = Point3d {
                x: x_axis.y * y_axis.z - x_axis.z * y_axis.y,
                y: x_axis.z * y_axis.x - x_axis.x * y_axis.z,
                z: x_axis.x * y_axis.y - x_axis.y * y_axis.x,
            };
            session.entities.insert(
                plane_id,
                Entity {
                    id: plane_id,
                    entity_type: EntityType::Plane,
                    parent_id: None,
                    children: vec![],
                    visible: true,
                },
            );
            Ok(OkModelingCmdResponse::MakePlane {
                data: MakePlaneData {
                    plane_id,
                    origin,
                    x_axis,
                    y_axis,
                    z_axis,
                },
            })
        }

        // -- Entity queries --
        ModelingCmd::EntityGetParentId { entity_id } => {
            let entity = session
                .entities
                .get(&entity_id)
                .ok_or_else(|| format!("Entity {entity_id} not found"))?;
            Ok(OkModelingCmdResponse::EntityGetParentId {
                data: EntityGetParentIdData {
                    entity_id: entity.parent_id.unwrap_or(entity_id),
                },
            })
        }

        ModelingCmd::EntityGetNumChildren { entity_id } => {
            let entity = session
                .entities
                .get(&entity_id)
                .ok_or_else(|| format!("Entity {entity_id} not found"))?;
            Ok(OkModelingCmdResponse::EntityGetNumChildren {
                data: EntityGetNumChildrenData {
                    num: entity.children.len() as u32,
                },
            })
        }

        ModelingCmd::EntityGetChildUuid {
            entity_id,
            child_index,
        } => {
            let entity = session
                .entities
                .get(&entity_id)
                .ok_or_else(|| format!("Entity {entity_id} not found"))?;
            let child_id = entity
                .children
                .get(child_index as usize)
                .ok_or_else(|| format!("Child index {child_index} out of range"))?;
            Ok(OkModelingCmdResponse::EntityGetChildUuid {
                data: EntityGetChildUuidData {
                    entity_id: *child_id,
                },
            })
        }

        ModelingCmd::EntityGetAllChildUuids { entity_id } => {
            let entity = session
                .entities
                .get(&entity_id)
                .ok_or_else(|| format!("Entity {entity_id} not found"))?;
            Ok(OkModelingCmdResponse::EntityGetAllChildUuids {
                data: EntityGetAllChildUuidsData {
                    entity_ids: entity.children.clone(),
                },
            })
        }

        ModelingCmd::EntityGetSketchPaths { entity_id } => {
            // Return all path children
            let entity = session
                .entities
                .get(&entity_id)
                .ok_or_else(|| format!("Entity {entity_id} not found"))?;
            let paths: Vec<Uuid> = entity
                .children
                .iter()
                .filter(|id| {
                    session
                        .entities
                        .get(id)
                        .is_some_and(|e| e.entity_type == EntityType::Path)
                })
                .copied()
                .collect();
            Ok(OkModelingCmdResponse::EntityGetSketchPaths {
                data: EntityGetSketchPathsData {
                    entity_ids: paths,
                },
            })
        }

        ModelingCmd::GetEntityType { entity_id } => {
            let entity = session
                .entities
                .get(&entity_id)
                .ok_or_else(|| format!("Entity {entity_id} not found"))?;
            Ok(OkModelingCmdResponse::GetEntityType {
                data: GetEntityTypeData {
                    entity_type: entity.entity_type.to_string(),
                },
            })
        }

        ModelingCmd::ObjectVisible {
            object_id, hidden, ..
        } => {
            if let Some(entity) = session.entities.get_mut(&object_id) {
                entity.visible = !hidden;
            }
            Ok(OkModelingCmdResponse::Empty {})
        }

        ModelingCmd::ObjectBringToFront { .. } => Ok(OkModelingCmdResponse::Empty {}),

        ModelingCmd::GetSketchModePlane {} => {
            if let Some(ref sm) = session.sketch_mode {
                Ok(OkModelingCmdResponse::GetSketchModePlane {
                    data: GetSketchModePlaneData {
                        origin: sm.plane_origin.clone(),
                        x_axis: sm.plane_x_axis.clone(),
                        y_axis: sm.plane_y_axis.clone(),
                        z_axis: sm.plane_normal.clone(),
                    },
                })
            } else {
                Err("Not in sketch mode".to_string())
            }
        }

        // -- Stub responses for queries that need OpenCASCADE --
        ModelingCmd::EntityGetDistance { .. } => Ok(OkModelingCmdResponse::EntityGetDistance {
            data: EntityGetDistanceData {
                min_distance: 0.0,
                max_distance: 0.0,
            },
        }),

        ModelingCmd::Solid3dGetAllEdgeFaces { .. } => {
            Ok(OkModelingCmdResponse::Solid3dGetAllEdgeFaces {
                data: Solid3dGetAllEdgeFacesData { faces: vec![] },
            })
        }

        ModelingCmd::Solid3dGetAllOppositeEdges { .. } => {
            Ok(OkModelingCmdResponse::Solid3dGetAllOppositeEdges {
                data: Solid3dGetAllOppositeEdgesData { edges: vec![] },
            })
        }

        ModelingCmd::Solid3dGetOppositeEdge {
            edge_id, ..
        } => Ok(OkModelingCmdResponse::Solid3dGetOppositeEdge {
            data: Solid3dGetOppositeEdgeData { edge: edge_id },
        }),

        ModelingCmd::Solid3dGetNextAdjacentEdge { .. } => {
            Ok(OkModelingCmdResponse::Solid3dGetNextAdjacentEdge {
                data: Solid3dGetAdjacentEdgeData { edge: None },
            })
        }

        ModelingCmd::Solid3dGetPrevAdjacentEdge { .. } => {
            Ok(OkModelingCmdResponse::Solid3dGetPrevAdjacentEdge {
                data: Solid3dGetAdjacentEdgeData { edge: None },
            })
        }

        ModelingCmd::FaceIsPlanar { .. } => Ok(OkModelingCmdResponse::FaceIsPlanar {
            data: FaceIsPlanarData { is_planar: true },
        }),

        ModelingCmd::FaceGetCenter { .. } => Ok(OkModelingCmdResponse::FaceGetCenter {
            data: FaceGetCenterData {
                pos: Point3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
        }),

        ModelingCmd::FaceGetGradient { .. } => Ok(OkModelingCmdResponse::FaceGetGradient {
            data: FaceGetGradientData {
                df_du: Point3d {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
                df_dv: Point3d {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
                normal: Point3d {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                },
            },
        }),

        ModelingCmd::FaceGetPosition { .. } => Ok(OkModelingCmdResponse::FaceGetPosition {
            data: FaceGetPositionData {
                pos: Point3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
        }),

        ModelingCmd::CurveGetControlPoints { .. } => {
            Ok(OkModelingCmdResponse::CurveGetControlPoints {
                data: CurveGetControlPointsData {
                    control_points: vec![],
                },
            })
        }

        ModelingCmd::CurveGetEndPoints { .. } => Ok(OkModelingCmdResponse::CurveGetEndPoints {
            data: CurveGetEndPointsData {
                start: Point3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                end: Point3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
        }),

        ModelingCmd::CurveGetType { .. } => Ok(OkModelingCmdResponse::CurveGetType {
            data: CurveGetTypeData {
                curve_type: "line".to_string(),
            },
        }),

        // -- Measurements (stub) --
        ModelingCmd::Mass { .. } => Ok(OkModelingCmdResponse::Mass {
            data: MassData {
                mass: 0.0,
                output_unit: "kg".to_string(),
            },
        }),

        ModelingCmd::Volume { .. } => Ok(OkModelingCmdResponse::Volume {
            data: VolumeData {
                volume: 0.0,
                output_unit: "m3".to_string(),
            },
        }),

        ModelingCmd::SurfaceArea { .. } => Ok(OkModelingCmdResponse::SurfaceArea {
            data: SurfaceAreaData {
                surface_area: 0.0,
                output_unit: "m2".to_string(),
            },
        }),

        ModelingCmd::CenterOfMass { .. } => Ok(OkModelingCmdResponse::CenterOfMass {
            data: CenterOfMassData {
                center_of_mass: Point3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                output_unit: "mm".to_string(),
            },
        }),

        ModelingCmd::Density { .. } => Ok(OkModelingCmdResponse::Density {
            data: DensityData {
                density: 0.0,
                output_unit: "kg_per_m3".to_string(),
            },
        }),

        // -- Export/Import (stub) --
        ModelingCmd::Export { .. } => Ok(OkModelingCmdResponse::Export {
            data: ExportData { files: vec![] },
        }),

        ModelingCmd::ImportFiles { .. } => {
            let object_id = Uuid::new_v4();
            Ok(OkModelingCmdResponse::ImportFiles {
                data: ImportFilesData { object_id },
            })
        }

        // -- Mouse/interaction (no-op) --
        ModelingCmd::MouseMove { .. }
        | ModelingCmd::MouseClick { .. }
        | ModelingCmd::HandleMouseDragStart { .. }
        | ModelingCmd::HandleMouseDragMove { .. }
        | ModelingCmd::HandleMouseDragEnd { .. } => Ok(OkModelingCmdResponse::Empty {}),

        // -- Catch-all --
        ModelingCmd::Unknown => {
            tracing::warn!("Received unknown/unimplemented modeling command");
            Ok(OkModelingCmdResponse::Empty {})
        }
    }
}

/// Compute the endpoint of a path segment.
fn segment_endpoint(
    from: &Point3d,
    segment: &protocol::modeling_cmd::PathSegment,
) -> Point3d {
    use protocol::modeling_cmd::PathSegment;
    match segment {
        PathSegment::Line { end, relative } => {
            if relative.unwrap_or(false) {
                Point3d {
                    x: from.x + end.x,
                    y: from.y + end.y,
                    z: from.z + end.z,
                }
            } else {
                end.clone()
            }
        }
        PathSegment::Arc { center, radius, end_angle, .. } => {
            Point3d {
                x: center.x + radius * end_angle.cos(),
                y: center.y + radius * end_angle.sin(),
                z: center.z,
            }
        }
        PathSegment::Bezier { end, .. } => end.clone(),
        PathSegment::TangentialArc { to, offset, .. } => {
            if let Some(to) = to {
                to.clone()
            } else if let Some(offset) = offset {
                Point3d {
                    x: from.x + offset.x,
                    y: from.y + offset.y,
                    z: from.z + offset.z,
                }
            } else {
                from.clone()
            }
        }
        PathSegment::TangentialArcTo { to, .. } => to.clone(),
    }
}
