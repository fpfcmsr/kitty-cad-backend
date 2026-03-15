use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modeling_cmd::Point3d;

/// Successful modeling command response variants.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OkModelingCmdResponse {
    Empty {},
    StartPath {
        data: StartPathData,
    },
    ClosePath {
        data: ClosePathData,
    },
    Extrude {
        data: ExtrudeData,
    },
    Revolve {
        data: RevolveData,
    },
    MakePlane {
        data: MakePlaneData,
    },
    Export {
        data: ExportData,
    },
    ImportFiles {
        data: ImportFilesData,
    },
    SelectAdd {
        data: SelectAddData,
    },
    DefaultCameraGetSettings {
        data: CameraSettings,
    },
    HighlightSetEntity {
        data: HighlightSetEntityData,
    },
    EntityGetParentId {
        data: EntityGetParentIdData,
    },
    EntityGetNumChildren {
        data: EntityGetNumChildrenData,
    },
    EntityGetChildUuid {
        data: EntityGetChildUuidData,
    },
    EntityGetAllChildUuids {
        data: EntityGetAllChildUuidsData,
    },
    EntityGetSketchPaths {
        data: EntityGetSketchPathsData,
    },
    EntityGetDistance {
        data: EntityGetDistanceData,
    },
    GetEntityType {
        data: GetEntityTypeData,
    },
    Solid3dGetAllEdgeFaces {
        data: Solid3dGetAllEdgeFacesData,
    },
    Solid3dGetAllOppositeEdges {
        data: Solid3dGetAllOppositeEdgesData,
    },
    Solid3dGetOppositeEdge {
        data: Solid3dGetOppositeEdgeData,
    },
    Solid3dGetNextAdjacentEdge {
        data: Solid3dGetAdjacentEdgeData,
    },
    Solid3dGetPrevAdjacentEdge {
        data: Solid3dGetAdjacentEdgeData,
    },
    FaceIsPlanar {
        data: FaceIsPlanarData,
    },
    FaceGetCenter {
        data: FaceGetCenterData,
    },
    FaceGetGradient {
        data: FaceGetGradientData,
    },
    FaceGetPosition {
        data: FaceGetPositionData,
    },
    GetSketchModePlane {
        data: GetSketchModePlaneData,
    },
    CurveGetControlPoints {
        data: CurveGetControlPointsData,
    },
    CurveGetEndPoints {
        data: CurveGetEndPointsData,
    },
    CurveGetType {
        data: CurveGetTypeData,
    },
    Mass {
        data: MassData,
    },
    Volume {
        data: VolumeData,
    },
    SurfaceArea {
        data: SurfaceAreaData,
    },
    CenterOfMass {
        data: CenterOfMassData,
    },
    Density {
        data: DensityData,
    },
    Solid3dShellFace {
        data: Solid3dShellFaceData,
    },
    Sweep {
        data: SweepData,
    },
    Loft {
        data: LoftData,
    },
    BoundingBox {
        data: BoundingBoxData,
    },
    Solid3dFilletEdge {
        data: Solid3dFilletEdgeData,
    },
    Modeling {
        data: ModelingData,
    },
}

// -- Response data types --

#[derive(Debug, Serialize, Deserialize)]
pub struct StartPathData {
    pub path_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClosePathData {
    pub face_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtrudeData {
    pub solid_id: Uuid,
    pub face_ids: Vec<Uuid>,
    pub edge_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevolveData {
    pub solid_id: Uuid,
    pub face_ids: Vec<Uuid>,
    pub edge_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MakePlaneData {
    pub plane_id: Uuid,
    pub origin: Point3d,
    pub x_axis: Point3d,
    pub y_axis: Point3d,
    pub z_axis: Point3d,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportData {
    pub files: Vec<ExportFileData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportFileData {
    pub name: String,
    pub contents: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportFilesData {
    pub object_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectAddData {}

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraSettings {
    pub pos: Point3d,
    pub center: Point3d,
    pub up: Point3d,
    pub fov_y: Option<f64>,
    pub ortho: bool,
    pub ortho_scale: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HighlightSetEntityData {
    pub entity_id: Option<Uuid>,
    pub sequence: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityGetParentIdData {
    pub entity_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityGetNumChildrenData {
    pub num: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityGetChildUuidData {
    pub entity_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityGetAllChildUuidsData {
    pub entity_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityGetSketchPathsData {
    pub entity_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityGetDistanceData {
    pub min_distance: f64,
    pub max_distance: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetEntityTypeData {
    pub entity_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Solid3dGetAllEdgeFacesData {
    pub faces: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Solid3dGetAllOppositeEdgesData {
    pub edges: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Solid3dGetOppositeEdgeData {
    pub edge: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Solid3dGetAdjacentEdgeData {
    pub edge: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FaceIsPlanarData {
    pub is_planar: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FaceGetCenterData {
    pub pos: Point3d,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FaceGetGradientData {
    pub df_du: Point3d,
    pub df_dv: Point3d,
    pub normal: Point3d,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FaceGetPositionData {
    pub pos: Point3d,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetSketchModePlaneData {
    pub origin: Point3d,
    pub x_axis: Point3d,
    pub y_axis: Point3d,
    pub z_axis: Point3d,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurveGetControlPointsData {
    pub control_points: Vec<Point3d>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurveGetEndPointsData {
    pub start: Point3d,
    pub end: Point3d,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurveGetTypeData {
    pub curve_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MassData {
    pub mass: f64,
    pub output_unit: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VolumeData {
    pub volume: f64,
    pub output_unit: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SurfaceAreaData {
    pub surface_area: f64,
    pub output_unit: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CenterOfMassData {
    pub center_of_mass: Point3d,
    pub output_unit: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DensityData {
    pub density: f64,
    pub output_unit: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Solid3dShellFaceData {
    pub solid_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SweepData {
    pub solid_id: Uuid,
    pub face_ids: Vec<Uuid>,
    pub edge_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoftData {
    pub solid_id: Uuid,
    pub face_ids: Vec<Uuid>,
    pub edge_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoundingBoxData {
    pub min: Point3d,
    pub max: Point3d,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Solid3dFilletEdgeData {
    pub solid_id: Uuid,
}

/// Mesh data for client-side Three.js rendering.
#[derive(Debug, Serialize, Deserialize)]
pub struct MeshData {
    pub vertices: Vec<f32>,
    pub normals: Vec<f32>,
    pub indices: Vec<u32>,
}

/// Data returned alongside modeling commands for scene updates.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelingData {
    pub entity_id: Uuid,
    pub mesh: Option<MeshData>,
}
