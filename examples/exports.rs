use erdos_unit_distance::UnitDistanceSet;
use erdos_unit_distance::utils::{export_to_csv, export_to_obj, export_to_svg};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all("output")?;

    let points = UnitDistanceSet::moser_spindle().generate_points(7)?;
    let point_arrays = points
        .iter()
        .copied()
        .map(|point| point.to_array())
        .collect::<Vec<_>>();

    export_to_csv(&point_arrays, "output/moser_spindle.csv")?;
    export_to_svg(&point_arrays, "output/moser_spindle.svg", 800, 800)?;
    export_to_obj(&point_arrays, "output/moser_spindle.obj")?;

    println!("wrote CSV, SVG, and OBJ files to output/");
    Ok(())
}
