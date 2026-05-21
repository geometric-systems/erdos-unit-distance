// Copyright (c) 2026 Lars Brusletto
// Under MIT and Apache-2.0 Licenses.
//
// This library implements certified finite prototypes for the Erdős unit-distance
// problem, based on OpenAI's 2026 disproof and the companion mathematical remarks.
// The current native algebraic implementation follows the Section 2 geometric shape;
// it does not yet implement the full class-field tower construction.

//! # Erdős Unit-Distance Constructions
//!
//! A Rust library for generating unit-distance point sets in the plane.
//! Classical constructions are exact; native finite multiquadratic prototype
//! inputs are validated and unsupported configurations are rejected before generation.
//!
//! ## Correctness Contract
//!
//! - Classical constructors return fixed, deterministic point sets.
//! - Native multiquadratic constructors validate squarefree generators and
//!   complete splitting of the requested odd prime before generating points.
//! - Native multiquadratic point generation is a bounded Section 2-style
//!   candidate search followed by projection, deduplication, and density pruning;
//!   it is not the full class-field tower construction.
//! - `generate_certified` returns a finite certificate whose edge claims are
//!   independently recomputed by `ConstructionCertificate::verify`.
//!
//! ## Quick Start
//!
//! ```
//! use erdos_unit_distance::UnitDistanceSet;
//! use erdos_unit_distance::utils::count_unit_distances;
//!
//! // Generate a classic Moser Spindle (7 points, 11 unit-distance edges)
//! let spindle = UnitDistanceSet::moser_spindle();
//! let points = spindle.generate_points(7).unwrap();
//! assert_eq!(points.len(), 7);
//!
//! let point_arrays = points.iter().map(|point| point.to_array()).collect::<Vec<_>>();
//! let edges = count_unit_distances(&point_arrays, 1e-4);
//! assert_eq!(edges, 11);
//! ```
//!
//! ## Native Finite Multiquadratic Prototype
//!
//! ```
//! use erdos_unit_distance::{MultiquadraticConfig, UnitDistanceSet};
//! use erdos_unit_distance::utils::count_unit_distances;
//!
//! let config = MultiquadraticConfig::builder(vec![5, 17], 101, 1).build().unwrap();
//! let builder = UnitDistanceSet::multiquadratic(config);
//! let points = builder.generate_points(50).unwrap();
//! assert_eq!(points.len(), 50);
//!
//! let point_arrays = points.iter().map(|point| point.to_array()).collect::<Vec<_>>();
//! let edges = count_unit_distances(&point_arrays, 1e-4);
//! assert!(edges > 0);
//! ```
//!
//! ## Certified Output
//!
//! ```
//! use erdos_unit_distance::{MultiquadraticConfig, UnitDistanceSet};
//!
//! let config = MultiquadraticConfig::builder(vec![5, 17], 101, 1).build().unwrap();
//! let builder = UnitDistanceSet::multiquadratic(config);
//! let certified = builder.generate_certified(20).unwrap();
//! let report = certified.verify().unwrap();
//! assert_eq!(report.point_count, 20);
//! assert!(report.edge_count > 0);
//! ```

pub mod algebraic {
    pub mod field;
    pub mod lattice;
    pub mod section2;
    pub mod units;
    pub mod window;
}
pub mod backend;
pub mod certificate;
pub mod classical;
pub mod error;
mod numeric;
pub mod utils;

use crate::algebraic::field::MultiQuadraticField;
use crate::algebraic::section2::generate_native_multiquadratic_section2;
pub use crate::certificate::{
    CertificateVerificationReport, CertifiedPointSet, ClassicalCertificate,
    ClassicalConstructionKind, ConstructionCertificate, FloatingAuditReport,
    MultiquadraticCertificate, Point2,
};
use crate::classical::{generate_moser_spindle, generate_square_grid, generate_triangular_grid};
pub use crate::error::{GenerationError, VerificationError};

const DEFAULT_MAX_PRIME_EXPONENT: usize = 3;
const DEFAULT_MAX_RADIUS_ATTEMPTS: usize = 15;
const DEFAULT_INITIAL_RADIUS: f64 = 2.0;
const DEFAULT_RADIUS_GROWTH: f64 = 1.5;
const DEFAULT_CANDIDATE_MULTIPLIER: usize = 10;
const DEFAULT_PROJECTION_TOLERANCE: f64 = 1e-7;
const DEFAULT_UNIT_DISTANCE_TOLERANCE: f64 = 1e-4;

/// Validated parameters for the native finite multiquadratic prototype.
#[derive(Clone, Debug)]
pub struct MultiquadraticConfig {
    generators: Vec<i64>,
    split_prime: i64,
    k: usize,
    max_prime_exponent: usize,
    max_radius_attempts: usize,
    initial_radius: f64,
    radius_growth: f64,
    candidate_multiplier: usize,
    projection_tolerance: f64,
    unit_distance_tolerance: f64,
}

