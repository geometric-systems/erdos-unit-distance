# erdos-unit-distance: Certified Unit-Distance Graphs and Point Sets

Generate and verify finite unit-distance point sets and unit distance graphs for discrete geometry in Rust and Python. Includes the Moser spindle, square and triangular grids, and certified multiquadratic constructions inspired by the Erdős/Erdos unit-distance breakthrough.

The first supported production target is a **finite Section 2 multiquadratic construction**. The crate does **not** claim to implement the full class-field tower proof. Exact certificate data is authoritative; floating-point coordinates are for application output, visualization, export, and independent numerical audit.

## Use Cases

- Generate unit-distance graphs for algorithm experiments.
- Build certified finite point sets for discrete geometry applications.
- Reproduce Moser spindle, square grid, and triangular grid examples.
- Export geometry to CSV, SVG, or OBJ.
- Verify construction certificates independently from floating-point output.

## Rust Quick Start

```rust
use erdos_unit_distance::{MultiquadraticConfig, UnitDistanceSet};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = MultiquadraticConfig::builder(vec![5, 17], 101, 1).build()?;
    let set = UnitDistanceSet::multiquadratic(config);
    let certified = set.generate_certified(100)?;
    let report = certified.verify()?;

    println!(
        "{} points, {} certified construction edges, {} floating audit edges",
        report.point_count, report.certified_edge_count, report.audit_edge_count
    );
    Ok(())
}
```

## Python Quick Start

```python
import erdos_unit_distance

certified = erdos_unit_distance.generate_multiquadratic_certified(
    generators=[5, 17],
    split_prime=101,
    k=1,
    n_target=100,
)

assert certified["verified"]
print(certified["point_count"], certified["certified_edge_count"])
```

## Correctness Contract

- Classical constructors are deterministic finite point sets.
- Multiquadratic construction validates generators, split primes, and search parameters before generation.
- Exact rational algebraic elements drive construction state, candidate paths, norm-one translation checks, deduplication keys, and certified construction edges.
- `CertifiedPointSet::verify()` verifies the exact construction certificate and then verifies the separate floating audit report against projected points.
- `FloatingAuditReport` is numerical and tolerance-based. It is useful for application inspection, but it is not the proof object.
- Unsupported class groups, ideals, and tower machinery are not hidden behind production APIs.

More detail: `docs/correctness-contract.md`.

## Certificate JSON

Certificates serialize with schema version `1`. The top-level JSON envelope contains `schema_version` and `certified` fields, and applications can compare against the Rust constant `CERTIFICATE_SCHEMA_VERSION`.

Exact certificate fields are the proof data. Floating fields are for display/export and tolerance-based audit. More detail: `docs/certificate-schema.md`.

## Supported APIs

Rust:

- `UnitDistanceSet::moser_spindle()`
- `UnitDistanceSet::square_grid(rows, cols)`
- `UnitDistanceSet::triangular_grid()`
- `MultiquadraticConfig::builder(generators, split_prime, k)`
- `UnitDistanceSet::multiquadratic(config)`
- `generate_points(n)`
- `generate_certified(n)`

Python:

- `generate_moser_spindle()`
- `generate_square_grid(rows, cols)`
- `generate_triangular_grid(n)`
- `generate_multiquadratic(generators, split_prime, k, n_target)`
- `generate_multiquadratic_certified(...)`

## Verification

```bash
./scripts/check-format.sh
./scripts/check-clippy-strict.sh
./scripts/check-tests.sh
./scripts/check-doctests.sh
./scripts/check-examples.sh
./scripts/check-bench-compile.sh
./scripts/check-python-smoke.sh
```

Run everything locally with `./scripts/check-all.sh`.

Run real Criterion benchmarks with:

```bash
./scripts/bench.sh
```

## Paper Traceability

`docs/paper-map.md` tracks what is implemented from the proof and what remains external-backend or future work.

## What This Is Not Yet

- no class-group computation
- no ideal arithmetic backend
- no Sage/PARI backend
- no full Golod-Shafarevich/class-field tower construction
- no proof of the asymptotic theorem beyond finite certified outputs

See `docs/paper-map.md` and `docs/correctness-contract.md` for the precise implementation boundary.

## License

Licensed under either Apache-2.0 or MIT, at your option.
