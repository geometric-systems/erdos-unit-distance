//! Machine-checkable certificates for finite point-set outputs.

use crate::MultiquadraticConfig;
use crate::algebraic::section2::{Section2Certificate, Section2Output};
use crate::backend::BackendProvenance;
use crate::classical::{generate_moser_spindle, generate_square_grid, generate_triangular_grid};
use crate::error::{GenerationError, VerificationError};
use crate::numeric::rounded_f64_to_i64;
use crate::utils::find_unit_distance_edges;

const PROJECTION_KEY_SCALE: f64 = 1e7;
const POINT_MATCH_TOLERANCE: f64 = 1e-9;
pub const CERTIFICATE_SCHEMA_VERSION: u32 = 1;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct CertifiedPointSet {
    pub points: Vec<Point2>,
    pub certificate: ConstructionCertificate,
    pub audit: FloatingAuditReport,
}

impl CertifiedPointSet {
    pub fn new_classical(
        construction: ClassicalConstructionKind,
        requested_points: usize,
        points: Vec<[f64; 2]>,
        tolerance: f64,
    ) -> Self {
        let edges = find_unit_distance_edges(&points, tolerance);
        let audit = FloatingAuditReport { tolerance, edges };
        let points = points.into_iter().map(Point2::from).collect::<Vec<_>>();
        Self {
            certificate: ConstructionCertificate::Classical(ClassicalCertificate {
                construction,
                requested_points,
                points: points.clone(),
            }),
            audit,
            points,
        }
    }

    pub fn new_algebraic_from_section2(output: Section2Output) -> Result<Self, GenerationError> {
        let points = output
            .projected_points
            .iter()
            .copied()
            .map(Point2::from)
            .collect::<Vec<_>>();
        let quantized_projection_keys = points
            .iter()
            .copied()
            .map(Point2::to_array)
            .map(quantized_point)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| GenerationError::InvalidSearchParameter {
                parameter: "quantized_projection_keys",
                reason: "could not quantize projected point",
            })?;
        let audit = output.certificate.audit.clone();
        Ok(Self {
            certificate: ConstructionCertificate::Multiquadratic(Box::new(
                MultiquadraticCertificate {
                    generators: output.certificate.generators.clone(),
                    split_prime: output.certificate.split_prime,
                    k: output.certificate.k,
                    requested_points: output.certificate.target_count,
                    backend: output.certificate.backend.clone(),
                    projected_points: points.clone(),
                    quantized_projection_keys,
                    section2: output.certificate,
                },
            )),
            audit,
            points,
        })
    }

    pub fn verify(&self) -> Result<CertificateVerificationReport, VerificationError> {
        let report = self.certificate.verify()?;
        self.audit.verify_against(&point_arrays(&self.points))?;
        Ok(CertificateVerificationReport {
            audit_edge_count: self.audit.edges.len(),
            audit_extra_edge_count: self
                .audit
                .edges
                .len()
                .saturating_sub(report.certified_edge_count),
            tolerance: self.audit.tolerance,
            ..report
        })
    }

    pub fn to_json(&self) -> Result<String, VerificationError> {
        serde_json::to_string_pretty(&CertifiedPointSetEnvelope {
            schema_version: CERTIFICATE_SCHEMA_VERSION,
            certified: self,
        })
        .map_err(|err| VerificationError::CertificateSchemaMismatch {
            reason: err.to_string(),
        })
    }

    pub fn from_json(input: &str) -> Result<Self, VerificationError> {
        let envelope: OwnedCertifiedPointSetEnvelope =
            serde_json::from_str(input).map_err(|err| {
                VerificationError::CertificateSchemaMismatch {
                    reason: err.to_string(),
                }
            })?;
        if envelope.schema_version != CERTIFICATE_SCHEMA_VERSION {
            return Err(VerificationError::CertificateSchemaMismatch {
                reason: format!(
                    "expected schema version {}, got {}",
                    CERTIFICATE_SCHEMA_VERSION, envelope.schema_version
                ),
            });
        }
        Ok(envelope.certified)
    }
}