impl MultiquadraticConfig {
    pub fn builder(
        generators: Vec<i64>,
        split_prime: i64,
        k: usize,
    ) -> MultiquadraticConfigBuilder {
        MultiquadraticConfigBuilder {
            generators,
            split_prime,
            k,
            max_prime_exponent: DEFAULT_MAX_PRIME_EXPONENT,
            max_radius_attempts: DEFAULT_MAX_RADIUS_ATTEMPTS,
            initial_radius: DEFAULT_INITIAL_RADIUS,
            radius_growth: DEFAULT_RADIUS_GROWTH,
            candidate_multiplier: DEFAULT_CANDIDATE_MULTIPLIER,
            projection_tolerance: DEFAULT_PROJECTION_TOLERANCE,
            unit_distance_tolerance: DEFAULT_UNIT_DISTANCE_TOLERANCE,
        }
    }

    /// Creates a validated multiquadratic configuration with conservative default search limits.
    pub fn new(generators: Vec<i64>, split_prime: i64, k: usize) -> Result<Self, GenerationError> {
        let mut generators = generators;
        if !generators.contains(&-1) {
            generators.push(-1);
        }
        MultiQuadraticField::try_new(&generators)?;
        validate_split_prime(split_prime)?;
        validate_complete_splitting(&generators, split_prime)?;

        Ok(Self {
            generators,
            split_prime,
            k,
            max_prime_exponent: DEFAULT_MAX_PRIME_EXPONENT,
            max_radius_attempts: DEFAULT_MAX_RADIUS_ATTEMPTS,
            initial_radius: DEFAULT_INITIAL_RADIUS,
            radius_growth: DEFAULT_RADIUS_GROWTH,
            candidate_multiplier: DEFAULT_CANDIDATE_MULTIPLIER,
            projection_tolerance: DEFAULT_PROJECTION_TOLERANCE,
            unit_distance_tolerance: DEFAULT_UNIT_DISTANCE_TOLERANCE,
        })
    }

    pub fn generators(&self) -> &[i64] {
        &self.generators
    }

    pub fn split_prime(&self) -> i64 {
        self.split_prime
    }

    pub fn k(&self) -> usize {
        self.k
    }

    pub fn max_prime_exponent(&self) -> usize {
        self.max_prime_exponent
    }

    pub fn max_radius_attempts(&self) -> usize {
        self.max_radius_attempts
    }

    pub fn initial_radius(&self) -> f64 {
        self.initial_radius
    }

    pub fn radius_growth(&self) -> f64 {
        self.radius_growth
    }

    pub fn candidate_multiplier(&self) -> usize {
        self.candidate_multiplier
    }

    pub fn projection_tolerance(&self) -> f64 {
        self.projection_tolerance
    }

    pub fn unit_distance_tolerance(&self) -> f64 {
        self.unit_distance_tolerance
    }

    pub fn with_prime_search_limit(
        mut self,
        max_prime_exponent: usize,
    ) -> Result<Self, GenerationError> {
        validate_nonzero_usize("max_prime_exponent", max_prime_exponent)?;
        self.max_prime_exponent = max_prime_exponent;
        Ok(self)
    }

    pub fn with_radius_search(
        mut self,
        max_radius_attempts: usize,
        initial_radius: f64,
        radius_growth: f64,
    ) -> Result<Self, GenerationError> {
        validate_nonzero_usize("max_radius_attempts", max_radius_attempts)?;
        validate_positive_f64("initial_radius", initial_radius)?;
        if !radius_growth.is_finite() || radius_growth <= 1.0 {
            return Err(GenerationError::InvalidSearchParameter {
                parameter: "radius_growth",
                reason: "expected a finite value greater than 1.0",
            });
        }
        self.max_radius_attempts = max_radius_attempts;
        self.initial_radius = initial_radius;
        self.radius_growth = radius_growth;
        Ok(self)
    }

    pub fn with_candidate_multiplier(
        mut self,
        candidate_multiplier: usize,
    ) -> Result<Self, GenerationError> {
        validate_nonzero_usize("candidate_multiplier", candidate_multiplier)?;
        self.candidate_multiplier = candidate_multiplier;
        Ok(self)
    }

    pub fn with_tolerances(
        mut self,
        projection_tolerance: f64,
        unit_distance_tolerance: f64,
    ) -> Result<Self, GenerationError> {
        validate_positive_f64("projection_tolerance", projection_tolerance)?;
        validate_positive_f64("unit_distance_tolerance", unit_distance_tolerance)?;
        self.projection_tolerance = projection_tolerance;
        self.unit_distance_tolerance = unit_distance_tolerance;
        Ok(self)
    }
}

