#[cfg(feature = "json_ontology")]
mod json_tests {
    use bitdag::bitdag::BitDag;
    use ontolius::io::OntologyLoaderBuilder;
    use ontolius::ontology::csr::FullCsrOntology;
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::sync::OnceLock;

    static SHARED_BIT_DAG: OnceLock<BitDag> = OnceLock::new();

    fn get_shared_dag() -> &'static BitDag {
        SHARED_BIT_DAG.get_or_init(|| {
            let manifest_dir = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
            let test_ontology = manifest_dir.join("tests/assets/mini_hp_2025-09-01.json");
            let loader = OntologyLoaderBuilder::new().obographs_parser().build();
            let ontolius: FullCsrOntology = loader.load_from_path(test_ontology).unwrap();

            BitDag::from_graph(&ontolius, "HP:0000118").unwrap()
        })
    }

    #[test]
    fn test_is_descendant() {
        let bit_dag = get_shared_dag();

        assert_eq!(
            bit_dag.is_descendant_of("HP:0000738", "HP:5200423"),
            Ok(true)
        );
        assert_eq!(
            bit_dag.is_descendant_of("HP:5200423", "HP:0000738"),
            Ok(false)
        );
    }

    #[test]
    fn test_test_is_ancestor() {
        let bit_dag = get_shared_dag();

        assert_eq!(bit_dag.is_ancestor_of("HP:5200423", "HP:0000738"), Ok(true));
        assert_eq!(
            bit_dag.is_ancestor_of("HP:0000738", "HP:5200423"),
            Ok(false)
        );
    }

    #[test]
    fn test_get_immediate_children() {
        let bit_dag = get_shared_dag();

        let children = bit_dag.get_immediate_children("HP:0011446"); // Abnormality of mental function
        assert!(children.is_ok());

        assert!(children.unwrap().contains(&"HP:5200423")); // Only one child because of mini hpo
    }

    #[test]
    fn test_get_immediate_parents() {
        let bit_dag = get_shared_dag();

        let parents = bit_dag.get_immediate_parents("HP:0000738");
        assert!(parents.is_ok());
    }

    #[test]
    fn test_get_n_deep_children() {
        let bit_dag = get_shared_dag();

        assert_eq!(
            bit_dag.get_n_deep_children("HP:5200423", 0),
            Ok(vec!["HP:5200423"])
        );

        let immediate = bit_dag.get_immediate_children("HP:5200423").unwrap();
        let deep_1 = bit_dag.get_n_deep_children("HP:5200423", 1).unwrap();
        assert_eq!(immediate, deep_1);
    }

    #[test]
    fn test_get_n_deep_parents() {
        let bit_dag = get_shared_dag();

        assert_eq!(
            bit_dag.get_n_deep_parents("HP:0000738", 0),
            Ok(vec!["HP:0000738"])
        );

        let immediate = bit_dag.get_immediate_parents("HP:0000738").unwrap();
        let deep_1 = bit_dag.get_n_deep_parents("HP:0000738", 1).unwrap();
        assert_eq!(immediate, deep_1);
    }

    #[test]
    fn test_get_descendants() {
        let bit_dag = get_shared_dag();
        let descendants = bit_dag.get_descendants("HP:5200423").unwrap();

        assert!(descendants.contains(&"HP:0000738"));
    }

    #[test]
    fn test_get_ancestors() {
        let bit_dag = get_shared_dag();
        let ancestors = bit_dag.get_ancestors("HP:0000738").unwrap();

        assert!(ancestors.contains(&"HP:5200423"));
    }

    #[test]
    fn test_get_common_descendants() {
        let bit_dag = get_shared_dag();

        let common_desc = bit_dag
            .get_common_descendants("HP:5200423", "HP:5200423")
            .unwrap();
        let descendants = bit_dag.get_descendants("HP:5200423").unwrap();

        assert_eq!(common_desc, descendants);
    }

    #[test]
    fn test_minimize_profile() {
        let bit_dag = get_shared_dag();

        let profile = vec!["HP:5200423", "HP:0000738"];
        let minimized = bit_dag.minimize_profile(&profile).unwrap();

        assert_eq!(minimized, vec!["HP:0000738"]);
    }

    #[test]
    fn test_get_leaves() {
        let bit_dag = get_shared_dag();
        let leaves = bit_dag.get_leaves();

        assert!(!leaves.is_empty());

        assert!(!leaves.contains(&"HP:5200423"));
    }

    #[test]
    fn test_unknown_id_errors() {
        let bit_dag = get_shared_dag();
        let invalid_id = "HP:NOT_REAL";

        assert!(bit_dag.get_immediate_children(invalid_id).is_err());
        assert!(bit_dag.get_immediate_parents(invalid_id).is_err());
        assert!(bit_dag.get_n_deep_children(invalid_id, 2).is_err());
        assert!(bit_dag.get_n_deep_parents(invalid_id, 2).is_err());
        assert!(bit_dag.get_descendants(invalid_id).is_err());
        assert!(bit_dag.get_ancestors(invalid_id).is_err());
        assert!(bit_dag.is_descendant_of(invalid_id, "HP:5200423").is_err());
        assert!(bit_dag.is_ancestor_of(invalid_id, "HP:5200423").is_err());
        assert!(
            bit_dag
                .get_common_descendants(invalid_id, "HP:5200423")
                .is_err()
        );
        assert!(bit_dag.minimize_profile(&[invalid_id]).is_err());
    }
}
