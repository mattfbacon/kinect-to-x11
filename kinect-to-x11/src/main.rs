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

use bytemuck::zeroed_box;
use freenect2::{Context, Device, FrameFormat, FrameType};

mod transformer;
use self::transformer::Transformer;

fn init_logging() {
	simplelog::TermLogger::init(
		log::LevelFilter::Trace,
		simplelog::ConfigBuilder::new()
			.add_filter_allow_str("kinect_to_x11")
			.build(),
		simplelog::TerminalMode::Stderr,
		simplelog::ColorChoice::Auto,
	)
	.unwrap();
}

fn main() {
	init_logging();

	let mut ctx = Context::new();
	if let Some(device) = ctx.open_default_device() {
		log::info!("opened device");

		run(device);
	} else {
		log::error!("no devices available");
	}
}

fn run(mut device: Device) {
	let (sender, recv) = std::sync::mpsc::sync_channel(4);

	log::debug!("setting frame listener");
	device.set_frame_listener(move |frame, ty| {
		log::debug!("frame listener got frame");
		let _ = sender.try_send((frame, ty));
	});

	log::info!("starting device");
	device.start().unwrap();

	let transformer = Transformer::for_device(&device);

	let mut color_thread = None;
	let mut depth_threads = None;

	log::debug!("starting frame loop");
	while let Ok((mut frame, ty)) = recv.recv() {
		log::debug!("receiver got message");
		log::debug!("message is a frame");

		match ty {
			FrameType::Color => {
				assert_eq!(frame.width(), 1920);
				assert_eq!(frame.height(), 1080);
				assert_eq!(frame.bytes_per_pixel(), 4);

				match frame.format() {
					FrameFormat::Rgbx => (),
					FrameFormat::Bgrx => {
						for rgbx in frame.data_mut().chunks_exact_mut(4) {
							rgbx.swap(0, 2);
						}
					}
					_ => unreachable!(),
				}

				if color_thread.is_none() {
					color_thread = Some(std::thread::spawn(move || {
						let mut image =
							image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(1920, 1080, frame.into_data())
								.unwrap();
						image::imageops::flip_vertical_in_place(&mut image);
						image.save("color.png").unwrap();
					}));
				}
			}
			FrameType::Depth => {
				assert_eq!(frame.width(), 512);
				assert_eq!(frame.height(), 424);
				assert_eq!(frame.bytes_per_pixel(), 4);
				assert_eq!(frame.format(), FrameFormat::Float);

				let raw_depth: &[f32] = bytemuck::cast_slice(frame.data());
				let mut depth_frame = zeroed_box::<[f32; 1920 * 1080]>();
				transformer.depth_to_color(raw_depth, &mut *depth_frame);
				let raw_depth = frame.into_data();

				if depth_threads.is_none() {
					depth_threads = Some((
						std::thread::spawn(move || {
							let image = image::ImageBuffer::from_fn(512, 424, |x, y| {
								let depth = f32::from_ne_bytes(
									raw_depth[az::cast::<_, usize>(y * 512 + x) * 4..][..4]
										.try_into()
										.unwrap(),
								);
								let proportion = depth / 4000.0;
								let coolor::Rgb { r, g, b } = coolor::Hsl {
									h: proportion * 240.0,
									s: 1.0,
									l: 0.5,
								}
								.to_rgb();
								image::Rgb([r, g, b])
							});
							image.save("depth-distorted.png").unwrap();
						}),
						std::thread::spawn(move || {
							let image = image::ImageBuffer::from_fn(1920, 1080, |x, y| {
								let depth = depth_frame[az::cast::<_, usize>(y * 1920 + x)];
								let proportion = depth / 4000.0;
								let coolor::Rgb { r, g, b } = coolor::Hsl {
									h: proportion * 240.0,
									s: 1.0,
									l: 0.5,
								}
								.to_rgb();
								image::Rgb([r, g, b])
							});
							image.save("depth.png").unwrap();
						}),
					));
				}
			}
			FrameType::Ir => (),
		}

		if color_thread.is_some() && depth_threads.is_some() {
			break;
		}
	}
	log::info!("frame loop finished");

	log::info!("stopping device");
	device.stop().unwrap();

	log::info!("waiting for threads");
	color_thread.unwrap().join().unwrap();
	let depth_threads = depth_threads.unwrap();
	depth_threads.0.join().unwrap();
	depth_threads.1.join().unwrap();
}
