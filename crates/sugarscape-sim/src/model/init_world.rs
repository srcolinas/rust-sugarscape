use std::collections::HashSet;

use crate::config::WorldParams;

use super::geometry::{coord_to_idx, euclidean_distance};
use super::Model;

impl Model {
    pub(super) fn init_world(world: &WorldParams) -> (Vec<f32>, Vec<i32>) {
        let max_capacity = world.capacity_distribution.max_capacity;
        let reduction_factor = world.capacity_distribution.reduction_factor;
        let num_cells = world.width as usize * world.height as usize;

        let mut capacities = vec![0.0; num_cells];
        let mut peaks = HashSet::new();
        for peak in world.capacity_distribution.peaks.iter() {
            let idx = coord_to_idx(peak.x, peak.y, world.width);
            capacities[idx] = max_capacity;
            peaks.insert((peak.x, peak.y));
        }

        for x in 0..world.width {
            for y in 0..world.height {
                if !peaks.contains(&(x, y)) {
                    let mut distance: f32 = f32::INFINITY;
                    for (px, py) in peaks.iter() {
                        let current = euclidean_distance(x, y, *px, *py);
                        distance = f32::min(distance, current);
                    }
                    let idx = coord_to_idx(x, y, world.width);
                    let capacity = f32::max(0.0, max_capacity - distance * reduction_factor);
                    capacities[idx] = capacity;
                }
            }
        }
        (capacities, vec![0; num_cells])
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use p_test::p_test;

    use crate::config::{CellCapacityDistribution, CellPosition, WorldParams};

    use super::super::geometry::coord_to_idx;
    use super::super::Model;

    #[p_test(
        "peak(1,1)", (2.0, 0.5, 1, 1, 2.0),
        "peak(3,3)", (2.0, 0.5, 3, 3, 2.0),
        "orthogonal(0,1)", (2.0, 0.5, 0, 1, 1.5),
        "orthogonal(1,0)", (2.0, 0.5, 1, 0, 1.5),
        "orthogonal(1,2)", (2.0, 0.5, 1, 2, 1.5),
        "orthogonal(2,1)", (2.0, 0.5, 2, 1, 1.5),
        "orthogonal(2,3)", (2.0, 0.5, 2, 3, 1.5),
        "orthogonal(3,2)", (2.0, 0.5, 3, 2, 1.5),
        "orthogonal(3,4)", (2.0, 0.5, 3, 4, 1.5),
        "orthogonal(4,3)", (2.0, 0.5, 4, 3, 1.5),
        "diagonal(0,0)", (2.0, 0.5, 0, 0, 1.292893),
        "diagonal(2,0)", (2.0, 0.5, 2, 0, 1.292893),
        "diagonal(0,2)", (2.0, 0.5, 0, 2, 1.292893),
        "diagonal(2,2)", (2.0, 0.5, 2, 2, 1.292893),
        "diagonal(4,2)", (2.0, 0.5, 4, 2, 1.292893),
        "diagonal(2,4)", (2.0, 0.5, 2, 4, 1.292893),
        "diagonal(4,4)", (2.0, 0.5, 4, 4, 1.292893),
        // Remaining non-peak cells (nearest-peak distance 2, √5, or √10)
        "dist2(3,1)", (2.0, 0.5, 3, 1, 1.0),
        "dist2(1,3)", (2.0, 0.5, 1, 3, 1.0),
        "dist_sqrt5(3,0)", (2.0, 0.5, 3, 0, 0.881966),
        "dist_sqrt5(4,1)", (2.0, 0.5, 4, 1, 0.881966),
        "dist_sqrt5(0,3)", (2.0, 0.5, 0, 3, 0.881966),
        "dist_sqrt5(1,4)", (2.0, 0.5, 1, 4, 0.881966),
        "dist_sqrt10(4,0)", (2.0, 0.5, 4, 0, 0.418861),
        "dist_sqrt10(0,4)", (2.0, 0.5, 0, 4, 0.418861),
    )]
    fn test_capacities_for_two_peaks_on_a_5x5_grid(
        max_capacity: f32,
        reduction_factor: f32,
        x: u8,
        y: u8,
        expected_value: f32,
    ) {
        // 5x5: P at (1,1) and (3,3); x→ columns, y↓ rows
        //       x: 0 1 2 3 4
        //     y0: . . . . .
        //     y1: . P . . .
        //     y2: . . . . .
        //     y3: . . . P .
        //     y4: . . . . .

        let width = 5;
        let height = 5;
        let peaks = [CellPosition { x: 1, y: 1 }, CellPosition { x: 3, y: 3 }];
        let model = Model::from_default(|w, a| {
            (
                WorldParams {
                    width,
                    height,
                    capacity_distribution: CellCapacityDistribution {
                        reduction_factor,
                        max_capacity,
                        peaks: peaks
                            .iter()
                            .map(|p| CellPosition { x: p.x, y: p.y })
                            .collect(),
                    },
                    ..w
                },
                a,
            )
        });
        let capacities = model.capacities;
        let idx = coord_to_idx(x, y, width);
        assert_relative_eq!(capacities[idx], expected_value, epsilon = 1e-5);
    }
}
