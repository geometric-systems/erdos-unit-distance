/// Generates a square grid of size `rows` x `cols`.
/// The spacing between adjacent points is exactly 1.0, producing many unit distances.
pub fn generate_square_grid(rows: usize, cols: usize) -> Vec<[f64; 2]> {
    (0..rows)
        .scan(0.0, |y, _| {
            let current = *y;
            *y += 1.0;
            Some(current)
        })
        .flat_map(|y| {
            (0..cols)
                .scan(0.0, |x, _| {
                    let current = *x;
                    *x += 1.0;
                    Some(current)
                })
                .map(move |x| [x, y])
        })
        .collect()
}

/// Generates a triangular grid arranged in a roughly hexagonal shape of `n_target` points.
/// The spacing between adjacent points is exactly 1.0.
pub fn generate_triangular_grid(n_target: usize) -> Vec<[f64; 2]> {
    // We expand outwards in layers of a triangular lattice
    // basis vectors: u = [1, 0], v = [0.5, sqrt(3)/2]
    const U: [f64; 2] = [1.0, 0.0];
    let v = [0.5, (3.0_f64).sqrt() / 2.0];

    (0_i32..)
        .flat_map(|layer| {
            (-layer..=layer)
                .flat_map(move |q| (-layer..=layer).map(move |r| (q, r)))
                .filter(move |&(q, r)| {
                    (q + r).abs() <= layer
                        && (q.abs() == layer
                            || r.abs() == layer
                            || (q + r).abs() == layer
                            || layer == 0)
                })
                .map(|(q, r)| {
                    let q = f64::from(q);
                    let r = f64::from(r);
                    [q * U[0] + r * v[0], q * U[1] + r * v[1]]
                })
        })
        .take(n_target)
        .collect()
}

/// Generates the Moser spindle (7 points, 11 unit-distance edges).
pub fn generate_moser_spindle() -> Vec<[f64; 2]> {
    // A: origin [0, 0]
    // D1 and D2 are the opposite vertices of two rhombi sharing A.
    // The angle alpha between D1 and D2 at the origin is arccos(5/6).
    let alpha = (5.0 / 6.0_f64).acos();
    let r = 3.0_f64.sqrt(); // Length of the diagonal of a unit rhombus

    let d1_angle = -alpha / 2.0;
    let d2_angle = alpha / 2.0;

    let a = [0.0, 0.0];
    let d1 = [r * d1_angle.cos(), r * d1_angle.sin()];
    let d2 = [r * d2_angle.cos(), r * d2_angle.sin()];

    let pi_6 = std::f64::consts::PI / 6.0;

    // Rhombus 1 vertices
    let b1 = [(d1_angle + pi_6).cos(), (d1_angle + pi_6).sin()];
    let c1 = [(d1_angle - pi_6).cos(), (d1_angle - pi_6).sin()];

    // Rhombus 2 vertices
    let b2 = [(d2_angle - pi_6).cos(), (d2_angle - pi_6).sin()];
    let c2 = [(d2_angle + pi_6).cos(), (d2_angle + pi_6).sin()];

    vec![a, b1, c1, d1, b2, c2, d2]
}

// Note: count_unit_distances lives in utils.rs as the single canonical implementation.
