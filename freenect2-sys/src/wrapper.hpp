#pragma once

#include <cstddef>
#include <cstdint>

extern "C" {
struct Fn2RustyBorrowedString {
	unsigned char const* data;
	size_t len;
};

typedef void (*Fn2StringCallback)(void*, Fn2RustyBorrowedString);

struct Fn2ColorCameraParams {
	float fx;
	float fy;
	float cx;
	float cy;

	float shift_d;
	float shift_m;
	float mx_x3y0;
	float mx_x0y3;
	float mx_x2y1;
	float mx_x1y2;
	float mx_x2y0;
	float mx_x0y2;
	float mx_x1y1;
	float mx_x1y0;
	float mx_x0y1;
	float mx_x0y0;
	float my_x3y0;
	float my_x0y3;
	float my_x2y1;
	float my_x1y2;
	float my_x2y0;
	float my_x0y2;
	float my_x1y1;
	float my_x1y0;
	float my_x0y1;
	float my_x0y0;
};

struct Fn2DeviceConfig {
	float min_depth;
	float max_depth;
	bool enable_bilateral_filter;
	bool enable_edge_aware_filter;
};

struct Fn2IrCameraParams {
	float fx;
	float fy;
	float cx;
	float cy;
	float k1;
	float k2;
	float k3;
	float p1;
	float p2;
};

enum Fn2FrameType {
	Color,
	Ir,
	Depth,
};

enum Fn2FrameFormat {
	Invalid,
	Raw,
	Float,
	Bgrx,
	Rgbx,
	Gray,
};

struct Fn2Frame {
	size_t width;
	size_t height;
	size_t bytes_per_pixel;
	unsigned char* data;  // owned
	uint32_t timestamp;
	uint32_t sequence;
	float exposure;
	float gain;
	float gamma;
	uint32_t status;
	Fn2FrameFormat format;
};

typedef void (*Fn2FrameCallback)(void*, Fn2Frame, Fn2FrameType);

enum Fn2LogLevel {
	None,
	Error,
	Warning,
	Info,
	Debug,
};

struct Fn2LoggerVTable {
	Fn2LogLevel (*level)(void const* this_);
	void (*log)(void* this_, Fn2LogLevel level, Fn2RustyBorrowedString message);
	void (*drop)(void* this_);
};

struct Fn2Device;
struct Fn2Context;

Fn2Context* fn2_context_new();
int fn2_context_enumerate_devices(Fn2Context* this_);
void fn2_context_get_device_serial_number(Fn2Context const* this_, int idx, Fn2StringCallback callback, void* callback_data);
void fn2_context_get_default_device_serial_number(Fn2Context const* this_, Fn2StringCallback callback, void* callback_data);
Fn2Device* fn2_context_open_device(Fn2Context* this_, int idx);
Fn2Device* fn2_context_open_device_by_serial(Fn2Context* this_, Fn2RustyBorrowedString serial);
Fn2Device* fn2_context_open_default_device(Fn2Context* this_);
void fn2_context_free(Fn2Context* this_);

void fn2_device_get_serial_number(Fn2Device const* this_, Fn2StringCallback callback, void* callback_data);
void fn2_device_get_firmware_version(Fn2Device const* this_, Fn2StringCallback callback, void* callback_data);
Fn2ColorCameraParams fn2_device_get_color_camera_params(Fn2Device const* this_);
Fn2IrCameraParams fn2_device_get_ir_camera_params(Fn2Device const* this_);
void fn2_device_set_color_camera_params(Fn2Device* this_, Fn2ColorCameraParams params);
void fn2_device_set_ir_camera_params(Fn2Device* this_, Fn2IrCameraParams params);
void fn2_device_set_config(Fn2Device* this_, Fn2DeviceConfig config);
void fn2_device_set_frame_listener(Fn2Device* this_, Fn2FrameCallback callback, void* user_data, void drop_user_data(void*));
bool fn2_device_start(Fn2Device* this_);
bool fn2_device_start_streams(Fn2Device* this_, bool rgb, bool depth);
bool fn2_device_stop(Fn2Device* this_);
bool fn2_device_close(Fn2Device* this_);
void fn2_device_free(Fn2Device* this_);

void fn2_set_logger(Fn2LoggerVTable vtable, void* user_data);
}
