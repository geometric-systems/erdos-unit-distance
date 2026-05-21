use erdos_unit_distance::UnitDistanceSet;
use erdos_unit_distance::algebraic::field::MultiQuadraticField;
use erdos_unit_distance::algebraic::section2::AlgebraicElementKey;
use erdos_unit_distance::algebraic::section2::Section2Certificate;
use erdos_unit_distance::certificate::ConstructionCertificate;
use erdos_unit_distance::error::VerificationError;
use serde_json::Value;

fn algebraic_section2(certified: &erdos_unit_distance::CertifiedPointSet) -> &Section2Certificate {
    match &certified.certificate {
        ConstructionCertificate::Multiquadratic(certificate) => &certificate.section2,
        _ => panic!("expected algebraic certificate"),
    }
}

fn algebraic_section2_mut(
    certified: &mut erdos_unit_distance::CertifiedPointSet,
) -> &mut Section2Certificate {
    match &mut certified.certificate {
        ConstructionCertificate::Multiquadratic(certificate) => &mut certificate.section2,
        _ => panic!("expected algebraic certificate"),
    }
}

#[test]
fn exact_algebraic_keys_multiply_and_conjugate_without_floats() {
    let field = MultiQuadraticField::try_new(&[5, -1]).unwrap();
    let sqrt5 = AlgebraicElementKey {
        coeffs: vec!["0".into(), "1".into(), "0".into(), "0".into()],
    };
    let i = AlgebraicElementKey {
        coeffs: vec!["0".into(), "0".into(), "1".into(), "0".into()],
    };
    let one = AlgebraicElementKey::one(&field);

    assert_eq!(
        sqrt5.mul(&sqrt5, &field).unwrap(),
        AlgebraicElementKey {
            coeffs: vec!["5".into(), "0".into(), "0".into(), "0".into()],
        }
    );
    assert_eq!(i.mul(&i, &field).unwrap().coeffs, vec!["-1", "0", "0", "0"]);
    assert_eq!(
        i.mul(&i.complex_conjugate(&field).unwrap(), &field)
            .unwrap(),
        one
    );
}

#[test]
fn paper_map_tracks_required_proof_components() {
    let paper_map = include_str!("../docs/paper-map.md");
    [
        "norm-one",
        "Minkowski embedding",
        "Polydisk",
        "Projection",
        "split-prime",
        "class groups",
        "tower",
    ]
    .into_iter()
    .for_each(|needle| {
        assert!(
            paper_map.to_lowercase().contains(&needle.to_lowercase()),
            "paper map should mention {needle}"
        );
    });
}

#[test]
fn multiquadratic_constructor_is_deterministic() {
    let multiquadratic = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1)
        .unwrap()
        .generate_certified(20)
        .unwrap();
    let algebraic = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1)
        .unwrap()
        .generate_certified(20)
        .unwrap();

    assert_eq!(multiquadratic.points, algebraic.points);
    assert_eq!(
        multiquadratic.audit.edges.len(),
        algebraic.audit.edges.len()
    );
}

#[test]
fn section2_fixture_verifies() {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/section2_multiquadratic_small.json")).unwrap();
    let field = &fixture["field"];
    let section2 = &fixture["section2"];
    let generators = field["generators"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_i64().unwrap())
        .collect::<Vec<_>>();
    let split_prime = field["split_prime"].as_i64().unwrap();
    let k = usize::try_from(field["k"].as_u64().unwrap()).unwrap();
    let target_count = usize::try_from(section2["target_count"].as_u64().unwrap()).unwrap();
    let expected_audit_edge_count =
        usize::try_from(section2["expected_edge_count"].as_u64().unwrap()).unwrap();

    let certified = UnitDistanceSet::try_new_multiquadratic(generators, split_prime, k)
        .unwrap()
        .generate_certified(target_count)
        .unwrap();
    let report = certified.verify().unwrap();
    let section2_certificate = algebraic_section2(&certified);

    assert_eq!(report.point_count, target_count);
    assert_eq!(report.audit_edge_count, expected_audit_edge_count);
    assert_eq!(
        report.certified_edge_count,
        section2_certificate.construction_edges.len()
    );
    assert!(report.audit_edge_count >= report.certified_edge_count);
    assert_eq!(section2_certificate.candidates.len(), target_count);
    assert!(!section2_certificate.translations.is_empty());
    assert!(!section2_certificate.construction_edges.is_empty());
    assert_eq!(
        section2_certificate.projection_index,
        usize::try_from(section2["projection_index"].as_u64().unwrap()).unwrap()
    );
}

