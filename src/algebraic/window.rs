use crate::algebraic::field::{FieldElement, MultiQuadraticField};
use crate::algebraic::lattice::project_to_plane;
use crate::error::GenerationError;

/// Generates bounded algebraic candidates whose complex embeddings lie within
/// the polydisk of radius R (i.e. |sigma_j(x)| <= R for all j).
/// This uses a BFS from the origin and translates by certified unit-modulus
/// elements. It is a bounded search, not an exhaustive lattice enumeration.
pub fn find_lattice_points_in_polydisk(
    field: &MultiQuadraticField,
    p: i64,
    k: usize,
    max_prime_exponent: usize,
    r_polydisk: f64,
    max_points: usize,
) -> Result<Vec<FieldElement>, GenerationError> {
    let units =
        crate::algebraic::units::generate_unit_modulus_elements(field, p, k, max_prime_exponent)?;

    // BFS queue and visited set
    let mut queue = std::collections::VecDeque::new();
    let mut visited = Vec::new();

    // Start with the origin
    let origin = FieldElement::zero(field);
    queue.push_back(origin.clone());
    visited.push(origin);

    while let Some(curr) = queue.pop_front() {
        if visited.len() >= max_points {
            break;
        }

        // Try translating by each unit-modulus element
        for u in &units {
            // We can translate by +u and -u
            let next_pos = curr.add(u);
            let next_neg = curr.sub(u);

            for next in &[next_pos, next_neg] {
                if !next
                    .embeddings(field)
                    .into_iter()
                    .all(|emb| emb.norm() <= r_polydisk + 1e-7)
                {
                    continue;
                }

                if !visited.iter().any(|v| coefficient_distance(next, v) < 1e-7) {
                    visited.push(next.clone());
                    queue.push_back(next.clone());
                    if visited.len() >= max_points {
                        break;
                    }
                }
            }
            if visited.len() >= max_points {
                break;
            }
        }
    }

    Ok(visited)
}

/// Projects a set of field elements to the 2D plane and returns unique coordinates.
pub fn project_and_deduplicate(
    elements: &[FieldElement],
    field: &MultiQuadraticField,
    tolerance: f64,
) -> Result<Vec<[f64; 2]>, GenerationError> {
    let alpha = FieldElement::one(field);
    elements.iter().try_fold(Vec::new(), |mut projected, elem| {
        let coords = project_to_plane(elem, field, &alpha)?;
        if !projected
            .iter()
            .any(|existing| point_distance(coords, *existing) < tolerance)
        {
            projected.push(coords);
        }
        Ok(projected)
    })
}

fn coefficient_distance(left: &FieldElement, right: &FieldElement) -> f64 {
    if left == right { 0.0 } else { f64::INFINITY }
}

fn point_distance(left: [f64; 2], right: [f64; 2]) -> f64 {
    let dx = left[0] - right[0];
    let dy = left[1] - right[1];
    (dx * dx + dy * dy).sqrt()
}
