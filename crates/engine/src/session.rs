use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use protocol::modeling_cmd::{Point3d, UnitLength};

/// Per-connection engine session holding all entity state.
pub struct Session {
    /// Map of entity UUID to entity data.
    pub entities: HashMap<Uuid, Entity>,
    /// Active path builders (in-progress sketches).
    pub paths: HashMap<Uuid, PathBuilder>,
    /// Currently selected entity IDs.
    pub selection: HashSet<Uuid>,
    /// Whether sketch mode is active.
    pub sketch_mode: Option<SketchMode>,
    /// Scene units.
    pub units: UnitLength,
    /// Camera state.
    pub camera: CameraState,
}

impl Session {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            paths: HashMap::new(),
            selection: HashSet::new(),
            sketch_mode: None,
            units: UnitLength::Mm,
            camera: CameraState::default(),
        }
    }

    /// Clears all entities and resets session state.
    pub fn clear(&mut self) {
        self.entities.clear();
        self.paths.clear();
        self.selection.clear();
        self.sketch_mode = None;
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a stored entity in the session.
#[derive(Debug, Clone)]
pub struct Entity {
    pub id: Uuid,
    pub entity_type: EntityType,
    pub parent_id: Option<Uuid>,
    pub children: Vec<Uuid>,
    pub visible: bool,
    // When OpenCASCADE is integrated, this will hold:
    // pub shape: Option<TopoDS_Shape>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    Path,
    Face,
    Solid,
    Edge,
    Plane,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityType::Path => write!(f, "path"),
            EntityType::Face => write!(f, "face"),
            EntityType::Solid => write!(f, "solid"),
            EntityType::Edge => write!(f, "edge"),
            EntityType::Plane => write!(f, "plane"),
        }
    }
}

/// In-progress path construction.
#[derive(Debug, Clone)]
pub struct PathBuilder {
    pub id: Uuid,
    pub pen_position: Option<Point3d>,
    pub segments: Vec<PathSegmentRecord>,
    pub closed: bool,
}

/// A recorded path segment for later geometry construction.
#[derive(Debug, Clone)]
pub struct PathSegmentRecord {
    pub from: Point3d,
    pub segment: protocol::modeling_cmd::PathSegment,
}

/// Sketch mode state.
#[derive(Debug, Clone)]
pub struct SketchMode {
    pub entity_id: Uuid,
    pub plane_origin: Point3d,
    pub plane_normal: Point3d,
    pub plane_x_axis: Point3d,
    pub plane_y_axis: Point3d,
}

/// Camera state (tracked server-side, rendered client-side).
#[derive(Debug, Clone)]
pub struct CameraState {
    pub position: Point3d,
    pub center: Point3d,
    pub up: Point3d,
    pub fov_y: f64,
    pub ortho: bool,
    pub ortho_scale: f64,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            position: Point3d {
                x: 0.0,
                y: 0.0,
                z: 100.0,
            },
            center: Point3d {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            up: Point3d {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            fov_y: 45.0,
            ortho: false,
            ortho_scale: 1.0,
        }
    }
}
