#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use flume::{Sender, Receiver, TryRecvError};

use portaudio::*;
use portaudio::stream::CallbackResult;
use crate::apu::filter::Filter;

const AUDIO_HERTZ: u16 = 44100;
const BUFFER_SIZE: u16 = 512;

pub struct Audio {
	stream: portaudio::Stream<NonBlocking, Output<f32>>,
	sender: Sender<f32>,

	// filters
	highPassFilter1: Filter,
	highPassFilter2: Filter,
	lowPassFilter: Filter
}

impl Audio {
	pub fn new() -> Self {

		let paudio = PortAudio::new().unwrap();
		let defaultDevice = paudio.default_output_device().unwrap();
		let outputInfo = paudio.device_info(defaultDevice).unwrap();
		let latency = outputInfo.default_low_output_latency;
		let params = portaudio::StreamParameters::<f32>::new(defaultDevice, 1, true, latency);
		let mut settings = portaudio::OutputStreamSettings::new(params, AUDIO_HERTZ as f64, BUFFER_SIZE as u32);

		let (tx , rx) = flume::unbounded();

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
			sender: tx,
			highPassFilter1: Filter::HighPassFilter(AUDIO_HERTZ as f32, 90 as f32),
			highPassFilter2: Filter::HighPassFilter(AUDIO_HERTZ as f32, 440 as f32),
			lowPassFilter: Filter::LowPassFilter(AUDIO_HERTZ as f32, 14000 as f32),
		}
	}

	pub fn pushSample(&mut self, sample: f32) -> () {
		let filteredSample = self.filterSample(sample);
		self.sender.send(filteredSample);
	}

	fn filterSample(&mut self, sample: f32) -> f32 {
		let mut fSample = self.highPassFilter1.Step(sample);
		fSample = self.highPassFilter2.Step(fSample);
		fSample = self.lowPassFilter.Step(fSample);
		return fSample;
	}
}

impl Drop for Audio {
	fn drop(&mut self) {
		self.stream.close().unwrap();
		self.stream.stop().unwrap();
	}
}
