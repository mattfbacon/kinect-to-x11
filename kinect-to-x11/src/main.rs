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

#[derive(Debug)]
enum Message {
    Frame(freenect2::Frame, freenect2::FrameType),
    Stop,
}

fn main() {
	simplelog::TermLogger::init(
		log::LevelFilter::Trace,
		simplelog::Config::default(),
		simplelog::TerminalMode::Stderr,
		simplelog::ColorChoice::Auto,
	)
	.unwrap();

	let mut ctx = Context::new();
	if let Some(mut device) = ctx.open_default_device() {
        let (sender, recv) = std::sync::mpsc::sync_channel(4);

        ctrlc::set_handler({
            let sender = sender.clone();
            move || {
                sender.send(Message::Stop).unwrap();
            }
        }).unwrap();

        device.set_frame_listener(move |frame, ty| {
            let _ = sender.send(Message::Frame(frame, ty));
        });
        device.start().unwrap();
        while let Ok(message) = recv.recv() {
            match message {
                Message::Frame(frame, ty) => {
                    if ty == FrameType::Color {
                        let mut image = image::RgbImage::new(
                    }
                }
                Message::Stop => break,
            }
        }
	} else {
		log::error!("no devices available");
	}
}
