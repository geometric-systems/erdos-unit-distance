use std::fmt;

/// Errors that can occur during point set generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerationError {
    /// A field generator is unsupported or violates the squarefree generator contract.
    InvalidGenerator {
        generator: i64,
        reason: &'static str,
    },
    /// The same generator was provided more than once.
    DuplicateGenerator { generator: i64 },
    /// The split prime is not an odd prime.
    InvalidSplitPrime { split_prime: i64 },
    /// The requested prime is not split completely in the configured field.
    PrimeNotSplit { split_prime: i64, generator: i64 },
    /// The coefficient vector length does not match the field degree.
    InvalidFieldElementDimension { expected: usize, actual: usize },
    /// A generation/search parameter is outside its supported range.
    InvalidSearchParameter {
        parameter: &'static str,
        reason: &'static str,
    },
    /// No suitable prime element theta found for the given field and prime.
    PrimeElementNotFound { prime: i64, max_exponent: usize },
    /// The field has no imaginary generator (-1), but the construction requires one.
    MissingImaginaryGenerator,
    /// Could not generate enough points within the radius expansion budget.
    InsufficientPoints { requested: usize, found: usize },
}

impl fmt::Display for GenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenerationError::InvalidGenerator { generator, reason } => {
                write!(f, "invalid generator {generator}: {reason}")
            }
            GenerationError::DuplicateGenerator { generator } => {
                write!(f, "duplicate generator {generator}")
            }
            GenerationError::InvalidSplitPrime { split_prime } => {
                write!(f, "split prime {split_prime} is not an odd prime")
            }
            GenerationError::PrimeNotSplit {
                split_prime,
                generator,
            } => write!(
                f,
                "prime {split_prime} is not split by generator {generator}"
            ),
            GenerationError::InvalidFieldElementDimension { expected, actual } => write!(
                f,
                "field element has {actual} coefficients, but field degree is {expected}"
            ),
            GenerationError::InvalidSearchParameter { parameter, reason } => {
                write!(f, "invalid search parameter {parameter}: {reason}")
            }
            GenerationError::PrimeElementNotFound {
                prime,
                max_exponent,
            } => write!(
                f,
                "could not find a suitable prime element theta for p = {prime} up to exponent {max_exponent}"
            ),
            GenerationError::MissingImaginaryGenerator => write!(
                f,
                "field is missing the imaginary generator -1 required by the construction"
            ),
            GenerationError::InsufficientPoints { requested, found } => write!(
                f,
                "could not generate enough points: requested {requested}, found {found}"
            ),
        }
    }
}

impl std::error::Error for GenerationError {}

/// Errors that can occur while verifying a construction certificate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationError {
    /// The certificate stores a different number of points than the construction claims.
    PointCountMismatch { expected: usize, actual: usize },
    /// A claimed unit-distance edge refers to a point index outside the point set.
    EdgeIndexOutOfBounds {
        edge: (usize, usize),
        point_count: usize,
    },
    /// A claimed unit-distance edge connects a point to itself.
    SelfEdge { index: usize },
    /// The claimed edge set does not match the independently recomputed edge set.
    EdgeSetMismatch {
        expected: Vec<(usize, usize)>,
        actual: Vec<(usize, usize)>,
    },
    /// A stored point does not match the deterministic construction.
    PointMismatch { index: usize },
    /// A stored projection key does not match the point it claims to certify.
    QuantizedProjectionMismatch {
        index: usize,
        expected: [i64; 2],
        actual: [i64; 2],
    },
    /// An algebraic certificate key has the wrong field dimension.
    AlgebraicKeyDimensionMismatch { expected: usize, actual: usize },
    /// An algebraic certificate key has a non-positive denominator.
    InvalidAlgebraicKeyDenominator { denominator: i64 },
    /// A claimed norm-one translation does not verify in the supported native model.
    TranslationNotNormOne { index: usize },
    /// A certified candidate is outside the claimed polydisk window.
    CandidateOutsideWindow {
        index: usize,
        radius: String,
        max_embedding_norm: String,
    },
    /// A stored projection point does not match the certified algebraic candidate.
    ProjectionMismatch { index: usize },
    /// A certified candidate path does not replay to the stored algebraic key.
    CandidatePathMismatch { index: usize },
    /// A candidate path references a missing translation.
    CandidatePathTranslationOutOfBounds {
        candidate_index: usize,
        translation_index: usize,
        translation_count: usize,
    },
    /// A certified construction edge is not explained by a norm-one translation.
    EdgeProvenanceMismatch { edge_index: usize },
    /// The certificate contains invalid construction metadata.
    InvalidConstruction { reason: String },
    /// The serialized certificate does not match the supported schema.
    CertificateSchemaMismatch { reason: String },
}

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerificationError::PointCountMismatch { expected, actual } => write!(
                f,
                "certificate has {actual} points, but expected {expected}"
            ),
            VerificationError::EdgeIndexOutOfBounds { edge, point_count } => write!(
                f,
                "certificate edge ({}, {}) is outside point count {point_count}",
                edge.0, edge.1
            ),
            VerificationError::SelfEdge { index } => {
                write!(f, "certificate contains self-edge at point {index}")
            }
            VerificationError::EdgeSetMismatch { expected, actual } => write!(
                f,
                "certificate edge set mismatch: expected {} edges, got {}",
                expected.len(),
                actual.len()
            ),
            VerificationError::PointMismatch { index } => {
                write!(f, "certificate point {index} does not match construction")
            }
            VerificationError::QuantizedProjectionMismatch {
                index,
                expected,
                actual,
            } => write!(
                f,
                "projection key {index} mismatch: expected {:?}, got {:?}",
                expected, actual
            ),
            VerificationError::AlgebraicKeyDimensionMismatch { expected, actual } => write!(
                f,
                "algebraic key has {actual} coefficients, but field degree is {expected}"
            ),
            VerificationError::InvalidAlgebraicKeyDenominator { denominator } => write!(
                f,
                "algebraic key denominator must be positive, got {denominator}"
            ),
            VerificationError::TranslationNotNormOne { index } => {
                write!(f, "translation {index} is not certified as norm-one")
            }
            VerificationError::CandidateOutsideWindow {
                index,
                radius,
                max_embedding_norm,
            } => write!(
                f,
                "candidate {index} is outside polydisk radius {radius}; max embedding norm is {max_embedding_norm}"
            ),
            VerificationError::ProjectionMismatch { index } => {
                write!(f, "projection {index} does not match algebraic candidate")
            }
            VerificationError::CandidatePathMismatch { index } => {
                write!(f, "candidate path {index} does not replay to stored key")
            }
            VerificationError::CandidatePathTranslationOutOfBounds {
                candidate_index,
                translation_index,
                translation_count,
            } => write!(
                f,
                "candidate {candidate_index} path references translation {translation_index}, but only {translation_count} translations exist"
            ),
            VerificationError::EdgeProvenanceMismatch { edge_index } => {
                write!(
                    f,
                    "construction edge {edge_index} is not explained by its translation provenance"
                )
            }
            VerificationError::InvalidConstruction { reason } => {
                write!(f, "invalid certificate construction: {reason}")
            }
            VerificationError::CertificateSchemaMismatch { reason } => {
                write!(f, "certificate schema mismatch: {reason}")
            }
        }
    }
}

impl std::error::Error for VerificationError {}
