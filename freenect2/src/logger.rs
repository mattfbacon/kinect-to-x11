use freenect2_sys as sys;

mod vtable {
	use std::os::raw::c_void;

	use freenect2_sys as sys;

	pub(in crate::logger) unsafe extern "C" fn drop(_this_: *mut c_void) {}
	pub(in crate::logger) unsafe extern "C" fn level(_this_: *const c_void) -> sys::Fn2LogLevel {
		// don't actually do anything here; filter in `log` instead
		sys::Fn2LogLevel_Debug
	}
	pub(in crate::logger) unsafe extern "C" fn log(
		_this_: *mut c_void,
		level: sys::Fn2LogLevel,
		message: sys::Fn2RustyBorrowedString,
	) {
		let message =
			String::from_utf8_lossy(std::slice::from_raw_parts(message.data, message.len)).into_owned();
		let level = match level {
			sys::Fn2LogLevel_Error => log::Level::Error,
			sys::Fn2LogLevel_Warning => log::Level::Warn,
			sys::Fn2LogLevel_Info => log::Level::Info,
			sys::Fn2LogLevel_Debug | sys::Fn2LogLevel_None => log::Level::Debug,
			_ => unreachable!(),
		};
		log::Log::log(
			log::logger(),
			&log::Record::builder()
				.args(std::format_args!("{message}"))
				.level(level)
				.module_path(Some("libfreenect2"))
				.build(),
		);
	}
}

pub(crate) fn initialize() {
	use once_cell::sync::OnceCell;
	static INIT: OnceCell<()> = OnceCell::new();

	INIT.get_or_init(|| unsafe {
		sys::fn2_set_logger(
			sys::Fn2LoggerVTable {
				drop: Some(vtable::drop),
				level: Some(vtable::level),
				log: Some(vtable::log),
			},
			std::ptr::null_mut(),
		);
	});
}
