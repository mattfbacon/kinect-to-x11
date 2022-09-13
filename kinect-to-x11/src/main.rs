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
#![warn(clippy::pedantic)]
#![allow(clippy::let_underscore_drop)]
#![forbid(unsafe_code)]

use freenect2::Context;

fn main() {
	simplelog::TermLogger::init(
		log::LevelFilter::Trace,
		simplelog::Config::default(),
		simplelog::TerminalMode::Stderr,
		simplelog::ColorChoice::Auto,
	)
	.unwrap();

	let mut ctx = Context::new();
	if let Some(device) = ctx.open_default_device() {
		log::info!(
			"there is a device. its serial number is {:?}",
			device.serial_number()
		);
	} else {
		log::error!("no devices available");
	}
}
