use erdos_unit_distance::certificate::{
    ClassicalCertificate, ConstructionCertificate, MultiquadraticCertificate,
};
use erdos_unit_distance::error::VerificationError;
use erdos_unit_distance::utils::find_unit_distance_edges;
use erdos_unit_distance::{CertifiedPointSet, UnitDistanceSet};

fn arrays(points: &[erdos_unit_distance::Point2]) -> Vec<[f64; 2]> {
    points
        .iter()
        .copied()
        .map(|point| point.to_array())
        .collect()
}

fn algebraic_cert_mut(certified: &mut CertifiedPointSet) -> &mut MultiquadraticCertificate {
    match &mut certified.certificate {
        ConstructionCertificate::Multiquadratic(certificate) => certificate,
        _ => panic!("expected algebraic certificate"),
    }
}

fn classical_cert_mut(certified: &mut CertifiedPointSet) -> &mut ClassicalCertificate {
    match &mut certified.certificate {
        ConstructionCertificate::Classical(certificate) => certificate,
        _ => panic!("expected classical certificate"),
    }
}

#[test]
fn certified_moser_spindle_verifies_exact_edge_count() {
    let certified = UnitDistanceSet::moser_spindle()
        .generate_certified(7)
        .unwrap();
    let report = certified.verify().unwrap();

    assert_eq!(certified.points.len(), 7);
    assert_eq!(report.point_count, 7);
    assert_eq!(report.edge_count, 11);
    assert_eq!(report.certified_edge_count, 11);
    assert_eq!(report.audit_edge_count, 11);
    assert_eq!(
        certified.audit.edges,
        find_unit_distance_edges(&arrays(&certified.points), 1e-4)
    );
}

#[test]
fn certified_square_grid_verifies_known_edge_count() {
    let certified = UnitDistanceSet::square_grid(3, 3)
        .generate_certified(0)
        .unwrap();
    let report = certified.verify().unwrap();

    assert_eq!(report.point_count, 9);
    assert_eq!(report.edge_count, 12);
    assert_eq!(report.certified_edge_count, 12);
    assert_eq!(report.audit_edge_count, 12);
}

#[test]
fn certified_algebraic_output_verifies_against_independent_edge_checker() {
    let certified = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1)
        .unwrap()
        .generate_certified(20)
        .unwrap();
    let report = certified.verify().unwrap();

    assert_eq!(report.point_count, 20);
    assert!(report.certified_edge_count > 0);
    assert!(report.audit_edge_count >= report.certified_edge_count);
    assert_eq!(
        certified.audit.edges,
        find_unit_distance_edges(&arrays(&certified.points), 1e-4)
    );
}

#[test]
fn tampered_classical_point_fails_verification() {
    let mut certified = UnitDistanceSet::moser_spindle()
        .generate_certified(7)
        .unwrap();
    classical_cert_mut(&mut certified).points[0].x += 0.25;

    assert!(matches!(
        certified.certificate.verify(),
        Err(VerificationError::PointMismatch { index: 0 })
    ));
}

#[test]
fn missing_certificate_edge_fails_verification() {
    let mut certified = UnitDistanceSet::moser_spindle()
        .generate_certified(7)
        .unwrap();
    certified.audit.edges.pop();

    assert!(matches!(
        certified.verify(),
        Err(VerificationError::EdgeSetMismatch { .. })
    ));
}

#[test]
fn invalid_certificate_edge_fails_verification() {
    let mut certified = UnitDistanceSet::moser_spindle()
        .generate_certified(7)
        .unwrap();
    certified.audit.edges.push((0, 0));

    assert!(matches!(
        certified.verify(),
        Err(VerificationError::SelfEdge { index: 0 })
    ));
}

#[test]
fn wrong_algebraic_split_prime_fails_verification() {
    let mut certified = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1)
        .unwrap()
        .generate_certified(20)
        .unwrap();
    algebraic_cert_mut(&mut certified).split_prime = 2;

    assert!(matches!(
        certified.certificate.verify(),
        Err(VerificationError::InvalidConstruction { .. })
    ));
}

#[test]
fn wrong_algebraic_projection_key_fails_verification() {
    let mut certified = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1)
        .unwrap()
        .generate_certified(20)
        .unwrap();
    algebraic_cert_mut(&mut certified).quantized_projection_keys[0][0] += 1;

    assert!(matches!(
        certified.certificate.verify(),
        Err(VerificationError::QuantizedProjectionMismatch { index: 0, .. })
    ));
}

#[test]
fn certified_point_set_json_round_trips_and_verifies() {
    let certified = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1)
        .unwrap()
        .generate_certified(20)
        .unwrap();

    let json = certified.to_json().unwrap();
    let decoded = CertifiedPointSet::from_json(&json).unwrap();
    let report = decoded.verify().unwrap();

    assert_eq!(report.point_count, 20);
    assert!(json.contains("\"schema_version\""));
    assert!(json.contains("\"certified\""));
}

#[test]
fn wrong_certificate_schema_version_fails() {
    let certified = UnitDistanceSet::moser_spindle()
        .generate_certified(7)
        .unwrap();
    let json = certified.to_json().unwrap().replacen(
        "\"schema_version\": 1",
        "\"schema_version\": 999",
        1,
    );

    assert!(matches!(
        CertifiedPointSet::from_json(&json),
        Err(VerificationError::CertificateSchemaMismatch { .. })
    ));
}
