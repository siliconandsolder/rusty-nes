#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

extern crate sdl2;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use self::sdl2::audio::{AudioQueue, AudioSpecDesired};
use std::thread::JoinHandle;
use flume::{Sender, Receiver};
use self::sdl2::AudioSubsystem;
use std::cell::RefCell;

const AUDIO_HERTZ: u16 = 44100;
const BUFFER_SIZE: u16 = 512;

struct AudioSubsystemSendWrapper(AudioSubsystem);

unsafe impl Sync for AudioSubsystemSendWrapper {}
unsafe impl Send for AudioSubsystemSendWrapper {}

pub struct Audio {
	isDone: Arc<AtomicBool>,
	handle: Option<JoinHandle<()>>,
	sender: Sender<f32>
}

impl Audio {
	pub fn new(audioSystem: AudioSubsystem) -> Self {

		let isDoneOrg = Arc::new(AtomicBool::from(false));
		let isDone = isDoneOrg.clone();
		let (tx, rx) = flume::unbounded();
		let system = AudioSubsystemSendWrapper(audioSystem);


		let handle = std::thread::spawn(move || {
			let mut buffer1: Vec<f32> = vec![0.0; BUFFER_SIZE as usize];
			let mut buffer2: Vec<f32> = vec![0.0; BUFFER_SIZE as usize];
			let mut bufferIdx: u16 = 0;
			let mut useBuffer1 = true;


			let specs = AudioSpecDesired {
				freq: Some(AUDIO_HERTZ as i32),
				channels: Some(1),
				samples: Some(BUFFER_SIZE)
			};

			let queue = system.0.open_queue::<f32, _>(None, &specs).unwrap();
			queue.resume();

			while !isDone.load(Ordering::SeqCst) {

				let mut sample: f32 = rx.recv().unwrap();
				//println!("Got a sample!");

				if useBuffer1 {
					buffer1[bufferIdx as usize] = sample;
				}
				else {
					buffer2[bufferIdx as usize] = sample;
				}
				bufferIdx += 1;

				if bufferIdx == BUFFER_SIZE {
					bufferIdx = 0;
					if useBuffer1 {
						queue.queue(buffer1.as_slice());
					}
					else {
						queue.queue(buffer2.as_slice());
					}
					useBuffer1 = !useBuffer1;
				}
			}
		});

		Audio {
			isDone: isDoneOrg,
			handle: Some(handle),
			sender: tx,
		}
	}

	pub fn pushSample(&mut self, sample: f32) -> () {
		self.sender.send(sample);
	}
}

impl Drop for Audio {
	fn drop(&mut self) {
		self.isDone.store(true, Ordering::SeqCst);
		self.handle.take().unwrap().join().unwrap();
	}
}
