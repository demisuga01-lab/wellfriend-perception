use core::fmt;
use std::collections::BTreeMap;

use crate::{PerceptionError, PerceptionResult, PipelineTrace};

/// Stable identifier for an observation supplied to the pipeline.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObservationId(pub String);

/// Timestamp supplied by an input provider in Unix milliseconds or another declared epoch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(pub i64);

/// Monotonic frame index within an input sequence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameIndex(pub u64);

/// Origin of an observation. New variants can be represented by `External`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservationSource {
    /// Live camera source.
    Camera,
    /// File or local media source.
    File,
    /// Video sequence source.
    Video,
    /// Generic sensor source.
    Sensor,
    /// Network-delivered source.
    Network,
    /// Future or domain-specific source.
    External(String),
}

/// Device information that may affect quality, timing, or model routing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeviceMetadata {
    /// Stable device family or class identifier.
    pub device_class: Option<String>,
    /// Optional vendor/model identifier.
    pub model: Option<String>,
    /// Extensible capability metadata.
    pub attributes: BTreeMap<String, String>,
}

/// Processing information carried with an observation without binding to a UI platform.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProcessingMetadata {
    /// Optional run identifier.
    pub run_id: Option<String>,
    /// Trace carried forward by an orchestrator.
    pub trace: PipelineTrace,
    /// Extensible processing metadata.
    pub attributes: BTreeMap<String, String>,
}

/// Extensible metadata attached to an observation without leaking domain fields into core.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ObservationMetadata {
    /// Timestamp supplied by the input provider.
    pub timestamp: Option<Timestamp>,
    /// Sequence position when available.
    pub frame_index: Option<FrameIndex>,
    /// Input source.
    pub source: Option<ObservationSource>,
    /// Device properties.
    pub device: Option<DeviceMetadata>,
    /// Processing trace and run metadata.
    pub processing: ProcessingMetadata,
    /// Domain-neutral extension fields.
    pub attributes: BTreeMap<String, String>,
}

/// Pixel layouts supported by the owned image model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    /// One unsigned eight-bit gray channel.
    Gray8,
    /// Three unsigned eight-bit red/green/blue channels.
    Rgb8,
    /// Three unsigned eight-bit blue/green/red channels.
    Bgr8,
    /// Four unsigned eight-bit red/green/blue/alpha channels.
    Rgba8,
    /// Four unsigned eight-bit blue/green/red/alpha channels.
    Bgra8,
    /// One native-endian IEEE 754 f32 gray channel.
    GrayF32,
    /// Three native-endian IEEE 754 f32 red/green/blue channels.
    RgbF32,
    /// Placeholder packed YUV 4:2:0 format; plane layout is not implemented in MP2.
    Yuv420,
    /// Placeholder packed YUV 4:2:2 format; plane layout is not implemented in MP2.
    Yuv422,
    /// Placeholder packed YUV 4:4:4 format; plane layout is not implemented in MP2.
    Yuv444,
    /// Placeholder unsigned sixteen-bit single-channel format.
    U16,
    /// Placeholder IEEE 754 half-float single-channel format.
    F16,
    /// Placeholder IEEE 754 single-float single-channel format.
    F32,
    /// Extension point for interleaved multi-band rasters.
    MultiBand { channels: u8, bytes_per_channel: u8 },
}

impl PixelFormat {
    /// Number of logical channels for supported interleaved layouts.
    pub const fn channel_count(self) -> Option<usize> {
        match self {
            Self::Gray8 | Self::GrayF32 | Self::U16 | Self::F16 | Self::F32 => Some(1),
            Self::Rgb8 | Self::Bgr8 | Self::RgbF32 | Self::Yuv444 => Some(3),
            Self::Rgba8 | Self::Bgra8 => Some(4),
            Self::Yuv420 | Self::Yuv422 => None,
            Self::MultiBand { channels, .. } => Some(channels as usize),
        }
    }

