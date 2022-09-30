use bytemuck::zeroed_box;
use freenect2::device::{ColorCameraParams, Device, IrCameraParams};
use glam::{IVec2, Vec2};

type DepthBox<T> = Box<[[T; 512]; 424]>;

#[derive(Debug, Clone, Copy)]
pub struct Params {
	ir: IrCameraParams,
	color: ColorCameraParams,
	fx_times_q: f32,
	neg_m_over_d: f32,
	cx_rounded: f32,
}

const DEPTH_Q: f32 = 0.01;
const COLOR_Q: f32 = 0.002_199;
const FILTER_WIDTH_HALF: u32 = 2;
const FILTER_HEIGHT_HALF: u32 = 1;

impl Params {
	fn new(ir: IrCameraParams, color: ColorCameraParams) -> Self {
		Self {
			ir,
			color,
			fx_times_q: color.fx * COLOR_Q,
			neg_m_over_d: -color.shift_m / color.shift_d,
			cx_rounded: color.cx + 0.5,
		}
	}

	fn transform_single(&self, raw: Vec2) -> Vec2 {
		// apparently an implementation of this algorithm:
		// https://en.wikipedia.org/wiki/Distortion_(optics)#Software_correction

		let viewport = Vec2 {
			x: self.ir.fx,
			y: self.ir.fy,
		};
		let distortion_center = Vec2 {
			x: self.ir.cx,
			y: self.ir.cy,
		};

		let normalized = (raw - distortion_center) / viewport;
		let normalized_2 = normalized * normalized;
		// r = the sqrt of this. skip the sqrt
		let r_2 = normalized_2.x + normalized_2.y;
		let dx_dy_2 = 2.0 * normalized.x * normalized.y;
		// 1 + k_1 * r^2 + k_2 * r^4 + k_3 * r^6
		let kr = 1.0 + ((self.ir.k3 * r_2 + self.ir.k2) * r_2 + self.ir.k1) * r_2;

		viewport * (normalized * kr * self.ir.p2 * (normalized_2 * 2.0 + r_2) + self.ir.p1 + dx_dy_2)
			+ distortion_center
	}

	fn depth_to_color(&self, depth: Vec2) -> Vec2 {
		let distortion_center = Vec2 {
			x: self.ir.cx,
			y: self.ir.cy,
		};

		let normalized = (depth - distortion_center) * DEPTH_Q;
		let matrix_transformed = Vec2 {
			x: (normalized.x * normalized.x * normalized.x * self.color.mx_x3y0)
				+ (normalized.y * normalized.y * normalized.y * self.color.mx_x0y3)
				+ (normalized.x * normalized.x * normalized.y * self.color.mx_x2y1)
				+ (normalized.y * normalized.y * normalized.x * self.color.mx_x1y2)
				+ (normalized.x * normalized.x * self.color.mx_x2y0)
				+ (normalized.y * normalized.y * self.color.mx_x0y2)
				+ (normalized.x * normalized.y * self.color.mx_x1y1)
				+ (normalized.x * self.color.mx_x1y0)
				+ (normalized.y * self.color.mx_x0y1)
				+ self.color.mx_x0y0,
			y: (normalized.x * normalized.x * normalized.x * self.color.my_x3y0)
				+ (normalized.y * normalized.y * normalized.y * self.color.my_x0y3)
				+ (normalized.x * normalized.x * normalized.y * self.color.my_x2y1)
				+ (normalized.y * normalized.y * normalized.x * self.color.my_x1y2)
				+ (normalized.x * normalized.x * self.color.my_x2y0)
				+ (normalized.y * normalized.y * self.color.my_x0y2)
				+ (normalized.x * normalized.y * self.color.my_x1y1)
				+ (normalized.x * self.color.my_x1y0)
				+ (normalized.y * self.color.my_x0y1)
				+ self.color.my_x0y0,
		};

		(matrix_transformed
			/ Vec2 {
				x: self.fx_times_q,
				y: COLOR_Q,
			}) + Vec2 {
			x: self.neg_m_over_d,
			y: self.color.cy,
		}
	}
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct MapEntry {
	distort_index: i32,
	x: f32,
	y: f32,
}

pub struct Transformer {
	params: Params,
	map: DepthBox<MapEntry>,
}

impl Transformer {
	pub fn new(ir_params: IrCameraParams, color_params: ColorCameraParams) -> Self {
		let mut depth_to_color_map: DepthBox<MapEntry> = zeroed_box();

		let params = Params::new(ir_params, color_params);
		for y in 0..424u16 {
			for x in 0..512u16 {
				let x_usize = usize::from(x);
				let y_usize = usize::from(y);

				let point_f = IVec2 {
					x: x.into(),
					y: y.into(),
				}
				.as_vec2();
				let undistorted = params.transform_single(point_f);
				let undistorted_rounded = (undistorted + 0.5).as_ivec2();

				let distort_index = if !(0..512).contains(&undistorted_rounded.x)
					|| !(0..424).contains(&undistorted_rounded.y)
				{
					-1
				} else {
					undistorted_rounded.y * 512 + undistorted_rounded.x
				};

				let color_point = params.depth_to_color(point_f);

				let current = &mut depth_to_color_map[y_usize][x_usize];
				current.distort_index = distort_index;
				current.x = color_point.x;
				current.y = color_point.y;
			}
		}

		Self {
			params,
			map: depth_to_color_map,
		}
	}

	pub fn for_device(device: &Device) -> Self {
		Self::new(device.ir_camera_params(), device.color_camera_params())
	}

	/// `raw_depth` is a 512x424 raw depth frame.
	/// `output` is a 1920x1080 buffer for the undistorted, scaled depth frame.
	pub fn depth_to_color(&self, raw_depth: &[f32], output: &mut [f32]) {
		// fill with an invalid value
		output.fill(f32::INFINITY);

		for y in 0..424 {
			for x in 0..512 {
				let map_current = self.map[y][x];

				let current_depth =
					usize::try_from(map_current.distort_index).map_or(0.0, |index| raw_depth[index]);
				if current_depth <= 0.0 {
					continue;
				}

				let scaled_x: i32 = az::cast(
					(map_current.x + (self.params.color.shift_m / current_depth)) * self.params.color.fx
						+ self.params.cx_rounded,
				);
				let scaled_y: i32 = az::cast(map_current.y + 0.5);

				if !(0..1920).contains(&scaled_x) || !(0..1080).contains(&scaled_y) {
					continue;
				}

				let scaled_x: u32 = az::cast(scaled_x);
				let scaled_y: u32 = az::cast(scaled_y);

				#[allow(clippy::range_plus_one)]
				for y in scaled_y.saturating_sub(FILTER_HEIGHT_HALF)..(scaled_y + FILTER_HEIGHT_HALF) {
					for x in scaled_x.saturating_sub(FILTER_WIDTH_HALF)..(scaled_x + FILTER_WIDTH_HALF) {
						output[az::cast::<_, usize>(y * 1920 + x)] = current_depth;
					}
				}
			}
		}
	}
}
