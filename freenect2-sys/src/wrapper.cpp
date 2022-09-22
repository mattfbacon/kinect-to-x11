#include <cstring>
#include <libfreenect2/libfreenect2.hpp>
#include <libfreenect2/logger.h>

#include "wrapper.hpp"

static inline Fn2RustyBorrowedString borrow_string(std::string const& cxx) {
	return Fn2RustyBorrowedString{ reinterpret_cast<unsigned char const*>(cxx.data()), cxx.length() };
}

static inline Fn2ColorCameraParams to_ours(libfreenect2::Freenect2Device::ColorCameraParams const lib) {
	return {
		lib.fx,      lib.fy,      lib.cx,      lib.cy,      lib.shift_d, lib.shift_m, lib.mx_x3y0, lib.mx_x0y3, lib.mx_x2y1,
		lib.mx_x1y2, lib.mx_x2y0, lib.mx_x0y2, lib.mx_x1y1, lib.mx_x1y0, lib.mx_x0y1, lib.mx_x0y0, lib.my_x3y0, lib.my_x0y3,
		lib.my_x2y1, lib.my_x1y2, lib.my_x2y0, lib.my_x0y2, lib.my_x1y1, lib.my_x1y0, lib.my_x0y1, lib.my_x0y0,
	};
}

static inline libfreenect2::Freenect2Device::ColorCameraParams from_ours(Fn2ColorCameraParams const our) {
	return {
		our.fx,      our.fy,      our.cx,      our.cy,      our.shift_d, our.shift_m, our.mx_x3y0, our.mx_x0y3, our.mx_x2y1,
		our.mx_x1y2, our.mx_x2y0, our.mx_x0y2, our.mx_x1y1, our.mx_x1y0, our.mx_x0y1, our.mx_x0y0, our.my_x3y0, our.my_x0y3,
		our.my_x2y1, our.my_x1y2, our.my_x2y0, our.my_x0y2, our.my_x1y1, our.my_x1y0, our.my_x0y1, our.my_x0y0,
	};
}

static inline Fn2IrCameraParams to_ours(libfreenect2::Freenect2Device::IrCameraParams const lib) {
	return {
		lib.fx, lib.fy, lib.cx, lib.cy, lib.k1, lib.k2, lib.k3, lib.p1, lib.p2,
	};
}

static inline libfreenect2::Freenect2Device::IrCameraParams from_ours(Fn2IrCameraParams const our) {
	return {
		our.fx, our.fy, our.cx, our.cy, our.k1, our.k2, our.k3, our.p1, our.p2,
	};
}

static inline Fn2DeviceConfig to_ours(libfreenect2::Freenect2Device::Config const lib) {
	return {
		lib.MinDepth,
		lib.MaxDepth,
		lib.EnableBilateralFilter,
		lib.EnableEdgeAwareFilter,
	};
}

static inline libfreenect2::Freenect2Device::Config from_ours(Fn2DeviceConfig const our) {
	libfreenect2::Freenect2Device::Config ret;
	ret.MinDepth = our.min_depth;
	ret.MaxDepth = our.max_depth;
	ret.EnableBilateralFilter = our.enable_bilateral_filter;
	ret.EnableEdgeAwareFilter = our.enable_edge_aware_filter;
	return ret;
}

static inline Fn2FrameType to_ours(libfreenect2::Frame::Type const lib) {
	switch (lib) {
		case libfreenect2::Frame::Type::Color:
			return Color;
		case libfreenect2::Frame::Type::Ir:
			return Ir;
		case libfreenect2::Frame::Type::Depth:
			return Depth;
		default:
			__builtin_unreachable();
	}
}

static inline Fn2FrameFormat to_ours(libfreenect2::Frame::Format const lib) {
	switch (lib) {
		case libfreenect2::Frame::Format::Invalid:
			return Invalid;
		case libfreenect2::Frame::Format::Raw:
			return Raw;
		case libfreenect2::Frame::Format::Float:
			return Bgrx;
		case libfreenect2::Frame::Format::BGRX:
			return Bgrx;
		case libfreenect2::Frame::Format::RGBX:
			return Rgbx;
		case libfreenect2::Frame::Format::Gray:
			return Gray;
		default:
			__builtin_unreachable();
	}
}

