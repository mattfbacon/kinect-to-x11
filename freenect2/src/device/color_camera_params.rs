use freenect2_sys as sys;

/// Color camera calibration parameters.
///
/// Some parameters are extrinsic.
/// Extrinsic parameters are used in a formula to map coordinates in the depth camera to the color camera.
/// They cannot be used for matrix transformation.
#[derive(Debug, Clone, Copy)]
pub struct ColorCameraParams {
	/// Focal length x (pixel)
	pub fx: f32,
	/// Focal length y (pixel)
	pub fy: f32,
	/// Principal point x (pixel)
	pub cx: f32,
	/// Principal point y (pixel)
	pub cy: f32,
	/// Extrinsic parameter.
	pub shift_d: f32,
	/// Extrinsic parameter.
	pub shift_m: f32,
	/// Extrinsic parameter.
	pub mx_x3y0: f32,
	/// Extrinsic parameter.
	pub mx_x0y3: f32,
	/// Extrinsic parameter.
	pub mx_x2y1: f32,
	/// Extrinsic parameter.
	pub mx_x1y2: f32,
	/// Extrinsic parameter.
	pub mx_x2y0: f32,
	/// Extrinsic parameter.
	pub mx_x0y2: f32,
	/// Extrinsic parameter.
	pub mx_x1y1: f32,
	/// Extrinsic parameter.
	pub mx_x1y0: f32,
	/// Extrinsic parameter.
	pub mx_x0y1: f32,
	/// Extrinsic parameter.
	pub mx_x0y0: f32,
	/// Extrinsic parameter.
	pub my_x3y0: f32,
	/// Extrinsic parameter.
	pub my_x0y3: f32,
	/// Extrinsic parameter.
	pub my_x2y1: f32,
	/// Extrinsic parameter.
	pub my_x1y2: f32,
	/// Extrinsic parameter.
	pub my_x2y0: f32,
	/// Extrinsic parameter.
	pub my_x0y2: f32,
	/// Extrinsic parameter.
	pub my_x1y1: f32,
	/// Extrinsic parameter.
	pub my_x1y0: f32,
	/// Extrinsic parameter.
	pub my_x0y1: f32,
	/// Extrinsic parameter.
	pub my_x0y0: f32,
}

impl From<sys::Fn2ColorCameraParams> for ColorCameraParams {
	fn from(sys: sys::Fn2ColorCameraParams) -> Self {
		Self {
			fx: sys.fx,
			fy: sys.fy,
			cx: sys.cx,
			cy: sys.cy,
			shift_d: sys.shift_d,
			shift_m: sys.shift_m,
			mx_x3y0: sys.mx_x3y0,
			mx_x0y3: sys.mx_x0y3,
			mx_x2y1: sys.mx_x2y1,
			mx_x1y2: sys.mx_x1y2,
			mx_x2y0: sys.mx_x2y0,
			mx_x0y2: sys.mx_x0y2,
			mx_x1y1: sys.mx_x1y1,
			mx_x1y0: sys.mx_x1y0,
			mx_x0y1: sys.mx_x0y1,
			mx_x0y0: sys.mx_x0y0,
			my_x3y0: sys.my_x3y0,
			my_x0y3: sys.my_x0y3,
			my_x2y1: sys.my_x2y1,
			my_x1y2: sys.my_x1y2,
			my_x2y0: sys.my_x2y0,
			my_x0y2: sys.my_x0y2,
			my_x1y1: sys.my_x1y1,
			my_x1y0: sys.my_x1y0,
			my_x0y1: sys.my_x0y1,
			my_x0y0: sys.my_x0y0,
		}
	}
}

impl From<ColorCameraParams> for sys::Fn2ColorCameraParams {
	fn from(our: ColorCameraParams) -> Self {
		Self {
			fx: our.fx,
			fy: our.fy,
			cx: our.cx,
			cy: our.cy,
			shift_d: our.shift_d,
			shift_m: our.shift_m,
			mx_x3y0: our.mx_x3y0,
			mx_x0y3: our.mx_x0y3,
			mx_x2y1: our.mx_x2y1,
			mx_x1y2: our.mx_x1y2,
			mx_x2y0: our.mx_x2y0,
			mx_x0y2: our.mx_x0y2,
			mx_x1y1: our.mx_x1y1,
			mx_x1y0: our.mx_x1y0,
			mx_x0y1: our.mx_x0y1,
			mx_x0y0: our.mx_x0y0,
			my_x3y0: our.my_x3y0,
			my_x0y3: our.my_x0y3,
			my_x2y1: our.my_x2y1,
			my_x1y2: our.my_x1y2,
			my_x2y0: our.my_x2y0,
			my_x0y2: our.my_x0y2,
			my_x1y1: our.my_x1y1,
			my_x1y0: our.my_x1y0,
			my_x0y1: our.my_x0y1,
			my_x0y0: our.my_x0y0,
		}
	}
}
