#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use flume::{Receiver, TryRecvError};
use sdl2::audio::AudioCallback;

pub struct Callback {
    pub rx: Receiver<f32>
}

impl AudioCallback for Callback {
    type Channel = f32;

    fn callback(&mut self, buffer: &mut [Self::Channel]) {

        for x in buffer.iter_mut() {

            let result = self.rx.try_recv();
            match result {
                Ok(sample) => { *x = sample; }
                Err(TryRecvError::Empty) => { *x = 0.0; }
                Err(TryRecvError::Disconnected) => { panic!("Audio channel disconnected! Shutting down...") }
            }
        }
    }
}