static inline Fn2Frame to_ours(libfreenect2::Frame* const lib) {
	auto size = lib->bytes_per_pixel;
	if (lib->format != libfreenect2::Frame::Format::Raw) {
		size *= lib->width * lib->height;
	}
	unsigned char* data = (unsigned char*)malloc(size);
	memcpy(data, lib->data, size);
	return {
		lib->width, lib->height, lib->bytes_per_pixel, data, lib->timestamp, lib->sequence, lib->exposure, lib->gain, lib->gamma, lib->status, to_ours(lib->format),
	};
}

struct ShimFrameListener : libfreenect2::FrameListener {
	Fn2FrameCallback callback;
	void* user_data;
	void (*drop_user_data)(void*);

	ShimFrameListener(Fn2FrameCallback const callback, void* const user_data, void (*const drop_user_data)(void*)) : callback(callback), user_data(user_data), drop_user_data(drop_user_data) {}

	virtual bool onNewFrame(libfreenect2::Frame::Type const type, libfreenect2::Frame* const frame) override {
		callback(user_data, to_ours(frame), to_ours(type));
		return false;
	}

	~ShimFrameListener() {
		drop_user_data(user_data);
	}
};

struct Fn2Device {
	libfreenect2::Freenect2Device* inner;
	ShimFrameListener* listener = nullptr;

	~Fn2Device() {
		delete inner;
		if (listener) {
			delete listener;
		}
	}
};

struct Fn2Context {
	libfreenect2::Freenect2 inner;

	Fn2Context() : inner{} {}
};

