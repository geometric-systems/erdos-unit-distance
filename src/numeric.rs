use crate::error::{GenerationError, VerificationError};
use num_traits::ToPrimitive;

const MAX_EXACT_F64_INTEGER: u64 = 1_u64 << 53;

pub(crate) fn i64_to_f64_generation(
    value: i64,
    parameter: &'static str,
) -> Result<f64, GenerationError> {
    if value.unsigned_abs() > MAX_EXACT_F64_INTEGER {
        return Err(GenerationError::InvalidSearchParameter {
            parameter,
            reason: "integer is too large to represent exactly as f64",
        });
    }
    value
        .to_f64()
        .ok_or(GenerationError::InvalidSearchParameter {
            parameter,
            reason: "could not convert integer to f64",
        })
}

pub(crate) fn i64_to_f64_verification(value: i64) -> Result<f64, VerificationError> {
    if value.unsigned_abs() > MAX_EXACT_F64_INTEGER {
        return Err(VerificationError::InvalidConstruction {
            reason: "integer is too large to represent exactly as f64".to_string(),
        });
    }
    value
        .to_f64()
        .ok_or_else(|| VerificationError::InvalidConstruction {
            reason: "could not convert integer to f64".to_string(),
        })
}

pub(crate) fn rounded_f64_to_i64(value: f64) -> Result<i64, VerificationError> {
    if !value.is_finite() {
        return Err(VerificationError::InvalidConstruction {
            reason: "cannot quantize a non-finite f64".to_string(),
        });
    }
    value
        .round()
        .to_i64()
        .ok_or_else(|| VerificationError::InvalidConstruction {
            reason: "quantized f64 is outside i64 range".to_string(),
        })
}
