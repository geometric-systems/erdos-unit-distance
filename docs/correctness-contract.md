# Correctness Contract

## Authoritative Data

Exact rational algebraic data is authoritative. It is used for field arithmetic, candidate paths, norm-one translation evidence, deduplication keys, and certified construction-edge provenance.

## Floating Data

Floating-point `Point2` coordinates are application-facing output. They are appropriate for visualization, export, spatial indexing, and independent numerical auditing. They are not the proof object.

## Certificates

`CertifiedPointSet::verify()` checks:

- construction metadata and validated configuration
- exact norm-one translation certificates
- exact replay of candidate paths
- window membership for projected finite candidates
- projection and deduplication keys
- exact construction-edge provenance
- floating audit edges against projected points

## Unsupported Claims

The crate does not currently certify class groups, ideal arithmetic, relative class numbers, Golod-Shafarevich tower data, or the full asymptotic theorem. Those require future backend traits and external or native number-theory implementations.