    /// Bytes required for one pixel for supported interleaved layouts.
    pub const fn bytes_per_pixel(self) -> Option<usize> {
        match self {
            Self::Gray8 => Some(1),
            Self::Rgb8 | Self::Bgr8 | Self::Yuv444 => Some(3),
            Self::Rgba8 | Self::Bgra8 => Some(4),
            Self::GrayF32 | Self::F32 => Some(4),
            Self::RgbF32 => Some(12),
            Self::U16 | Self::F16 => Some(2),
            Self::Yuv420 | Self::Yuv422 => None,
            Self::MultiBand {
                channels,
                bytes_per_channel,
            } => Some(channels as usize * bytes_per_channel as usize),
        }
    }

    /// Returns true for scalar MP2 operations that work on interleaved u8 pixels.
    pub const fn is_u8_interleaved(self) -> bool {
        matches!(
            self,
            Self::Gray8 | Self::Rgb8 | Self::Bgr8 | Self::Rgba8 | Self::Bgra8
        )
    }
}

impl fmt::Display for PixelFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

/// Validated image width and height.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageShape {
    /// Pixel width.
    pub width: u32,
    /// Pixel height.
    pub height: u32,
}

impl ImageShape {
    /// Creates non-zero image dimensions.
    pub fn new(width: u32, height: u32) -> PerceptionResult<Self> {
        if width == 0 || height == 0 {
            return Err(PerceptionError::InvalidDimensions { width, height });
        }
        Ok(Self { width, height })
    }
}

/// Validated byte distance between adjacent image rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Stride(pub usize);

