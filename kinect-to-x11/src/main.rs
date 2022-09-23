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

use freenect2::{Context, Frame, FrameFormat, FrameType};

#[derive(Debug)]
enum Message {
	Frame(Frame, FrameType),
	Stop,
}

fn main() {
	simplelog::TermLogger::init(
		log::LevelFilter::Info,
		simplelog::Config::default(),
		simplelog::TerminalMode::Stderr,
		simplelog::ColorChoice::Auto,
	)
	.unwrap();

	let mut ctx = Context::new();
	if let Some(mut device) = ctx.open_default_device() {
		println!("opened device");

		let (sender, recv) = std::sync::mpsc::sync_channel(4);

		println!("setting frame listener");
		device.set_frame_listener({
			let sender = sender.clone();
			move |frame, ty| {
				println!("frame listener got frame");
				let _ = sender.try_send(Message::Frame(frame, ty));
			}
		});

		println!("setting ctrl-c handler");
		ctrlc::set_handler({
			let sender = sender.clone();
			move || {
				println!("received ctrl-c, sending stop message");
				sender.send(Message::Stop).unwrap();
			}
		})
		.unwrap();

		println!("starting device");
		device.start().unwrap();

		let mut color_thread = None;
		let mut depth_thread = None;

		println!("starting frame loop");
		while let Ok(message) = recv.recv() {
			println!("receiver got message");
			match message {
				Message::Frame(mut frame, ty) => {
					println!("message is a frame");

					if ty == FrameType::Color && color_thread.is_none() {
						println!("frame is color");

						if frame.format() == FrameFormat::Bgrx {
							println!("bgrx! transforming.");
							for rgbx in frame.data_mut().chunks_exact_mut(4) {
								rgbx.swap(0, 2);
							}
						}

						let image = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
							frame.width().try_into().unwrap(),
							frame.height().try_into().unwrap(),
							frame.into_data(),
						)
						.unwrap();
						color_thread = Some(std::thread::spawn(move || {
							image
								.save_with_format("color.png", image::ImageFormat::Png)
								.unwrap();
							println!("done saving");
						}));
					} else if ty == FrameType::Depth && depth_thread.is_none() {
						println!(
							"frame is depth, width={:?}, height={:?}, format={:?}",
							frame.width(),
							frame.height(),
							frame.format()
						);

						debug_assert_eq!(frame.bytes_per_pixel(), 4);
						debug_assert_eq!(frame.format(), FrameFormat::Float);

						let width = frame.width().try_into().unwrap();
						let data = frame.data();
						let image =
							image::ImageBuffer::from_fn(width, frame.height().try_into().unwrap(), |x, y| {
								const MAX: f32 = 4000.0; // 4 meters, conservative maximum

								let idx = y * width + x;
								let distance = f32::from_ne_bytes(
									data[usize::try_from(idx).unwrap() * 4..][..4]
										.try_into()
										.unwrap(),
								);
								let proportion = distance / MAX;
								let hsl = coolor::Hsl {
									h: (1.0 - proportion) * 240.0,
									s: 1.0,
									l: 0.5,
								};
								let coolor::Rgb { r, g, b } = hsl.to_rgb();
								image::Rgb([r, g, b])
							});

						depth_thread = Some(std::thread::spawn(move || {
							image
								.save_with_format("depth.png", image::ImageFormat::Png)
								.unwrap();
							println!("done saving");
						}));
					}

					if color_thread.is_some() && depth_thread.is_some() {
						break;
					}
				}
				Message::Stop => {
					println!("message is Stop");
					break;
				}
			}
		}
		println!("frame loop finished");

		println!("stopping device");
		device.stop().unwrap();

		color_thread.unwrap().join().unwrap();
		depth_thread.unwrap().join().unwrap();
	} else {
		log::error!("no devices available");
	}
}