#[derive(Clone, Debug)]
pub struct MultiquadraticConfigBuilder {
    generators: Vec<i64>,
    split_prime: i64,
    k: usize,
    max_prime_exponent: usize,
    max_radius_attempts: usize,
    initial_radius: f64,
    radius_growth: f64,
    candidate_multiplier: usize,
    projection_tolerance: f64,
    unit_distance_tolerance: f64,
}

impl MultiquadraticConfigBuilder {
    pub fn max_prime_exponent(mut self, value: usize) -> Self {
        self.max_prime_exponent = value;
        self
    }

    pub fn radius_search(mut self, attempts: usize, initial_radius: f64, growth: f64) -> Self {
        self.max_radius_attempts = attempts;
        self.initial_radius = initial_radius;
        self.radius_growth = growth;
        self
    }

    pub fn candidate_multiplier(mut self, value: usize) -> Self {
        self.candidate_multiplier = value;
        self
    }

    pub fn tolerances(mut self, projection_tolerance: f64, unit_distance_tolerance: f64) -> Self {
        self.projection_tolerance = projection_tolerance;
        self.unit_distance_tolerance = unit_distance_tolerance;
        self
    }

    pub fn build(self) -> Result<MultiquadraticConfig, GenerationError> {
        MultiquadraticConfig::new(self.generators, self.split_prime, self.k)?
            .with_prime_search_limit(self.max_prime_exponent)?
            .with_radius_search(
                self.max_radius_attempts,
                self.initial_radius,
                self.radius_growth,
            )?
            .with_candidate_multiplier(self.candidate_multiplier)?
            .with_tolerances(self.projection_tolerance, self.unit_distance_tolerance)
    }
}

/// Describes the type of unit-distance construction to generate.
#[derive(Clone, Debug)]
pub enum ConstructionType {
    /// A rectangular grid with unit spacing.
    SquareGrid { rows: usize, cols: usize },
    /// A hexagonal/triangular grid with unit spacing.
    TriangularGrid,
    /// The canonical 7-point, 11-edge Moser Spindle.
    MoserSpindle,
    /// Native finite Section 2 prototype over a multiquadratic field.
    Multiquadratic(MultiquadraticConfig),
}

/// Builder for generating unit-distance point sets.
///
/// # Behaviour by construction type
///
/// - **`SquareGrid`**: Returns `rows × cols` points; `n_target` is ignored.
/// - **`MoserSpindle`**: Always returns exactly 7 points; `n_target` is ignored.
/// - **`TriangularGrid`**: Returns exactly `n_target` points.
/// - **`Multiquadratic`**: Returns exactly `n_target` points from the native multiquadratic prototype.
#[derive(Clone, Debug)]
pub struct UnitDistanceSet {
    construction: ConstructionType,
}

impl UnitDistanceSet {
    /// Creates a new builder for a square grid.
    pub fn square_grid(rows: usize, cols: usize) -> Self {
        UnitDistanceSet {
            construction: ConstructionType::SquareGrid { rows, cols },
        }
    }

    /// Creates a new builder for a triangular grid.
    pub fn triangular_grid() -> Self {
        UnitDistanceSet {
            construction: ConstructionType::TriangularGrid,
        }
    }

    /// Creates a new builder for the Moser Spindle.
    pub fn moser_spindle() -> Self {
        UnitDistanceSet {
            construction: ConstructionType::MoserSpindle,
        }
    }

    /// Creates a validated builder for the native finite multiquadratic prototype.
    ///
    /// `generators` are the positive squarefree integers generating the real subfield.
    /// The imaginary generator `-1` is appended automatically if not already present.
    /// `split_prime` must be an odd prime split completely in K.
    /// `k` is the exponent/depth parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// use erdos_unit_distance::{ConstructionType, MultiquadraticConfig, UnitDistanceSet};
    ///
    /// let config = MultiquadraticConfig::builder(vec![5, 17], 101, 1).build().unwrap();
    /// let set = UnitDistanceSet::multiquadratic(config);
    /// match set.construction() {
    ///     ConstructionType::Multiquadratic(config) => assert!(config.generators().contains(&-1)),
    ///     _ => unreachable!(),
    /// }
    /// ```
    pub fn try_new_multiquadratic(
        generators: Vec<i64>,
        split_prime: i64,
        k: usize,
    ) -> Result<Self, GenerationError> {
        Ok(UnitDistanceSet {
            construction: ConstructionType::Multiquadratic(MultiquadraticConfig::new(
                generators,
                split_prime,
                k,
            )?),
        })
    }

    pub fn multiquadratic(config: MultiquadraticConfig) -> Self {
        UnitDistanceSet {
            construction: ConstructionType::Multiquadratic(config),
        }
    }

