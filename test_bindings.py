import erdos_unit_distance

def test_moser_spindle():
    print("Testing Moser Spindle...")
    pts = erdos_unit_distance.generate_moser_spindle()
    print(f"Generated {len(pts)} points: {pts}")
    assert len(pts) == 7, f"Expected 7 points, got {len(pts)}"
    
    # Moser Spindle has exactly 11 unit distance edges
    edges = erdos_unit_distance.count_unit_distances(pts, 1e-5)
    print(f"Number of unit distances: {edges}")
    assert edges == 11, f"Expected 11 unit distance edges, got {edges}"
    print("Moser Spindle test passed!")

def test_square_grid():
    print("Testing Square Grid...")
    # 5x5 grid has 25 points
    pts = erdos_unit_distance.generate_square_grid(5, 5)
    assert len(pts) == 25, f"Expected 25 points, got {len(pts)}"
    # 5x5 grid has 5*4 (horiz) + 5*4 (vert) = 40 unit distances
    edges = erdos_unit_distance.count_unit_distances(pts, 1e-5)
    print(f"Number of unit distances in 5x5 grid: {edges}")
    assert edges == 40, f"Expected 40 unit distances, got {edges}"
    print("Square Grid test passed!")

def test_triangular_grid():
    print("Testing Triangular Grid...")
    # Request 15 points
    pts = erdos_unit_distance.generate_triangular_grid(15)
    assert len(pts) == 15, f"Expected 15 points, got {len(pts)}"
    edges = erdos_unit_distance.count_unit_distances(pts, 1e-5)
    print(f"Number of unit distances in triangular grid: {edges}")
    print("Triangular Grid test passed!")

def test_multiquadratic():
    print("Testing native multiquadratic prototype...")
    # Generators [5, 17], split prime 101, k=1, target 50
    pts = erdos_unit_distance.generate_multiquadratic([5, 17], 101, 1, 50)
    assert len(pts) == 50, f"Expected 50 points, got {len(pts)}"
    edges = erdos_unit_distance.count_unit_distances(pts, 1e-5)
    print(f"Number of unit distances in native multiquadratic prototype: {edges}")
    print(f"Ratio of unit distances to points: {edges / len(pts):.3f}")
    assert edges > 0, "Expected some unit distances"
    print("Native multiquadratic prototype test passed!")

def test_invalid_multiquadratic():
    print("Testing invalid native multiquadratic input...")
    try:
        erdos_unit_distance.generate_multiquadratic([4], 101, 1, 10)
    except RuntimeError as err:
        assert "invalid generator 4" in str(err)
        print("Invalid native multiquadratic test passed!")
        return
    raise AssertionError("Expected invalid generator to raise RuntimeError")

def test_certified_multiquadratic():
    print("Testing certified native multiquadratic prototype...")
    certified = erdos_unit_distance.generate_multiquadratic_certified(
        [5, 17], 101, 1, 20
    )
    pts = certified["points"]
    certified_edges = certified["certified_edges"]
    audit_edges = certified["audit_edges"]
    assert certified["verified"], "Expected certificate verification to pass"
    assert len(pts) == 20, f"Expected 20 points, got {len(pts)}"
    assert len(certified_edges) > 0, "Expected certified construction edges"
    assert len(audit_edges) >= len(certified_edges), "Expected audit edges to cover certified edges"
    independent_edges = erdos_unit_distance.count_unit_distances(pts, 1e-4)
    assert independent_edges == len(audit_edges), (
        f"Expected audit edges to match independent checker, got "
        f"{len(audit_edges)} audit and {independent_edges} independent"
    )
    print("Certified native multiquadratic prototype test passed!")

if __name__ == "__main__":
    test_moser_spindle()
    print("-" * 40)
    test_square_grid()
    print("-" * 40)
    test_triangular_grid()
    print("-" * 40)
    test_multiquadratic()
    print("-" * 40)
    test_invalid_multiquadratic()
    print("-" * 40)
    test_certified_multiquadratic()
    print("-" * 40)
    print("All python bindings tests passed successfully!")
