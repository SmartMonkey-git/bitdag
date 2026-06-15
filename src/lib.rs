//! # BitDag
//!
//! A fast, memory-efficient Directed Acyclic Graph (DAG) representation utilizing bit matrices.
//!
//! This crate computes the transitive closure of a graph upon construction, translating the
//! hierarchy into a dense bit matrix. This architecture provides O(1) time complexity for
//! immediate ancestry or descendant checks, and allows bulk relationship queries (like finding
//! common descendants) to be highly parallelized using bitwise operations.
//!
//! Included in this crate are the core [`bitdag::BitDag`] structure, fundamental elements like
//! [`edge::Edge`], and standard traits such as [`traits::ToEdges`] to facilitate parsing
//! industry-standard formats into a digestible graph structure.
//!
//! ## Feature Flags
//!
//! This crate provides several optional features to tailor functionality and manage dependencies:
//!
//! * **`serde`** — Implements standard serialization and deserialization traits for core structures.
//! * **`miniserde`** — Implements lightweight serialization traits via `miniserde` for minimal binary overhead.
//! * **`obo`** — Enables the OBO format adapter, unlocking the `dag_edges::obo` module (requires `fastobo`).
//! * **`json_ontology`** — Enables the JSON ontology adapter, unlocking the `dag_edges::json` module (requires `ontolius`).
//! ## Examples
//!
//! ```rust
//! use bitdag::bitdag::BitDag;
//! use bitdag::edge::Edge;
//!
//! fn main() {
//!     // 1. Define your edges (Parent -> Child)
//!     let edges: Vec<Edge> = vec![
//!         ("A", "B").into(),
//!         ("B", "C").into(),
//!         ("C", "D").into(),
//!         ("C", "F").into(),
//!     ];
//!
//!     // 2. Build the BitDag (computes transitive closure automatically)
//!     let dag = BitDag::from_edges(&edges);
//!
//!     // 3. Perform O(1) checks
//!     assert_eq!(dag.is_ancestor_of("A", "D").unwrap(), true);
//!     assert_eq!(dag.is_descendant_of("A", "C").unwrap(), false);
//!
//!     // 4. Extract generations or sub-graphs
//!     let descendants = dag.get_descendants("A").unwrap();
//!     println!("Descendants of A: {:?}", descendants); // ["B", "C", "D", "F"]
//!
//!     let leaves = dag.get_leaves();
//!     println!("Leaves in the graph: {:?}", leaves); // ["D", "F"]
//! }
//! ```
//!
//! ```rust
//!     // Requires the `json_ontology` feature
//!     use ontolius::io::OntologyLoaderBuilder;
//!     use ontolius::ontology::csr::FullCsrOntology;
//!     use bitdag::traits::ToEdges;
//!     use bitdag::bitdag::BitDag;
//!     use std::path::PathBuf;
//!     use std::str::FromStr;
//!
//!     let loader = OntologyLoaderBuilder::new().obographs_parser().build();
//!     let manifest_dir = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
//!     let test_ontology = manifest_dir.join("tests/assets/mini_hp_2025-09-01.json");
//!     let ontology: FullCsrOntology = loader.load_from_path(test_ontology).unwrap();
//!
//!     // Convert directly into a BitDag starting from a root node
//!     let dag = BitDag::from_graph(&ontology, "ROOT:0000000").unwrap();
//! ```
use crate::error::BitDagError;

/// A specialized `Result` type for `BitDag` operations.
///
/// This alias is used throughout the crate to standardize error handling, particularly
/// for failed graph traversals and unknown identifier lookups.
pub type Result<T> = std::result::Result<T, BitDagError>;

pub mod bitdag;
pub mod dag_edges;
pub mod edge;
pub mod error;
pub mod traits;
