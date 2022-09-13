use freenect2_sys as sys;

/// IR camera intrinsic calibration parameters.
#[derive(Debug, Clone, Copy)]
pub struct IrCameraParams {
	/// Focal length x (pixel)
	pub fx: f32,
	/// Focal length y (pixel)
	pub fy: f32,
	/// Principal point x (pixel)
	pub cx: f32,
	/// Principal point y (pixel)
	pub cy: f32,
	/// Radial distortion coefficient, first order
	pub k1: f32,
	/// Radial distortion coefficient, second order
	pub k2: f32,
	/// Radial distortion coefficient, third order
	pub k3: f32,
	/// Tangential distortion coefficient
	pub p1: f32,
	/// Tangential distortion coefficient
	pub p2: f32,
}

impl From<sys::Fn2IrCameraParams> for IrCameraParams {
	fn from(sys: sys::Fn2IrCameraParams) -> Self {
		Self {
			fx: sys.fx,
			fy: sys.fy,
			cx: sys.cx,
			cy: sys.cy,
			k1: sys.k1,
			k2: sys.k2,
			k3: sys.k3,
			p1: sys.p1,
			p2: sys.p2,
		}
	}
}

impl From<IrCameraParams> for sys::Fn2IrCameraParams {
	fn from(our: IrCameraParams) -> Self {
		Self {
			fx: our.fx,
			fy: our.fy,
			cx: our.cx,
			cy: our.cy,
			k1: our.k1,
			k2: our.k2,
			k3: our.k3,
			p1: our.p1,
			p2: our.p2,
		}
	}
}
