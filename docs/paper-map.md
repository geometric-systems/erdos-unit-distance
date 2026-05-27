# Paper Map

This map keeps the crate honest about what is implemented from the unit-distance proof and companion remarks.

| Proof component | Status | Crate mapping |
| --- | --- | --- |
| Finite point-set output | Implemented | Classical constructors and `CertifiedPointSet` |
| Norm-one / unit-modulus elements `u c(u) = 1` | Implemented | Exact rational `Section2Certificate` translation checks for native finite multiquadratic outputs |
| Minkowski embedding | Implemented | `algebraic::lattice` projection utilities |
| Polydisk / window membership | Implemented | `Section2Certificate` verifies candidate embedding bounds for native finite multiquadratic outputs |
| Projection to one complex coordinate | Implemented | Projected `Point2` output plus checked projection keys |
| Exact candidate paths | Implemented | Candidate paths replay from exact rational translations |
| Split-prime validation | Implemented | `MultiquadraticConfig` validates odd primes and Legendre-symbol splitting |
| Unit-distance construction edges | Implemented | `ConstructionEdgeCertificate` uses exact translation provenance |
| Floating unit-distance audit | Implemented | `FloatingAuditReport`, explicitly numerical and tolerance-based |
| Section 2 finite geometric construction | Implemented | `algebraic::section2` for native finite multiquadratic parameters |
| Section 3 class groups and class-number pigeonhole | Not implemented | External backend needed |
| Ideals and relative norms | Not implemented | External backend needed |
| Infinite unramified pro-3 tower | Not implemented | External backend needed |
| Golod-Shafarevich / Hajir-Maire tower argument | Not implemented | Future research API |
| Full asymptotic theorem `nu(n) >= n^(1+delta)` | Not implemented | Current crate certifies finite outputs only |

## Claim Boundary

The production API implements finite certified multiquadratic Section 2 outputs. It does not implement class groups, ideals, external CAS backends, or the full tower construction.
