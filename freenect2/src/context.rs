//! Provides [`Context`].

use std::ptr::{addr_of_mut, NonNull};

use freenect2_sys as sys;

use crate::device::Device;

/// The context used to discover and open devices.
#[derive(Debug)]
pub struct Context {
	inner: NonNull<sys::Fn2Context>,
	num_devices: u32,
}

impl Default for Context {
	fn default() -> Self {
		Self::new()
	}
}

impl Context {
	/// Create a new context from a raw pointer to the unsafe equivalent, as well as the number of devices.
	///
	/// You should have gotten the number of devices by calling `fn2_context_enumerate_devices` or its C++ equivalent `Freenect2::enumerateDevices`.
	///
	/// # Safety
	///
	/// The pointer in `raw` should have been obtained from `fn2_context_new` or its C++ equivalent `new Freenect2`.
	///
	/// # Panics
	///
	/// Panics if `raw` is null.
	#[must_use]
	pub unsafe fn from_raw(raw: *mut sys::Fn2Context, num_devices: u32) -> Self {
		crate::logger::initialize();

		Self {
			inner: NonNull::new(raw).unwrap(),
			num_devices,
		}
	}

	/// Create a new context.
	///
	/// This automatically enumerates the devices.
	#[must_use]
	pub fn new() -> Self {
		crate::logger::initialize();

		let raw = unsafe { sys::fn2_context_new() };
		let inner = NonNull::new(raw).expect("`new` returned nullptr");
		let mut ret = Self {
			inner,
			num_devices: 0,
		};
		ret.enumerate_devices();
		ret
	}

	/// Gets the number of devices that were discovered during enumeration.
	#[must_use]
	pub fn num_devices(&self) -> u32 {
		self.num_devices
	}

	/// Get the serial number of a device by its index.
	///
	/// Returns `None` if the device index is invalid (`>= num_devices()`).
	#[must_use]
	pub fn device_serial_number(&self, device_index: u32) -> Option<String> {
		let mut ret = String::new();
		unsafe {
			sys::fn2_context_get_device_serial_number(
				self.inner.as_ptr(),
				device_index.try_into().ok()?,
				Some(crate::string_closure),
				addr_of_mut!(ret).cast(),
			);
		}
		Some(ret).filter(|ret| !ret.is_empty())
	}

	/// Get the serial number of the default device.
	///
	/// Returns `None` if no devices were discovered and thus there is no default device.
	#[must_use]
	pub fn default_device_serial_number(&self) -> Option<String> {
		let mut ret = String::new();
		unsafe {
			sys::fn2_context_get_default_device_serial_number(
				self.inner.as_ptr(),
				Some(crate::string_closure),
				addr_of_mut!(ret).cast(),
			);
		}
		Some(ret).filter(|ret| !ret.is_empty())
	}

	/// Open a device by its index.
	///
	/// Returns `None` if the device index is invalid (`>= num_devices()`), or if the device is already open.
	pub fn open_device(&mut self, device_idx: u32) -> Option<Device> {
		let raw =
			unsafe { sys::fn2_context_open_device(self.inner.as_ptr(), device_idx.try_into().ok()?) };
		if raw.is_null() {
			None
		} else {
			Some(unsafe { Device::from_raw(raw, false) })
		}
	}

	/// Open the default device.
	///
	/// Returns `None` if no devices were discovered and thus there is no default device, or if the device is already open.
	pub fn open_default_device(&mut self) -> Option<Device> {
		let raw = unsafe { sys::fn2_context_open_default_device(self.inner.as_ptr()) };
		if raw.is_null() {
			None
		} else {
			Some(unsafe { Device::from_raw(raw, false) })
		}
	}

	/// Opens a device based on its serial number.
	///
	/// Returns `None` if there is no device by that serial number, or if the device is already open.
	pub fn open_device_by_serial(&mut self, serial: &str) -> Option<Device> {
		let raw = unsafe {
			sys::fn2_context_open_device_by_serial(
				self.inner.as_ptr(),
				sys::Fn2RustyBorrowedString {
					data: serial.as_ptr(),
					len: serial.len(),
				},
			)
		};
		if raw.is_null() {
			None
		} else {
			Some(unsafe { Device::from_raw(raw, false) })
		}
	}

	fn enumerate_devices(&mut self) {
		self.num_devices = unsafe { sys::fn2_context_enumerate_devices(self.inner.as_ptr()) }
			.try_into()
			.unwrap();
	}
}

impl Drop for Context {
	fn drop(&mut self) {
		unsafe {
			sys::fn2_context_free(self.inner.as_ptr());
		}
	}
}
