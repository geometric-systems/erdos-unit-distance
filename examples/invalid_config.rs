use erdos_unit_distance::{MultiquadraticConfig, UnitDistanceSet};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    match MultiquadraticConfig::builder(vec![4], 101, 1).build() {
        Ok(config) => {
            let _ = UnitDistanceSet::multiquadratic(config);
            Err("expected nonsquarefree generator 4 to be rejected".into())
        }
        Err(error) => {
            println!("{}: {error}", error.code());
            Ok(())
        }
    }
}
