use crate::error::{GenerationError, VerificationError};
use crate::numeric::i64_to_f64_generation;
use num_bigint::BigInt;
use num_complex::Complex;
use num_rational::BigRational;
use num_traits::{One, ToPrimitive, Zero};
use std::collections::HashSet;

pub type ExactRational = BigRational;

#[derive(Clone, Debug)]
pub struct MultiQuadraticField {
    generators: Vec<i64>,
    m: usize,
    n: usize,
    scale_matrix: Vec<i64>,
    base_values: Vec<Complex<f64>>,
    i_idx: Option<usize>,
}

impl MultiQuadraticField {
    /// Creates a validated multi-quadratic field from squarefree generators.
    ///
    /// Positive generators must be squarefree integers greater than 1. The only
    /// supported negative generator is `-1`, which represents the imaginary unit.
    ///
    /// # Examples
    ///
    /// ```
    /// use erdos_unit_distance::algebraic::field::MultiQuadraticField;
    ///
    /// let field = MultiQuadraticField::try_new(&[5, -1]).unwrap();
    /// assert_eq!(field.degree(), 4);
    /// assert_eq!(field.imaginary_generator_index(), Some(1));
    /// ```
    pub fn try_new(generators: &[i64]) -> Result<Self, GenerationError> {
        validate_generators(generators)?;
        Self::new_unchecked(generators)
    }

