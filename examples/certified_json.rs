use erdos_unit_distance::{
    CERTIFICATE_SCHEMA_VERSION, CertifiedPointSet, MultiquadraticConfig, UnitDistanceSet,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let config = MultiquadraticConfig::builder(vec![5, 17], 101, 1).build()?;
    let certified = UnitDistanceSet::multiquadratic(config).generate_certified(20)?;
    let report = certified.verify()?;

    let json = certified.to_json()?;
    let decoded = CertifiedPointSet::from_json(&json)?;
    let decoded_report = decoded.verify()?;

    println!(
        "schema v{}: {} points, {} certified edges, {} audit edges",
        CERTIFICATE_SCHEMA_VERSION,
        decoded_report.point_count,
        report.certified_edge_count,
        report.audit_edge_count
    );
    println!("certificate JSON bytes: {}", json.len());
    Ok(())
}
