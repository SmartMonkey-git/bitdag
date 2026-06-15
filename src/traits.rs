use crate::edge::Edge;

pub trait ToEdges {
    fn edges(&self, root_node: &str) -> crate::Result<Vec<Edge>>;
}