/// Region within an image, measured in pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegionOfInterest {
    /// Left edge in pixels.
    pub x: u32,
    /// Top edge in pixels.
    pub y: u32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl RegionOfInterest {
    /// Creates a region and validates it against an image shape.
    pub fn within(
        shape: ImageShape,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> PerceptionResult<Self> {
        if width == 0
            || height == 0
            || x.checked_add(width).is_none_or(|right| right > shape.width)
            || y.checked_add(height)
                .is_none_or(|bottom| bottom > shape.height)
        {
            return Err(PerceptionError::OutOfBounds {
                reason: format!(
                    "ROI {x},{y} {width}x{height} is outside {}x{}",
                    shape.width, shape.height
                ),
            });
        }
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }
}

/// Owned image bytes with checked shape and row-stride invariants.
#[derive(Clone, Debug, PartialEq)]
pub struct ImageBuffer {
    shape: ImageShape,
    pixel_format: PixelFormat,
    stride: Stride,
    data: Vec<u8>,
}

impl ImageBuffer {
    /// Creates a tightly packed image buffer.
    pub fn new(
        width: u32,
        height: u32,
        pixel_format: PixelFormat,
        data: Vec<u8>,
    ) -> PerceptionResult<Self> {
        let shape = ImageShape::new(width, height)?;
        let bytes_per_pixel =
            pixel_format
                .bytes_per_pixel()
                .ok_or(PerceptionError::UnsupportedFormat {
                    operation: "ImageBuffer::new",
                    format: pixel_format.to_string(),
                })?;
        let stride = (width as usize)
            .checked_mul(bytes_per_pixel)
            .ok_or(PerceptionError::Overflow)?;
        Self::new_with_stride(shape, pixel_format, Stride(stride), data)
    }

    /// Creates an image buffer with an explicit validated row stride.
    pub fn new_with_stride(
        shape: ImageShape,
        pixel_format: PixelFormat,
        stride: Stride,
        data: Vec<u8>,
    ) -> PerceptionResult<Self> {
        let bytes_per_pixel =
            pixel_format
                .bytes_per_pixel()
                .ok_or(PerceptionError::UnsupportedFormat {
                    operation: "ImageBuffer::new_with_stride",
                    format: pixel_format.to_string(),
                })?;
        let minimum = (shape.width as usize)
            .checked_mul(bytes_per_pixel)
            .ok_or(PerceptionError::Overflow)?;
        if stride.0 < minimum {
            return Err(PerceptionError::StrideMismatch {
                minimum,
                actual: stride.0,
            });
        }
        let expected = stride
            .0
            .checked_mul(shape.height as usize)
            .ok_or(PerceptionError::Overflow)?;
        if data.len() != expected {
            return Err(PerceptionError::InvalidBuffer {
                expected,
                actual: data.len(),
            });
        }
        Ok(Self {
            shape,
            pixel_format,
            stride,
            data,
        })
    }

    /// Creates a packed f32 image using native-endian component encoding.
    pub fn from_f32(
        width: u32,
        height: u32,
        pixel_format: PixelFormat,
        values: Vec<f32>,
    ) -> PerceptionResult<Self> {
        if !matches!(
            pixel_format,
            PixelFormat::GrayF32 | PixelFormat::RgbF32 | PixelFormat::F32
        ) {
            return Err(PerceptionError::UnsupportedFormat {
                operation: "ImageBuffer::from_f32",
                format: pixel_format.to_string(),
            });
        }
        if values.iter().any(|value| !value.is_finite()) {
            return Err(PerceptionError::NumericFailure {
                reason: "f32 image contains non-finite value".into(),
            });
        }
        let channels = pixel_format
            .channel_count()
            .ok_or(PerceptionError::UnsupportedFormat {
                operation: "ImageBuffer::from_f32",
                format: pixel_format.to_string(),
            })?;
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|size| size.checked_mul(channels))
            .ok_or(PerceptionError::Overflow)?;
        if values.len() != expected {
            return Err(PerceptionError::InvalidBuffer {
                expected,
                actual: values.len(),
            });
        }
        let data = values.into_iter().flat_map(f32::to_ne_bytes).collect();
        Self::new(width, height, pixel_format, data)
    }

    /// Decodes a supported f32 buffer into owned components.
    pub fn to_f32(&self) -> PerceptionResult<Vec<f32>> {
        if !matches!(
            self.pixel_format,
            PixelFormat::GrayF32 | PixelFormat::RgbF32 | PixelFormat::F32
        ) {
            return Err(PerceptionError::UnsupportedFormat {
                operation: "ImageBuffer::to_f32",
                format: self.pixel_format.to_string(),
            });
        }
        let mut values = Vec::with_capacity(self.data.len() / 4);
        for bytes in self.data.chunks_exact(4) {
            values.push(f32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));
        }
        Ok(values)
    }

    /// Image shape.
    pub const fn shape(&self) -> ImageShape {
        self.shape
    }
    /// Pixel width.
    pub const fn width(&self) -> u32 {
        self.shape.width
    }
    /// Pixel height.
    pub const fn height(&self) -> u32 {
        self.shape.height
    }
    /// Pixel format.
    pub const fn pixel_format(&self) -> PixelFormat {
        self.pixel_format
    }
    /// Row stride in bytes.
    pub const fn stride(&self) -> Stride {
        self.stride
    }
    /// Immutable encoded bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
    /// Mutable encoded bytes. Length and stride remain immutable.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
    /// Validated immutable full-image view.
    pub fn view(&self) -> ImageView<'_> {
        ImageView {
            shape: self.shape,
            pixel_format: self.pixel_format,
            stride: self.stride,
            data: &self.data,
        }
    }
    /// Validated mutable full-image view.
    pub fn view_mut(&mut self) -> ImageViewMut<'_> {
        ImageViewMut {
            shape: self.shape,
            pixel_format: self.pixel_format,
            stride: self.stride,
            data: &mut self.data,
        }
    }
    /// Creates an immutable ROI view without copying pixels.
    pub fn roi(&self, roi: RegionOfInterest) -> PerceptionResult<ImageView<'_>> {
        self.view().roi(roi)
    }
    /// Returns a checked byte offset to a u8 interleaved pixel channel.
    pub fn pixel_offset(&self, x: u32, y: u32, channel: usize) -> PerceptionResult<usize> {
        self.view().pixel_offset(x, y, channel)
    }
    /// Gets one u8 interleaved component.
    pub fn get_u8(&self, x: u32, y: u32, channel: usize) -> PerceptionResult<u8> {
        Ok(self.data[self.pixel_offset(x, y, channel)?])
    }
    /// Sets one u8 interleaved component.
    pub fn set_u8(&mut self, x: u32, y: u32, channel: usize, value: u8) -> PerceptionResult<()> {
        let index = self.pixel_offset(x, y, channel)?;
        self.data[index] = value;
        Ok(())
    }
}

