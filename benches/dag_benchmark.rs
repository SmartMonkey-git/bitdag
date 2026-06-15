#![cfg(feature = "json_ontology")]
use bitdag::bitdag::BitDag;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ontolius::TermId;
use ontolius::io::OntologyLoaderBuilder;
use ontolius::ontology::HierarchyWalks;
use ontolius::ontology::csr::FullCsrOntology;
use ontology_registry::{
    BioRegistryMetadataProvider, FileSystemOntologyRegistry, FileType, OboLibraryProvider,
    OntologyRegistration, RegistryKey, SupportedOntology, Version,
};
use std::path::PathBuf;
use std::str::FromStr;

static ROOT_NODE: &str = "HP:0000118";
static TARGET_NODE: &str = "HP:0032708";
fn load_test_data() -> FullCsrOntology {
    let manifest_dir = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    let registry_dir = manifest_dir.join("tests/assets/");
    let registry = FileSystemOntologyRegistry::new(
        registry_dir,
        BioRegistryMetadataProvider::default(),
        OboLibraryProvider::default(),
    );

    let reg_key = RegistryKey::new(SupportedOntology::HP, Version::Latest, FileType::Json);

    let reader = registry.register(reg_key).unwrap();

    OntologyLoaderBuilder::new()
        .obographs_parser()
        .build()
        .load_from_read(reader)
        .expect("Failed to load test ontology")
}

fn bench_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("Descendant Queries");

    let ontology = load_test_data();
    let target_node_ontolius = TermId::from_str(TARGET_NODE).unwrap();

    let bitdag = BitDag::from_graph(&ontology, ROOT_NODE).unwrap();

    group.bench_function("BitDag::get_descendants", |b| {
        b.iter(|| {
            let descendants = bitdag.get_ancestors(black_box(TARGET_NODE)).unwrap();
            black_box(descendants);
        })
    });

    group.bench_function("Ontolius::native_descendants", |b| {
        b.iter(|| {
            let mut all_descendants = Vec::new();
            for child in ontology.iter_ancestor_ids(black_box(&target_node_ontolius)) {
                all_descendants.push(child);
            }
            black_box(all_descendants);
        })
    });

    group.finish();
}

fn bench_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("Graph Construction");
    let ontology = load_test_data();

    group.bench_function("BitDag::from_graph", |b| {
        b.iter(|| {
            let dag = BitDag::from_graph(black_box(&ontology), black_box(ROOT_NODE)).unwrap();
            black_box(dag);
        })
    });

    group.finish();
}

criterion_group!(benches, bench_queries, bench_construction);
criterion_main!(benches);
