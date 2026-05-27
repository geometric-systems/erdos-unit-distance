# Correctness Contract

## Supported

The production API supports finite point sets:

- classical Moser spindle, square grid, and triangular grid constructors
- finite native multiquadratic Section 2-style generation
- certificate JSON schema version `1`
- deterministic certificate verification for the supported finite outputs

For multiquadratic generation, supported inputs are validated before search starts:

- positive squarefree generators greater than `1`
- optional `-1` imaginary generator, appended automatically by the public config
- an odd split prime that splits for every generator
- positive, finite search limits and tolerances

## Verified

Exact rational algebraic data is authoritative. It is used for field arithmetic, candidate paths, norm-one translation evidence, deduplication keys, and certified construction-edge provenance.

`CertifiedPointSet::verify()` checks:

- construction metadata and validated configuration
- exact norm-one translation certificates in the supported native multiquadratic model
- exact replay of candidate paths
- window membership for finite candidates
- projection and deduplication keys
- exact construction-edge provenance
- floating audit edges against projected points

Classical certificates are verified by recomputing the deterministic construction and its unit-distance edges.

## Numerical Audit

Floating-point `Point2` coordinates are application-facing output. They are appropriate for visualization, export, spatial indexing, and independent numerical auditing. They are not the proof object.

`FloatingAuditReport` uses an absolute unit-distance tolerance. It is useful for checking the displayed/exported coordinates, but it is not a replacement for exact construction provenance.

## Not Implemented

The crate does not currently implement or certify:

- class-group computation
- ideal arithmetic
- relative class numbers
- Sage/PARI or other external CAS backends
- Golod-Shafarevich or Hajir-Maire tower data
- the full class-field tower construction
- a proof of the asymptotic theorem beyond finite certified outputs

Those are future research/backend work, not hidden behavior in the current production API.
