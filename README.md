[![Crates.io](https://img.shields.io/crates/v/bitdag.svg)](https://crates.io/crates/bitdag)
[![Docs.rs](https://docs.rs/bitdag/badge.svg)](https://docs.rs/bitdag)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![RobisonGroup](https://img.shields.io/badge/Robinson%20Group-blue)](https://robinsongroup.github.io/)

# BitDag

A fast, memory-efficient Directed Acyclic Graph (DAG) representation in Rust, optimized for lightning-fast relationship
queries using dense bit matrices.

By computing the transitive closure of the graph upon construction, `BitDag` achieves $O(1)$ time complexity for
immediate ancestry or descendant checks. For bulk operations—such as finding common descendants or extracting leaf
nodes—it leverages `rayon` to perform highly parallelized bitwise operations across the matrix.

---

## Key Features

* **$O(1)$ Relationship Checks:** Instant `is_ancestor_of` and `is_descendant_of` lookups.
* **Parallelized Bulk Queries:** Uses `rayon` to execute bitwise `AND`/`OR` operations across rows and columns for rapid
  descendant/ancestor extraction.
* **Profile Minimization:** Easily filter out redundant ancestor terms from a profile, leaving only the most specific (
  deepest) nodes.
* **Format Adapters:** Built-in support for parsing standard ontology formats (OBO and JSON) directly into a bit matrix.
* **Flexible Serialization:** Optional support for both `serde` and `miniserde`.

---

## Feature Flags

This crate is highly modular. You can opt-in to specific dependencies to keep your binary size and compilation times as
low as possible.

- serde — Implements standard serialization and deserialization for core structures.
- miniserde — Implements lightweight serialization for minimal binary overhead.
- obo — Enables the OBO format adapter to ingest .obo files (requires fastobo).
- json_ontology — Enables the JSON ontology adapter to ingest JSON-formatted ontologies (requires ontolius).

---

## Quick Start

Here is a basic example of building a BitDag from a raw list of edges and querying relationships.
use bitdag::edge::Edge;
use bitdag::bitdag::BitDag;

```rust
fn main() {
    // 1. Define your edges (Parent -> Child)
    let edges: Vec<Edge> = vec![
        ("A", "B").into(),
        ("B", "C").into(),
        ("C", "D").into(),
        ("C", "F").into(),
    ];

    // 2. Build the BitDag (computes transitive closure automatically)
    let dag = BitDag::from_edges(&edges);

    // 3. Perform O(1) checks
    assert_eq!(dag.is_ancestor_of("A", "D").unwrap(), true);
    assert_eq!(dag.is_descendant_of("A", "C").unwrap(), false);

    // 4. Extract generations or sub-graphs
    let descendants = dag.get_descendants("A").unwrap();
    println!("Descendants of A: {:?}", descendants); // ["B", "C", "D", "F"]

    let leaves = dag.get_leaves();
    println!("Leaves in the graph: {:?}", leaves); // ["D", "F"]

}
```

## Using Format Adapters

If you are working with external ontologies, you can use the built-in adapters (ensure you have the correct feature
flags enabled).

```rust
fn main() {
    // Requires the `json_ontology` feature
    use ontolius::io::OntologyLoaderBuilder;
    use ontolius::ontology::csr::FullCsrOntology;
    use bitdag::traits::ToEdges;
    use bitdag::bitdag::BitDag;

    let loader = OntologyLoaderBuilder::new().obographs_parser().build();
    let ontology: FullCsrOntology = loader.load_from_path("path/to/ontology.json").unwrap();

    // Convert directly into a BitDag starting from a root node
    let dag = BitDag::from_graph(&ontology, "ROOT:0000000").unwrap();
}
```