/// Non-owning immutable image view that can represent a strided ROI.
#[derive(Clone, Copy, Debug)]
pub struct ImageView<'a> {
    shape: ImageShape,
    pixel_format: PixelFormat,
    stride: Stride,
    data: &'a [u8],
}

impl<'a> ImageView<'a> {
    /// Shape of this view.
    pub const fn shape(&self) -> ImageShape {
        self.shape
    }
    /// Pixel format of this view.
    pub const fn pixel_format(&self) -> PixelFormat {
        self.pixel_format
    }
    /// Original row stride in bytes.
    pub const fn stride(&self) -> Stride {
        self.stride
    }
    /// Returns a checked row containing only this view's logical width.
    pub fn row(&self, y: u32) -> PerceptionResult<&[u8]> {
        if y >= self.shape.height {
            return Err(PerceptionError::OutOfBounds {
                reason: format!("row {y} exceeds height {}", self.shape.height),
            });
        }
        let row_bytes = self.shape.width as usize
            * self
                .pixel_format
                .bytes_per_pixel()
                .ok_or(PerceptionError::UnsupportedFormat {
                    operation: "ImageView::row",
                    format: self.pixel_format.to_string(),
                })?;
        let start = y as usize * self.stride.0;
        self.data
            .get(start..start + row_bytes)
            .ok_or(PerceptionError::InvalidBuffer {
                expected: start + row_bytes,
                actual: self.data.len(),
            })
    }
    /// Returns a non-copying nested ROI view.
    pub fn roi(&self, roi: RegionOfInterest) -> PerceptionResult<Self> {
        RegionOfInterest::within(self.shape, roi.x, roi.y, roi.width, roi.height)?;
        let bytes_per_pixel =
            self.pixel_format
                .bytes_per_pixel()
                .ok_or(PerceptionError::UnsupportedFormat {
                    operation: "ImageView::roi",
                    format: self.pixel_format.to_string(),
                })?;
        let start = roi.y as usize * self.stride.0 + roi.x as usize * bytes_per_pixel;
        let required = (roi.height as usize - 1)
            .checked_mul(self.stride.0)
            .and_then(|offset| offset.checked_add(roi.width as usize * bytes_per_pixel))
            .ok_or(PerceptionError::Overflow)?;
        let data =
            self.data
                .get(start..start + required)
                .ok_or(PerceptionError::InvalidBuffer {
                    expected: start + required,
                    actual: self.data.len(),
                })?;
        Ok(Self {
            shape: ImageShape::new(roi.width, roi.height)?,
            pixel_format: self.pixel_format,
            stride: self.stride,
            data,
        })
    }
    /// Copies this possibly-strided view into a tightly packed owned buffer.
    pub fn to_owned(&self) -> PerceptionResult<ImageBuffer> {
        let mut data = Vec::with_capacity(
            self.shape.width as usize
                * self.shape.height as usize
                * self.pixel_format.bytes_per_pixel().ok_or(
                    PerceptionError::UnsupportedFormat {
                        operation: "ImageView::to_owned",
                        format: self.pixel_format.to_string(),
                    },
                )?,
        );
        for y in 0..self.shape.height {
            data.extend_from_slice(self.row(y)?);
        }
        ImageBuffer::new(self.shape.width, self.shape.height, self.pixel_format, data)
    }
    /// Returns a checked component offset for u8 interleaved formats.
    pub fn pixel_offset(&self, x: u32, y: u32, channel: usize) -> PerceptionResult<usize> {
        if !self.pixel_format.is_u8_interleaved() {
            return Err(PerceptionError::UnsupportedFormat {
                operation: "ImageView::pixel_offset",
                format: self.pixel_format.to_string(),
            });
        }
        if x >= self.shape.width || y >= self.shape.height {
            return Err(PerceptionError::OutOfBounds {
                reason: format!(
                    "pixel {x},{y} exceeds {}x{}",
                    self.shape.width, self.shape.height
                ),
            });
        }
        let channels =
            self.pixel_format
                .channel_count()
                .ok_or(PerceptionError::UnsupportedFormat {
                    operation: "ImageView::pixel_offset",
                    format: self.pixel_format.to_string(),
                })?;
        if channel >= channels {
            return Err(PerceptionError::OutOfBounds {
                reason: format!("channel {channel} exceeds {channels}"),
            });
        }
        Ok(y as usize * self.stride.0 + x as usize * channels + channel)
    }
}

