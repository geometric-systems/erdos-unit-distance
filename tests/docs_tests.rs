#[test]
fn readme_and_docs_name_unsupported_full_paper_components() {
    let readme = include_str!("../README.md").to_lowercase();
    let correctness = include_str!("../docs/correctness-contract.md").to_lowercase();
    let paper_map = include_str!("../docs/paper-map.md").to_lowercase();
    let combined = format!("{readme}\n{correctness}\n{paper_map}");

    [
        "class-group",
        "ideal arithmetic",
        "sage/pari",
        "golod-shafarevich",
        "class-field tower",
        "asymptotic theorem",
    ]
    .into_iter()
    .for_each(|needle| {
        assert!(
            combined.contains(needle),
            "public docs should mention unsupported component: {needle}"
        );
    });
}

#[test]
fn certificate_schema_doc_pins_schema_version_one() {
    let schema = include_str!("../docs/certificate-schema.md");
    assert!(schema.contains("schema version `1`"));
    assert!(schema.contains("CERTIFICATE_SCHEMA_VERSION"));
    assert!(schema.contains("certificate_schema_mismatch"));
    assert!(schema.contains("FloatingAuditReport"));
}