#[test]
fn tampered_section2_candidate_path_fails() {
    let mut certified = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1)
        .unwrap()
        .generate_certified(20)
        .unwrap();
    algebraic_section2_mut(&mut certified).candidates[1]
        .path
        .push(
            erdos_unit_distance::algebraic::section2::SignedTranslationStep {
                translation_index: 0,
                sign: 1,
            },
        );

    assert!(matches!(
        certified.certificate.verify(),
        Err(VerificationError::CandidatePathMismatch { index: 1 })
            | Err(VerificationError::InvalidConstruction { .. })
    ));
}

#[test]
fn tampered_section2_edge_provenance_fails() {
    let mut certified = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1)
        .unwrap()
        .generate_certified(20)
        .unwrap();
    algebraic_section2_mut(&mut certified).construction_edges[0].sign *= -1;

    assert!(matches!(
        certified.certificate.verify(),
        Err(VerificationError::EdgeProvenanceMismatch { edge_index: 0 })
            | Err(VerificationError::InvalidConstruction { .. })
    ));
}

#[test]
fn tampered_section2_norm_one_evidence_fails() {
    let mut certified = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1)
        .unwrap()
        .generate_certified(20)
        .unwrap();
    algebraic_section2_mut(&mut certified).translations[0]
        .norm
        .coeffs[0] = "2".to_string();

    assert!(matches!(
        certified.certificate.verify(),
        Err(VerificationError::TranslationNotNormOne { index: 0 })
            | Err(VerificationError::InvalidConstruction { .. })
    ));
}

#[test]
fn tampered_section2_translation_rational_fails() {
    let mut certified = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1)
        .unwrap()
        .generate_certified(20)
        .unwrap();
    algebraic_section2_mut(&mut certified).translations[0]
        .element
        .coeffs[0] = "1/0".to_string();

    assert!(matches!(
        certified.certificate.verify(),
        Err(VerificationError::CertificateSchemaMismatch { .. })
            | Err(VerificationError::InvalidConstruction { .. })
    ));
}

#[test]
fn tampered_section2_candidate_key_fails() {
    let mut certified = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1)
        .unwrap()
        .generate_certified(20)
        .unwrap();
    algebraic_section2_mut(&mut certified).candidates[0]
        .element
        .coeffs[0] = "1".to_string();

    assert!(matches!(
        certified.certificate.verify(),
        Err(VerificationError::ProjectionMismatch { index: 0 })
            | Err(VerificationError::CandidatePathMismatch { index: 0 })
            | Err(VerificationError::CandidateOutsideWindow { .. })
            | Err(VerificationError::EdgeSetMismatch { .. })
            | Err(VerificationError::InvalidConstruction { .. })
    ));
}

#[test]
fn tampered_section2_window_membership_fails() {
    let mut certified = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1)
        .unwrap()
        .generate_certified(20)
        .unwrap();
    algebraic_section2_mut(&mut certified).window_radius = 0.01;

    assert!(matches!(
        certified.certificate.verify(),
        Err(VerificationError::CandidateOutsideWindow { .. })
            | Err(VerificationError::InvalidConstruction { .. })
    ));
}

#[test]
fn tampered_section2_projection_fails() {
    let mut certified = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1)
        .unwrap()
        .generate_certified(20)
        .unwrap();
    algebraic_section2_mut(&mut certified).candidates[0].projection[0] += 0.25;

    assert!(matches!(
        certified.certificate.verify(),
        Err(VerificationError::ProjectionMismatch { index: 0 })
            | Err(VerificationError::InvalidConstruction { .. })
    ));
}
