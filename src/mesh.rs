use glam::Vec3;

use crate::Point;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MeshPoint {
    pub(crate) pos: Vec3,
    pub(crate) normal: Option<Vec3>,
    pub(crate) used: bool,
    pub(crate) edges: Option<Vec<MeshEdge>>,
}

impl MeshPoint {
    pub(crate) fn new(pos: Vec3) -> Self {
        Self {
            pos,
            normal: None,
            used: false,
            edges: None,
        }
    }

    pub(crate) fn add_edge(&mut self, edge: &MeshEdge) {
        match self.edges {
            Some(ref mut edges) => edges.push(edge.clone()),
            None => self.edges = Some(vec![edge.clone()]),
        }
    }
}

impl From<&Point> for MeshPoint {
    fn from(point: &Point) -> Self {
        Self {
            pos: point.pos,
            normal: Some(point.normal),
            used: false,
            edges: None,
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

#[derive(Debug)]
pub(crate) struct MeshFace(pub(crate) [MeshPoint; 3]);

impl MeshFace {
    pub(crate) fn normal(&self) -> Vec3 {
        let cross = (self.0[0].pos - self.0[1].pos).cross(self.0[0].pos - self.0[2].pos);
        cross.normalize()
    }
}
