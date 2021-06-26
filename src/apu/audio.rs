#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use flume::{Sender, Receiver, TryRecvError};

use portaudio::*;
use portaudio::stream::CallbackResult;
use sdl2::audio::{AudioQueue, AudioSpecDesired};
use sdl2::AudioSubsystem;
use crate::apu::filter::Filter;
use std::borrow::Borrow;

const AUDIO_HERTZ: u16 = 44100;
const BUFFER_SIZE: usize = 512;

pub struct Audio {
    queue: AudioQueue<f32>,
    buffer: Vec<f32>,
    bufferIdx: usize,

    // filters
    highPassFilter1: Filter,
    highPassFilter2: Filter,
    lowPassFilter: Filter,
}

impl Audio {
    pub fn new(audioSystem: AudioSubsystem) -> Self {

        let specs = AudioSpecDesired {
            freq: Some(AUDIO_HERTZ as i32),
            channels: Some(1),
            samples: Some(BUFFER_SIZE as u16)
        };

        let queue = audioSystem.open_queue::<f32, _>(None, &specs).unwrap();
        queue.resume();

        Audio {
            queue,
            buffer: vec![0.0; BUFFER_SIZE],
            bufferIdx: 0,
            highPassFilter1: Filter::HighPassFilter(AUDIO_HERTZ as f32, 90 as f32),
            highPassFilter2: Filter::HighPassFilter(AUDIO_HERTZ as f32, 440 as f32),
            lowPassFilter: Filter::LowPassFilter(AUDIO_HERTZ as f32, 14000 as f32),
        }
    }

    pub fn pushSample(&mut self, sample: f32) -> () {
        let filteredSample = self.filterSample(sample);
        self.buffer[self.bufferIdx] = filteredSample;
        self.bufferIdx += 1;

        if self.bufferIdx == BUFFER_SIZE {
            self.queue.queue(self.buffer.as_slice());
            self.bufferIdx = 0;
        }
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
        self.queue.pause();
        self.queue.clear();
    }
}
