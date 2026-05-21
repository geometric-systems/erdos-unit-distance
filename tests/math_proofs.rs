use erdos_unit_distance::algebraic::field::{FieldElement, MultiQuadraticField};
use erdos_unit_distance::algebraic::window::find_lattice_points_in_polydisk;
use erdos_unit_distance::utils::{find_unit_distance_edges, prune_to_target_density};

#[test]
fn test_pruning_disconnected_graph() {
    // 4 points: A and B are connected by a unit-distance edge.
    // C and D are completely isolated.
    let points = vec![
        [0.0, 0.0],   // A
        [1.0, 0.0],   // B
        [10.0, 10.0], // C
        [20.0, 20.0], // D
    ];

    // Prune down to 2 points
    let pruned = prune_to_target_density(&points, 2, 1e-4);
    assert_eq!(pruned.len(), 2);

    // It should keep A and B (since they have degree 1) and discard C and D (degree 0)
    let edges = find_unit_distance_edges(&pruned, 1e-4);
    assert_eq!(edges.len(), 1);

    // Verify A and B are preserved (within floating point tolerances)
    let has_origin = pruned
        .iter()
        .any(|p| p[0].abs() < 1e-5 && p[1].abs() < 1e-5);
    let has_one = pruned
        .iter()
        .any(|p| (p[0] - 1.0).abs() < 1e-5 && p[1].abs() < 1e-5);
    assert!(has_origin, "Should contain [0, 0]");
    assert!(has_one, "Should contain [1, 0]");
}

#[test]
fn test_pruning_dense_subgraph() {
    // 5 points:
    // A, B, C form a unit-distance triangle (each has degree 2).
    // D, E form a single unit-distance edge (each has degree 1).
    let sqrt_3_over_2 = (3.0_f64).sqrt() / 2.0;
    let points = vec![
        [0.0, 0.0],           // A
        [1.0, 0.0],           // B
        [0.5, sqrt_3_over_2], // C
        [10.0, 0.0],          // D
        [11.0, 0.0],          // E
    ];

    // Prune down to 3 points
    let pruned = prune_to_target_density(&points, 3, 1e-4);
    assert_eq!(pruned.len(), 3);

    // It should keep the triangle A, B, C (higher degree density) and discard D, E
    let edges = find_unit_distance_edges(&pruned, 1e-4);
    assert_eq!(edges.len(), 3); // A triangle has 3 unit distance edges

    // Verify D and E are discarded
    for p in &pruned {
        assert!(
            p[0] < 5.0,
            "Pruned set should only contain the dense triangle, but got point {:?}",
            p
        );
    }
}

#[test]
fn test_pruning_edge_cases_empty_and_trivial() {
    // 1. Empty input
    let empty_points: Vec<[f64; 2]> = Vec::new();
    let pruned_empty = prune_to_target_density(&empty_points, 5, 1e-4);
    assert!(
        pruned_empty.is_empty(),
        "Pruning empty points should return empty"
    );

    let pruned_empty_zero = prune_to_target_density(&empty_points, 0, 1e-4);
    assert!(
        pruned_empty_zero.is_empty(),
        "Pruning empty points to target 0 should return empty"
    );

    // 2. n_target >= n_initial
    let points = vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]];
    let pruned_large = prune_to_target_density(&points, 5, 1e-4);
    assert_eq!(
        pruned_large.len(),
        3,
        "Should return all points if target is larger than initial size"
    );
    assert_eq!(pruned_large[0], [0.0, 0.0]);
    assert_eq!(pruned_large[1], [1.0, 0.0]);
    assert_eq!(pruned_large[2], [2.0, 0.0]);

    // 3. n_target == 0
    let pruned_zero = prune_to_target_density(&points, 0, 1e-4);
    assert!(
        pruned_zero.is_empty(),
        "Pruning to target 0 should return empty"
    );
}

