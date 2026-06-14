use crate::error::BitDagError;
use crate::traits::GetDAGEdges;
use ontolius::TermId;
use ontolius::ontology::HierarchyWalks;
use ontolius::ontology::csr::FullCsrOntology;
use std::collections::VecDeque;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

impl<T> GetDAGEdges for Arc<T>
where
    T: GetDAGEdges,
{
    fn edges(&self, root_node: &str) -> crate::Result<Vec<(String, String)>> {
        self.deref().edges(root_node)
    }
}

impl<T> GetDAGEdges for Box<T>
where
    T: GetDAGEdges,
{
    fn edges(&self, root_node: &str) -> crate::Result<Vec<(String, String)>> {
        self.deref().edges(root_node)
    }
}

impl GetDAGEdges for FullCsrOntology {
    fn edges(&self, root_node: &str) -> Result<Vec<(String, String)>, BitDagError> {
        let root_node = TermId::from_str(root_node)
            .map_err(|_| BitDagError::UnknownID(root_node.to_string()))?;

        let mut schedule: VecDeque<&TermId> = VecDeque::new();
        schedule.push_front(&root_node);

        let mut edges = Vec::new();

        let mut i = 0;
        while i < schedule.len() {
            let parent = schedule[i];
            for child in self.iter_child_ids(parent) {
                edges.push((parent.to_string(), child.to_string()));

                if !schedule.contains(&child) {
                    schedule.push_back(child);
                }
            }
            i += 1;
        }

        Ok(edges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ontolius::io::OntologyLoaderBuilder;
    use std::path::PathBuf;
    #[test]
    fn test_json() {
        let a = PathBuf::from_str("/Users/rouvenreuter/Downloads/hp.json").unwrap();
        let loader = OntologyLoaderBuilder::new().obographs_parser().build();

        let ontolius: FullCsrOntology = loader.load_from_path(a).unwrap();

        let edges = ontolius.edges("HP:0000118").unwrap();
    }
}
