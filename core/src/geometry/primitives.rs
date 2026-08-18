//! Geometry primitive helpers that augment the stable core data model.

use crate::{
    BoundingBox, PerceptionError, PerceptionResult, Point2, Point3, Polygon, Quad, Segment2,
};

/// Circle in a declared 2D coordinate system.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Circle {
    /// Center point.
    pub center: Point2,
    /// Non-negative radius.
    pub radius: f32,
}

/// Rigid 2D pose using translation and radians counter-clockwise rotation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose2D {
    /// Position component.
    pub translation: Point2,
    /// Counter-clockwise orientation in radians.
    pub rotation_radians: f32,
}

/// Placeholder 3D pose for future calibrated camera and reconstruction modules.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose3D {
    /// Position component.
    pub translation: Point3,
    /// Unit quaternion in xyzw ordering when populated by a future module.
    pub rotation_xyzw: [f32; 4],
}

impl Point2 {
    /// Creates a point from coordinates.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    /// Euclidean distance to another point.
    pub fn distance(self, other: Self) -> f32 {
        (self.x - other.x).hypot(self.y - other.y)
    }
    /// Vector angle from this point to another in radians.
    pub fn angle_to(self, other: Self) -> f32 {
        (other.y - self.y).atan2(other.x - self.x)
    }
}

impl Segment2 {
    /// Length of the segment.
    pub fn length(self) -> f32 {
        self.start.distance(self.end)
    }
    /// Axis-aligned bounding box.
    pub fn bounding_box(self) -> BoundingBox {
        BoundingBox {
            x: self.start.x.min(self.end.x),
            y: self.start.y.min(self.end.y),
            width: (self.start.x - self.end.x).abs(),
            height: (self.start.y - self.end.y).abs(),
        }
    }
}

impl BoundingBox {
    /// Creates non-negative axis-aligned bounds.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> PerceptionResult<Self> {
        if ![x, y, width, height].iter().all(|value| value.is_finite())
            || width < 0.0
            || height < 0.0
        {
            return Err(PerceptionError::DegenerateGeometry {
                reason: "bounding box must be finite with non-negative extent".into(),
            });
        }
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }
    /// Returns whether a point lies inside or on the box.
    pub fn contains(self, point: Point2) -> bool {
        point.x >= self.x
            && point.y >= self.y
            && point.x <= self.x + self.width
            && point.y <= self.y + self.height
    }
}

impl Polygon {
    /// Axis-aligned bounds of polygon vertices.
    pub fn bounding_box(&self) -> PerceptionResult<BoundingBox> {
        if self.points.is_empty() {
            return Err(PerceptionError::InsufficientPoints {
                required: 1,
                actual: 0,
            });
        }
        let (mut min_x, mut max_x, mut min_y, mut max_y) = (
            self.points[0].x,
            self.points[0].x,
            self.points[0].y,
            self.points[0].y,
        );
        for point in &self.points[1..] {
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }
        BoundingBox::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }
}

impl Quad {
    /// Returns the edges in declared corner order.
    pub fn edges(self) -> [Segment2; 4] {
        let [a, b, c, d] = self.points;
        [
            Segment2 { start: a, end: b },
            Segment2 { start: b, end: c },
            Segment2 { start: c, end: d },
            Segment2 { start: d, end: a },
        ]
    }
    /// Converts the quad to a polygon.
    pub fn polygon(self) -> Polygon {
        Polygon {
            points: self.points.to_vec(),
        }
    }
}
