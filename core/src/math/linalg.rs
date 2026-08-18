//! Small checked linear-system solvers used by exact low-dimensional geometry.

use super::EPSILON;
use crate::{PerceptionError, PerceptionResult};

/// Solves a square linear system by partial-pivot Gauss-Jordan elimination.
pub fn solve_linear_system(matrix: &[Vec<f32>], vector: &[f32]) -> PerceptionResult<Vec<f32>> {
    let size = matrix.len();
    if size == 0 || vector.len() != size || matrix.iter().any(|row| row.len() != size) {
        return Err(PerceptionError::NumericFailure {
            reason: "linear system must be non-empty and square".into(),
        });
    }
    let mut augmented = vec![vec![0.0; size + 1]; size];
    for row in 0..size {
        augmented[row][..size].copy_from_slice(&matrix[row]);
        augmented[row][size] = vector[row];
    }
    for pivot in 0..size {
        let best = (pivot..size)
            .max_by(|a, b| {
                augmented[*a][pivot]
                    .abs()
                    .partial_cmp(&augmented[*b][pivot].abs())
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
            .ok_or(PerceptionError::NonInvertibleMatrix)?;
        if augmented[best][pivot].abs() <= EPSILON || !augmented[best][pivot].is_finite() {
            return Err(PerceptionError::NonInvertibleMatrix);
        }
        augmented.swap(pivot, best);
        let divisor = augmented[pivot][pivot];
        for value in &mut augmented[pivot] {
            *value /= divisor;
        }
        for row in 0..size {
            if row != pivot {
                let factor = augmented[row][pivot];
                let pivot_values = augmented[pivot].clone();
                for (value, pivot_value) in augmented[row]
                    .iter_mut()
                    .skip(pivot)
                    .zip(pivot_values.iter().skip(pivot))
                {
                    *value -= factor * *pivot_value;
                }
            }
        }
    }
    Ok((0..size).map(|row| augmented[row][size]).collect())
}
