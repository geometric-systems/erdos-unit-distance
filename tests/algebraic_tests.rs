use erdos_unit_distance::algebraic::field::{FieldElement, MultiQuadraticField};
use erdos_unit_distance::algebraic::units::{find_prime_element, generate_unit_modulus_elements};
use erdos_unit_distance::error::GenerationError;
use erdos_unit_distance::utils::count_unit_distances;
use erdos_unit_distance::{MultiquadraticConfig, UnitDistanceSet};
use num_bigint::BigInt;
use num_rational::BigRational;

fn q(value: i64) -> BigRational {
    BigRational::from_integer(BigInt::from(value))
}

fn field(generators: &[i64]) -> MultiQuadraticField {
    MultiQuadraticField::try_new(generators).unwrap()
}

fn elem(coeffs: Vec<f64>, field: &MultiQuadraticField) -> FieldElement {
    FieldElement::try_from_f64_coeffs(coeffs, field).unwrap()
}

fn assert_coeffs_close(left: &FieldElement, right: &FieldElement) {
    assert_eq!(left.coeffs().len(), right.coeffs().len());
    for (a, b) in left.coeffs().iter().zip(right.coeffs()) {
        assert_eq!(a, b, "{a} != {b}");
    }
}

#[test]
fn test_generator_validation_rejects_invalid_inputs() {
    let cases = [
        (
            vec![0, -1],
            GenerationError::InvalidGenerator {
                generator: 0,
                reason: "expected a positive squarefree integer greater than 1, or -1",
            },
        ),
        (
            vec![1, -1],
            GenerationError::InvalidGenerator {
                generator: 1,
                reason: "expected a positive squarefree integer greater than 1, or -1",
            },
        ),
        (
            vec![4, -1],
            GenerationError::InvalidGenerator {
                generator: 4,
                reason: "positive generators must be squarefree",
            },
        ),
        (
            vec![-5, -1],
            GenerationError::InvalidGenerator {
                generator: -5,
                reason: "expected a positive squarefree integer greater than 1, or -1",
            },
        ),
        (
            vec![5, 5, -1],
            GenerationError::DuplicateGenerator { generator: 5 },
        ),
        (
            vec![5, -1, -1],
            GenerationError::DuplicateGenerator { generator: -1 },
        ),
    ];

    for (generators, expected) in cases {
        assert_eq!(
            MultiQuadraticField::try_new(&generators).unwrap_err(),
            expected
        );
    }
}

#[test]
fn test_split_prime_validation() {
    assert!(UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1).is_ok());
    assert_eq!(
        UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 2, 1).unwrap_err(),
        GenerationError::InvalidSplitPrime { split_prime: 2 }
    );
    assert_eq!(
        UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 91, 1).unwrap_err(),
        GenerationError::InvalidSplitPrime { split_prime: 91 }
    );
    assert_eq!(
        UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 103, 1).unwrap_err(),
        GenerationError::PrimeNotSplit {
            split_prime: 103,
            generator: 5
        }
    );
}

#[test]
fn test_algebraic_config_search_parameter_validation() {
    let config = MultiquadraticConfig::new(vec![5, 17], 101, 1).unwrap();
    assert_eq!(config.max_prime_exponent(), 3);
    assert_eq!(
        config.clone().with_prime_search_limit(0).unwrap_err(),
        GenerationError::InvalidSearchParameter {
            parameter: "max_prime_exponent",
            reason: "expected a value greater than zero",
        }
    );
    assert_eq!(
        config.clone().with_radius_search(1, 2.0, 1.0).unwrap_err(),
        GenerationError::InvalidSearchParameter {
            parameter: "radius_growth",
            reason: "expected a finite value greater than 1.0",
        }
    );
    assert!(
        config
            .with_prime_search_limit(4)
            .unwrap()
            .with_radius_search(12, 1.5, 1.25)
            .unwrap()
            .with_candidate_multiplier(8)
            .unwrap()
            .with_tolerances(1e-8, 1e-5)
            .is_ok()
    );
}

#[test]
fn test_field_element_dimension_validation() {
    let field = field(&[5, -1]);
    assert_eq!(
        FieldElement::try_from_coeffs(vec![q(1), q(2)], &field).unwrap_err(),
        GenerationError::InvalidFieldElementDimension {
            expected: 4,
            actual: 2,
        }
    );
}

