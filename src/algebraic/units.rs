use crate::algebraic::field::{FieldElement, MultiQuadraticField};
use crate::error::GenerationError;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Zero};

/// Recursive helper to find all integer coefficient vectors that satisfy the sum-of-squares constraint.
fn find_coefficients(
    index: usize,
    current_sum: i64,
    target: i64,
    current_coeffs: &mut Vec<i64>,
    scale_factors: &[i64],
    results: &mut Vec<Vec<i64>>,
) {
    if index == scale_factors.len() {
        if current_sum == target {
            results.push(current_coeffs.clone());
        }
        return;
    }

    let scale = scale_factors[index];
    let remaining = target - current_sum;
    if remaining < 0 {
        return;
    }

    let max_val = integer_sqrt_floor(remaining / scale);
    for val in -max_val..=max_val {
        current_coeffs.push(val);
        find_coefficients(
            index + 1,
            current_sum + val * val * scale,
            target,
            current_coeffs,
            scale_factors,
            results,
        );
        current_coeffs.pop();
    }
}

/// Searches for an element \theta \in K of norm p^y.
/// Returns \theta as a FieldElement.
#[allow(clippy::needless_range_loop)]
pub fn find_prime_element(
    field: &MultiQuadraticField,
    p: i64,
    max_y: usize,
) -> Result<(FieldElement, usize), GenerationError> {
    let m = field.generator_count();
    let scale_factors: Vec<i64> = (0..field.degree())
        .map(|j| {
            (0..m)
                .filter(|&i| ((j >> i) & 1) == 1 && field.generators()[i] != -1)
                .map(|i| field.generators()[i].abs())
                .product()
        })
        .collect();

    for y in 1..=max_y {
        let target = checked_i64_pow(p, y, "prime_norm_target")?;
        let mut candidates = Vec::new();
        let mut current_coeffs = Vec::new();
        find_coefficients(
            0,
            0,
            target,
            &mut current_coeffs,
            &scale_factors,
            &mut candidates,
        );

        if let Some(theta) = candidates
            .into_iter()
            .map(|cand| {
                FieldElement::try_from_coeffs(
                    cand.into_iter()
                        .map(|x| BigRational::from_integer(BigInt::from(x)))
                        .collect(),
                    field,
                )
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .find(|theta| is_valid_prime_element(theta, field, target))
        {
            return Ok((theta, y));
        }
    }

    Err(GenerationError::PrimeElementNotFound {
        prime: p,
        max_exponent: max_y,
    })
}

/// Generates the Galois conjugates of a given theta element.
/// Since the Galois group of F/Q is (Z/2Z)^{m-1}, we conjugate the real generators.
pub fn generate_conjugates(theta: &FieldElement, field: &MultiQuadraticField) -> Vec<FieldElement> {
    let real_gens: Vec<usize> = (0..field.generator_count())
        .filter(|&i| field.generators()[i] != -1)
        .collect();
    let num_conjugates = 1 << real_gens.len();

    (0..num_conjugates)
        .map(|mask| {
            real_gens
                .iter()
                .enumerate()
                .filter(|(i, _)| ((mask >> i) & 1) == 1)
                .fold(theta.clone(), |cand, (_, &gen_idx)| {
                    cand.conjugate_gen(gen_idx)
                })
        })
        .collect()
}

/// Generates all unit-modulus elements in K by computing all combinations
/// \prod_{i=1}^d \theta_i^{a_i} \bar{\theta}_i^{k - a_i} and scaling by p^{-y * k / 2}.
#[allow(clippy::needless_range_loop)]
pub fn generate_unit_modulus_elements(
    field: &MultiQuadraticField,
    p: i64,
    k: usize,
    max_prime_exponent: usize,
) -> Result<Vec<FieldElement>, GenerationError> {
    field
        .imaginary_generator_index()
        .ok_or(GenerationError::MissingImaginaryGenerator)?;

    // 1. Find the base prime element theta
    let (theta, y) = find_prime_element(field, p, max_prime_exponent)?;

    // 2. Generate Galois conjugates of theta
    let conjugates = generate_conjugates(&theta, field);
    let d = conjugates.len(); // Degree of F over Q = 2^{m-1}

    let num_combinations = checked_usize_pow(k + 1, d, "unit_modulus_combination_count")?;

    let units = (0..num_combinations)
        .map(|idx| decode_exponents(idx, d, k + 1))
        .map(|exponents| {
            let beta = conjugates.iter().zip(exponents).fold(
                FieldElement::one(field),
                |beta, (theta_i, a)| {
                    let conj_theta_i = theta_i.complex_conjugate(field);
                    let beta = multiply_power(beta, theta_i, a, field);
                    multiply_power(beta, &conj_theta_i, k - a, field)
                },
            );

            let exponent = y
                .checked_mul(k)
                .and_then(|value| value.checked_mul(d))
                .ok_or(GenerationError::InvalidSearchParameter {
                    parameter: "unit_scale_exponent",
                    reason: "integer exponentiation overflowed",
                })?;
            if !exponent.is_multiple_of(2) {
                return Err(GenerationError::InvalidSearchParameter {
                    parameter: "unit_scale_exponent",
                    reason: "expected even scaling exponent",
                });
            }
            let denominator = checked_i64_pow(p, exponent / 2, "unit_scale_denominator")?;
            let scale = BigRational::new(BigInt::one(), BigInt::from(denominator));
            Ok(beta
                .into_coeffs()
                .into_iter()
                .map(|coeff| coeff * scale.clone())
                .collect::<Vec<_>>())
        })
        .map(|coeffs| FieldElement::try_from_coeffs(coeffs?, field))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(units
        .into_iter()
        .fold(Vec::new(), |mut unique_units, unit| {
            if !unique_units
                .iter()
                .any(|existing| coefficient_distance(&unit, existing) < 1e-7)
            {
                unique_units.push(unit);
            }
            unique_units
        }))
}

fn is_valid_prime_element(theta: &FieldElement, field: &MultiQuadraticField, target: i64) -> bool {
    let norm = theta.mul(&theta.complex_conjugate(field), field);
    norm.coeffs()[0] == BigRational::from_integer(BigInt::from(target))
        && norm.coeffs()[1..].iter().all(BigRational::is_zero)
        && theta.coeffs()[1..].iter().any(|coeff| !coeff.is_zero())
}

fn decode_exponents(mut idx: usize, len: usize, radix: usize) -> Vec<usize> {
    (0..len)
        .map(|_| {
            let exponent = idx % radix;
            idx /= radix;
            exponent
        })
        .collect()
}

fn integer_sqrt_floor(value: i64) -> i64 {
    if value <= 0 {
        return 0;
    }

    let mut low = 0_i64;
    let mut high = value.min(3_037_000_499);
    while low <= high {
        let mid = low + (high - low) / 2;
        match mid.checked_mul(mid) {
            Some(square) if square <= value => low = mid + 1,
            _ => high = mid - 1,
        }
    }
    high
}

fn checked_i64_pow(
    base: i64,
    exponent: usize,
    parameter: &'static str,
) -> Result<i64, GenerationError> {
    (0..exponent).try_fold(1_i64, |product, _| {
        product
            .checked_mul(base)
            .ok_or(GenerationError::InvalidSearchParameter {
                parameter,
                reason: "integer exponentiation overflowed",
            })
    })
}

fn checked_usize_pow(
    base: usize,
    exponent: usize,
    parameter: &'static str,
) -> Result<usize, GenerationError> {
    (0..exponent).try_fold(1_usize, |product, _| {
        product
            .checked_mul(base)
            .ok_or(GenerationError::InvalidSearchParameter {
                parameter,
                reason: "integer exponentiation overflowed",
            })
    })
}

fn multiply_power(
    base: FieldElement,
    factor: &FieldElement,
    exponent: usize,
    field: &MultiQuadraticField,
) -> FieldElement {
    (0..exponent).fold(base, |acc, _| acc.mul(factor, field))
}

fn coefficient_distance(left: &FieldElement, right: &FieldElement) -> f64 {
    if left == right { 0.0 } else { f64::INFINITY }
}
