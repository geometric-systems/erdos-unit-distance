# Certificate JSON Schema

`erdos-unit-distance` certificate JSON uses schema version `1`.

The Rust constant `CERTIFICATE_SCHEMA_VERSION` is re-exported from the crate root so applications can check the supported schema before accepting stored certificate data.

## Envelope

Every serialized certificate is wrapped in a top-level envelope:

```json
{
  "schema_version": 1,
  "certified": {
    "points": [],
    "certificate": {},
    "audit": {}
  }
}
```

Missing envelope fields, malformed JSON, or a schema version other than `1` are rejected with the machine-readable verification code `certificate_schema_mismatch`.

## Exact Data

Exact construction data is authoritative. In multiquadratic certificates, algebraic elements are encoded as coefficient vectors over the field basis.

Rationals are serialized as strings:

- integers: `"123"`
- rational numbers: `"123/456"`

The verifier parses these strings back into exact rational values. Malformed rational strings, zero denominators, wrong coefficient dimensions, invalid path replay, invalid translation evidence, and invalid edge provenance make verification fail.

## Floating Data

`points`, candidate projections, and `FloatingAuditReport` entries are floating-point audit data. They exist for display, export, and independent numerical checks.

The floating audit is tolerance-based and is verified separately from exact construction provenance. Applications should treat exact certificate fields as proof data and floating fields as rendering/audit data.

## Compatibility Policy

Schema version `1` is stable for the `0.2` release line. Display error messages are not part of the compatibility contract; applications should match `GenerationError::code()` and `VerificationError::code()` instead.

Any incompatible certificate layout change must bump the schema version. Additive fields may be introduced only when old schema `1` certificates still deserialize and verify.
