use crate::edge::Edge;
use crate::error::BitDagError;
use crate::traits::ToEdges;
use bit_matrix::BitMatrix;
use bit_matrix::block::BITS;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use std::collections::HashMap;

/// A memory-efficient Directed Acyclic Graph (DAG) representation optimized for fast
/// ancestor, descendant, and relationship queries using an underlying bit matrix.
///
/// `BitDag` computes the transitive closure of the graph upon construction, enabling
/// $O(1)$ complexity for ancestry checks and highly parallelized bitwise operations
/// for bulk relationship queries.
#[cfg_attr(
    feature = "miniserde",
    derive(miniserde::Serialize, miniserde::Deserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct BitDag {
    /// Dense bit matrix representing the transitive closure of the DAG.
    /// `matrix[(i, j)] == true` indicates that node `i` is an ancestor of node `j`.
    matrix: BitMatrix,
    /// Adjacency list storing the immediate children indices for each node.
    direct_children: Vec<Vec<usize>>,
    /// Adjacency list storing the immediate parent indices for each node.
    direct_parents: Vec<Vec<usize>>,
    /// Bidirectional mapping lookup: string term identifier to its internal matrix index.
    term_to_idx: HashMap<String, usize>,
    /// Bidirectional mapping lookup: internal matrix index to its string term identifier.
    idx_to_term: Vec<String>,
}

impl BitDag {
    /// Constructs a `BitDag` from an object implementing `ToEdges`, beginning traversal from a root node.
    ///
    /// # Errors
    ///
    /// Returns a `crate::Result::Err` if the graph traversal or edge extraction fails.
    pub fn from_graph(dag: &impl ToEdges, root_node: &str) -> crate::Result<Self> {
        let edges = dag.edges(root_node)?;
        Ok(Self::build(edges.as_slice()))
    }

    /// Constructs a `BitDag` from a flat slice of [`Edge`] relationships.
    pub fn from_edges(edges: &[Edge]) -> BitDag {
        Self::build(edges)
    }

    /// Internal builder that extracts unique terms, alphabetizes them for deterministic indexing,
    /// populates adjacency vectors, and computes the transitive closure matrix.
    fn build(edges: &[Edge]) -> BitDag {
        let mut terms: Vec<String> = edges
            .iter()
            .flat_map(|e| [e.parent().to_string(), e.child().to_string()])
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        terms.sort();

        let term_to_idx: HashMap<_, _> = terms
            .iter()
            .enumerate()
            .map(|(i, t)| (t.clone(), i))
            .collect();

        let n_terms = terms.len();
        let mut matrix = BitMatrix::new(n_terms, n_terms);

        let mut direct_children = vec![Vec::new(); n_terms];
        let mut direct_parents = vec![Vec::new(); n_terms];

        for e in edges {
            let parent_idx = term_to_idx[e.parent()];
            let child_idx = term_to_idx[e.child()];

            matrix.set(parent_idx, child_idx, true);

            direct_children[parent_idx].push(child_idx);
            direct_parents[child_idx].push(parent_idx);
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
            direct_parents,
            term_to_idx,
            idx_to_term: terms,
        }
    }

    /// Retrieves the immediate (1-hop) children of a given subject node.
    ///
    /// # Errors
    ///
    /// Returns [`BitDagError::UnknownID`] if the subject node does not exist in the DAG.
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

    /// Retrieves the immediate (1-hop) parents of a given subject node.
    ///
    /// # Errors
    ///
    /// Returns [`BitDagError::UnknownID`] if the subject node does not exist in the DAG.
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

    /// Retrieves all children located exactly `depth` steps down from the subject node.
    ///
    /// * A `depth` of `0` returns the subject itself.
    /// * A `depth` of `1` is equivalent to calling [`Self::get_immediate_children`].
    ///
    /// # Errors
    ///
    /// Returns [`BitDagError::UnknownID`] if the subject node does not exist in the DAG.
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

    /// Retrieves all parents located exactly `depth` steps up from the subject node.
    ///
    /// * A `depth` of `0` returns the subject itself.
    /// * A `depth` of `1` is equivalent to calling [`Self::get_immediate_parents`].
    ///
    /// # Errors
    ///
    /// Returns [`BitDagError::UnknownID`] if the subject node does not exist in the DAG.
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

    /// Retrieves all descendants (sub-nodes) of a subject node across all generations.
    ///
    /// # Errors
    ///
    /// Returns [`BitDagError::UnknownID`] if the subject node does not exist in the DAG.
    pub fn get_descendants(&self, subject: &str) -> crate::Result<Vec<&str>> {
        let Some(row_idx) = self.term_to_idx.get(subject) else {
            return Err(BitDagError::UnknownID(subject.to_string()));
        };

        let n_cols = self.matrix.size().1;
        let row = &self.matrix[*row_idx];
        let mut descendants = Vec::new();

        for (word_idx, &word) in row.iter_blocks().enumerate() {
            let mut w: u32 = word;
            while w != 0 {
                let bit = w.trailing_zeros() as usize;
                let col_idx = word_idx * BITS + bit;
                if col_idx >= n_cols {
                    break;
                }
                descendants.push(self.idx_to_term[col_idx].as_str());
                w &= w - 1;
            }
        }

        Ok(descendants)
    }

    /// Retrieves all ancestors (super-nodes) of a subject node across all generations.
    ///
    /// # Errors
    ///
    /// Returns [`BitDagError::UnknownID`] if the subject node does not exist in the DAG.
    pub fn get_ancestors(&self, subject: &str) -> crate::Result<Vec<&str>> {
        let Some(&target_col_idx) = self.term_to_idx.get(subject) else {
            return Err(BitDagError::UnknownID(subject.to_string()));
        };

        let mut ancestors = Vec::new();
        let n_rows = self.matrix.size().0;

        let word_idx = target_col_idx / BITS;

        let bit_mask: u32 = 1 << (target_col_idx % BITS);

        for row_idx in 0..n_rows {
            let row = &self.matrix[row_idx];

            if let Some(&word) = row.iter_blocks().nth(word_idx)
                && (word & bit_mask) != 0
            {
                ancestors.push(self.idx_to_term[row_idx].as_str());
            }
        }

        Ok(ancestors)
    }

    /// Checks if a `child` node is a descendant of an `ancestor` node.
    ///
    /// # Errors
    ///
    /// Returns [`BitDagError::UnknownID`] if either provided identifier is missing from the graph.
    pub fn is_descendant_of(&self, child: &str, ancestor: &str) -> crate::Result<bool> {
        match (self.term_to_idx.get(child), self.term_to_idx.get(ancestor)) {
            (Some(child_idx), Some(parent_idx)) => Ok(self.matrix[(*parent_idx, *child_idx)]),
            (None, Some(_)) => Err(BitDagError::UnknownID(child.to_string())),
            (Some(_), None) => Err(BitDagError::UnknownID(ancestor.to_string())),
            (None, None) => Err(BitDagError::UnknownID(format!(
                "both '{}' and '{}'",
                child, ancestor
            ))),
        }
    }

    /// Checks if a `parent` node is an ancestor of a `child` node.
    ///
    /// # Errors
    ///
    /// Returns [`BitDagError::UnknownID`] if either provided identifier is missing from the graph.
    pub fn is_ancestor_of(&self, parent: &str, child: &str) -> crate::Result<bool> {
        match (self.term_to_idx.get(parent), self.term_to_idx.get(child)) {
            (Some(parent_idx), Some(child_idx)) => Ok(self.matrix[(*parent_idx, *child_idx)]),
            (None, Some(_)) => Err(BitDagError::UnknownID(parent.to_string())),
            (Some(_), None) => Err(BitDagError::UnknownID(child.to_string())),
            (None, None) => Err(BitDagError::UnknownID(format!(
                "both '{}' and '{}'",
                parent, child
            ))),
        }
    }

    /// Finds all terms that are common descendants of both node `a` and node `b`.
    ///
    /// This evaluates a parallel bitwise `AND` across the matrix rows corresponding to both terms.
    ///
    /// # Errors
    ///
    /// Returns [`BitDagError::UnknownID`] if either node `a` or `b` does not exist in the DAG.
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

    /// Minimizes a sub-slice profile of terms by eliminating redundant ancestors.
    ///
    /// If a term in the provided list is an ancestor of *any other* term in that list,
    /// it is deemed redundant and filtered out. Only the most specific (deepest) nodes remain.
    ///
    /// # Errors
    ///
    /// Returns [`BitDagError::UnknownID`] if any term in the input slice is missing from the DAG.
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

    /// Computes the structural (topology-based) Information Content for every term in the DAG.
    ///
    /// Uses the Sánchez et al. (2011) formulation, derived purely from graph topology
    /// without any annotation data:
    ///
    /// ```text
    /// IC(t) = −ln( |leaves(t)| / ( |anc(t)| × |leaves_total| ) )
    /// ```
    ///
    /// * `|leaves(t)|`    — leaf nodes reachable from `t` (including `t` itself if it is a leaf)
    /// * `|anc(t)|`       — ancestors of `t`, including `t` itself
    /// * `|leaves_total|` — total leaf count in the DAG
    ///
    /// Complexity: O(n² / BITS) for the ancestor pass; O(n / BITS) per term for leaf descendants.
    ///
    /// | Term            | IC        |
    /// |-----------------|-----------|
    /// | Root            | 0.0       |
    /// | Shallow, broad  | low       |
    /// | Deep, narrow    | high      |
    /// | Leaf at depth d | ln(d × L) |
    pub fn structural_ic(&self) -> HashMap<String, f64> {
        let (n_rows, n_cols) = self.matrix.size();
        let n_terms = self.idx_to_term.len();

        let is_leaf: Vec<bool> = (0..n_rows)
            .map(|r| self.matrix[r].iter_blocks().all(|&w| w == 0))
            .collect();

        let total_leaves = is_leaf.iter().filter(|&&l| l).count() as f64;

        let n_blocks = n_terms.div_ceil(BITS);
        let mut leaf_blocks = vec![0u32; n_blocks];
        for (idx, &leaf) in is_leaf.iter().enumerate() {
            if leaf {
                leaf_blocks[idx / BITS] |= 1u32 << (idx % BITS);
            }
        }

        let mut ancestor_counts = vec![1usize; n_terms];
        for row_idx in 0..n_rows {
            for (block_pos, &word) in self.matrix[row_idx].iter_blocks().enumerate() {
                let mut w = word;
                while w != 0 {
                    let bit = w.trailing_zeros() as usize;
                    let col_idx = block_pos * BITS + bit;
                    if col_idx < n_cols {
                        ancestor_counts[col_idx] += 1;
                    }
                    w &= w - 1; // clear lowest set bit
                }
            }
        }

        (0..n_terms)
            .into_par_iter()
            .map(|term_idx| {
                let n_leaf_desc = self.matrix[term_idx]
                    .iter_blocks()
                    .zip(leaf_blocks.iter())
                    .map(|(&row_word, &leaf_word)| (row_word & leaf_word).count_ones() as usize)
                    .sum::<usize>()
                    + if is_leaf[term_idx] { 1 } else { 0 };

                let freq = n_leaf_desc as f64 / (ancestor_counts[term_idx] as f64 * total_leaves);

                (self.idx_to_term[term_idx].clone(), -freq.ln())
            })
            .collect()
    }

    /// Computes Information Content (IC) for every term in the DAG relative to an annotated cohort.
    ///
    /// IC is defined by Resnik's corpus-based formulation:
    ///
    /// ```text
    /// IC(t) = −ln( count(t) / N )
    /// ```
    ///
    /// where `N` is the total number of cohort members and `count(t)` is the number of members
    /// annotated with `t` — either **directly** or because one of their direct terms is a
    /// **descendant** of `t` (i.e., ancestor propagation is applied automatically).
    ///
    /// A per-member boolean mask is used during propagation so that ancestors shared by
    /// multiple direct terms are counted **at most once** per member.
    ///
    /// Terms absent (directly or by ancestry) from every cohort member are omitted from the
    /// result. Terms present in all members receive `IC = 0.0`.
    ///
    /// # Arguments
    ///
    /// * `cohort` — One `Vec<&str>` per member, each listing that member's direct term IDs.
    ///
    /// # Errors
    ///
    /// Returns [`BitDagError::UnknownID`] if any term identifier in `cohort` is absent from the DAG.
    pub fn cohort_ic(&self, cohort: Vec<Vec<&str>>) -> crate::Result<HashMap<String, f64>> {
        let n_members = cohort.len();
        if n_members == 0 {
            return Ok(HashMap::new());
        }

        let n_terms = self.idx_to_term.len();

        let member_indices: crate::Result<Vec<Vec<usize>>> = cohort
            .iter()
            .map(|member_terms| {
                member_terms
                    .iter()
                    .map(|&term| {
                        self.term_to_idx
                            .get(term)
                            .copied()
                            .ok_or_else(|| BitDagError::UnknownID(term.to_string()))
                    })
                    .collect()
            })
            .collect();

        let member_indices = member_indices?;

        let mut counts = vec![0usize; n_terms];

        for term_indices in &member_indices {
            let mut member_mask = vec![false; n_terms];

            for &term_idx in term_indices {
                member_mask[term_idx] = true;

                let word_idx = term_idx / BITS;
                let bit_mask: u32 = 1 << (term_idx % BITS);

                #[allow(clippy::needless_range_loop)]
                for row_idx in 0..n_terms {
                    if let Some(&word) = self.matrix[row_idx].iter_blocks().nth(word_idx)
                        && (word & bit_mask) != 0
                    {
                        member_mask[row_idx] = true;
                    }
                }
            }

            for (idx, present) in member_mask.into_iter().enumerate() {
                if present {
                    counts[idx] += 1;
                }
            }
        }

        let ic_map = counts
            .into_iter()
            .enumerate()
            .filter(|&(_, count)| count > 0)
            .map(|(idx, count)| {
                let freq = count as f64 / n_members as f64;
                (self.idx_to_term[idx].clone(), -freq.ln())
            })
            .collect();

        Ok(ic_map)
    }

    /// Finds and returns all leaf nodes in the DAG.
    ///
    /// A leaf node is defined as a node that has no outgoing descendants (its corresponding row in
    /// the bit matrix evaluates completely to `false`).
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

    fn test_edges() -> Vec<Edge> {
        vec![
            ("A", "B").into(),
            ("B", "C").into(),
            ("G", "C").into(),
            ("C", "D").into(),
            ("C", "F").into(),
        ]
    }

    fn test_bitdag(edges: Vec<Edge>) -> BitDag {
        BitDag::from_edges(edges.as_slice())
    }

    #[test]
    fn test_is_descendant_of() {
        let bitdag = test_bitdag(test_edges());
        assert_eq!(bitdag.is_descendant_of("C", "A"), Ok(true));
        assert_eq!(bitdag.is_descendant_of("A", "C"), Ok(false));
    }

    #[test]
    fn test_is_ancestor_of() {
        let bitdag = test_bitdag(test_edges());
        assert_eq!(bitdag.is_ancestor_of("A", "C"), Ok(true));
        assert_eq!(bitdag.is_ancestor_of("C", "A"), Ok(false));
    }

    #[test]
    fn test_get_descendants() {
        let bitdag = test_bitdag(test_edges());
        assert_eq!(bitdag.get_descendants("A"), Ok(vec!["B", "C", "D", "F"]));
    }

    #[test]
    fn test_get_ancestors() {
        let bitdag = test_bitdag(test_edges());
        assert_eq!(bitdag.get_ancestors("F"), Ok(vec!["A", "B", "C", "G"]));
    }

    #[test]
    fn test_get_n_children_deep() {
        let bitdag = test_bitdag(test_edges());
        assert_eq!(bitdag.get_n_deep_children("A", 1), Ok(vec!["B"]));
    }

    #[test]
    fn test_get_n_parents_deep() {
        let bitdag = test_bitdag(test_edges());
        assert_eq!(bitdag.get_n_deep_parents("D", 1), Ok(vec!["C"]));
    }

    #[test]
    fn test_minimize_profile_several() {
        let bitdag = test_bitdag(test_edges());

        let profile = ["A", "B", "C", "D", "F"];

        let most_specific_children = bitdag.minimize_profile(&profile).unwrap();

        assert_eq!(most_specific_children.len(), 2);
        assert_eq!(most_specific_children, ["D", "F"]);
    }

    #[test]
    fn test_minimize_profile_single() {
        let bitdag = test_bitdag(test_edges());

        let profile = ["A", "B", "C", "D"];

        let most_specific_children = bitdag.minimize_profile(&profile).unwrap();

        let msc = most_specific_children.first().unwrap();

        assert_eq!(most_specific_children.len(), 1);
        assert_eq!(*msc, "D");
    }
    #[test]
    fn test_get_immediate_children() {
        let bitdag = test_bitdag(test_edges());

        assert_eq!(bitdag.get_immediate_children("C"), Ok(vec!["D", "F"]));
    }

    #[test]
    fn test_get_immediate_parents() {
        let bitdag = test_bitdag(test_edges());

        assert_eq!(bitdag.get_immediate_parents("C"), Ok(vec!["B", "G"]));
    }

    #[test]
    fn test_get_leaves() {
        let bitdag = test_bitdag(test_edges());
        assert_eq!(bitdag.get_leaves(), vec!["D", "F"]);
    }
}
