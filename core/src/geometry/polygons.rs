//! Polygon operations that preserve winding and reject degenerate centroids.

use crate::{BoundingBox, PerceptionError, PerceptionResult, Point2, Polygon, Quad};

/// Winding direction for a non-degenerate polygon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Winding {
    /// Counter-clockwise vertex order.
    CounterClockwise,
    /// Clockwise vertex order.
    Clockwise,
    /// Area too small to determine a direction.
    Degenerate,
}

impl Polygon {
    /// Signed shoelace area; positive means counter-clockwise ordering.
    pub fn signed_area(&self) -> f32 {
        if self.points.len() < 3 {
            return 0.0;
        }
        self.points
            .iter()
            .zip(self.points.iter().cycle().skip(1))
            .take(self.points.len())
            .map(|(a, b)| a.x * b.y - b.x * a.y)
            .sum::<f32>()
            * 0.5
    }
    /// Absolute area.
    pub fn area(&self) -> f32 {
        self.signed_area().abs()
    }
    /// Boundary length.
    pub fn perimeter(&self) -> f32 {
        if self.points.len() < 2 {
            return 0.0;
        }
        self.points
            .iter()
            .zip(self.points.iter().cycle().skip(1))
            .take(self.points.len())
            .map(|(a, b)| a.distance(*b))
            .sum()
    }
    /// Winding classification.
    pub fn winding(&self) -> Winding {
        let area = self.signed_area();
        if area.abs() <= crate::math::EPSILON {
            Winding::Degenerate
        } else if area > 0.0 {
            Winding::CounterClockwise
        } else {
            Winding::Clockwise
        }
    }
    /// Area-weighted centroid.
    pub fn centroid(&self) -> PerceptionResult<Point2> {
        let area = self.signed_area();
        if area.abs() <= crate::math::EPSILON {
            return Err(PerceptionError::DegenerateGeometry {
                reason: "polygon centroid requires non-zero area".into(),
            });
        }
        let (x, y) = self
            .points
            .iter()
            .zip(self.points.iter().cycle().skip(1))
            .take(self.points.len())
            .fold((0.0, 0.0), |(sum_x, sum_y), (a, b)| {
                let cross = a.x * b.y - b.x * a.y;
                (sum_x + (a.x + b.x) * cross, sum_y + (a.y + b.y) * cross)
            });
        Ok(Point2::new(x / (6.0 * area), y / (6.0 * area)))
    }
    /// Even-odd point-in-polygon test; the boundary is considered inside.
    pub fn contains_point(&self, point: Point2) -> bool {
        if self.points.len() < 3 {
            return false;
        }
        let mut inside = false;
        for (a, b) in self
            .points
            .iter()
            .zip(self.points.iter().cycle().skip(1))
            .take(self.points.len())
        {
            if crate::geometry::point_on_segment(point, *a, *b, crate::math::EPSILON) {
                return true;
            }
            let crosses = (a.y > point.y) != (b.y > point.y);
            if crosses {
                let intersection_x = (b.x - a.x) * (point.y - a.y) / (b.y - a.y) + a.x;
                if point.x < intersection_x {
                    inside = !inside;
                }
            }
        }
        inside
    }
    /// Reverses point order without changing geometry.
    pub fn reversed(&self) -> Self {
        let mut points = self.points.clone();
        points.reverse();
        Self { points }
    }
}

impl Quad {
    /// Verifies distinct corners, non-zero area, and consistent convex winding.
    pub fn validate(self) -> PerceptionResult<()> {
        let polygon = self.polygon();
        if polygon.area() <= crate::math::EPSILON {
            return Err(PerceptionError::DegenerateGeometry {
                reason: "quad has near-zero area".into(),
            });
        }
        let points = self.points;
        let mut sign = 0.0f32;
        for index in 0..4 {
            let a = points[index];
            let b = points[(index + 1) % 4];
            let c = points[(index + 2) % 4];
            let cross = (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x);
            if cross.abs() <= crate::math::EPSILON {
                return Err(PerceptionError::DegenerateGeometry {
                    reason: "quad has collinear corners".into(),
                });
            }
            if sign == 0.0 {
                sign = cross.signum();
            } else if sign.signum() != cross.signum() {
                return Err(PerceptionError::DegenerateGeometry {
                    reason: "quad is concave or self-intersecting".into(),
                });
            }
        }
        Ok(())
    }
    /// Bounds around all four points.
    pub fn bounding_box(self) -> PerceptionResult<BoundingBox> {
        self.polygon().bounding_box()
    }
}
