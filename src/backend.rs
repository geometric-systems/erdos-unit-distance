//! Backend provenance for certificate metadata.

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackendMode {
    NativeMultiquadratic,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackendProvenance {
    pub name: String,
    pub mode: BackendMode,
}

impl BackendProvenance {
    pub fn native_multiquadratic() -> Self {
        Self {
            name: NativeMultiquadraticBackend::NAME.to_string(),
            mode: BackendMode::NativeMultiquadratic,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NativeMultiquadraticBackend;

impl NativeMultiquadraticBackend {
    pub const NAME: &'static str = "native-multiquadratic";
}
