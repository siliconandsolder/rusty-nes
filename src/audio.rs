#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

extern crate sdl2;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use self::sdl2::audio::{AudioQueue, AudioSpecDesired};
use std::thread::JoinHandle;
use flume::{Sender, Receiver, TryRecvError};
use self::sdl2::AudioSubsystem;
use std::cell::RefCell;

use portaudio::*;
use portaudio::stream::CallbackResult;

const AUDIO_HERTZ: u16 = 44100;
const BUFFER_SIZE: u16 = 2048;

struct AudioSubsystemSendWrapper(AudioSubsystem);

unsafe impl Sync for AudioSubsystemSendWrapper {}
unsafe impl Send for AudioSubsystemSendWrapper {}

pub struct Audio {
	stream: portaudio::Stream<NonBlocking, Output<f32>>,
	sender: Sender<f32>
}

impl Audio {
	pub fn new() -> Self {

		let paudio = PortAudio::new().unwrap();
		let defaultDevice = paudio.default_output_device().unwrap();
		let outputInfo = paudio.device_info(defaultDevice).unwrap();
		let latency = outputInfo.default_high_input_latency;
		let params = portaudio::StreamParameters::<f32>::new(defaultDevice, 1, true, latency);
		let mut settings = portaudio::OutputStreamSettings::new(params, AUDIO_HERTZ as f64, BUFFER_SIZE as u32);
		settings.flags = portaudio::stream_flags::CLIP_OFF;

		let (tx, rx) = flume::unbounded();

		let callback
			= move |portaudio::OutputStreamCallbackArgs { buffer, frames, .. }| {

			for i in 0..frames {
				let result = rx.try_recv();

				match result {
					Ok(sample) => { buffer[i] = sample; }
					Err(TryRecvError::Empty) => { buffer[i] = 0.0f32; }
					Err(TryRecvError::Disconnected) => { panic!("Audio channel disconnected! Shutting down...") }
				}
			}

			return portaudio::Continue;
		};

		let mut stream = paudio.open_non_blocking_stream(settings, callback).unwrap();

		stream.start().unwrap();

		Audio {
			stream: stream,
			sender: tx
		}
	}

	pub fn pushSample(&mut self, sample: f32) -> () {
		self.sender.send(sample);
	}
}

impl Drop for Audio {
	fn drop(&mut self) {
		self.stream.close().unwrap();
		self.stream.stop().unwrap();
	}
}
