//! Provides [`Frame`], [`Type`], and [`Format`].

use freenect2_sys as sys;

/// A frame from one of the device's cameras.
#[derive(Debug)]
pub struct Frame {
	width: usize,
	height: usize,
	bytes_per_pixel: usize,
	data: Box<[u8]>,
	timestamp: u32,
	sequence: u32,
	exposure: f32,
	gain: f32,
	errors_occurred: bool,
	format: Format,
}

impl Frame {
	/// Convert from the unsafe equivalent.
	///
	/// # Safety
	///
	/// This conversion is unsafe because it makes assumptions about the validity and size of the `data` pointer.
	/// It should never be null.
	/// It should be aligned to at least 8-byte boundary.
	/// If `format` is `Fn2FrameFormat_Raw`, it should be `bytes_per_pixel` bytes long, otherwise it should be `width * height * bytes_per_pixel` bytes long.
	///
	/// # Panics
	///
	/// Panics if the frame format is invalid.
	#[must_use]
	pub unsafe fn from_sys(sys: sys::Fn2Frame) -> Self {
		let format = sys.format.try_into().unwrap();

		let data_len = match format {
			Format::Raw => sys.bytes_per_pixel,
			_ => sys.width * sys.height * sys.bytes_per_pixel,
		};

		Self {
			width: sys.width,
			height: sys.height,
			bytes_per_pixel: sys.bytes_per_pixel,
			data: Box::from_raw(std::ptr::slice_from_raw_parts_mut(sys.data, data_len)),
			timestamp: sys.timestamp,
			sequence: sys.sequence,
			exposure: sys.exposure,
			gain: sys.gain,
			errors_occurred: sys.status > 0,
			format,
		}
	}

	/// The width of the frame, in pixels.
	#[must_use]
	pub fn width(&self) -> usize {
		self.width
	}

	/// The height of the frame, in pixels.
	#[must_use]
	pub fn height(&self) -> usize {
		self.height
	}

	/// The number of data bytes per pixel.
	///
	/// If `format` is `Raw`, this is simply the length of `data`.
	#[must_use]
	pub fn bytes_per_pixel(&self) -> usize {
		self.bytes_per_pixel
	}

	/// The data itself.
	///
	/// It will be aligned to at least an 8-byte boundary.
	#[must_use]
	pub fn data(&self) -> &[u8] {
		&self.data
	}

	/// The data itself, mutably.
	///
	/// It will be aligned to at least an 8-byte boundary.
	#[must_use]
	pub fn data_mut(&mut self) -> &mut [u8] {
		&mut self.data
	}

	/// Consume the frame and return the owned data.
	#[must_use]
	pub fn into_data(self) -> Box<[u8]> {
		self.data
	}

	/// In units of 100 microseconds.
	#[must_use]
	pub fn timestamp(&self) -> u32 {
		self.timestamp
	}

	/// The monotonically increasing sequence number.
	#[must_use]
	pub fn sequence(&self) -> u32 {
		self.sequence
	}

	/// From 0.5 (very bright) to about 60.0 (fully dark).
	#[must_use]
	pub fn exposure(&self) -> f32 {
		self.exposure
	}

	/// From 1.0 (bright) to 1.5 (dark).
	#[must_use]
	pub fn gain(&self) -> f32 {
		self.gain
	}

	/// Indicates if any errors occurred.
	#[must_use]
	pub fn errors_occurred(&self) -> bool {
		self.errors_occurred
	}

	/// The format of the image data returned by `data`.
	#[must_use]
	pub fn format(&self) -> Format {
		self.format
	}
}

/// The possible image formats of a frame's data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
	/// 4 bytes of B, G, R, and unused per pixel.
	Bgrx,
	/// One `f32` per pixel.
	Float,
	/// One gray byte per pixel.
	Gray,
	/// An invalid format.
	Invalid,
	/// A raw data stream.
	Raw,
	/// 4 bytes of R, G, B, and unused per pixel.
	Rgbx,
}

/// Error returned when converting from unsafe equivalent if the data represents an invalid variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownFormat;

impl std::fmt::Display for UnknownFormat {
	fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		formatter.write_str("unknown frame format")
	}
}

impl std::error::Error for UnknownFormat {}

impl TryFrom<sys::Fn2FrameFormat> for Format {
	type Error = UnknownFormat;

	fn try_from(raw: sys::Fn2FrameFormat) -> Result<Self, Self::Error> {
		Ok(match raw {
			sys::Fn2FrameFormat_Bgrx => Self::Bgrx,
			sys::Fn2FrameFormat_Float => Self::Float,
			sys::Fn2FrameFormat_Gray => Self::Gray,
			sys::Fn2FrameFormat_Invalid => Self::Invalid,
			sys::Fn2FrameFormat_Raw => Self::Raw,
			sys::Fn2FrameFormat_Rgbx => Self::Rgbx,
			_ => return Err(UnknownFormat),
		})
	}
}

/// Error returned when converting from unsafe equivalent if the data represents an invalid variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownType;

impl std::fmt::Display for UnknownType {
	fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		formatter.write_str("unknown frame type")
	}
}

impl std::error::Error for UnknownType {}

/// The source of a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
	/// From the color camera.
	///
	/// The image will be 1920 by 1080 and in the Bgrx or Rgbx format.
	Color,
	/// From the depth camera.
	///
	/// The image will be 512 by 424 and in the Float format, with values ranging from 0.0 to 65536.0.
	Depth,
	/// From the IR camera.
	///
	/// The image will be 512 by 424 and in the Float format, in millimeters.
	/// Non-positive, NaN, and infinities represent invalid or missing data.
	Ir,
}

impl TryFrom<sys::Fn2FrameType> for Type {
	type Error = UnknownType;

	fn try_from(raw: sys::Fn2FrameType) -> Result<Self, Self::Error> {
		Ok(match raw {
			sys::Fn2FrameType_Color => Self::Color,
			sys::Fn2FrameType_Depth => Self::Depth,
			sys::Fn2FrameType_Ir => Self::Ir,
			_ => return Err(UnknownType),
		})
	}
}
