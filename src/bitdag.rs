use crate::error::BitDagError;
use crate::traits::GetDAGEdges;
use bit_matrix::BitMatrix;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use std::collections::{HashMap, HashSet};

#[cfg_attr(
    feature = "miniserde",
    derive(miniserde::Serialize, miniserde::Deserialize)
)]
#[derive(Debug, Clone)]
pub struct BitDag {
    matrix: BitMatrix,
    direct_children: Vec<Vec<usize>>,
    direct_parents: Vec<Vec<usize>>,
    term_to_idx: HashMap<String, usize>,
    idx_to_term: Vec<String>,
}

impl BitDag {
    pub fn new(dag: &impl GetDAGEdges, root_node: &str) -> crate::Result<Self> {
        let edges = dag.edges(root_node)?;
        Ok(Self::build(edges.as_slice()))
    }

    pub fn from_edges(edges: &[(String, String)]) -> BitDag {
        Self::build(edges)
    }

    fn build(edges: &[(String, String)]) -> BitDag {
        let mut terms: Vec<String> = edges
            .iter()
            .flat_map(|(a, b)| [a.clone(), b.clone()])
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        terms.sort();

        let term_to_idx: HashMap<_, _> = terms
            .iter()
            .enumerate()
            .map(|(i, t)| (t.clone(), i))
            .collect();

        let n = terms.len();
        let mut matrix = BitMatrix::new(n, n);

        let mut direct_children = vec![Vec::new(); n];
        let mut direct_parents = vec![Vec::new(); n]; // NEW

        for (parent, child) in edges {
            let i = term_to_idx[parent];
            let j = term_to_idx[child];

            matrix.set(i, j, true);

            direct_children[i].push(j);
            direct_parents[j].push(i); // NEW: Child points back to parent
        }

        for children in direct_children.iter_mut() {
            children.sort_unstable();
            children.dedup();
        }

        for parents in direct_parents.iter_mut() {
            parents.sort_unstable();
            parents.dedup();
        }

        matrix.transitive_closure();

        Self {
            matrix,
            direct_children,
            direct_parents, // NEW
            term_to_idx,
            idx_to_term: terms,
        }
    }

    pub fn get_immediate_children(&self, subject: &str) -> crate::Result<Vec<&str>> {
        if let Some(idx) = self.term_to_idx.get(subject) {
            let children = self.direct_children[*idx]
                .iter()
                .map(|&child_idx| self.idx_to_term[child_idx].as_str())
                .collect();
            Ok(children)
        } else {
            Err(BitDagError::UnknownID(subject.to_string()))
        }
    }

    pub fn get_immediate_parents(&self, subject: &str) -> crate::Result<Vec<&str>> {
        if let Some(idx) = self.term_to_idx.get(subject) {
            let parents = self.direct_parents[*idx]
                .iter()
                .map(|&parent_idx| self.idx_to_term[parent_idx].as_str())
                .collect();
            Ok(parents)
        } else {
            Err(BitDagError::UnknownID(subject.to_string()))
        }
    }

