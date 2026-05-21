use erdos_unit_distance::UnitDistanceSet;
use erdos_unit_distance::utils::{export_to_csv, export_to_obj, export_to_svg};
use std::fs;
use std::path::Path;

fn arrays(points: &[erdos_unit_distance::Point2]) -> Vec<[f64; 2]> {
    points
        .iter()
        .copied()
        .map(|point| point.to_array())
        .collect()
}

fn main() {
    println!("=== Erdős Unit-Distance Certified Export ===");

    if !Path::new("output").exists() {
        fs::create_dir("output").expect("Failed to create output directory");
    }

    println!("\nGenerating Moser Spindle...");
    let spindle = UnitDistanceSet::moser_spindle()
        .generate_certified(7)
        .expect("generation failed");
    let spindle_report = spindle.certificate.verify().expect("certificate failed");
    println!(
        "Moser Spindle: {} points, {} certified unit-distance edges.",
        spindle_report.point_count, spindle_report.edge_count
    );
    let spindle_points = arrays(&spindle.points);
    export_to_csv(&spindle_points, "output/moser_spindle.csv").unwrap();
    export_to_obj(&spindle_points, "output/moser_spindle.obj").unwrap();
    export_to_svg(&spindle_points, "output/moser_spindle.svg", 800, 800).unwrap();

    println!("\nGenerating 10x10 Square Grid...");
    let grid = UnitDistanceSet::square_grid(10, 10)
        .generate_certified(100)
        .expect("generation failed");
    let grid_report = grid.certificate.verify().expect("certificate failed");
    println!(
        "Square Grid: {} points, {} certified unit-distance edges.",
        grid_report.point_count, grid_report.edge_count
    );
    let grid_points = arrays(&grid.points);
    export_to_csv(&grid_points, "output/square_grid_100.csv").unwrap();
    export_to_obj(&grid_points, "output/square_grid_100.obj").unwrap();
    export_to_svg(&grid_points, "output/square_grid_100.svg", 800, 800).unwrap();

    println!("\nGenerating native finite multiquadratic prototype (size 100)...");
    let algebraic = UnitDistanceSet::try_new_multiquadratic(vec![5, 17], 101, 1)
        .expect("invalid algebraic configuration")
        .generate_certified(100)
        .expect("generation failed");
    let algebraic_report = algebraic.certificate.verify().expect("certificate failed");
    println!(
        "Native multiquadratic prototype (100): {} points, {} certified unit-distance edges.",
        algebraic_report.point_count, algebraic_report.edge_count
    );
    let algebraic_points = arrays(&algebraic.points);
    export_to_csv(&algebraic_points, "output/multiquadratic_100.csv").unwrap();
    export_to_obj(&algebraic_points, "output/multiquadratic_100.obj").unwrap();
    export_to_svg(&algebraic_points, "output/multiquadratic_100.svg", 800, 800).unwrap();

    println!("\nCertified exports written to the 'output' directory.");
}