    pub fn construction(&self) -> &ConstructionType {
        &self.construction
    }

    /// Generates the point set of target size `n_target`.
    ///
    /// For `SquareGrid` and `MoserSpindle`, the `n_target` parameter is ignored
    /// and the natural point count of the construction is returned.
    pub fn generate(&self, n_target: usize) -> Result<Vec<[f64; 2]>, GenerationError> {
        match &self.construction {
            ConstructionType::SquareGrid { rows, cols } => Ok(generate_square_grid(*rows, *cols)),
            ConstructionType::TriangularGrid => Ok(generate_triangular_grid(n_target)),
            ConstructionType::MoserSpindle => Ok(generate_moser_spindle()),
            ConstructionType::Multiquadratic(config) => {
                Ok(generate_native_multiquadratic_section2(config, n_target)?.projected_points)
            }
        }
    }

    pub fn generate_points(&self, n_target: usize) -> Result<Vec<Point2>, GenerationError> {
        Ok(self
            .generate(n_target)?
            .into_iter()
            .map(Point2::from)
            .collect())
    }

    /// Generates a point set plus a machine-checkable certificate for the finite output.
    ///
    /// The certificate is authoritative for verification; floating coordinates are
    /// still exposed for display, export, and downstream numerical workflows.
    pub fn generate_certified(
        &self,
        n_target: usize,
    ) -> Result<CertifiedPointSet, GenerationError> {
        Ok(match &self.construction {
            ConstructionType::SquareGrid { rows, cols } => {
                let points = generate_square_grid(*rows, *cols);
                CertifiedPointSet::new_classical(
                    ClassicalConstructionKind::SquareGrid {
                        rows: *rows,
                        cols: *cols,
                    },
                    n_target,
                    points,
                    DEFAULT_UNIT_DISTANCE_TOLERANCE,
                )
            }
            ConstructionType::TriangularGrid => {
                let points = generate_triangular_grid(n_target);
                CertifiedPointSet::new_classical(
                    ClassicalConstructionKind::TriangularGrid,
                    n_target,
                    points,
                    DEFAULT_UNIT_DISTANCE_TOLERANCE,
                )
            }
            ConstructionType::MoserSpindle => {
                let points = generate_moser_spindle();
                CertifiedPointSet::new_classical(
                    ClassicalConstructionKind::MoserSpindle,
                    n_target,
                    points,
                    DEFAULT_UNIT_DISTANCE_TOLERANCE,
                )
            }
            ConstructionType::Multiquadratic(config) => {
                CertifiedPointSet::new_algebraic_from_section2(
                    generate_native_multiquadratic_section2(config, n_target)?,
                )?
            }
        })
    }
}

fn validate_split_prime(split_prime: i64) -> Result<(), GenerationError> {
    if split_prime <= 2 || !is_prime(split_prime) {
        return Err(GenerationError::InvalidSplitPrime { split_prime });
    }
    Ok(())
}

fn validate_nonzero_usize(parameter: &'static str, value: usize) -> Result<(), GenerationError> {
    if value == 0 {
        return Err(GenerationError::InvalidSearchParameter {
            parameter,
            reason: "expected a value greater than zero",
        });
    }
    Ok(())
}

fn validate_positive_f64(parameter: &'static str, value: f64) -> Result<(), GenerationError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(GenerationError::InvalidSearchParameter {
            parameter,
            reason: "expected a finite value greater than zero",
        });
    }
    Ok(())
}

fn validate_complete_splitting(
    generators: &[i64],
    split_prime: i64,
) -> Result<(), GenerationError> {
    generators.iter().try_for_each(|&generator| {
        if legendre_symbol(generator, split_prime) == 1 {
            Ok(())
        } else {
            Err(GenerationError::PrimeNotSplit {
                split_prime,
                generator,
            })
        }
    })
}

fn is_prime(n: i64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    !(3..)
        .step_by(2)
        .take_while(|d| d * d <= n)
        .any(|d| n % d == 0)
}

fn legendre_symbol(a: i64, p: i64) -> i64 {
    let a = a.rem_euclid(p);
    if a == 0 {
        return 0;
    }
    let value = mod_pow(a, (p - 1) / 2, p);
    if value == p - 1 { -1 } else { value }
}

fn mod_pow(mut base: i64, exp: i64, modulus: i64) -> i64 {
    base = base.rem_euclid(modulus);
    std::iter::successors(Some((base, exp)), |&(base, exp)| {
        (exp / 2 > 0).then_some(((base * base).rem_euclid(modulus), exp / 2))
    })
    .filter(|&(_, exp)| exp % 2 == 1)
    .fold(1, |result, (base, _)| (result * base).rem_euclid(modulus))
}
