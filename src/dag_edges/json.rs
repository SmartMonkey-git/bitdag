use crate::edge::Edge;
use crate::error::BitDagError;
use crate::traits::ToEdges;
use ontolius::TermId;
use ontolius::ontology::HierarchyWalks;
use ontolius::ontology::csr::FullCsrOntology;
use std::collections::VecDeque;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

impl<T> ToEdges for Arc<T>
where
    T: ToEdges,
{
    fn edges(&self, root_node: &str) -> crate::Result<Vec<Edge>> {
        self.deref().edges(root_node)
    }
}

impl<T> ToEdges for Box<T>
where
    T: ToEdges,
{
    fn edges(&self, root_node: &str) -> crate::Result<Vec<Edge>> {
        self.deref().edges(root_node)
    }
}

impl ToEdges for FullCsrOntology {
    fn edges(&self, root_node: &str) -> Result<Vec<Edge>, BitDagError> {
        let root_node = TermId::from_str(root_node)
            .map_err(|_| BitDagError::UnknownID(root_node.to_string()))?;

        let mut schedule: VecDeque<&TermId> = VecDeque::new();
        schedule.push_front(&root_node);

        let mut edges = Vec::new();

        let mut i = 0;
        while i < schedule.len() {
            let parent = schedule[i];
            for child in self.iter_child_ids(parent) {
                edges.push((parent.to_string(), child.to_string()).into());

                if !schedule.contains(&child) {
                    schedule.push_back(child);
                }
            }
            i += 1;
        }

        Ok(edges)
    }
}
