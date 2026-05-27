use erdos_unit_distance::UnitDistanceSet;
use erdos_unit_distance::utils::count_unit_distances;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let points = UnitDistanceSet::moser_spindle().generate_points(7)?;
    let point_arrays = points
        .iter()
        .copied()
        .map(|point| point.to_array())
        .collect::<Vec<_>>();
    let edge_count = count_unit_distances(&point_arrays, 1e-4);

    println!(
        "Moser spindle: {} points, {edge_count} unit-distance edges",
        points.len()
    );
    Ok(())
}
