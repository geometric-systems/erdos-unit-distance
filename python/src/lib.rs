// Copyright (c) 2026 Lars Brusletto
// Under MIT and Apache-2.0 Licenses.
//
// Python bindings wrapping the erdos-unit-distance Rust library.
// Based on the disproof of the Erdős unit distance conjecture by Lijie Chen,
// Mark Sellke, and Mehtaab Sawhney (2026), with explicit exposition by Will Sawin
// (arXiv:2605.20579) and Noga Alon et al. (arXiv:2605.20695).

use ::erdos_unit_distance::classical::{
    generate_moser_spindle as rust_moser, generate_square_grid as rust_square,
    generate_triangular_grid as rust_triangular,
};
use ::erdos_unit_distance::utils::count_unit_distances as rust_count_distances;
use ::erdos_unit_distance::{ConstructionCertificate, UnitDistanceSet};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Generate a square grid of points of size `rows` x `cols`.
///
/// Returns a list of coordinates (tuples of float).
#[pyfunction]
#[pyo3(signature = (rows, cols))]
pub fn generate_square_grid(rows: usize, cols: usize) -> PyResult<Vec<(f64, f64)>> {
    let pts = rust_square(rows, cols);
    Ok(pts.into_iter().map(|p| (p[0], p[1])).collect())
}

/// Generate a triangular grid of points of size `n_target`.
///
/// Returns a list of coordinates (tuples of float).
#[pyfunction]
#[pyo3(signature = (n_target))]
pub fn generate_triangular_grid(n_target: usize) -> PyResult<Vec<(f64, f64)>> {
    let pts = rust_triangular(n_target);
    Ok(pts.into_iter().map(|p| (p[0], p[1])).collect())
}

/// Generate the Moser spindle (7 points).
///
/// Returns a list of coordinates (tuples of float).
#[pyfunction]
pub fn generate_moser_spindle() -> PyResult<Vec<(f64, f64)>> {
    let pts = rust_moser();
    Ok(pts.into_iter().map(|p| (p[0], p[1])).collect())
}

/// Generate a point set with the native finite multiquadratic prototype.
///
/// * `generators`: List of positive squarefree integers generating the real subfield.
/// * `split_prime`: A prime split completely in K.
/// * `k`: Exponent/depth parameter.
/// * `n_target`: Target number of points.
///
/// Returns a list of coordinates (tuples of float).
#[pyfunction]
#[pyo3(signature = (generators, split_prime, k, n_target))]
pub fn generate_multiquadratic(
    generators: Vec<i64>,
    split_prime: i64,
    k: usize,
    n_target: usize,
) -> PyResult<Vec<(f64, f64)>> {
    let builder = UnitDistanceSet::try_new_multiquadratic(generators, split_prime, k)
        .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;
    let pts = builder
        .generate_points(n_target)
        .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;
    Ok(pts.into_iter().map(|p| (p.x, p.y)).collect())
}

/// Generate the native finite multiquadratic prototype with a machine-checkable certificate.
///
/// Returns a dictionary with named point, certificate edge, audit edge, and report fields.
#[pyfunction]
#[pyo3(signature = (generators, split_prime, k, n_target))]
pub fn generate_multiquadratic_certified(
    py: Python<'_>,
    generators: Vec<i64>,
    split_prime: i64,
    k: usize,
    n_target: usize,
) -> PyResult<PyObject> {
    let builder = UnitDistanceSet::try_new_multiquadratic(generators, split_prime, k)
        .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;
    let certified = builder
        .generate_certified(n_target)
        .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;
    let report = certified
        .verify()
        .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;

    let certified_edges: Vec<(usize, usize)> = match certified.certificate {
        ConstructionCertificate::Multiquadratic(certificate) => certificate
            .section2
            .construction_edges
            .into_iter()
            .map(|edge| edge.endpoints)
            .collect(),
        _ => Vec::new(),
    };
    let dict = PyDict::new_bound(py);
    dict.set_item(
        "points",
        certified
            .points
            .into_iter()
            .map(|point| (point.x, point.y))
            .collect::<Vec<_>>(),
    )?;
    dict.set_item("certified_edges", certified_edges)?;
    dict.set_item("audit_edges", certified.audit.edges)?;
    dict.set_item("verified", true)?;
    dict.set_item("point_count", report.point_count)?;
    dict.set_item("certified_edge_count", report.certified_edge_count)?;
    dict.set_item("audit_edge_count", report.audit_edge_count)?;
    dict.set_item("construction", report.construction)?;
    Ok(dict.into())
}

/// Helper to count the number of unit distances in a set of points.
///
/// * `points`: A list of coordinates (tuples of float).
/// * `tolerance`: The absolute tolerance (e.g. 1e-5).
#[pyfunction]
#[pyo3(signature = (points, tolerance = 1e-5))]
pub fn count_unit_distances(points: Vec<(f64, f64)>, tolerance: f64) -> PyResult<usize> {
    let rust_pts: Vec<[f64; 2]> = points.into_iter().map(|(x, y)| [x, y]).collect();
    let count = rust_count_distances(&rust_pts, tolerance);
    Ok(count)
}

/// Python bindings for the erdos-unit-distance library
#[pymodule]
pub fn erdos_unit_distance(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate_square_grid, m)?)?;
    m.add_function(wrap_pyfunction!(generate_triangular_grid, m)?)?;
    m.add_function(wrap_pyfunction!(generate_moser_spindle, m)?)?;
    m.add_function(wrap_pyfunction!(generate_multiquadratic, m)?)?;
    m.add_function(wrap_pyfunction!(generate_multiquadratic_certified, m)?)?;
    m.add_function(wrap_pyfunction!(count_unit_distances, m)?)?;
    Ok(())
}
