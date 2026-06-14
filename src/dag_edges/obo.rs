use crate::error::BitDagError;
use crate::traits::GetDAGEdges;
use fastobo::ast::{EntityFrame, OboDoc, TermClause};
use std::collections::{HashMap, HashSet, VecDeque};

impl GetDAGEdges for OboDoc {
    fn edges(&self, root_node: &str) -> Result<Vec<(String, String)>, BitDagError> {
        let mut parent_to_children: HashMap<String, Vec<String>> = HashMap::new();

        for entity in self.entities() {
            if let EntityFrame::Term(term) = entity {
                let child_id = term.id().to_string();

                for clause in term.clauses() {
                    if let TermClause::IsA(parent_ident) = clause.as_inner() {
                        parent_to_children
                            .entry(parent_ident.to_string())
                            .or_default()
                            .push(child_id.clone());
                    }
                }
            }
        }

        let mut edges = Vec::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        let mut visited: HashSet<String> = HashSet::new();

        queue.push_back(root_node.to_string());
        visited.insert(root_node.to_string());

        while let Some(parent) = queue.pop_front() {
            if let Some(children) = parent_to_children.get(&parent) {
                for child in children {
                    edges.push((parent.clone(), child.clone()));

                    if visited.insert(child.clone()) {
                        queue.push_back(child.clone());
                    }
                }
            }
        }

        Ok(edges)
    }
}