/// Non-owning mutable image view that can expose checked rows.
#[derive(Debug)]
pub struct ImageViewMut<'a> {
    shape: ImageShape,
    pixel_format: PixelFormat,
    stride: Stride,
    data: &'a mut [u8],
}

impl ImageViewMut<'_> {
    /// Returns a checked mutable row containing only the logical width.
    pub fn row_mut(&mut self, y: u32) -> PerceptionResult<&mut [u8]> {
        if y >= self.shape.height {
            return Err(PerceptionError::OutOfBounds {
                reason: format!("row {y} exceeds height {}", self.shape.height),
            });
        }
        let row_bytes = self.shape.width as usize
            * self
                .pixel_format
                .bytes_per_pixel()
                .ok_or(PerceptionError::UnsupportedFormat {
                    operation: "ImageViewMut::row_mut",
                    format: self.pixel_format.to_string(),
                })?;
        let start = y as usize * self.stride.0;
        let actual = self.data.len();
        self.data
            .get_mut(start..start + row_bytes)
            .ok_or(PerceptionError::InvalidBuffer {
                expected: start + row_bytes,
                actual,
            })
    }
}

/// Generic sensor packet for a future IMU, depth, or hardware input provider.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SensorFrame {
    /// Sensor family identifier.
    pub sensor_type: String,
    /// Named scalar samples.
    pub values: BTreeMap<String, f32>,
}

/// Descriptor for a future raster payload that need not be a 2D image buffer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RasterDescriptor {
    /** Raster dimensions. */
    pub dimensions: Vec<u32>,
    /** Band count. */
    pub bands: u32,
}

/// Descriptor for a future volume payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VolumeDescriptor {
    /** Voxel dimensions. */
    pub dimensions: [u32; 3],
    /** Scalar encoding name. */
    pub scalar_type: String,
}

/// Payload carried by a generic observation.
#[derive(Clone, Debug, PartialEq)]
pub enum ObservationPayload {
    /// Owned two-dimensional image.
    Image(ImageBuffer),
    /// Generic sensor frame.
    Sensor(SensorFrame),
    /// Future multi-band raster descriptor.
    Raster(RasterDescriptor),
    /// Future volumetric descriptor.
    Volume(VolumeDescriptor),
}

/// A single sensor observation and an optional domain-neutral payload.
#[derive(Clone, Debug, PartialEq)]
pub struct Observation {
    /// Stable observation identifier.
    pub id: ObservationId,
    /// Source, timing, and processing metadata.
    pub metadata: ObservationMetadata,
    /// Optional payload; metadata-only observations are valid in a sequence.
    pub payload: Option<ObservationPayload>,
}

