use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Modeling commands mirroring Zoo's kittycad-modeling-cmds.
/// We start with P0 commands and add more over time.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelingCmd {
    // -- Sketch operations --
    StartPath {},
    MovePathPen {
        path: Uuid,
        to: Point3d,
    },
    ExtendPath {
        path: Uuid,
        segment: PathSegment,
    },
    ClosePath {
        path_id: Uuid,
    },

    // -- Sketch mode --
    EnableSketchMode {
        entity_id: Uuid,
        planar_normal: Option<Point3d>,
        animated: Option<bool>,
        adjust_camera: Option<bool>,
        ortho: Option<bool>,
    },
    SketchModeDisable {},

    // -- 3D solid operations --
    Extrude {
        target: Uuid,
        distance: f64,
        cap: Option<bool>,
    },
    Revolve {
        target: Uuid,
        axis: Point3d,
        angle: f64,
        tolerance: Option<f64>,
    },
    Solid3dFilletEdge {
        object_id: Uuid,
        edge_id: Uuid,
        radius: f64,
        tolerance: Option<f64>,
        cut_type: Option<String>,
    },

    // -- Booleans --
    BooleanUnion {
        targets: Vec<Uuid>,
    },
    BooleanSubtract {
        target: Uuid,
        tool: Uuid,
    },
    BooleanIntersect {
        targets: Vec<Uuid>,
    },

    // -- Scene/camera --
    DefaultCameraLookAt {
        center: Option<Point3d>,
        eye: Option<Point3d>,
        up: Option<Point3d>,
        vantage: Option<Point3d>,
        sequence: Option<u32>,
    },
    DefaultCameraGetSettings {},
    DefaultCameraZoom {
        magnitude: f64,
    },
    ZoomToFit {
        object_ids: Option<Vec<Uuid>>,
        padding: Option<f64>,
        animated: Option<bool>,
    },
    ViewIsometric {
        padding: Option<f64>,
    },

    // -- Selection --
    SceneClearAll {},
    SelectAdd {
        entities: Vec<Uuid>,
    },
    SelectRemove {
        entities: Vec<Uuid>,
    },
    SelectClear {},
    HighlightSetEntity {
        selected_at_window: Point2d,
        sequence: Option<u32>,
    },

    // -- Planes & units --
    MakePlane {
        origin: Point3d,
        x_axis: Point3d,
        y_axis: Point3d,
        size: f64,
        clobber: Option<bool>,
        hide: Option<bool>,
    },
    SetSceneUnits {
        unit: UnitLength,
    },
    SetBackgroundColor {
        color: Color,
    },
    EdgeLinesVisible {
        hidden: bool,
    },

    // -- Entity info --
    EntityGetParentId {
        entity_id: Uuid,
    },
    EntityGetNumChildren {
        entity_id: Uuid,
    },
    EntityGetChildUuid {
        entity_id: Uuid,
        child_index: u32,
    },
    EntityGetAllChildUuids {
        entity_id: Uuid,
    },
    EntityGetSketchPaths {
        entity_id: Uuid,
    },
    EntityGetDistance {
        entity_id_a: Uuid,
        entity_id_b: Uuid,
        distance_type: Option<String>,
    },

    // -- Export/Import --
    Export {
        entity_ids: Vec<Uuid>,
        format: ExportFormat,
    },
    ImportFiles {
        files: Vec<ImportFile>,
        format: ImportFormat,
    },

    // -- Mouse/interaction (stub/no-op for now) --
    MouseMove {
        window: Point2d,
        sequence: Option<u32>,
    },
    MouseClick {
        window: Point2d,
        entities_modified: Option<Vec<Uuid>>,
    },
    HandleMouseDragStart {
        window: Point2d,
    },
    HandleMouseDragMove {
        window: Point2d,
        sequence: Option<u32>,
    },
    HandleMouseDragEnd {
        window: Point2d,
    },

    // -- Misc --
    ObjectVisible {
        object_id: Uuid,
        hidden: bool,
    },
    ObjectBringToFront {
        object_id: Uuid,
    },
    GetEntityType {
        entity_id: Uuid,
    },
    Solid3dGetAllEdgeFaces {
        object_id: Uuid,
        edge_id: Uuid,
    },
    Solid3dGetAllOppositeEdges {
        object_id: Uuid,
        edge_id: Uuid,
        along_vector: Option<Point3d>,
    },
    Solid3dGetOppositeEdge {
        object_id: Uuid,
        edge_id: Uuid,
        face_id: Uuid,
    },
    Solid3dGetNextAdjacentEdge {
        object_id: Uuid,
        edge_id: Uuid,
        face_id: Uuid,
    },
    Solid3dGetPrevAdjacentEdge {
        object_id: Uuid,
        edge_id: Uuid,
        face_id: Uuid,
    },
    FaceIsPlanar {
        object_id: Uuid,
        face_id: Uuid,
    },
    FaceGetCenter {
        object_id: Uuid,
        face_id: Uuid,
    },
    FaceGetGradient {
        object_id: Uuid,
        face_id: Uuid,
        uv: Point2d,
    },
    FaceGetPosition {
        object_id: Uuid,
        face_id: Uuid,
        uv: Point2d,
    },
    GetSketchModePlane {},
    CurveGetControlPoints {
        curve_id: Uuid,
    },
    CurveGetEndPoints {
        curve_id: Uuid,
    },
    CurveGetType {
        curve_id: Uuid,
    },
    Mass {
        entity_ids: Vec<Uuid>,
        material_density: f64,
        material_density_unit: Option<UnitDensity>,
        output_unit: Option<UnitMass>,
    },
    Volume {
        entity_ids: Vec<Uuid>,
        output_unit: Option<UnitVolume>,
    },
    SurfaceArea {
        entity_ids: Vec<Uuid>,
        output_unit: Option<UnitArea>,
    },
    CenterOfMass {
        entity_ids: Vec<Uuid>,
        output_unit: Option<UnitLength>,
    },
    Density {
        entity_ids: Vec<Uuid>,
        material_mass: f64,
        material_mass_unit: Option<UnitMass>,
        output_unit: Option<UnitDensity>,
    },

    // Catch-all for unimplemented commands
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Point3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Point2d {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PathSegment {
    Line {
        end: Point3d,
        relative: Option<bool>,
    },
    Arc {
        center: Point3d,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
    },
    Bezier {
        control1: Point3d,
        control2: Point3d,
        end: Point3d,
    },
    TangentialArc {
        offset: Option<Point3d>,
        radius: Option<f64>,
        to: Option<Point3d>,
    },
    TangentialArcTo {
        angle_snap_increment: Option<f64>,
        to: Point3d,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExportFormat {
    Step {},
    Stl { coords: Option<StlCoords> },
    Gltf { storage: Option<String> },
    Obj {},
    Ply {},
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StlCoords {
    pub up: Option<String>,
    pub forward: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImportFile {
    pub path: String,
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImportFormat {
    Step {},
    Stl {},
    Gltf {},
    Obj {},
    Ply {},
    Fbx {},
    Sldprt {},
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitLength {
    Cm,
    Ft,
    In,
    M,
    Mm,
    Yd,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitDensity {
    LbPerFt3,
    KgPerM3,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitMass {
    G,
    Kg,
    Lb,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitVolume {
    Cm3,
    Ft3,
    In3,
    M3,
    Mm3,
    Yd3,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitArea {
    Cm2,
    Ft2,
    In2,
    M2,
    Mm2,
    Yd2,
}
