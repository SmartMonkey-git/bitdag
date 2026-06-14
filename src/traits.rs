pub trait GetDAGEdges {
    fn edges(&self, root_node: &str) -> crate::Result<Vec<(String, String)>>;
}