extern "C" {

Fn2Context* fn2_context_new() {
	return new Fn2Context{};
}

int fn2_context_enumerate_devices(Fn2Context* const this_) {
	return this_->inner.enumerateDevices();
}

void fn2_context_get_device_serial_number(Fn2Context const* const this_, int const idx, Fn2StringCallback const callback, void* const callback_data) {
	// SAFETY: it's just a getter, so `const_cast` is fine
	auto const cxx = const_cast<libfreenect2::Freenect2&>(this_->inner).getDeviceSerialNumber(idx);
	callback(callback_data, borrow_string(cxx));
}

void fn2_context_get_default_device_serial_number(Fn2Context const* const this_, Fn2StringCallback const callback, void* const callback_data) {
	// SAFETY: it's just a getter, so `const_cast` is fine
	auto const cxx = const_cast<libfreenect2::Freenect2&>(this_->inner).getDefaultDeviceSerialNumber();
	callback(callback_data, borrow_string(cxx));
}

Fn2Device* fn2_context_open_device(Fn2Context* const this_, int const idx) {
	auto* const inner = this_->inner.openDevice(idx);
	if (inner) {
		return new Fn2Device{ inner };
	} else {
		return nullptr;
	}
}

Fn2Device* fn2_context_open_device_by_serial(Fn2Context* const this_, Fn2RustyBorrowedString const serial) {
	std::string serial_cxx{ reinterpret_cast<char const*>(serial.data), serial.len };
	auto* const inner = this_->inner.openDevice(serial_cxx);
	if (inner) {
		return new Fn2Device{ inner };
	} else {
		return nullptr;
	}
}

Fn2Device* fn2_context_open_default_device(Fn2Context* const this_) {
	auto* const inner = this_->inner.openDefaultDevice();
	if (inner) {
		return new Fn2Device{ inner };
	} else {
		return nullptr;
	}
}

void fn2_context_free(Fn2Context* const this_) {
	delete this_;
}

void fn2_device_get_serial_number(Fn2Device const* const this_, Fn2StringCallback const callback, void* const callback_data) {
	auto const cxx = this_->inner->getSerialNumber();
	callback(callback_data, borrow_string(cxx));
}

void fn2_device_get_firmware_version(Fn2Device const* const this_, Fn2StringCallback const callback, void* const callback_data) {
	auto const cxx = this_->inner->getFirmwareVersion();
	callback(callback_data, borrow_string(cxx));
}

Fn2ColorCameraParams fn2_device_get_color_camera_params(Fn2Device const* const this_) {
	return to_ours(this_->inner->getColorCameraParams());
}

Fn2IrCameraParams fn2_device_get_ir_camera_params(Fn2Device const* const this_) {
	return to_ours(this_->inner->getIrCameraParams());
}

void fn2_device_set_color_camera_params(Fn2Device* const this_, Fn2ColorCameraParams const params) {
	this_->inner->setColorCameraParams(from_ours(params));
}

void fn2_device_set_ir_camera_params(Fn2Device* const this_, Fn2IrCameraParams const params) {
	this_->inner->setIrCameraParams(from_ours(params));
}

void fn2_device_set_config(Fn2Device* const this_, Fn2DeviceConfig const config) {
	this_->inner->setConfiguration(from_ours(config));
}

void fn2_device_set_frame_listener(Fn2Device* const this_, void (*const callback)(void*, Fn2Frame, Fn2FrameType), void* const user_data, void (*const drop_user_data)(void*)) {
	if (this_->listener == nullptr) {
		this_->listener = new ShimFrameListener{ callback, user_data, drop_user_data };
		this_->inner->setColorFrameListener(this_->listener);
		this_->inner->setIrAndDepthFrameListener(this_->listener);
	} else {
		this_->listener->callback = callback;
		this_->listener->user_data = user_data;
		this_->listener->drop_user_data = drop_user_data;
	}
}

bool fn2_device_start(Fn2Device* const this_) {
	return this_->inner->start();
}

bool fn2_device_start_streams(Fn2Device* const this_, bool const rgb, bool const depth) {
	return this_->inner->startStreams(rgb, depth);
}

bool fn2_device_stop(Fn2Device* const this_) {
	return this_->inner->stop();
}

bool fn2_device_close(Fn2Device* const this_) {
	return this_->inner->close();
}

void fn2_device_free(Fn2Device* const this_) {
	delete this_;
}

struct Logger : libfreenect2::Logger {
	Fn2LoggerVTable vtable;
	void* user_data;

	Logger(Fn2LoggerVTable vtable, void* user_data) : vtable(vtable), user_data(user_data) {}

	virtual libfreenect2::Logger::Level level() const override {
		switch ((vtable.level)(user_data)) {
			case Fn2LogLevel::None:
				return libfreenect2::Logger::Level::None;
			case Fn2LogLevel::Error:
				return libfreenect2::Logger::Level::Error;
			case Fn2LogLevel::Warning:
				return libfreenect2::Logger::Level::Warning;
			case Fn2LogLevel::Info:
				return libfreenect2::Logger::Level::Info;
			case Fn2LogLevel::Debug:
				return libfreenect2::Logger::Level::Debug;
			default:
				__builtin_unreachable();
		}
	}

	virtual void log(libfreenect2::Logger::Level level, std::string const& message) override {
		Fn2LogLevel our_level = Fn2LogLevel::None;
		switch (level) {
			case libfreenect2::Logger::Level::Error:
				our_level = Fn2LogLevel::Error;
				break;
			case libfreenect2::Logger::Level::Warning:
				our_level = Fn2LogLevel::Warning;
				break;
			case libfreenect2::Logger::Level::Info:
				our_level = Fn2LogLevel::Info;
				break;
			case libfreenect2::Logger::Level::Debug:
				our_level = Fn2LogLevel::Debug;
				break;
			case libfreenect2::Logger::Level::None:
				our_level = Fn2LogLevel::None;
				break;
		}

		(vtable.log)(user_data, our_level, borrow_string(message));
	}

	~Logger() {
		(vtable.drop)(user_data);
	}
};

void fn2_set_logger(Fn2LoggerVTable vtable, void* user_data) {
	libfreenect2::setGlobalLogger(new Logger{ vtable, user_data });
}
}