#[test]
fn test_pruning_component_density_comparison() {
    // We set up two disconnected components:
    // Component 1: A 3-cycle (triangle)
    //   A: [0, 0]
    //   B: [1, 0]
    //   C: [0.5, sqrt(3)/2]
    // Component 2: A 4-cycle (square)
    //   D: [10, 10]
    //   E: [11, 10]
    //   F: [11, 11]
    //   G: [10, 11]
    // Total vertices = 7. All start with degree 2.
    // If we prune from 7 down to 4:
    // The algorithm will pick one vertex (say A) to remove.
    // This drops degrees of B and C to 1, while D, E, F, G remain at degree 2.
    // Next, the algorithm will prune B (degree 1), leaving C at degree 0.
    // Next, it will prune C (degree 0).
    // Thus, the remaining 4 vertices must be exactly D, E, F, G (the 4-cycle).
    let sqrt_3_over_2 = (3.0_f64).sqrt() / 2.0;
    let points = vec![
        [0.0, 0.0],           // 0: A
        [1.0, 0.0],           // 1: B
        [0.5, sqrt_3_over_2], // 2: C
        [10.0, 10.0],         // 3: D
        [11.0, 10.0],         // 4: E
        [11.0, 11.0],         // 5: F
        [10.0, 11.0],         // 6: G
    ];

    let pruned = prune_to_target_density(&points, 4, 1e-4);
    assert_eq!(pruned.len(), 4);

    // Verify all remaining points belong to Component 2 (the 4-cycle)
    for p in &pruned {
        assert!(
            p[0] >= 9.0 && p[1] >= 9.0,
            "Pruned set must isolate the intact 4-cycle, but got point {:?}",
            p
        );
    }

    let edges = find_unit_distance_edges(&pruned, 1e-4);
    assert_eq!(
        edges.len(),
        4,
        "The isolated 4-cycle should have exactly 4 unit-distance edges"
    );
}

#[test]
fn test_multiquadratic_density_and_polydisk_bound() {
    let field = MultiQuadraticField::try_new(&[5, -1]).unwrap(); // Q(sqrt(5), i)
    let p = 41;
    let k = 1;
    let r_polydisk = 1.5;

    let elements = find_lattice_points_in_polydisk(&field, p, k, 3, r_polydisk, 50).unwrap();
    assert!(!elements.is_empty());

    // Verify that every generated element satisfies the polydisk bound:
    // For every complex embedding, the absolute value is <= r_polydisk
    for elem in &elements {
        let embs = elem.embeddings(&field);
        for &emb in &embs {
            assert!(
                emb.norm() <= r_polydisk + 1e-5,
                "Lattice point embedding {:.6} + {:.6}i exceeds polydisk radius {}",
                emb.re,
                emb.im,
                r_polydisk
            );
        }
    }
}

#[test]
fn test_minkowski_and_lemma_2_3_verification() {
    // K = Q(sqrt(5), sqrt(17), i)
    let field = MultiQuadraticField::try_new(&[5, 17, -1]).unwrap();
    let p = 101;
    let k = 1;
    let r_polydisk = 2.0;

    // 1. Generate lattice points in the polydisk
    let elements = find_lattice_points_in_polydisk(&field, p, k, 3, r_polydisk, 100).unwrap();
    assert!(!elements.is_empty());

    // 2. Project points to the plane and prune to a target size of 20
    let alpha = FieldElement::one(&field);
    let projected: Vec<[f64; 2]> = elements
        .iter()
        .map(|e| {
            erdos_unit_distance::algebraic::lattice::project_to_plane(e, &field, &alpha).unwrap()
        })
        .collect();

    // Deduplicate projected points
    let mut unique_projected: Vec<[f64; 2]> = Vec::new();
    let mut unique_elements = Vec::new();
    for (i, &p_coord) in projected.iter().enumerate() {
        let mut is_dup = false;
        for &existing in &unique_projected {
            let dx = p_coord[0] - existing[0];
            let dy = p_coord[1] - existing[1];
            if (dx * dx + dy * dy).sqrt() < 1e-7 {
                is_dup = true;
                break;
            }
        }
        if !is_dup {
            unique_projected.push(p_coord);
            unique_elements.push(elements[i].clone());
        }
    }

    let n = unique_projected.len();
    assert!(n > 0);

    // 3. Find all unit distance edges
    let edges = find_unit_distance_edges(&unique_projected, 1e-4);

    // 4. Verify Lemma 2.3: Any unit distance edge in the plane corresponds to a difference
    // in the number field whose complex embeddings all have modulus exactly 1.0.
    for (i, j) in edges {
        let diff = unique_elements[i].sub(&unique_elements[j]);
        let embs = diff.embeddings(&field);

        // Every embedding of the difference must have modulus 1.0 (within float tolerance)
        for &emb in &embs {
            let mod_val = emb.norm();
            assert!(
                (mod_val - 1.0).abs() < 1e-5,
                "Embedding of algebraic difference between unit-distance points must have modulus 1.0, but got {:.6}",
                mod_val
            );
        }
    }
}
