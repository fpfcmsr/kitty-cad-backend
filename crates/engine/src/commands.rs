use glam::DVec3;
use uuid::Uuid;

use protocol::modeling_cmd::{ModelingCmd, Point3d};
use protocol::responses::*;

use crate::geometry::{boolean, query, sketch, solid};
use crate::{export, tessellation};
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
                    shape: None,
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
            let new_pos = compute_segment_endpoint(&from, &segment);
            builder
                .segments
                .push(crate::session::PathSegmentRecord { from, segment });
            builder.pen_position = Some(new_pos);
            Ok(OkModelingCmdResponse::Empty {})
        }

        ModelingCmd::ClosePath { path_id } => {
            let builder = session
                .paths
                .get_mut(&path_id)
                .ok_or_else(|| format!("Path {path_id} not found"))?;
            builder.closed = true;

            // Build actual geometry: wire -> face
            let segments = builder.segments.clone();
            let wire = sketch::build_wire(&segments, true)?;
            let face = sketch::build_face_from_wire(&wire)?;

            let face_id = Uuid::new_v4();

            // Convert face to shape for tessellation and storage
            let face_shape: opencascade::primitives::Shape = face.into();
            let mesh = tessellation::tessellate(&face_shape);

            tracing::info!(
                %face_id, %path_id,
                vertex_count = mesh.vertices.len() / 3,
                "ClosePath: built face from wire"
            );

            session.entities.insert(
                face_id,
                Entity {
                    id: face_id,
                    entity_type: EntityType::Face,
                    parent_id: Some(path_id),
                    children: vec![],
                    visible: true,
                    shape: Some(face_shape),
                },
            );
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

        // -- 3D Solid operations with real OpenCASCADE geometry --
        ModelingCmd::Extrude {
            target, distance, ..
        } => {
            let target_shape = session
                .get_shape(&target)
                .ok_or_else(|| format!("Shape entity {target} not found for extrusion"))?;

            // Reconstruct Face from the stored shape
            let face = opencascade::primitives::Face::from_shape(target_shape);

            // Default extrusion along Z axis
            let direction = if let Some(ref sm) = session.sketch_mode {
                sketch::to_dvec3(&sm.plane_normal)
            } else {
                DVec3::Z
            };

            let extruded_shape = solid::extrude(&face, direction, distance);

            let solid_id = Uuid::new_v4();
            let cap_face_id = Uuid::new_v4();
            let edge_id = Uuid::new_v4();

            let mesh = tessellation::tessellate(&extruded_shape);

            tracing::info!(
                %solid_id, %target, distance,
                vertex_count = mesh.vertices.len() / 3,
                "Extrude"
            );

            session.entities.insert(
                solid_id,
                Entity {
                    id: solid_id,
                    entity_type: EntityType::Solid,
                    parent_id: Some(target),
                    children: vec![cap_face_id, edge_id],
                    visible: true,
                    shape: Some(extruded_shape),
                },
            );
            session.entities.insert(
                cap_face_id,
                Entity {
                    id: cap_face_id,
                    entity_type: EntityType::Face,
                    parent_id: Some(solid_id),
                    children: vec![],
                    visible: true,
                    shape: None,
                },
            );
            session.entities.insert(
                edge_id,
                Entity {
                    id: edge_id,
                    entity_type: EntityType::Edge,
                    parent_id: Some(solid_id),
                    children: vec![],
                    visible: true,
                    shape: None,
                },
            );

            Ok(OkModelingCmdResponse::Extrude {
                data: ExtrudeData {
                    solid_id,
                    face_ids: vec![cap_face_id],
                    edge_ids: vec![edge_id],
                },
            })
        }

        ModelingCmd::Revolve {
            target,
            axis,
            angle,
            ..
        } => {
            let target_shape = session
                .get_shape(&target)
                .ok_or_else(|| format!("Shape entity {target} not found for revolve"))?;

            let face = opencascade::primitives::Face::from_shape(target_shape);

            let axis_dir = sketch::to_dvec3(&axis);
            let origin = DVec3::ZERO;
            let angle_deg = angle.to_degrees();

            let revolved_shape = solid::revolve(&face, origin, axis_dir, angle_deg);

            let solid_id = Uuid::new_v4();

            tracing::info!(%solid_id, %target, angle, "Revolve");

            session.entities.insert(
                solid_id,
                Entity {
                    id: solid_id,
                    entity_type: EntityType::Solid,
                    parent_id: Some(target),
                    children: vec![],
                    visible: true,
                    shape: Some(revolved_shape),
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

        ModelingCmd::Solid3dFilletEdge {
            object_id, radius, ..
        } => {
            // Fillet requires specific edge references which we don't track yet.
            // For now, log and return success without modifying the shape.
            tracing::info!(%object_id, radius, "Solid3dFilletEdge (edge tracking not yet implemented)");
            Ok(OkModelingCmdResponse::Empty {})
        }

        // -- Booleans --
        ModelingCmd::BooleanUnion { targets } => {
            if targets.len() < 2 {
                return Err("Boolean union requires at least 2 targets".to_string());
            }
            // Collect shape references, then perform unions
            let shapes: Vec<_> = targets
                .iter()
                .map(|id| {
                    session
                        .get_shape(id)
                        .ok_or_else(|| format!("Shape {id} not found"))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let mut result = boolean::union(shapes[0], shapes[1]);
            for shape in &shapes[2..] {
                result = boolean::union(&result, shape);
            }

            if let Some(entity) = session.entities.get_mut(&targets[0]) {
                entity.shape = Some(result);
            }
            tracing::info!(?targets, "Boolean union");
            Ok(OkModelingCmdResponse::Empty {})
        }

        ModelingCmd::BooleanSubtract { target, tool } => {
            let target_shape = session
                .get_shape(&target)
                .ok_or_else(|| format!("Shape {target} not found"))?;
            let tool_shape = session
                .get_shape(&tool)
                .ok_or_else(|| format!("Shape {tool} not found"))?;

            let result = boolean::subtract(target_shape, tool_shape);

            if let Some(entity) = session.entities.get_mut(&target) {
                entity.shape = Some(result);
            }
            tracing::info!(%target, %tool, "Boolean subtract");
            Ok(OkModelingCmdResponse::Empty {})
        }

        ModelingCmd::BooleanIntersect { targets } => {
            if targets.len() < 2 {
                return Err("Boolean intersect requires at least 2 targets".to_string());
            }
            let shapes: Vec<_> = targets
                .iter()
                .map(|id| {
                    session
                        .get_shape(id)
                        .ok_or_else(|| format!("Shape {id} not found"))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let mut result = boolean::intersect(shapes[0], shapes[1]);
            for shape in &shapes[2..] {
                result = boolean::intersect(&result, shape);
            }

            if let Some(entity) = session.entities.get_mut(&targets[0]) {
                entity.shape = Some(result);
            }
            tracing::info!(?targets, "Boolean intersect");
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
                    shape: None,
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

        // -- Face queries (real OCCT implementations) --
        ModelingCmd::FaceIsPlanar {
            object_id, face_id,
        } => {
            // Try to get face shape; fall back to parent object
            let shape = session
                .get_shape(&face_id)
                .or_else(|| session.get_shape(&object_id))
                .ok_or_else(|| format!("Shape not found for face query"))?;
            Ok(OkModelingCmdResponse::FaceIsPlanar {
                data: FaceIsPlanarData {
                    is_planar: query::face_is_planar(shape),
                },
            })
        }

        ModelingCmd::FaceGetCenter {
            object_id, face_id,
        } => {
            let shape = session
                .get_shape(&face_id)
                .or_else(|| session.get_shape(&object_id))
                .ok_or_else(|| format!("Shape not found for face query"))?;
            Ok(OkModelingCmdResponse::FaceGetCenter {
                data: FaceGetCenterData {
                    pos: query::face_get_center(shape),
                },
            })
        }

        ModelingCmd::FaceGetGradient {
            object_id, face_id, ..
        } => {
            let shape = session
                .get_shape(&face_id)
                .or_else(|| session.get_shape(&object_id))
                .ok_or_else(|| format!("Shape not found for face query"))?;
            let normal = query::face_get_normal_at_center(shape);
            // Compute tangent vectors from normal
            let n = sketch::to_dvec3(&normal);
            let (df_du, df_dv) = compute_tangent_frame(n);
            Ok(OkModelingCmdResponse::FaceGetGradient {
                data: FaceGetGradientData {
                    df_du: sketch::from_dvec3(df_du),
                    df_dv: sketch::from_dvec3(df_dv),
                    normal,
                },
            })
        }

        ModelingCmd::FaceGetPosition {
            object_id, face_id, ..
        } => {
            let shape = session
                .get_shape(&face_id)
                .or_else(|| session.get_shape(&object_id))
                .ok_or_else(|| format!("Shape not found for face query"))?;
            Ok(OkModelingCmdResponse::FaceGetPosition {
                data: FaceGetPositionData {
                    pos: query::face_get_center(shape),
                },
            })
        }

        // -- Edge/topology queries --
        ModelingCmd::EntityGetDistance { .. } => {
            // BRepExtrema_DistShapeShape not exposed in opencascade-rs
            Ok(OkModelingCmdResponse::EntityGetDistance {
                data: EntityGetDistanceData {
                    min_distance: 0.0,
                    max_distance: 0.0,
                },
            })
        }

        ModelingCmd::Solid3dGetAllEdgeFaces { object_id, .. } => {
            let face_ids: Vec<Uuid> = session
                .entities
                .get(&object_id)
                .map(|e| {
                    e.children
                        .iter()
                        .filter(|id| {
                            session
                                .entities
                                .get(id)
                                .is_some_and(|e| e.entity_type == EntityType::Face)
                        })
                        .copied()
                        .collect()
                })
                .unwrap_or_default();
            Ok(OkModelingCmdResponse::Solid3dGetAllEdgeFaces {
                data: Solid3dGetAllEdgeFacesData { faces: face_ids },
            })
        }

        ModelingCmd::Solid3dGetAllOppositeEdges { object_id, .. } => {
            let edge_ids: Vec<Uuid> = session
                .entities
                .get(&object_id)
                .map(|e| {
                    e.children
                        .iter()
                        .filter(|id| {
                            session
                                .entities
                                .get(id)
                                .is_some_and(|e| e.entity_type == EntityType::Edge)
                        })
                        .copied()
                        .collect()
                })
                .unwrap_or_default();
            Ok(OkModelingCmdResponse::Solid3dGetAllOppositeEdges {
                data: Solid3dGetAllOppositeEdgesData { edges: edge_ids },
            })
        }

        ModelingCmd::Solid3dGetOppositeEdge { edge_id, .. } => {
            Ok(OkModelingCmdResponse::Solid3dGetOppositeEdge {
                data: Solid3dGetOppositeEdgeData { edge: edge_id },
            })
        }

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

        // -- Curve queries --
        ModelingCmd::CurveGetControlPoints { .. } => {
            Ok(OkModelingCmdResponse::CurveGetControlPoints {
                data: CurveGetControlPointsData {
                    control_points: vec![],
                },
            })
        }

        ModelingCmd::CurveGetEndPoints { curve_id } => {
            // Try to get shape and extract edge endpoints
            if let Some(shape) = session.get_shape(&curve_id) {
                let mut edges = shape.edges();
                if let Some(edge) = edges.next() {
                    return Ok(OkModelingCmdResponse::CurveGetEndPoints {
                        data: CurveGetEndPointsData {
                            start: sketch::from_dvec3(edge.start_point()),
                            end: sketch::from_dvec3(edge.end_point()),
                        },
                    });
                }
            }
            Ok(OkModelingCmdResponse::CurveGetEndPoints {
                data: CurveGetEndPointsData {
                    start: Point3d { x: 0.0, y: 0.0, z: 0.0 },
                    end: Point3d { x: 0.0, y: 0.0, z: 0.0 },
                },
            })
        }

        ModelingCmd::CurveGetType { .. } => Ok(OkModelingCmdResponse::CurveGetType {
            data: CurveGetTypeData {
                curve_type: "line".to_string(),
            },
        }),

        // -- Measurements (real OCCT implementations) --
        ModelingCmd::Mass {
            entity_ids,
            material_density,
            ..
        } => {
            let mut total_volume = 0.0;
            for id in &entity_ids {
                if let Some(shape) = session.get_shape(id) {
                    total_volume += query::volume(shape);
                }
            }
            let mass = total_volume * material_density;
            Ok(OkModelingCmdResponse::Mass {
                data: MassData {
                    mass,
                    output_unit: "kg".to_string(),
                },
            })
        }

        ModelingCmd::Volume { entity_ids, .. } => {
            let mut total_volume = 0.0;
            for id in &entity_ids {
                if let Some(shape) = session.get_shape(id) {
                    total_volume += query::volume(shape);
                }
            }
            Ok(OkModelingCmdResponse::Volume {
                data: VolumeData {
                    volume: total_volume,
                    output_unit: "mm3".to_string(),
                },
            })
        }

        ModelingCmd::SurfaceArea { entity_ids, .. } => {
            let mut total_area = 0.0;
            for id in &entity_ids {
                if let Some(shape) = session.get_shape(id) {
                    total_area += query::surface_area(shape);
                }
            }
            Ok(OkModelingCmdResponse::SurfaceArea {
                data: SurfaceAreaData {
                    surface_area: total_area,
                    output_unit: "mm2".to_string(),
                },
            })
        }

        ModelingCmd::CenterOfMass { entity_ids, .. } => {
            // Use center of mass of the first entity
            if let Some(id) = entity_ids.first() {
                if let Some(shape) = session.get_shape(id) {
                    let com = query::center_of_mass(shape);
                    return Ok(OkModelingCmdResponse::CenterOfMass {
                        data: CenterOfMassData {
                            center_of_mass: com,
                            output_unit: "mm".to_string(),
                        },
                    });
                }
            }
            Ok(OkModelingCmdResponse::CenterOfMass {
                data: CenterOfMassData {
                    center_of_mass: Point3d { x: 0.0, y: 0.0, z: 0.0 },
                    output_unit: "mm".to_string(),
                },
            })
        }

        ModelingCmd::Density {
            entity_ids,
            material_mass,
            ..
        } => {
            let mut total_volume = 0.0;
            for id in &entity_ids {
                if let Some(shape) = session.get_shape(id) {
                    total_volume += query::volume(shape);
                }
            }
            let density = if total_volume > 0.0 {
                material_mass / total_volume
            } else {
                0.0
            };
            Ok(OkModelingCmdResponse::Density {
                data: DensityData {
                    density,
                    output_unit: "kg_per_m3".to_string(),
                },
            })
        }

        // -- Export/Import (real implementations) --
        ModelingCmd::Export {
            entity_ids, format,
        } => {
            use protocol::modeling_cmd::ExportFormat;

            let mut files = vec![];
            for id in &entity_ids {
                if let Some(shape) = session.get_shape(id) {
                    let (name, contents) = match &format {
                        ExportFormat::Step {} => {
                            let data = export::export_step(shape)?;
                            ("export.step".to_string(), data)
                        }
                        ExportFormat::Stl { .. } => {
                            let data = export::export_stl(shape)?;
                            ("export.stl".to_string(), data)
                        }
                        _ => {
                            return Err(format!("Export format {format:?} not yet supported"));
                        }
                    };
                    files.push(ExportFileData { name, contents });
                }
            }
            Ok(OkModelingCmdResponse::Export {
                data: ExportData { files },
            })
        }

        ModelingCmd::ImportFiles { files, .. } => {
            let object_id = Uuid::new_v4();
            // Try to import the first file
            if let Some(file) = files.first() {
                if let Some(data) = &file.data {
                    match export::import_step(data) {
                        Ok(shape) => {
                            session.entities.insert(
                                object_id,
                                Entity {
                                    id: object_id,
                                    entity_type: EntityType::Solid,
                                    parent_id: None,
                                    children: vec![],
                                    visible: true,
                                    shape: Some(shape),
                                },
                            );
                        }
                        Err(e) => {
                            tracing::warn!("Import failed: {e}");
                        }
                    }
                }
            }
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
fn compute_segment_endpoint(from: &Point3d, segment: &protocol::modeling_cmd::PathSegment) -> Point3d {
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
        PathSegment::Arc {
            center,
            radius,
            end_angle,
            ..
        } => Point3d {
            x: center.x + radius * end_angle.cos(),
            y: center.y + radius * end_angle.sin(),
            z: center.z,
        },
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

/// Compute two orthogonal tangent vectors from a normal vector.
fn compute_tangent_frame(n: DVec3) -> (DVec3, DVec3) {
    // Pick a vector not parallel to n
    let up = if n.x.abs() < 0.9 {
        DVec3::X
    } else {
        DVec3::Y
    };
    let df_du = n.cross(up).normalize();
    let df_dv = n.cross(df_du).normalize();
    (df_du, df_dv)
}