/// Time-ordered collection of observations, such as a camera stream segment.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObservationFrame {
    /// Optional sequence index supplied by the provider.
    pub frame_index: Option<FrameIndex>,
    /// Time-ordered observations.
    pub observations: Vec<Observation>,
}

/// A finite, validated score in the closed unit interval.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Confidence(f32);

impl Confidence {
    /// Creates a confidence after rejecting non-finite and out-of-range values.
    pub fn new(value: f32) -> PerceptionResult<Self> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(PerceptionError::InvalidConfidence { value });
        }
        Ok(Self(value))
    }
    /// Returns the bounded scalar value.
    pub const fn value(self) -> f32 {
        self.0
    }
}

impl Default for Confidence {
    fn default() -> Self {
        Self(0.0)
    }
}

/// Bounded score alias used when a caller wants semantic distinction from confidence.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Score(Confidence);
impl Score {
    /** Creates a bounded score. */
    pub fn new(value: f32) -> PerceptionResult<Self> {
        Ok(Self(Confidence::new(value)?))
    }
    /** Returns its bounded value. */
    pub const fn value(self) -> f32 {
        self.0.value()
    }
}
/// Bounded probability alias.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Probability(Confidence);
impl Probability {
    /** Creates a probability. */
    pub fn new(value: f32) -> PerceptionResult<Self> {
        Ok(Self(Confidence::new(value)?))
    }
    /** Returns its bounded value. */
    pub const fn value(self) -> f32 {
        self.0.value()
    }
}
/// Bounded reliability alias.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Reliability(Confidence);
impl Reliability {
    /** Creates a reliability score. */
    pub fn new(value: f32) -> PerceptionResult<Self> {
        Ok(Self(Confidence::new(value)?))
    }
    /** Returns its bounded value. */
    pub const fn value(self) -> f32 {
        self.0.value()
    }
}
/// Standard quality metrics shared by all domain packs.
#[derive(Clone, Debug, PartialEq)]
pub enum QualityMetric {
    /** Blur measure. */
    Blur,
    /** Noise measure. */
    Noise,
    /** Exposure measure. */
    Exposure,
    /** Saturation measure. */
    Saturation,
    /** Contrast measure. */
    Contrast,
    /** Motion measure. */
    Motion,
    /** Glare measure. */
    Glare,
    /** Occlusion measure. */
    Occlusion,
    /** Confidence measure. */
    Confidence,
    /** Domain-defined metric. */
    DomainSpecific(String),
}

/// Named normalized quality values; directionality is defined by the analyzer.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct QualityVector(pub BTreeMap<String, f32>);

/// Measured quality plus detector diagnostics.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct QualityReport {
    /** Values by metric name. */
    pub vector: QualityVector,
    /** Implementation diagnostics. */
    pub diagnostics: Vec<String>,
}

