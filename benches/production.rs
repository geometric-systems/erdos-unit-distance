use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use erdos_unit_distance::algebraic::field::{ExactElement, MultiQuadraticField};
use erdos_unit_distance::algebraic::section2::generate_native_multiquadratic_section2;
use erdos_unit_distance::algebraic::units::generate_unit_modulus_elements;
use erdos_unit_distance::utils::find_unit_distance_edges;
use erdos_unit_distance::{MultiquadraticConfig, UnitDistanceSet};
use num_bigint::BigInt;
use num_rational::BigRational;

fn exact_element_multiplication(c: &mut Criterion) {
    let field = MultiQuadraticField::try_new(&[5, 17, -1]).unwrap();
    let coeffs = (0..field.degree())
        .map(|idx| BigRational::from_integer(BigInt::from(idx + 1)))
        .collect::<Vec<_>>();
    let element = ExactElement::try_from_coeffs(coeffs, &field).unwrap();

    c.bench_function("exact_element_multiplication", |b| {
        b.iter(|| element.mul(&element, &field))
    });
}

fn unit_generation(c: &mut Criterion) {
    let field = MultiQuadraticField::try_new(&[5, 17, -1]).unwrap();
    c.bench_function("unit_generation_p101_k1", |b| {
        b.iter(|| generate_unit_modulus_elements(&field, 101, 1, 3).unwrap())
    });
}

fn multiquadratic_generation(c: &mut Criterion) {
    let config = MultiquadraticConfig::builder(vec![5, 17], 101, 1)
        .build()
        .unwrap();
    let set = UnitDistanceSet::multiquadratic(config);

    let mut group = c.benchmark_group("multiquadratic_generation");
    [20_usize, 50].into_iter().for_each(|target| {
        group.bench_with_input(
            BenchmarkId::from_parameter(target),
            &target,
            |b, &target| b.iter(|| set.generate_certified(target).unwrap()),
        );
    });
    group.finish();
}

fn section2_engine(c: &mut Criterion) {
    let config = MultiquadraticConfig::builder(vec![5, 17], 101, 1)
        .build()
        .unwrap();

    c.bench_function("section2_engine_generate_20", |b| {
        b.iter(|| generate_native_multiquadratic_section2(&config, 20).unwrap())
    });
}

fn certificate_verification(c: &mut Criterion) {
    let certified = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1)
        .unwrap()
        .generate_certified(20)
        .unwrap();

    c.bench_function("certified_point_set_verify_20", |b| {
        b.iter(|| certified.verify().unwrap())
    });
}

fn float_audit(c: &mut Criterion) {
    let points = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1)
        .unwrap()
        .generate(20)
        .unwrap();

    c.bench_function("float_audit_20", |b| {
        b.iter(|| find_unit_distance_edges(&points, 1e-4))
    });
}

criterion_group!(
    production,
    exact_element_multiplication,
    unit_generation,
    multiquadratic_generation,
    section2_engine,
    certificate_verification,
    float_audit
);
criterion_main!(production);
