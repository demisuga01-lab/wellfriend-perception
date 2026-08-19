//! Geometry-track smoothing and stability evidence without optical-flow dependence.

use wellfriend_perception_core::{
    Confidence, DetectionSource, FrameIndex, PerceptionResult, Point2, Quad, Score, TemporalState,
};

use crate::detection::document_quad_candidate;

/// Configuration for deterministic exponential quad tracking.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuadTemporalConfig {
    /// Contribution of the newest candidate to the smoothed geometry.
    pub smoothing_alpha: f32,
    /// History depth used to compute stability evidence.
    pub history_length: usize,
    /// Consecutive misses before a track is declared lost.
    pub max_lost_frames: u32,
    /// Stability score required to mark a tracked candidate stable.
    pub stable_threshold: f32,
}

impl Default for QuadTemporalConfig {
    fn default() -> Self {
        Self {
            smoothing_alpha: 0.35,
            history_length: 8,
            max_lost_frames: 4,
            stable_threshold: 0.78,
        }
    }
}

/// Temporal output retaining the smoothed geometry separately from state metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct TemporalUpdate {
    /// Smoothed candidate when the track is active.
    pub quad: Option<Quad>,
    /// Machine-readable state for later capture readiness.
    pub state: TemporalState,
}

/// Deterministic quad tracker based on geometry agreement rather than optical flow.
#[derive(Clone, Debug, Default)]
pub struct QuadTemporalTracker {
    /// Public smoothing and track-loss configuration.
    pub config: QuadTemporalConfig,
    next_track_id: u64,
    track_id: Option<u64>,
    smoothed: Option<Quad>,
    history: Vec<Quad>,
    lost_frames: u32,
}

impl QuadTemporalTracker {
    /// Updates the active geometry track with an optional refined quad.
    pub fn update(
        &mut self,
        frame_index: FrameIndex,
        candidate: Option<Quad>,
    ) -> PerceptionResult<TemporalUpdate> {
        match candidate {
            Some(quad) => {
                quad.validate()?;
                if self.track_id.is_none() {
                    self.next_track_id += 1;
                    self.track_id = Some(self.next_track_id);
                    self.history.clear();
                }
                let smoothed = self
                    .smoothed
                    .map(|previous| blend_quad(previous, quad, self.config.smoothing_alpha))
                    .unwrap_or(quad);
                self.smoothed = Some(smoothed);
                self.history.push(smoothed);
                if self.history.len() > self.config.history_length {
                    self.history.remove(0);
                }
                self.lost_frames = 0;
                let stability = stability_score(&self.history);
                let velocity = self
                    .history
                    .len()
                    .checked_sub(2)
                    .and_then(|index| self.history.get(index).copied())
                    .and_then(|previous| velocity(previous, smoothed));
                let state = TemporalState {
                    stable: self.history.len() >= 3 && stability >= self.config.stable_threshold,
                    confidence: Confidence::new((0.40 + 0.60 * stability).clamp(0.0, 1.0))?,
                    track_id: self.track_id,
                    frame_index: Some(frame_index),
                    velocity,
                    stability_score: Score::new(stability)?,
                    lost_frames: 0,
                    diagnostics: vec!["geometry-only temporal smoothing; optical flow and IMU integration remain adapters".into()],
                };
                Ok(TemporalUpdate {
                    quad: Some(smoothed),
                    state,
                })
            }
            None => {
                self.lost_frames += 1;
                if self.lost_frames > self.config.max_lost_frames {
                    self.smoothed = None;
                    self.history.clear();
                    self.track_id = None;
                }
                Ok(TemporalUpdate {
                    quad: self.smoothed,
                    state: TemporalState {
                        stable: false,
                        confidence: Confidence::new(0.0)?,
                        track_id: self.track_id,
                        frame_index: Some(frame_index),
                        velocity: None,
                        stability_score: Score::new(0.0)?,
                        lost_frames: self.lost_frames,
                        diagnostics: vec!["no candidate accepted for current frame".into()],
                    },
                })
            }
        }
    }

    /// Converts the active smoothed quad into a low-reliability temporal detector candidate.
    pub fn temporal_candidate(
        &self,
    ) -> PerceptionResult<Option<wellfriend_perception_core::DetectionCandidate>> {
        self.smoothed
            .map(|quad| {
                document_quad_candidate(DetectionSource::Temporal, quad, 0.60, "quad-temporal-v1")
            })
            .transpose()
    }
}

fn blend_quad(previous: Quad, next: Quad, alpha: f32) -> Quad {
    let alpha = alpha.clamp(0.0, 1.0);
    Quad {
        points: std::array::from_fn(|index| {
            let a = previous.points[index];
            let b = next.points[index];
            Point2::new(a.x + (b.x - a.x) * alpha, a.y + (b.y - a.y) * alpha)
        }),
    }
}

fn stability_score(history: &[Quad]) -> f32 {
    if history.len() < 2 {
        return 0.5;
    }
    let pairs = history.windows(2).collect::<Vec<_>>();
    let position = pairs
        .iter()
        .map(|pair| center(pair[0]).distance(center(pair[1])) / diagonal(pair[1]).max(1.0))
        .sum::<f32>()
        / pairs.len() as f32;
    let scale = pairs
        .iter()
        .map(|pair| (quad_area(pair[0]) - quad_area(pair[1])).abs() / quad_area(pair[1]).max(1.0))
        .sum::<f32>()
        / pairs.len() as f32;
    let rotation = pairs
        .iter()
        .map(|pair| {
            angle_difference(edge_angle(pair[0]), edge_angle(pair[1])) / core::f32::consts::PI
        })
        .sum::<f32>()
        / pairs.len() as f32;
    (1.0 - (0.50 * position + 0.30 * scale + 0.20 * rotation)).clamp(0.0, 1.0)
}

fn velocity(previous: Quad, current: Quad) -> Option<Point2> {
    Some(Point2::new(
        center(current).x - center(previous).x,
        center(current).y - center(previous).y,
    ))
}

fn center(quad: Quad) -> Point2 {
    quad.polygon().centroid().unwrap_or_default()
}

fn quad_area(quad: Quad) -> f32 {
    quad.polygon().area()
}

fn diagonal(quad: Quad) -> f32 {
    quad.points[0]
        .distance(quad.points[2])
        .max(quad.points[1].distance(quad.points[3]))
}

fn edge_angle(quad: Quad) -> f32 {
    quad.points[0].angle_to(quad.points[1])
}

fn angle_difference(left: f32, right: f32) -> f32 {
    let mut difference = (left - right).abs();
    while difference > core::f32::consts::PI {
        difference = (2.0 * core::f32::consts::PI - difference).abs();
    }
    difference.abs()
}
