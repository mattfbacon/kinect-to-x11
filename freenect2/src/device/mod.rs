//! Provides [`Device`] and related types.

use std::os::raw::c_void;
use std::ptr::{addr_of_mut, NonNull};

use freenect2_sys as sys;

use crate::{Frame, FrameType};

mod color_camera_params;
mod depth_config;
mod ir_camera_params;

pub use color_camera_params::ColorCameraParams;
pub use depth_config::DepthConfig;
pub use ir_camera_params::IrCameraParams;

/// Errors that can occur related to the device.
#[derive(Debug, Clone, Copy)]
pub enum Error {
	/// Certain operations cannot be performed if streams are running. This error is returned if you attempt to perform one such operation.
	StreamsRunning,
	/// Certain operations cannot be performed if streams are not running. This error is returned if you attempt to perform one such operation.
	NotRunning,
	/// A library error occurred.
	///
	/// `libfreenect2` does not provide any information about the underlying error.
	Library,
}

impl std::fmt::Display for Error {
	fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		formatter.write_str(match self {
			Self::StreamsRunning => {
				"streams are already running; cannot update configuration or start again"
			}
			Self::NotRunning => "streams are not running; nothing to stop",
			Self::Library => "libfreenect2 library error",
		})
	}
}

impl std::error::Error for Error {}

/// A Kinect V2 device.
#[derive(Debug)]
pub struct Device {
	inner: NonNull<sys::Fn2Device>,
	started: bool,
}

impl Device {
	/// Create a device from a raw pointer to the unsafe equivalent as well as whether streams have started.
	///
	/// Unless you have started streams you can provide `false` for `started`.
	///
	/// # Safety
	///
	/// The pointer in `raw` should have been obtained from a variant of `fn2_context_open_device` or its C++ equivalent `Freenect2::openDevice`.
	/// This includes `fn2_context_open_default_device` and its C++ equivalent `Freenect2::openDefaultDevice`.
	///
	/// # Panics
	///
	/// Panics if `raw` is null.
	#[must_use]
	pub unsafe fn from_raw(raw: *mut sys::Fn2Device, started: bool) -> Self {
		Self {
			inner: NonNull::new(raw).unwrap(),
			started,
		}
	}

	/// Get the serial number of the device.
	#[must_use]
	pub fn serial_number(&self) -> String {
		let mut ret = String::new();
		unsafe {
			sys::fn2_device_get_serial_number(
				self.inner.as_ptr(),
				Some(crate::string_closure),
				addr_of_mut!(ret).cast(),
			);
		}
		ret
	}

	/// Get the firmware version of the device.
	#[must_use]
	pub fn firmware_version(&self) -> String {
		let mut ret = String::new();
		unsafe {
			sys::fn2_device_get_firmware_version(
				self.inner.as_ptr(),
				Some(crate::string_closure),
				addr_of_mut!(ret).cast(),
			);
		}
		ret
	}

	/// Get the parameters of the color camera.
	#[must_use]
	pub fn color_camera_params(&self) -> ColorCameraParams {
		unsafe { sys::fn2_device_get_color_camera_params(self.inner.as_ptr()) }.into()
	}

	/// Set the parameters of the color camera.
	///
	/// # Errors
	///
	/// Fails if streams are running, in which case the camera cannot be reconfigured.
	pub fn set_color_camera_params(&mut self, params: ColorCameraParams) -> Result<(), Error> {
		if self.started {
			Err(Error::StreamsRunning)
		} else {
			unsafe {
				sys::fn2_device_set_color_camera_params(self.inner.as_ptr(), params.into());
			}
			Ok(())
		}
	}

	/// Get the parameters of the IR camera.
	#[must_use]
	pub fn ir_camera_params(&self) -> IrCameraParams {
		unsafe { sys::fn2_device_get_ir_camera_params(self.inner.as_ptr()) }.into()
	}

	/// Set the parameters of the IR camera.
	///
	/// # Errors
	///
	/// Fails if streams are running, in which case the camera cannot be reconfigured.
	pub fn set_ir_camera_params(&mut self, params: IrCameraParams) -> Result<(), Error> {
		if self.started {
			Err(Error::StreamsRunning)
		} else {
			unsafe {
				sys::fn2_device_set_ir_camera_params(self.inner.as_ptr(), params.into());
			}
			Ok(())
		}
	}

	/// Set the device's depth processing configuration.
	pub fn set_depth_config(&self, config: DepthConfig) {
		unsafe { sys::fn2_device_set_config(self.inner.as_ptr(), config.into()) }
	}

	/// Set the frame listener, which is called when the Kinect device has a frame available.
	pub fn set_frame_listener<F: FnMut(Frame, FrameType) + Send + 'static>(&mut self, listener: F) {
		unsafe extern "C" fn call_listener<F: FnMut(Frame, FrameType) + 'static>(
			user_data: *mut c_void,
			frame: sys::Fn2Frame,
			ty: sys::Fn2FrameType,
		) {
			let ty = ty.try_into().unwrap();
			(*user_data.cast::<F>())(Frame::from_sys(frame), ty);
		}

		unsafe extern "C" fn drop_listener<F: FnMut(Frame, FrameType) + 'static>(
			user_data: *mut c_void,
		) {
			drop(Box::from_raw(user_data.cast::<F>()));
		}

		let listener = Box::into_raw(Box::new(listener));
		unsafe {
			sys::fn2_device_set_frame_listener(
				self.inner.as_ptr(),
				Some(call_listener::<F>),
				listener.cast(),
				Some(drop_listener::<F>),
			);
		}
	}

	/// Whether streams have started.
	#[must_use]
	pub fn started(&self) -> bool {
		self.started
	}

	/// Start all streams and data processing.
	///
	/// # Errors
	///
	/// Fails if streams are already running or if the library returns an error.
	pub fn start(&mut self) -> Result<(), Error> {
		if self.started {
			Err(Error::StreamsRunning)
		} else if unsafe { sys::fn2_device_start(self.inner.as_ptr()) } {
			self.started = true;
			Ok(())
		} else {
			Err(Error::Library)
		}
	}

	/// Start some streams and data processing.
	///
	/// # Errors
	///
	/// Fails if streams are already running or if the library returns an error.
	pub fn start_streams(&mut self, rgb: bool, depth: bool) -> Result<(), Error> {
		if self.started {
			Err(Error::StreamsRunning)
		} else if unsafe { sys::fn2_device_start_streams(self.inner.as_ptr(), rgb, depth) } {
			self.started = true;
			Ok(())
		} else {
			Err(Error::Library)
		}
	}

	/// Stop the streams and data processing.
	///
	/// # Errors
	///
	/// Fails if streams are not running or if the library returns an error.
	pub fn stop(&mut self) -> Result<(), Error> {
		if !self.started {
			Err(Error::NotRunning)
		} else if unsafe { sys::fn2_device_stop(self.inner.as_ptr()) } {
			self.started = false;
			Ok(())
		} else {
			Err(Error::Library)
		}
	}
}

impl Drop for Device {
	fn drop(&mut self) {
		unsafe { sys::fn2_device_free(self.inner.as_ptr()) }
	}
}