#[derive(serde::Serialize)]
struct CertifiedPointSetEnvelope<'a> {
    schema_version: u32,
    certified: &'a CertifiedPointSet,
}

#[derive(serde::Deserialize)]
struct OwnedCertifiedPointSetEnvelope {
    schema_version: u32,
    certified: CertifiedPointSet,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl From<[f64; 2]> for Point2 {
    fn from(point: [f64; 2]) -> Self {
        Self {
            x: point[0],
            y: point[1],
        }
    }
}

impl Point2 {
    pub fn to_array(self) -> [f64; 2] {
        [self.x, self.y]
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum ConstructionCertificate {
    Classical(ClassicalCertificate),
    Multiquadratic(Box<MultiquadraticCertificate>),
}

impl ConstructionCertificate {
    pub fn verify(&self) -> Result<CertificateVerificationReport, VerificationError> {
        match self {
            ConstructionCertificate::Classical(certificate) => certificate.verify(),
            ConstructionCertificate::Multiquadratic(certificate) => certificate.verify(),
        }
    }

    pub fn certified_edge_count(&self) -> usize {
        match self {
            ConstructionCertificate::Classical(certificate) => {
                find_unit_distance_edges(&point_arrays(&certificate.points), 1e-4).len()
            }
            ConstructionCertificate::Multiquadratic(certificate) => {
                certificate.section2.construction_edges.len()
            }
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ClassicalCertificate {
    pub construction: ClassicalConstructionKind,
    pub requested_points: usize,
    pub points: Vec<Point2>,
}

impl ClassicalCertificate {
    pub fn verify(&self) -> Result<CertificateVerificationReport, VerificationError> {
        let expected_points = match self.construction {
            ClassicalConstructionKind::SquareGrid { rows, cols } => {
                generate_square_grid(rows, cols)
            }
            ClassicalConstructionKind::TriangularGrid => {
                generate_triangular_grid(self.requested_points)
            }
            ClassicalConstructionKind::MoserSpindle => generate_moser_spindle(),
        };

        let actual_points = point_arrays(&self.points);
        verify_point_count(expected_points.len(), actual_points.len())?;
        verify_points_match(&expected_points, &actual_points)?;
        let certified_edges = find_unit_distance_edges(&actual_points, 1e-4);

        Ok(CertificateVerificationReport {
            construction: self.construction.label(),
            point_count: self.points.len(),
            edge_count: certified_edges.len(),
            certified_edge_count: certified_edges.len(),
            audit_edge_count: 0,
            audit_extra_edge_count: 0,
            tolerance: 0.0,
            backend: None,
        })
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClassicalConstructionKind {
    SquareGrid { rows: usize, cols: usize },
    TriangularGrid,
    MoserSpindle,
}

impl ClassicalConstructionKind {
    pub fn label(&self) -> String {
        match self {
            ClassicalConstructionKind::SquareGrid { rows, cols } => {
                format!("square-grid({rows}x{cols})")
            }
            ClassicalConstructionKind::TriangularGrid => "triangular-grid".to_string(),
            ClassicalConstructionKind::MoserSpindle => "moser-spindle".to_string(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct MultiquadraticCertificate {
    pub generators: Vec<i64>,
    pub split_prime: i64,
    pub k: usize,
    pub requested_points: usize,
    pub backend: BackendProvenance,
    pub projected_points: Vec<Point2>,
    pub quantized_projection_keys: Vec<[i64; 2]>,
    pub section2: Section2Certificate,
}

impl MultiquadraticCertificate {
    pub fn verify(&self) -> Result<CertificateVerificationReport, VerificationError> {
        MultiquadraticConfig::new(
            self.generators_without_auto_imaginary(),
            self.split_prime,
            self.k,
        )
        .map_err(|err| VerificationError::InvalidConstruction {
            reason: err.to_string(),
        })?;
        self.section2.verify()?;
        if self
            .section2
            .projected_points()
            .into_iter()
            .map(Point2::from)
            .collect::<Vec<_>>()
            != self.projected_points
        {
            return Err(VerificationError::InvalidConstruction {
                reason: "algebraic certificate points do not match Section 2 certificate"
                    .to_string(),
            });
        }
        verify_point_count(self.requested_points, self.projected_points.len())?;
        verify_point_count(
            self.projected_points.len(),
            self.quantized_projection_keys.len(),
        )?;
        self.projected_points
            .iter()
            .copied()
            .map(Point2::to_array)
            .map(quantized_point)
            .zip(&self.quantized_projection_keys)
            .enumerate()
            .try_for_each(|(index, (expected, &actual))| {
                let expected = expected?;
                if expected == actual {
                    Ok(())
                } else {
                    Err(VerificationError::QuantizedProjectionMismatch {
                        index,
                        expected,
                        actual,
                    })
                }
            })?;
        let certified_edge_count = self.section2.construction_edges.len();

        Ok(CertificateVerificationReport {
            construction: "multiquadratic-section2".to_string(),
            point_count: self.projected_points.len(),
            edge_count: certified_edge_count,
            certified_edge_count,
            audit_edge_count: 0,
            audit_extra_edge_count: 0,
            tolerance: 0.0,
            backend: Some(self.backend.clone()),
        })
    }

    fn generators_without_auto_imaginary(&self) -> Vec<i64> {
        self.generators
            .iter()
            .copied()
            .filter(|&generator| generator != -1)
            .collect()
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct FloatingAuditReport {
    /// Absolute distance tolerance used when auditing unit edges in floating output.
    pub tolerance: f64,
    pub edges: Vec<(usize, usize)>,
}

impl FloatingAuditReport {
    pub fn verify_against(&self, points: &[[f64; 2]]) -> Result<(), VerificationError> {
        verify_unit_distance_certificate(points, self)
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct CertificateVerificationReport {
    pub construction: String,
    pub point_count: usize,
    /// Backward-compatible edge count. For algebraic certificates this is the certified edge count.
    pub edge_count: usize,
    pub certified_edge_count: usize,
    pub audit_edge_count: usize,
    pub audit_extra_edge_count: usize,
    pub tolerance: f64,
    pub backend: Option<BackendProvenance>,
}

fn verify_point_count(expected: usize, actual: usize) -> Result<(), VerificationError> {
    if expected == actual {
        Ok(())
    } else {
        Err(VerificationError::PointCountMismatch { expected, actual })
    }
}

fn verify_points_match(
    expected: &[[f64; 2]],
    actual: &[[f64; 2]],
) -> Result<(), VerificationError> {
    expected
        .iter()
        .zip(actual)
        .enumerate()
        .try_for_each(|(index, (&expected, &actual))| {
            let dx = (expected[0] - actual[0]).abs();
            let dy = (expected[1] - actual[1]).abs();
            if dx <= POINT_MATCH_TOLERANCE && dy <= POINT_MATCH_TOLERANCE {
                Ok(())
            } else {
                Err(VerificationError::PointMismatch { index })
            }
        })
}

fn verify_unit_distance_certificate(
    points: &[[f64; 2]],
    certificate: &FloatingAuditReport,
) -> Result<(), VerificationError> {
    certificate.edges.iter().try_for_each(|&edge| {
        if edge.0 == edge.1 {
            Err(VerificationError::SelfEdge { index: edge.0 })
        } else if edge.0 >= points.len() || edge.1 >= points.len() {
            Err(VerificationError::EdgeIndexOutOfBounds {
                edge,
                point_count: points.len(),
            })
        } else {
            Ok(())
        }
    })?;

    let mut expected = find_unit_distance_edges(points, certificate.tolerance);
    let mut actual = certificate.edges.clone();
    expected.sort_unstable();
    actual.sort_unstable();
    if expected == actual {
        Ok(())
    } else {
        Err(VerificationError::EdgeSetMismatch { expected, actual })
    }
}

fn quantized_point(point: [f64; 2]) -> Result<[i64; 2], VerificationError> {
    Ok([
        rounded_f64_to_i64(point[0] * PROJECTION_KEY_SCALE)?,
        rounded_f64_to_i64(point[1] * PROJECTION_KEY_SCALE)?,
    ])
}

fn point_arrays(points: &[Point2]) -> Vec<[f64; 2]> {
    points.iter().copied().map(Point2::to_array).collect()
}
