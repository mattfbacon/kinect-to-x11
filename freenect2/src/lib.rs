//! # freenect2
//!
//! A safe wrapper around [libfreenect2](https://github.com/OpenKinect/libfreenect2).
//!
//! To get started, create a [`Context`] which can be used to discover and open [Device]s.

#![deny(
	absolute_paths_not_starting_with_crate,
	future_incompatible,
	keyword_idents,
	macro_use_extern_crate,
	meta_variable_misuse,
	missing_abi,
	missing_copy_implementations,
	non_ascii_idents,
	nonstandard_style,
	noop_method_call,
	pointer_structural_match,
	private_in_public,
	rust_2018_idioms,
	unused_qualifications
)]
#![warn(
	clippy::pedantic,
	missing_docs,
	missing_copy_implementations,
	missing_debug_implementations
)]
#![allow(clippy::let_underscore_drop)]

use std::os::raw::c_void;

use freenect2_sys as sys;

pub mod context;
pub mod device;
pub mod frame;
mod logger;

pub use context::Context;
pub use device::Device;
pub use frame::{Format as FrameFormat, Frame, Type as FrameType};

unsafe extern "C" fn string_closure(user_data: *mut c_void, borrowed: sys::Fn2RustyBorrowedString) {
	let s = &mut *user_data.cast::<String>();
	*s = String::from_utf8_lossy(std::slice::from_raw_parts(
		borrowed.data.cast(),
		borrowed.len,
	))
	.into_owned();
}
