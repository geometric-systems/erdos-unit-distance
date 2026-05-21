# Paper Map

This map keeps the crate honest about what is implemented from the unit-distance proof and companion remarks.

| Proof component | Status | Crate mapping |
| --- | --- | --- |
| Finite point-set output | Implemented | Classical constructors and `CertifiedPointSet` |
| Norm-one / unit-modulus elements `u c(u) = 1` | Implemented for finite multiquadratic backend | Exact rational `Section2Certificate` translation checks |
| Minkowski embedding | Implemented | `algebraic::lattice` projection utilities |
| Polydisk / window membership | Implemented for finite multiquadratic backend | `Section2Certificate` verifies candidate embedding bounds |
| Projection to one complex coordinate | Implemented | Projected `Point2` output plus checked projection keys |
| Exact candidate paths | Implemented | Candidate paths replay from exact rational translations |
| Split-prime validation | Implemented | `MultiquadraticConfig` validates odd primes and Legendre-symbol splitting |
| Unit-distance construction edges | Implemented | `ConstructionEdgeCertificate` uses exact translation provenance |
| Floating unit-distance audit | Implemented | `FloatingAuditReport`, explicitly numerical and tolerance-based |
| Section 2 finite geometric construction | Implemented for native multiquadratic parameters | `algebraic::section2` |
| Section 3 class groups and class-number pigeonhole | External backend needed | Not implemented |
| Ideals and relative norms | External backend needed | Not implemented |
| Infinite unramified pro-3 tower | External backend needed | Not implemented |
| Golod-Shafarevich / Hajir-Maire tower argument | Missing | Future research API |
| Full asymptotic theorem `nu(n) >= n^(1+delta)` | Missing | Current crate certifies finite outputs only |

## Claim Boundary

The production API implements finite certified multiquadratic Section 2 outputs. It does not implement class groups, ideals, external CAS backends, or the full tower construction.
