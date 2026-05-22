# erdos-unit-distance Python bindings

Python bindings for certified unit-distance point sets and unit distance graphs.

```python
import erdos_unit_distance

points = erdos_unit_distance.generate_moser_spindle()
edges = erdos_unit_distance.count_unit_distances(points)

print(len(points), edges)
```

The package exposes classical constructors, native finite multiquadratic generation,
and certified multiquadratic output for applications that want machine-checkable
construction metadata.

Project repository: https://github.com/geometric-systems/erdos-unit-distance
