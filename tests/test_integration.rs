use bitdag::bitdag::BitDag;
use bitdag::traits::ToEdges;
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