#[test]
fn test_field_arithmetic() {
    let field = field(&[5, -1]);

    let a = FieldElement::from_integer(2, &field);
    let b = FieldElement::from_integer(3, &field);
    let c = a.add(&b);
    assert_eq!(c.coeffs()[0], q(5));

    let mut sqrt5_coeffs = vec![0.0; field.degree()];
    sqrt5_coeffs[1] = 1.0;
    let sqrt5 = elem(sqrt5_coeffs, &field);
    let sqrt5_sq = sqrt5.mul(&sqrt5, &field);
    assert_eq!(sqrt5_sq.coeffs()[0], q(5));

    let two_plus_i = elem(vec![2.0, 0.0, 1.0, 0.0], &field);
    let inv = two_plus_i.inv(&field).unwrap();
    let inv_coeffs = inv.coeffs_as_f64().unwrap();
    assert!((inv_coeffs[0] - 0.4).abs() < 1e-7);
    assert!((inv_coeffs[2] + 0.2).abs() < 1e-7);

    let one = two_plus_i.mul(&inv, &field);
    let one_coeffs = one.coeffs_as_f64().unwrap();
    assert!((one_coeffs[0] - 1.0).abs() < 1e-7);
    one_coeffs
        .iter()
        .take(field.degree())
        .skip(1)
        .for_each(|coeff| assert!(coeff.abs() < 1e-7));
}

#[test]
fn test_complex_embedding_norms() {
    let field = field(&[5, -1]);
    let element = elem(vec![2.0, 1.0, -3.0, 0.5], &field);

    for embedding in element.embeddings(&field) {
        assert!((embedding.norm_sqr().sqrt() - embedding.norm()).abs() < 1e-12);
    }
}

#[test]
fn test_field_multiplication_is_commutative_associative_with_identity() {
    let field = field(&[2, 3, -1]);
    let one = FieldElement::one(&field);
    let samples = [
        elem(vec![1.0, 2.0, 0.0, -3.0, 4.0, 0.5, -1.0, 2.5], &field),
        elem(vec![0.0, -1.0, 3.0, 1.5, -2.0, 0.0, 0.25, -0.75], &field),
        elem(vec![2.0, 0.0, -0.5, 4.0, 1.0, -3.0, 0.0, 0.125], &field),
    ];

    for a in &samples {
        assert_coeffs_close(&a.mul(&one, &field), a);
        for b in &samples {
            assert_coeffs_close(&a.mul(b, &field), &b.mul(a, &field));
            for c in &samples {
                assert_coeffs_close(
                    &a.mul(&b.mul(c, &field), &field),
                    &a.mul(b, &field).mul(c, &field),
                );
            }
        }
    }
}

#[test]
fn test_embeddings_and_conjugate() {
    let field = field(&[3, -1]);
    let x = elem(vec![1.0, 1.0, 2.0, 0.0], &field);

    let x_bar = x.complex_conjugate(&field);
    assert_eq!(x_bar.coeffs()[0], q(1));
    assert_eq!(x_bar.coeffs()[1], q(1));
    assert_eq!(x_bar.coeffs()[2], q(-2));

    let embs = x.embeddings(&field);
    let sqrt3 = (3.0_f64).sqrt();
    let expected0_re = 1.0 + sqrt3;
    let expected1_re = 1.0 - sqrt3;

    assert!((embs[0].re - expected0_re).abs() < 1e-7);
    assert!((embs[0].im - 2.0).abs() < 1e-7);
    assert!((embs[1].re - expected1_re).abs() < 1e-7);
    assert!((embs[1].im - 2.0).abs() < 1e-7);
}

#[test]
fn test_prime_element_and_units() {
    let field = field(&[5, 17, -1]);

    let (theta, y) = find_prime_element(&field, 101, 2).unwrap();
    assert!(y >= 1);

    let norm = theta.mul(&theta.complex_conjugate(&field), &field);
    let target = (0..y).fold(q(1), |acc, _| acc * q(101));
    assert_eq!(norm.coeffs()[0], target);
    for i in 1..field.degree() {
        assert_eq!(norm.coeffs()[i], q(0));
    }

    let units = generate_unit_modulus_elements(&field, 101, 1, 3).unwrap();
    assert!(!units.is_empty());

    for u in units {
        let embs = u.embeddings(&field);
        for emb in embs {
            assert!((emb.norm() - 1.0).abs() < 1e-6);
        }
    }
}

#[test]
fn test_unit_generation_requires_imaginary_generator() {
    let field = field(&[5, 17]);
    let err = generate_unit_modulus_elements(&field, 101, 1, 3).unwrap_err();
    assert_eq!(err, GenerationError::MissingImaginaryGenerator);
}

#[test]
fn test_unit_distance_set_classical() {
    let spindle = UnitDistanceSet::moser_spindle();
    let pts = spindle.generate(7).unwrap();
    assert_eq!(pts.len(), 7);
    let count = count_unit_distances(&pts, 1e-4);
    assert_eq!(count, 11);

    let grid = UnitDistanceSet::square_grid(3, 3);
    let pts = grid.generate(9).unwrap();
    assert_eq!(pts.len(), 9);
    let count = count_unit_distances(&pts, 1e-4);
    assert_eq!(count, 12);
}

#[test]
fn test_unit_distance_set_algebraic() {
    let builder = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1).unwrap();
    let pts = builder.generate(20).unwrap();
    assert_eq!(pts.len(), 20);

    let count = count_unit_distances(&pts, 1e-4);
    assert!(count > 0);
}
