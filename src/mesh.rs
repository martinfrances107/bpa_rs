use glam::Vec3;

#[derive(Debug, Default, PartialEq)]
pub(crate) enum EdgeStatus {
    #[default]
    Active,
    Inner,
    Boundary,
}

#[derive(Debug, PartialEq)]
pub(crate) struct MeshPoint<'a> {
    pub(crate) pos: Vec3,
    pub(crate) normal: Option<Vec3>,
    pub(crate) used: bool,
    pub(crate) edges: Option<Vec<&'a MeshEdge<'a>>>,
}

impl<'a> MeshPoint<'a> {
    pub(crate) fn new(pos: Vec3) -> Self {
        Self {
            pos,
            normal: None,
            used: false,
            edges: None,
        }
    }

    pub(crate) fn add_edge(&mut self, edge: &'a MeshEdge<'a>) {
        match self.edges {
            Some(ref mut edges) => edges.push(edge),
            None => self.edges = Some(vec![edge]),
        }
    }
}
#[derive(Debug, PartialEq)]
pub(crate) struct MeshEdge<'a> {
    pub(crate) a: &'a MeshPoint<'a>,
    pub(crate) b: &'a MeshPoint<'a>,
    pub(crate) opposite: &'a MeshPoint<'a>,
    pub(crate) center: Vec3,
    pub(crate) prev: Option<&'a MeshEdge<'a>>,
    pub(crate) next: Option<&'a MeshEdge<'a>>,
    pub(crate) status: EdgeStatus,
}

impl<'a> MeshEdge<'a> {
    pub(crate) fn new(
        a: &'a MeshPoint<'a>,
        b: &'a MeshPoint<'a>,
        opposite: &'a MeshPoint<'a>,
        center: Vec3,
    ) -> Self {
        Self {
            a,
            b,
            opposite,
            center,
            prev: None,
            next: None,
            status: EdgeStatus::Active,
        }
    }
}

#[derive(Debug)]
pub(crate) struct MeshFace<'a>(pub(crate) [MeshPoint<'a>; 3]);

impl MeshFace<'_> {
    pub(crate) fn normal(&self) -> Vec3 {
        let cross = (self.0[0].pos - self.0[1].pos).cross(self.0[0].pos - self.0[2].pos);
        cross.normalize()
    }
}