    pub fn get_n_deep_children<'a>(
        &'a self,
        subject: &'a str,
        depth: usize,
    ) -> crate::Result<Vec<&'a str>> {
        let start_idx = self
            .term_to_idx
            .get(subject)
            .ok_or_else(|| BitDagError::UnknownID(subject.to_string()))?;

        if depth == 0 {
            return Ok(vec![subject]);
        }
        if depth == 1 {
            return self.get_immediate_children(subject);
        }

        let mut current_level = vec![*start_idx];
        let mut next_level = Vec::new();

        for _ in 0..depth {
            next_level.clear();

            for &node in &current_level {
                next_level.extend_from_slice(&self.direct_children[node]);
            }

            next_level.sort_unstable();
            next_level.dedup();

            if next_level.is_empty() {
                return Ok(Vec::new());
            }
            std::mem::swap(&mut current_level, &mut next_level);
        }

        let n_deep_children = current_level
            .into_iter()
            .map(|child_idx| self.idx_to_term[child_idx].as_str())
            .collect();

        Ok(n_deep_children)
    }

    pub fn get_n_deep_parents<'a>(
        &'a self,
        subject: &'a str,
        depth: usize,
    ) -> crate::Result<Vec<&'a str>> {
        let start_idx = self
            .term_to_idx
            .get(subject)
            .ok_or_else(|| BitDagError::UnknownID(subject.to_string()))?;

        if depth == 0 {
            return Ok(vec![subject]);
        }
        if depth == 1 {
            return self.get_immediate_parents(subject);
        }

        let mut current_level = vec![*start_idx];
        let mut next_level = Vec::new();

        for _ in 0..depth {
            next_level.clear();

            for &node in &current_level {
                next_level.extend_from_slice(&self.direct_parents[node]);
            }

            next_level.sort_unstable();
            next_level.dedup();

            if next_level.is_empty() {
                return Ok(Vec::new());
            }

            std::mem::swap(&mut current_level, &mut next_level);
        }

        let n_deep_parents = current_level
            .into_iter()
            .map(|parent_idx| self.idx_to_term[parent_idx].as_str())
            .collect();

        Ok(n_deep_parents)
    }

    pub fn get_descendants(&self, subject: &str) -> crate::Result<Vec<&str>> {
        if let Some(row_idx) = self.term_to_idx.get(subject) {
            let n_cols = self.matrix.size().1;
            let row = &self.matrix[*row_idx];

            let descendants: Vec<&str> = (0..n_cols)
                .into_par_iter()
                .filter_map(|col_idx| {
                    let is_descendant = row[col_idx];
                    is_descendant.then(|| self.idx_to_term[col_idx].as_str())
                })
                .collect();

            Ok(descendants)
        } else {
            Err(BitDagError::UnknownID(subject.to_string()))
        }
    }

    pub fn get_ancestors(&self, subject: &str) -> crate::Result<Vec<&str>> {
        if let Some(col_idx) = self.term_to_idx.get(subject) {
            let n_rows = self.matrix.size().0;

            let ancestors: Vec<&str> = (0..n_rows)
                .into_par_iter()
                .filter_map(|row_idx| {
                    let is_ancestor = self.matrix[row_idx][*col_idx];

                    is_ancestor.then(|| self.idx_to_term[row_idx].as_str())
                })
                .collect();

            Ok(ancestors)
        } else {
            Err(BitDagError::UnknownID(subject.to_string()))
        }
    }
    pub fn is_descendant_of(&self, child: &str, ancestor: &str) -> crate::Result<bool> {
        if let Some(child_idx) = self.term_to_idx.get(child)
            && let Some(parent_idx) = self.term_to_idx.get(ancestor)
        {
            Ok(self.matrix[(*parent_idx, *child_idx)])
        } else {
            Err(BitDagError::UnknownID(child.to_string()))
        }
    }

    pub fn is_ancestor_of(&self, parent: &str, child: &str) -> crate::Result<bool> {
        if let Some(child_idx) = self.term_to_idx.get(child)
            && let Some(parent_idx) = self.term_to_idx.get(parent)
        {
            Ok(self.matrix[(*parent_idx, *child_idx)])
        } else {
            Err(BitDagError::UnknownID(child.to_string()))
        }
    }

    pub fn get_common_descendants(&self, a: &str, b: &str) -> crate::Result<Vec<&str>> {
        let a_idx = self
            .term_to_idx
            .get(a)
            .ok_or_else(|| BitDagError::UnknownID(a.to_string()))?;
        let b_idx = self
            .term_to_idx
            .get(b)
            .ok_or_else(|| BitDagError::UnknownID(b.to_string()))?;

        let row_a = &self.matrix[*a_idx];
        let row_b = &self.matrix[*b_idx];

        let n_cols = self.matrix.size().1;
        let common = (0..n_cols)
            .into_par_iter()
            .filter_map(|col_idx| {
                (row_a[col_idx] && row_b[col_idx]).then(|| self.idx_to_term[col_idx].as_str())
            })
            .collect();

        Ok(common)
    }

    pub fn minimize_profile<'a>(&self, terms: &[&'a str]) -> crate::Result<Vec<&'a str>> {
        let mut indices = Vec::with_capacity(terms.len());
        for &term in terms {
            let idx = self
                .term_to_idx
                .get(term)
                .ok_or_else(|| BitDagError::UnknownID(term.to_string()))?;
            indices.push(*idx);
        }

        let minimized = terms
            .iter()
            .enumerate()
            .filter_map(|(i, &term)| {
                let term_idx = indices[i];

                let is_redundant = indices
                    .iter()
                    .enumerate()
                    .any(|(j, &other_idx)| i != j && self.matrix[(term_idx, other_idx)]);

                if is_redundant { None } else { Some(term) }
            })
            .collect();

        Ok(minimized)
    }

    pub fn get_leaves(&self) -> Vec<&str> {
        let (n_rows, n_cols) = self.matrix.size();
        (0..n_rows)
            .into_par_iter()
            .filter_map(|row_idx| {
                let is_leaf = (0..n_cols).all(|col_idx| !self.matrix[(row_idx, col_idx)]);
                is_leaf.then(|| self.idx_to_term[row_idx].as_str())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ontolius::io::OntologyLoaderBuilder;
    use ontolius::ontology::csr::FullCsrOntology;
    use std::path::PathBuf;
    use std::str::FromStr;

    #[test]
    fn test_json() {
        let a = PathBuf::from_str("/Users/rouvenreuter/Downloads/hp.json").unwrap();
        let loader = OntologyLoaderBuilder::new().obographs_parser().build();

        let ontolius: FullCsrOntology = loader.load_from_path(a).unwrap();

        let edges = ontolius.edges("HP:0000118").unwrap();
        let bit_dag = BitDag::from_edges(edges.as_slice());

        println!("{:?}", bit_dag.is_descendant_of("HP:5200203", "HP:0000738"));
        println!("{:?}", bit_dag.is_descendant_of("HP:0002367", "HP:0000738"));
    }
    #[test]

    fn test_json_2() {
        let a = PathBuf::from_str("/Users/rouvenreuter/Downloads/hp.json").unwrap();
        let loader = OntologyLoaderBuilder::new().obographs_parser().build();

        let ontolius: FullCsrOntology = loader.load_from_path(a).unwrap();

        let edges = ontolius.edges("HP:0000118").unwrap();
        let bit_dag = BitDag::from_edges(edges.as_slice());

        println!("{:?}", bit_dag.get_descendants("HP:0000738"));
        println!("{:?}", bit_dag.is_descendant_of("HP:0002367", "HP:0000738"));
    }
}
