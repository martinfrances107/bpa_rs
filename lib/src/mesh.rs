use core::cell::RefCell;
use std::rc::Rc;

use glam::Vec3;

use crate::Point;

/// A point in 3D space with a normal vector, and list of edges
#[derive(Clone, Debug)]
pub struct MeshPoint {
    pub(crate) pos: Vec3,
    pub(crate) normal: Vec3,
    pub(crate) used: bool,
    pub(crate) edges: Vec<Rc<RefCell<MeshEdge>>>,
}

// Defining is MeshPoint without a normal
// is useful for testing ONLY.
impl MeshPoint {
    /// Constructor
    #[must_use]
    pub const fn new(pos: Vec3) -> Self {
        Self {
            pos,
            normal: glam::vec3(0.0, 0.0, 0.0),
            used: false,
            edges: vec![],
        }
    }
}

impl From<&Point> for MeshPoint {
    fn from(point: &Point) -> Self {
        Self {
            pos: point.pos,
            normal: point.normal,
            used: false,
            edges: vec![],
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum EdgeStatus {
    #[default]
    Active,
    Inner,
    Boundary,
}

#[derive(Clone, Debug)]
pub(crate) struct MeshEdge {
    pub(crate) a: Rc<RefCell<MeshPoint>>,
    pub(crate) b: Rc<RefCell<MeshPoint>>,
    pub(crate) opposite: Rc<RefCell<MeshPoint>>,
    pub(crate) center: Vec3,
    pub(crate) prev: Option<Rc<RefCell<MeshEdge>>>,
    pub(crate) next: Option<Rc<RefCell<MeshEdge>>>,
    pub(crate) status: EdgeStatus,
}

impl MeshEdge {
    pub(crate) fn new(
        a: &Rc<RefCell<MeshPoint>>,
        b: &Rc<RefCell<MeshPoint>>,
        opposite: &Rc<RefCell<MeshPoint>>,
        center: Vec3,
    ) -> Self {
        Self {
            a: a.clone(),
            b: b.clone(),
            opposite: opposite.clone(),
            center,
            prev: None,
            next: None,
            status: EdgeStatus::Active,
        }
    }
}

/// A triangle in 3D space defined by three points
#[derive(Clone, Debug)]
pub struct MeshFace(pub [Rc<RefCell<MeshPoint>>; 3]);

impl MeshFace {
    pub(crate) fn normal(&self) -> Vec3 {
        let cross = (self.0[0].borrow().pos - self.0[1].borrow().pos)
            .cross(self.0[0].borrow().pos - self.0[2].borrow().pos);
        cross.normalize()
    }
}
