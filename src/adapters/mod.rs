//! Format-specific graph adapters and parsers.
//!
//! This module contains implementations of the [`ToEdges`](crate::traits::ToEdges) trait
//! for various standard ontology and graph representation formats. By leveraging the
//! parsers in these sub-modules, you can seamlessly convert external documents into a
//! flat vector of edges that the `BitDag` can ingest.

/// Adapter implementations for JSON-based ontology formats (via the `ontolius` crate).
#[cfg(feature = "json_ontology")]
pub mod json;

/// Adapter implementations for OBO-formatted ontology documents (via the `fastobo` crate).
#[cfg(feature = "obo")]
pub mod obo;
