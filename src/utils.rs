use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

fn point_pairs(points: &[[f64; 2]]) -> impl Iterator<Item = (usize, usize)> {
    (0..points.len()).flat_map(|i| ((i + 1)..points.len()).map(move |j| (i, j)))
}

fn unit_distance_bounds(tolerance: f64) -> (f64, f64) {
    (
        (1.0 - tolerance) * (1.0 - tolerance),
        (1.0 + tolerance) * (1.0 + tolerance),
    )
}

fn squared_distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}

/// Counts the number of unit distances in a set of 2D points.
/// Two points are at unit distance if their Euclidean distance is within `tolerance` of 1.0.
///
/// # Examples
///
/// ```
/// use erdos_unit_distance::utils::count_unit_distances;
///
/// let points = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]];
/// assert_eq!(count_unit_distances(&points, 1e-9), 2);
/// ```
pub fn count_unit_distances(points: &[[f64; 2]], tolerance: f64) -> usize {
    let (lo, hi) = unit_distance_bounds(tolerance);
    point_pairs(points)
        .filter(|&(i, j)| {
            let dist_sq = squared_distance(points[i], points[j]);
            dist_sq >= lo && dist_sq <= hi
        })
        .count()
}

/// Finds all unit-distance pairs (edges) in the point set.
///
/// # Examples
///
/// ```
/// use erdos_unit_distance::utils::find_unit_distance_edges;
///
/// let points = [[0.0, 0.0], [1.0, 0.0], [0.0, 2.0]];
/// assert_eq!(find_unit_distance_edges(&points, 1e-9), vec![(0, 1)]);
/// ```
pub fn find_unit_distance_edges(points: &[[f64; 2]], tolerance: f64) -> Vec<(usize, usize)> {
    let (lo, hi) = unit_distance_bounds(tolerance);
    point_pairs(points)
        .filter(|&(i, j)| {
            let dist_sq = squared_distance(points[i], points[j]);
            dist_sq >= lo && dist_sq <= hi
        })
        .collect()
}

/// Exports the point set to a CSV file.
///
/// # Examples
///
/// ```
/// use erdos_unit_distance::utils::export_to_csv;
///
/// let path = std::env::temp_dir().join("erdos_unit_distance_export_to_csv.csv");
/// export_to_csv(&[[0.0, 0.0], [1.0, 0.0]], &path).unwrap();
/// std::fs::remove_file(path).unwrap();
/// ```
pub fn export_to_csv(points: &[[f64; 2]], path: impl AsRef<Path>) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "x,y")?;
    points
        .iter()
        .try_for_each(|p| writeln!(file, "{},{}", p[0], p[1]))
}

/// Exports the point set and its unit-distance edges to an OBJ file.
/// Points are represented as vertices and unit-distance edges as lines.
pub fn export_to_obj(points: &[[f64; 2]], path: impl AsRef<Path>) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "# erdos-unit-distance OBJ export")?;

    points
        .iter()
        .try_for_each(|p| writeln!(file, "v {} {} 0.0", p[0], p[1]))?;

    find_unit_distance_edges(points, 1e-4)
        .into_iter()
        .try_for_each(|(i, j)| writeln!(file, "l {} {}", i + 1, j + 1))
}

/// Exports the point set and its unit-distance edges to an SVG file.
/// It dynamically computes the bounding box and draws points and edges.
pub fn export_to_svg(
    points: &[[f64; 2]],
    path: impl AsRef<Path>,
    width: u32,
    height: u32,
) -> io::Result<()> {
    if points.is_empty() {
        return Ok(());
    }

    let (min_x, max_x, min_y, max_y) = points.iter().skip(1).fold(
        (points[0][0], points[0][0], points[0][1], points[0][1]),
        |(min_x, max_x, min_y, max_y), p| {
            (
                min_x.min(p[0]),
                max_x.max(p[0]),
                min_y.min(p[1]),
                max_y.max(p[1]),
            )
        },
    );

    let dx = max_x - min_x;
    let dy = max_y - min_y;
    let margin = 0.1 * (if dx > dy { dx } else { dy }).max(1.0);

    let min_x = min_x - margin;
    let max_x = max_x + margin;
    let min_y = min_y - margin;
    let max_y = max_y + margin;

    let view_w = max_x - min_x;
    let view_h = max_y - min_y;

    // Helper to map coordinates to SVG space
    let svg_width = f64::from(width);
    let svg_height = f64::from(height);
    let map_x = |x: f64| -> f64 { ((x - min_x) / view_w) * svg_width };
    let map_y = |y: f64| -> f64 {
        // Flip y-axis for SVG coordinate system
        (1.0 - (y - min_y) / view_h) * svg_height
    };

    let mut file = File::create(path)?;

    // Write SVG header
    writeln!(
        file,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">"#,
        width, height, width, height
    )?;

    // Background style - modern dark scheme
    writeln!(
        file,
        r##"  <rect width="100%" height="100%" fill="#121214"/>"##
    )?;

    find_unit_distance_edges(points, 1e-4)
        .into_iter()
        .try_for_each(|(i, j)| {
            writeln!(
                file,
                r##"  <line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="#3b82f6" stroke-width="1.5" opacity="0.6"/>"##,
                map_x(points[i][0]),
                map_y(points[i][1]),
                map_x(points[j][0]),
                map_y(points[j][1])
            )
        })?;

    points.iter().try_for_each(|p| {
        writeln!(
            file,
            r##"  <circle cx="{:.2}" cy="{:.2}" r="3.5" fill="#f43f5e" stroke="#121214" stroke-width="0.5"/>"##,
            map_x(p[0]),
            map_y(p[1])
        )
    })?;

    writeln!(file, "</svg>")?;
    Ok(())
}

/// Prunes a set of points to a target size `n_target` by greedily removing the points with the lowest unit-distance degrees.
/// It uses an adjacency list to track neighbor degrees in O(N^2) time.
pub fn prune_to_target_density(
    points: &[[f64; 2]],
    n_target: usize,
    tolerance: f64,
) -> Vec<[f64; 2]> {
    let n_initial = points.len();
    if n_initial <= n_target {
        return points.to_vec();
    }

    // Build adjacency lists. For each node, we store a list of neighbor indices.
    let mut adj = vec![Vec::new(); n_initial];
    let mut degrees = vec![0; n_initial];
    let mut active = vec![true; n_initial];
    let (lo, hi) = unit_distance_bounds(tolerance);

    point_pairs(points)
        .filter(|&(i, j)| {
            let dist_sq = squared_distance(points[i], points[j]);
            dist_sq >= lo && dist_sq <= hi
        })
        .for_each(|(i, j)| {
            adj[i].push(j);
            adj[j].push(i);
            degrees[i] += 1;
            degrees[j] += 1;
        });

    let mut active_count = n_initial;
    while active_count > n_target {
        let Some(min_idx) = (0..n_initial)
            .filter(|&i| active[i])
            .min_by_key(|&i| degrees[i])
        else {
            break;
        };

        // Deactivate min_idx
        active[min_idx] = false;
        active_count -= 1;

        // Decrement degrees of all active neighbors
        let neighbors_to_decrement: Vec<usize> = adj[min_idx]
            .iter()
            .copied()
            .filter(|&neighbor| active[neighbor] && degrees[neighbor] > 0)
            .collect();
        neighbors_to_decrement
            .into_iter()
            .for_each(|neighbor| degrees[neighbor] -= 1);
    }

    points
        .iter()
        .zip(active)
        .filter_map(|(&point, is_active)| is_active.then_some(point))
        .collect()
}
