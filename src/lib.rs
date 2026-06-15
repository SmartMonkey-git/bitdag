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