/// A two-dimensional point in a declared coordinate system.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point2 {
    /** Horizontal coordinate. */
    pub x: f32,
    /** Vertical coordinate. */
    pub y: f32,
}
/// A three-dimensional point in a declared coordinate system.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point3 {
    /** X coordinate. */
    pub x: f32,
    /** Y coordinate. */
    pub y: f32,
    /** Z coordinate. */
    pub z: f32,
}
/// Infinite 2D line represented by two distinct points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line2 {
    /** First point. */
    pub a: Point2,
    /** Second point. */
    pub b: Point2,
}
/// Finite 2D line segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment2 {
    /** Start point. */
    pub start: Point2,
    /** End point. */
    pub end: Point2,
}
/// Ordered 2D polygon boundary.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Polygon {
    /** Ordered vertices. */
    pub points: Vec<Point2>,
}
/// Four-corner polygon with stable order defined by its producer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quad {
    /** Ordered corners. */
    pub points: [Point2; 4],
}
/// Axis-aligned 2D bounds.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoundingBox {
    /** Left edge. */
    pub x: f32,
    /** Top edge. */
    pub y: f32,
    /** Non-negative width. */
    pub width: f32,
    /** Non-negative height. */
    pub height: f32,
}
/// A binary image mask; coordinate mapping belongs to associated output.
#[derive(Clone, Debug, PartialEq)]
pub struct Mask {
    /** Width. */
    pub width: u32,
    /** Height. */
    pub height: u32,
    /** One byte per mask sample. */
    pub values: Vec<u8>,
}
/// Generic sampled or parametric surface representation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Surface {
    /** Surface vertices. */
    pub vertices: Vec<Point3>,
    /** Triangle indices. */
    pub indices: Vec<u32>,
}
/// Position and orientation expressed by the declaring domain pack.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    /** Translation. */
    pub translation: Point3,
    /** XYZW quaternion. */
    pub rotation_xyzw: [f32; 4],
}
/// Homogeneous 2D transform in row-major order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform2D {
    /** Row-major 3x3 matrix. */
    pub matrix: [[f32; 3]; 3],
}
/// Homogeneous 3D transform in row-major order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform3D {
    /** Row-major 4x4 matrix. */
    pub matrix: [[f32; 4]; 4],
}
/// Dense mapping from output pixels to source coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct DenseWarpField {
    /** Output width. */
    pub width: u32,
    /** Output height. */
    pub height: u32,
    /** Source coordinate per output pixel. */
    pub vectors: Vec<Point2>,
}

/// How a detector obtained a candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DetectionSource {
    /** Classical algorithm. */
    Classical,
    /** Machine-learning runtime. */
    Ml,
    /** Temporal estimator. */
    Temporal,
    /** User correction. */
    Manual,
    /** Other module. */
    External(String),
}
/// Calibrated confidence with an explicit interval.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DetectionConfidence {
    /** Point confidence. */
    pub score: Confidence,
    /** Lower interval bound. */
    pub lower: Confidence,
    /** Upper interval bound. */
    pub upper: Confidence,
}
/// Qualitative and numerical uncertainty information.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Uncertainty {
    /** Optional covariance components in declared order. */
    pub covariance: Vec<f32>,
    /** Optional scalar variance. */
    pub variance: Option<f32>,
    /** Diagnostics. */
    pub notes: Vec<String>,
}

impl Uncertainty {
    /// Creates a scalar variance after rejecting negative and non-finite values.
    pub fn with_variance(variance: f32) -> PerceptionResult<Self> {
        if !variance.is_finite() || variance < 0.0 {
            return Err(PerceptionError::NumericFailure {
                reason: "variance must be finite and non-negative".into(),
            });
        }
        Ok(Self {
            covariance: Vec::new(),
            variance: Some(variance),
            notes: Vec::new(),
        })
    }
}

/// Domain-neutral candidate payload; interpretation is declared by kind and domain pack.
#[derive(Clone, Debug, PartialEq)]
pub struct DetectionCandidate {
    /** Candidate kind. */
    pub kind: String,
    /** Candidate source. */
    pub source: DetectionSource,
    /** Calibrated confidence. */
    pub confidence: DetectionConfidence,
    /** Optional polygon geometry. */
    pub geometry: Option<Polygon>,
    /** Uncertainty payload. */
    pub uncertainty: Uncertainty,
    /** Extension values. */
    pub attributes: BTreeMap<String, String>,
}
/// Candidates produced for one observation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DetectionSet {
    /** Candidates. */
    pub candidates: Vec<DetectionCandidate>,
    /** Diagnostics. */
    pub diagnostics: Vec<String>,
}
/// Fusion output with traceable contributing candidates.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FusionResult {
    /** Fused candidates. */
    pub candidates: DetectionSet,
    /** Diagnostics. */
    pub diagnostics: Vec<String>,
}
/// Refinement output for one selected target.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RefinementResult {
    /** Refined candidates. */
    pub candidates: DetectionSet,
    /** Diagnostics. */
    pub diagnostics: Vec<String>,
}
/// Persistent temporal state represented without imposing a tracker implementation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TemporalState {
    /** Stability flag. */
    pub stable: bool,
    /** Bounded confidence. */
    pub confidence: Confidence,
    /** Diagnostics. */
    pub diagnostics: Vec<String>,
}

