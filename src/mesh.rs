use glam::Vec3;

use crate::Point;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MeshPoint {
    pub(crate) pos: Vec3,
    pub(crate) normal: Option<Vec3>,
    pub(crate) used: bool,
    pub(crate) edges: Vec<MeshEdge>,
}

impl MeshPoint {
    pub(crate) fn new(pos: Vec3) -> Self {
        Self {
            pos,
            normal: None,
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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MeshEdge {
    pub(crate) a: MeshPoint,
    pub(crate) b: MeshPoint,
    pub(crate) opposite: MeshPoint,
    pub(crate) center: Vec3,
    pub(crate) prev: Option<Box<MeshEdge>>,
    pub(crate) next: Option<Box<MeshEdge>>,
    pub(crate) status: EdgeStatus,
}

impl MeshEdge {
    pub(crate) fn new(a: &MeshPoint, b: &MeshPoint, opposite: MeshPoint, center: Vec3) -> Self {
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

#[derive(Clone, Debug)]
pub(crate) struct MeshFace(pub(crate) [MeshPoint; 3]);

impl MeshFace {
    pub(crate) fn normal(&self) -> Vec3 {
        let cross = (self.0[0].pos - self.0[1].pos).cross(self.0[0].pos - self.0[2].pos);
        cross.normalize()
    }
}