    pub(crate) fn new_unchecked(generators: &[i64]) -> Result<Self, GenerationError> {
        let m = generators.len();
        let n = 1 << m;

        let scale_matrix = (0..n)
            .flat_map(|a| {
                (0..n).map(move |b| {
                    (0..m)
                        .filter(|&j| ((a >> j) & 1) == 1 && ((b >> j) & 1) == 1)
                        .map(|j| generators[j])
                        .try_fold(1_i64, |product, generator| {
                            product.checked_mul(generator).ok_or(
                                GenerationError::InvalidSearchParameter {
                                    parameter: "field_scale_matrix",
                                    reason: "basis scale factor overflowed i64",
                                },
                            )
                        })
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let base_values = (0..n)
            .map(|a| {
                (0..m)
                    .filter(|&j| ((a >> j) & 1) == 1)
                    .map(|j| {
                        let dj = generators[j];
                        if dj > 0 {
                            i64_to_f64_generation(dj, "field_generator")
                                .map(|value| Complex::new(value.sqrt(), 0.0))
                        } else {
                            i64_to_f64_generation(dj.abs(), "field_generator")
                                .map(|value| Complex::new(0.0, value.sqrt()))
                        }
                    })
                    .try_fold(Complex::new(1.0, 0.0), |product, value| {
                        value.map(|value| product * value)
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let i_idx = generators.iter().position(|&x| x == -1);

        Ok(MultiQuadraticField {
            generators: generators.to_vec(),
            m,
            n,
            scale_matrix,
            base_values,
            i_idx,
        })
    }

    pub fn generators(&self) -> &[i64] {
        &self.generators
    }

    pub fn generator_count(&self) -> usize {
        self.m
    }

    pub fn degree(&self) -> usize {
        self.n
    }

    pub fn imaginary_generator_index(&self) -> Option<usize> {
        self.i_idx
    }

    pub fn integer_scale_factor(&self, a: usize, b: usize) -> i64 {
        self.scale_matrix[a * self.n + b]
    }

    fn base_values(&self) -> &[Complex<f64>] {
        &self.base_values
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ExactElement {
    coeffs: Vec<ExactRational>,
}

impl ExactElement {
    pub fn try_from_coeffs(
        coeffs: Vec<ExactRational>,
        field: &MultiQuadraticField,
    ) -> Result<Self, GenerationError> {
        if coeffs.len() != field.degree() {
            return Err(GenerationError::InvalidFieldElementDimension {
                expected: field.degree(),
                actual: coeffs.len(),
            });
        }
        Ok(Self { coeffs })
    }

    pub fn try_from_f64_coeffs(
        coeffs: Vec<f64>,
        field: &MultiQuadraticField,
    ) -> Result<Self, GenerationError> {
        Self::try_from_coeffs(
            coeffs
                .into_iter()
                .map(|coeff| {
                    ExactRational::from_float(coeff).ok_or(
                        GenerationError::InvalidSearchParameter {
                            parameter: "field_element_coeff",
                            reason: "expected a finite rational f64 coefficient",
                        },
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
            field,
        )
    }

    pub fn coeffs(&self) -> &[ExactRational] {
        &self.coeffs
    }

    pub(crate) fn into_coeffs(self) -> Vec<ExactRational> {
        self.coeffs
    }

    pub fn coeffs_as_f64(&self) -> Result<Vec<f64>, GenerationError> {
        self.coeffs
            .iter()
            .map(rational_to_f64)
            .collect::<Result<Vec<_>, _>>()
    }

    pub fn zero(field: &MultiQuadraticField) -> Self {
        Self {
            coeffs: vec![ExactRational::zero(); field.degree()],
        }
    }

    pub fn one(field: &MultiQuadraticField) -> Self {
        let mut coeffs = vec![ExactRational::zero(); field.degree()];
        coeffs[0] = ExactRational::one();
        Self { coeffs }
    }

    pub fn from_rational(val: ExactRational, field: &MultiQuadraticField) -> Self {
        let mut coeffs = vec![ExactRational::zero(); field.degree()];
        coeffs[0] = val;
        Self { coeffs }
    }

    pub fn from_integer(val: i64, field: &MultiQuadraticField) -> Self {
        Self::from_rational(ExactRational::from_integer(BigInt::from(val)), field)
    }

    pub fn add(&self, other: &ExactElement) -> ExactElement {
        debug_assert_eq!(self.coeffs.len(), other.coeffs.len());
        Self {
            coeffs: self
                .coeffs
                .iter()
                .zip(&other.coeffs)
                .map(|(c, o)| c + o)
                .collect(),
        }
    }

    pub fn sub(&self, other: &ExactElement) -> ExactElement {
        debug_assert_eq!(self.coeffs.len(), other.coeffs.len());
        Self {
            coeffs: self
                .coeffs
                .iter()
                .zip(&other.coeffs)
                .map(|(c, o)| c - o)
                .collect(),
        }
    }

    /// Multiplies two field elements using the field's basis multiplication table.
    ///
    /// # Examples
    ///
    /// ```
    /// use erdos_unit_distance::algebraic::field::{ExactElement, MultiQuadraticField};
    /// use num_rational::BigRational;
    ///
    /// let field = MultiQuadraticField::try_new(&[5, -1]).unwrap();
    /// let mut sqrt5_coeffs = vec![BigRational::from_integer(0.into()); field.degree()];
    /// sqrt5_coeffs[1] = BigRational::from_integer(1.into());
    /// let sqrt5 = ExactElement::try_from_coeffs(sqrt5_coeffs, &field).unwrap();
    ///
    /// let square = sqrt5.mul(&sqrt5, &field);
    /// assert_eq!(square.coeffs()[0], BigRational::from_integer(5.into()));
    /// ```
    pub fn mul(&self, other: &ExactElement, field: &MultiQuadraticField) -> ExactElement {
        debug_assert_eq!(self.coeffs.len(), field.degree());
        debug_assert_eq!(other.coeffs.len(), field.degree());
        let n = field.degree();
        let mut res = vec![ExactRational::zero(); n];
        self.coeffs.iter().enumerate().for_each(|(a, ca)| {
            other.coeffs.iter().enumerate().for_each(|(b, cb)| {
                let dest = a ^ b;
                res[dest] += ca
                    * cb
                    * ExactRational::from_integer(BigInt::from(field.integer_scale_factor(a, b)));
            });
        });
        Self { coeffs: res }
    }

    pub fn conjugate_gen(&self, gen_idx: usize) -> ExactElement {
        Self {
            coeffs: self
                .coeffs
                .iter()
                .enumerate()
                .map(|(a, coeff)| {
                    if ((a >> gen_idx) & 1) == 1 {
                        -coeff
                    } else {
                        coeff.clone()
                    }
                })
                .collect(),
        }
    }

    pub fn complex_conjugate(&self, field: &MultiQuadraticField) -> ExactElement {
        if let Some(idx) = field.imaginary_generator_index() {
            self.conjugate_gen(idx)
        } else {
            self.clone()
        }
    }

    /// Computes all complex embeddings of this element using the Fast Walsh-Hadamard Transform (FWHT).
    pub fn embeddings(&self, field: &MultiQuadraticField) -> Vec<Complex<f64>> {
        debug_assert_eq!(self.coeffs.len(), field.degree());
        let mut temp: Vec<Complex<f64>> = self
            .coeffs
            .iter()
            .zip(field.base_values())
            .map(|(c, &b)| b * rational_to_f64(c).unwrap_or(f64::NAN))
            .collect();

        fwht(&mut temp);
        temp
    }

    /// Computes the multiplicative inverse of this element.
    pub fn inv(&self, field: &MultiQuadraticField) -> Option<ExactElement> {
        let n = field.degree();
        let mut matrix = vec![vec![ExactRational::zero(); n + 1]; n];
        self.coeffs.iter().enumerate().for_each(|(basis, coeff)| {
            (0..n).for_each(|column| {
                let row = basis ^ column;
                matrix[row][column] += coeff
                    * ExactRational::from_integer(BigInt::from(
                        field.integer_scale_factor(basis, column),
                    ));
            });
        });
        matrix[0][n] = ExactRational::one();

        solve_linear_system(matrix).map(|coeffs| ExactElement { coeffs })
    }

    pub fn div(&self, other: &ExactElement, field: &MultiQuadraticField) -> Option<ExactElement> {
        let inv_other = other.inv(field)?;
        Some(self.mul(&inv_other, field))
    }
}

pub type FieldElement = ExactElement;

pub(crate) fn rational_to_f64(value: &ExactRational) -> Result<f64, GenerationError> {
    let numerator = value
        .numer()
        .to_f64()
        .ok_or(GenerationError::InvalidSearchParameter {
            parameter: "exact_rational",
            reason: "rational numerator is too large to project to f64",
        })?;
    let denominator = value
        .denom()
        .to_f64()
        .ok_or(GenerationError::InvalidSearchParameter {
            parameter: "exact_rational",
            reason: "rational denominator is too large to project to f64",
        })?;
    Ok(numerator / denominator)
}

pub(crate) fn rational_to_string(value: &ExactRational) -> String {
    if value.denom().is_one() {
        value.numer().to_string()
    } else {
        format!("{}/{}", value.numer(), value.denom())
    }
}

pub(crate) fn rational_from_string(value: &str) -> Result<ExactRational, VerificationError> {
    let Some((numerator, denominator)) = value.split_once('/') else {
        return value
            .parse::<BigInt>()
            .map(ExactRational::from_integer)
            .map_err(|_| VerificationError::CertificateSchemaMismatch {
                reason: format!("invalid rational {value}"),
            });
    };
    let numerator =
        numerator
            .parse::<BigInt>()
            .map_err(|_| VerificationError::CertificateSchemaMismatch {
                reason: format!("invalid rational numerator {value}"),
            })?;
    let denominator = denominator.parse::<BigInt>().map_err(|_| {
        VerificationError::CertificateSchemaMismatch {
            reason: format!("invalid rational denominator {value}"),
        }
    })?;
    if denominator.is_zero() {
        return Err(VerificationError::CertificateSchemaMismatch {
            reason: "rational denominator must be nonzero".to_string(),
        });
    }
    Ok(ExactRational::new(numerator, denominator))
}

fn solve_linear_system(mut matrix: Vec<Vec<ExactRational>>) -> Option<Vec<ExactRational>> {
    let n = matrix.len();
    for pivot_col in 0..n {
        let pivot_row = (pivot_col..n).find(|&row| !matrix[row][pivot_col].is_zero())?;
        matrix.swap(pivot_col, pivot_row);

        let pivot = matrix[pivot_col][pivot_col].clone();
        (pivot_col..=n).for_each(|col| matrix[pivot_col][col] /= pivot.clone());

        (0..n).filter(|&row| row != pivot_col).for_each(|row| {
            let factor = matrix[row][pivot_col].clone();
            if !factor.is_zero() {
                (pivot_col..=n).for_each(|col| {
                    let pivot_value = matrix[pivot_col][col].clone();
                    matrix[row][col] -= factor.clone() * pivot_value;
                });
            }
        });
    }

    Some(matrix.into_iter().map(|row| row[n].clone()).collect())
}

fn validate_generators(generators: &[i64]) -> Result<(), GenerationError> {
    let mut seen = HashSet::new();
    generators.iter().try_for_each(|&generator| {
        if !seen.insert(generator) {
            return Err(GenerationError::DuplicateGenerator { generator });
        }

        match generator {
            -1 => Ok(()),
            generator if generator <= 1 => Err(GenerationError::InvalidGenerator {
                generator,
                reason: "expected a positive squarefree integer greater than 1, or -1",
            }),
            generator if !is_squarefree(generator) => Err(GenerationError::InvalidGenerator {
                generator,
                reason: "positive generators must be squarefree",
            }),
            _ => Ok(()),
        }
    })
}

fn is_squarefree(n: i64) -> bool {
    debug_assert!(n > 1);
    !(2..)
        .map(|factor| factor * factor)
        .take_while(|&square| square <= n)
        .any(|square| n % square == 0)
}

/// In-place Fast Walsh-Hadamard Transform.
fn fwht(a: &mut [Complex<f64>]) {
    let n = a.len();
    let mut h = 1;
    while h < n {
        for i in (0..n).step_by(h * 2) {
            for j in i..i + h {
                let x = a[j];
                let y = a[j + h];
                a[j] = x + y;
                a[j + h] = x - y;
            }
        }
        h *= 2;
    }
}