/// Domain-selected mathematical reconstruction model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GeometryModel {
    /** Planar model. */
    Planar,
    /** Surface model. */
    Surface,
    /** Volume model. */
    Volumetric,
    /** Geospatial model. */
    Geospatial,
    /** Photogrammetric model. */
    Photogrammetric,
    /** Domain-defined model. */
    Custom(String),
}
/// Reconstruction product and its optional geometry artifacts.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReconstructionResult {
    /** Optional 2D transform. */
    pub transform_2d: Option<Transform2D>,
    /** Optional 3D transform. */
    pub transform_3d: Option<Transform3D>,
    /** Optional surface. */
    pub surface: Option<Surface>,
    /** Diagnostics. */
    pub diagnostics: Vec<String>,
}

/// Condition scores used to route specialized processors.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConditionVector(pub BTreeMap<String, f32>);
/// Ordered processor identifiers selected by the router.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcessingPlan {
    /** Processor ids. */
    pub processor_ids: Vec<String>,
    /** Diagnostics. */
    pub diagnostics: Vec<String>,
}
/// Standardized processor result for graph execution and observability.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcessorResult {
    /** Optional image output. */
    pub output: Option<ImageBuffer>,
    /** Bounded confidence. */
    pub confidence: Confidence,
    /** Diagnostics. */
    pub diagnostics: Vec<String>,
}
/// Declared operating envelope used by the specialist router.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcessorCapabilities {
    /** Capability names. */
    pub capabilities: Vec<String>,
    /** Expected normalized benefit. */
    pub expected_benefit: Confidence,
    /** Estimated device time. */
    pub estimated_cost_ms: u32,
    /** Supported device classes. */
    pub supported_device_classes: Vec<String>,
    /** Reliability estimate. */
    pub confidence: Reliability,
    /** Diagnostics. */
    pub diagnostics: Vec<String>,
}

/// A typed semantic region, independent of OCR or document-specific labels.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticRegion {
    /** Region kind. */
    pub kind: String,
    /** Optional geometry. */
    pub geometry: Option<Polygon>,
    /** Bounded confidence. */
    pub confidence: Confidence,
    /** Extension attributes. */
    pub attributes: BTreeMap<String, String>,
}
/// Semantic regions and relationships emitted by a semantic engine.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SemanticResult {
    /** Regions. */
    pub regions: Vec<SemanticRegion>,
    /** Directed relationships. */
    pub relationships: Vec<(usize, usize, String)>,
}
/// Exportable, domain-owned structured output.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StructuredOutput {
    /** Schema identifier. */
    pub schema: String,
    /** Serialized payload. */
    pub payload: String,
    /** Diagnostics. */
    pub diagnostics: Vec<String>,
}

/// Immutable per-run context passed across pipeline stages.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PipelineContext {
    /** Run id. */
    pub run_id: String,
    /** Selected domain pack id. */
    pub domain: String,
    /** Extension attributes. */
    pub attributes: BTreeMap<String, String>,
}
/// Ordered pipeline stages. Packs may intentionally omit stages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PipelineStage {
    /** Input acquisition. */
    Input,
    /** Quality analysis. */
    Quality,
    /** Candidate detection. */
    Detection,
    /** Candidate fusion. */
    Fusion,
    /** Precision refinement. */
    Refinement,
    /** Temporal update. */
    Temporal,
    /** Geometry reconstruction. */
    Reconstruction,
    /** Condition analysis. */
    Condition,
    /** Processor routing. */
    Routing,
    /** Restoration. */
    Restoration,
    /** Semantic analysis. */
    Semantics,
    /** Structured export. */
    Export,
}
