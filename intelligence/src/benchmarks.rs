//! Generated ScanBench fixtures that keep MP3 benchmarks free of binary assets.

use wellfriend_perception_core::{ImageBuffer, PerceptionResult, PixelFormat, Point2, Quad};

/// Named synthetic image classes used by correctness tests and scalar benchmarks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyntheticDocumentFixtureKind {
    /// Centered axis-aligned bright quadrilateral.
    PlainCentered,
    /// Rotated bright quadrilateral.
    Rotated,
    /// Convex perspective quadrilateral.
    Perspective,
    /// Page/background separation with low contrast.
    LowContrast,
    /// Page intentionally extending outside the visible frame.
    PartialCutOff,
    /// One page plus a separate rectangular distractor.
    MultipleDistractors,
    /// Uniform frame without a document candidate.
    NoDocument,
}

/// Generated fixture and optional known page geometry in image coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct SyntheticDocumentFixture {
    /// Checked Gray8 image buffer.
    pub image: ImageBuffer,
    /// Expected document quad when one is fully representable.
    pub expected_quad: Option<Quad>,
}

/// Creates a deterministic 160×120 fixture without external data provenance.
pub fn synthetic_document_fixture(
    kind: SyntheticDocumentFixtureKind,
) -> PerceptionResult<SyntheticDocumentFixture> {
    let width = 160;
    let height = 120;
    let background = if kind == SyntheticDocumentFixtureKind::LowContrast {
        90
    } else {
        12
    };
    let mut data = vec![background; width as usize * height as usize];
    let page = match kind {
        SyntheticDocumentFixtureKind::PlainCentered => Some(Quad {
            points: [
                p(28.0, 18.0),
                p(132.0, 18.0),
                p(132.0, 102.0),
                p(28.0, 102.0),
            ],
        }),
        SyntheticDocumentFixtureKind::Rotated => Some(Quad {
            points: [
                p(45.0, 14.0),
                p(138.0, 37.0),
                p(113.0, 106.0),
                p(20.0, 82.0),
            ],
        }),
        SyntheticDocumentFixtureKind::Perspective => Some(Quad {
            points: [
                p(45.0, 18.0),
                p(126.0, 28.0),
                p(142.0, 105.0),
                p(24.0, 96.0),
            ],
        }),
        SyntheticDocumentFixtureKind::LowContrast => Some(Quad {
            points: [
                p(30.0, 20.0),
                p(130.0, 20.0),
                p(130.0, 100.0),
                p(30.0, 100.0),
            ],
        }),
        SyntheticDocumentFixtureKind::PartialCutOff => Some(Quad {
            points: [
                p(-10.0, 8.0),
                p(150.0, 8.0),
                p(170.0, 112.0),
                p(-16.0, 112.0),
            ],
        }),
        SyntheticDocumentFixtureKind::MultipleDistractors => Some(Quad {
            points: [
                p(36.0, 16.0),
                p(130.0, 20.0),
                p(126.0, 104.0),
                p(32.0, 98.0),
            ],
        }),
        SyntheticDocumentFixtureKind::NoDocument => None,
    };
    if let Some(quad) = page {
        draw_quad(
            &mut data,
            width,
            height,
            quad,
            if kind == SyntheticDocumentFixtureKind::LowContrast {
                150
            } else {
                245
            },
        );
    }
    if kind == SyntheticDocumentFixtureKind::MultipleDistractors {
        draw_quad(
            &mut data,
            width,
            height,
            Quad {
                points: [p(4.0, 4.0), p(24.0, 4.0), p(24.0, 24.0), p(4.0, 24.0)],
            },
            190,
        );
    }
    Ok(SyntheticDocumentFixture {
        image: ImageBuffer::new(width, height, PixelFormat::Gray8, data)?,
        expected_quad: page,
    })
}

fn draw_quad(data: &mut [u8], width: u32, height: u32, quad: Quad, value: u8) {
    let polygon = quad.polygon();
    for y in 0..height {
        for x in 0..width {
            if polygon.contains_point(p(x as f32 + 0.5, y as f32 + 0.5)) {
                data[y as usize * width as usize + x as usize] = value;
            }
        }
    }
}

fn p(x: f32, y: f32) -> Point2 {
    Point2::new(x, y)
}
