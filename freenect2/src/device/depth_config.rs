use freenect2_sys as sys;

/// Depth processing configuration.
#[derive(Debug, Clone, Copy)]
pub struct DepthConfig {
	/// Clip at this minimum depth, in meters.
	pub min_depth: f32,
	/// Clip at this maximum depth, in meters.
	pub max_depth: f32,
	/// Remove some "flying pixels".
	pub enable_bilateral_filter: bool,
	/// Remove pixels on edges because ToF cameras produce noisy edges.
	pub enable_edge_aware_filter: bool,
}

impl Default for DepthConfig {
	fn default() -> Self {
		Self {
			min_depth: 0.5,
			max_depth: 4.5,
			enable_bilateral_filter: true,
			enable_edge_aware_filter: true,
		}
	}
}

impl From<DepthConfig> for sys::Fn2DeviceConfig {
	fn from(our: DepthConfig) -> Self {
		Self {
			min_depth: our.min_depth,
			max_depth: our.max_depth,
			enable_bilateral_filter: our.enable_bilateral_filter,
			enable_edge_aware_filter: our.enable_edge_aware_filter,
		}
	}
}
