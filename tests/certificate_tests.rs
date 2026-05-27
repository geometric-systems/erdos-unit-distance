use erdos_unit_distance::certificate::{
    CERTIFICATE_SCHEMA_VERSION, ClassicalCertificate, ConstructionCertificate,
    MultiquadraticCertificate,
};
use erdos_unit_distance::error::VerificationError;
use erdos_unit_distance::utils::find_unit_distance_edges;
use erdos_unit_distance::{CertifiedPointSet, UnitDistanceSet};
use serde_json::{Value, json};

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

fn json_value(certified: &CertifiedPointSet) -> Value {
    serde_json::from_str(&certified.to_json().unwrap()).unwrap()
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
    assert_eq!(
        certified.certificate.verify().unwrap_err().code(),
        "point_mismatch"
    );
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
    assert_eq!(certified.verify().unwrap_err().code(), "edge_set_mismatch");
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
    assert_eq!(certified.verify().unwrap_err().code(), "self_edge");
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
    assert_eq!(
        certified.certificate.verify().unwrap_err().code(),
        "invalid_construction"
    );
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
    assert_eq!(
        certified.certificate.verify().unwrap_err().code(),
        "quantized_projection_mismatch"
    );
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
fn golden_moser_spindle_certificate_json_shape_is_stable() {
    let certified = UnitDistanceSet::moser_spindle()
        .generate_certified(7)
        .unwrap();
    let value = json_value(&certified);

    assert_eq!(value["schema_version"], json!(CERTIFICATE_SCHEMA_VERSION));
    assert!(value["certified"]["certificate"]["Classical"].is_object());
    assert_eq!(
        value["certified"]["certificate"]["Classical"]["construction"],
        json!("MoserSpindle")
    );
    assert_eq!(value["certified"]["points"].as_array().unwrap().len(), 7);
    assert_eq!(
        value["certified"]["audit"]["edges"]
            .as_array()
            .unwrap()
            .len(),
        11
    );
}

#[test]
fn golden_square_grid_certificate_json_shape_is_stable() {
    let certified = UnitDistanceSet::square_grid(3, 3)
        .generate_certified(0)
        .unwrap();
    let value = json_value(&certified);

    assert_eq!(value["schema_version"], json!(CERTIFICATE_SCHEMA_VERSION));
    assert_eq!(
        value["certified"]["certificate"]["Classical"]["construction"],
        json!({ "SquareGrid": { "rows": 3, "cols": 3 } })
    );
    assert_eq!(value["certified"]["points"].as_array().unwrap().len(), 9);
    assert_eq!(
        value["certified"]["audit"]["edges"]
            .as_array()
            .unwrap()
            .len(),
        12
    );
}

#[test]
fn golden_multiquadratic_certificate_json_shape_is_stable() {
    let certified = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1)
        .unwrap()
        .generate_certified(20)
        .unwrap();
    let value = json_value(&certified);

    assert_eq!(value["schema_version"], json!(CERTIFICATE_SCHEMA_VERSION));
    let certificate = &value["certified"]["certificate"]["Multiquadratic"];
    assert_eq!(certificate["generators"], json!([5, 17, -1]));
    assert_eq!(certificate["split_prime"], json!(101));
    assert_eq!(certificate["k"], json!(1));
    assert_eq!(
        certificate["section2"]["backend"]["name"],
        json!("native-multiquadratic")
    );
    assert_eq!(certificate["section2"]["target_count"], json!(20));
    assert!(
        !certificate["section2"]["translations"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        !certificate["section2"]["construction_edges"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn missing_certificate_envelope_field_fails_with_code() {
    let certified = UnitDistanceSet::moser_spindle()
        .generate_certified(7)
        .unwrap();
    let mut value = json_value(&certified);
    value.as_object_mut().unwrap().remove("certified");
    let err = CertifiedPointSet::from_json(&value.to_string()).unwrap_err();

    assert_eq!(err.code(), "certificate_schema_mismatch");
}

#[test]
fn malformed_rational_certificate_fails_with_code() {
    let certified = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1)
        .unwrap()
        .generate_certified(20)
        .unwrap();
    let mut value = json_value(&certified);
    value["certified"]["certificate"]["Multiquadratic"]["section2"]["translations"][0]["element"]
        ["coeffs"][0] = json!("1/0");

    let decoded = CertifiedPointSet::from_json(&value.to_string()).unwrap();
    let err = decoded.verify().unwrap_err();

    assert_eq!(err.code(), "certificate_schema_mismatch");
}

#[test]
fn invalid_edge_provenance_fails_with_code() {
    let mut certified = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1)
        .unwrap()
        .generate_certified(20)
        .unwrap();
    algebraic_cert_mut(&mut certified)
        .section2
        .construction_edges[0]
        .sign = -1;
    let err = certified.certificate.verify().unwrap_err();

    assert_eq!(err.code(), "edge_provenance_mismatch");
}

#[test]
fn changed_projected_point_fails_with_code() {
    let mut certified = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1)
        .unwrap()
        .generate_certified(20)
        .unwrap();
    algebraic_cert_mut(&mut certified).projected_points[0].x += 0.25;
    let err = certified.certificate.verify().unwrap_err();

    assert_eq!(err.code(), "invalid_construction");
}

#[test]
fn changed_audit_edge_fails_with_code() {
    let mut certified = UnitDistanceSet::moser_spindle()
        .generate_certified(7)
        .unwrap();
    certified.audit.edges.push((0, 0));
    let err = certified.verify().unwrap_err();

    assert_eq!(err.code(), "self_edge");
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
    assert_eq!(
        CertifiedPointSet::from_json(&json).unwrap_err().code(),
        "certificate_schema_mismatch"
    );
}
