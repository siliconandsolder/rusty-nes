#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

extern crate sdl2;

use crate::cpu::Cpu;
use crate::ppu::Ppu;
use crate::cartridge::Cartridge;
use crate::data_bus::DataBus;
use crate::clock::Clocked;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use sdl2::AudioSubsystem;
use crate::ppu_bus::PpuBus;
use crate::controller::Controller;
use crate::apu::Apu;
use crate::save_load::SaveState;
use pixels::{Pixels, SurfaceTexture};
use rfd::FileDialog;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event::Event::WindowEvent;
use winit::event::WindowEvent::CloseRequested;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::{Window, WindowBuilder};
use winit_input_helper::WinitInputHelper;
use crate::gui::Gui;
use crate::gui_commands::GuiCommands;

const CPU_HERTZ_PER_CYCLE: f64 = 1.0 / 1789773.0;
const AUDIO_HERTZ_PER_SAMPLE: f64 = 1.0 / 44100.0;

const SCREEN_WIDTH: u32 = 768;
const SCREEN_HEIGHT: u32 = 720;

const PIXEL_WIDTH: u32 = 256;
const PIXEL_HEIGHT: u32 = 240;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    NotLoaded,
    Loaded
}

pub struct Console {
    window: Window,
    eventLoop: Option<EventLoop<()>>,
    pixels: Pixels,
    gui: Gui,
    guiCommands: Rc<RefCell<GuiCommands>>,
    audioSystem: Rc<RefCell<AudioSubsystem>>,
    cpu: Rc<RefCell<Cpu>>,
    ppu: Rc<RefCell<Ppu>>,
    apu: Rc<RefCell<Apu>>,
    bus: Rc<RefCell<DataBus>>,
    cartridge: Option<Rc<RefCell<Cartridge>>>,
    gameState: GameState
}

impl Console {
    pub fn new(game: Option<&str>) -> Self {
        let sdl = sdl2::init().unwrap();
        let audioSystem = Rc::new(RefCell::new(sdl.audio().unwrap()));

        let guiCommands = Rc::new(RefCell::new(GuiCommands::Default));
        let eventLoop = EventLoop::new();
        let size = LogicalSize::new(SCREEN_WIDTH, SCREEN_HEIGHT);
        let window =
            WindowBuilder::new()
                .with_title("RustyNES")
                .with_inner_size(size)
                .with_min_inner_size(size)
                .with_max_inner_size(size)
                .build(&eventLoop)
                .unwrap();

        let windowSize = window.inner_size();
        let scale = window.scale_factor();
        let texture = SurfaceTexture::new(windowSize.width, windowSize.height, &window);
        let pixels = Pixels::new(PIXEL_WIDTH, PIXEL_HEIGHT, texture).unwrap();
        let gui = Gui::new(windowSize.width, windowSize.height, scale as f32, guiCommands.clone(), &pixels);


        let bus = Rc::new(RefCell::new(DataBus::new()));
        bus.borrow_mut().attachController1(Rc::new(RefCell::new(Controller::new())));
        let cpu = Rc::new(RefCell::new(Cpu::new(bus.clone())));
        bus.borrow_mut().attachCpu(cpu.clone());
        let apu = Rc::new(RefCell::new(Apu::new(bus.clone(), audioSystem.clone())));
        bus.borrow_mut().attachApu(apu.clone());

        let mut ppuBus = PpuBus::new();
        let mut cartridge: Option<Rc<RefCell<Cartridge>>> = None;
        let mut gameState = GameState::NotLoaded;

        if game.is_some() {
            cartridge = Some(Rc::new(RefCell::new(Cartridge::new(Path::new(game.unwrap())))));
            bus.borrow_mut().attachCartridge(cartridge.as_ref().unwrap().clone());
            ppuBus.attachCartridge(cartridge.as_ref().unwrap().clone());
            cpu.borrow_mut().init();
            gameState = GameState::Loaded;
        }

        let ppu = Rc::new(RefCell::new(Ppu::new(bus.clone(), ppuBus)));
        bus.borrow_mut().attachPpu(ppu.clone());


        Console {
            window,
            eventLoop: Some(eventLoop),
            pixels,
            gui,
            guiCommands,
            audioSystem,
            cpu,
            ppu,
            apu,
            bus,
            cartridge,
            gameState
        }
    }

    fn returnToSplashScreen(&mut self) -> () {
        self.bus = Rc::new(RefCell::new(DataBus::new()));
        self.bus.borrow_mut().attachController1(Rc::new(RefCell::new(Controller::new())));
        self.cpu = Rc::new(RefCell::new(Cpu::new(self.bus.clone())));
        self.bus.borrow_mut().attachCpu(self.cpu.clone());
        self.apu = Rc::new(RefCell::new(Apu::new(self.bus.clone(), self.audioSystem.clone())));
        self.bus.borrow_mut().attachApu(self.apu.clone());
        
        let ppuBus = PpuBus::new();
        self.ppu = Rc::new(RefCell::new(Ppu::new(self.bus.clone(), ppuBus)));
        self.bus.borrow_mut().attachPpu(self.ppu.clone());
        
        self.gameState = GameState::NotLoaded;
    }

    fn copyBufferToPixels(&mut self, buffer: &Vec<u8>) -> () {
        let frame = self.pixels.get_frame();
        for (idx, pixel) in frame.chunks_exact_mut(4).into_iter().enumerate() {
            let realIdx = idx * 3;
            pixel[0] = buffer[realIdx];
            pixel[1] = buffer[realIdx + 1];
            pixel[2] = buffer[realIdx + 2];
            pixel[3] = 0xFF;
        }
    }

    pub fn run(mut self) {

        let mut audioTime: f64 = 0.0;
        let mut canPressEscape: bool = true;

        let mut input = WinitInputHelper::new();
        let img = include_bytes!("./resources/rustynes_splash_screen.png");
        let imgBytes = image::load_from_memory(img).unwrap().to_rgb8().into_raw();

        let mut pixelBuffer: Vec<u8> = vec![0; 256 * 240 * 3];

        if self.gameState == GameState::NotLoaded {
            pixelBuffer = imgBytes.clone();
            self.copyBufferToPixels(&pixelBuffer);
        }

        self.eventLoop.take().unwrap().run(move |event, _, controlFlow | {

            if input.update(&event) {
                let state = &self.gameState;
                match *state {
                    GameState::NotLoaded => {

                        if canPressEscape && input.key_pressed(VirtualKeyCode::Escape) {
                            canPressEscape = false;
                            *controlFlow = ControlFlow::Exit;
                        }

                        self.window.request_redraw();
                    }
                    GameState::Loaded => {
                        
                        for _ in 0..29781 {
                            for _ in 0..3 {
                                if let Some(buffer) = self.ppu.borrow_mut().cycleAndPrepareTexture().cloned() {
                                    pixelBuffer = buffer;
                                }
                            }
    
                            self.cpu.borrow_mut().cycle();
                            self.apu.borrow_mut().cycle();
    
                            audioTime += CPU_HERTZ_PER_CYCLE;
                            if audioTime >= AUDIO_HERTZ_PER_SAMPLE {
                                audioTime -= AUDIO_HERTZ_PER_SAMPLE;
                                self.apu.borrow_mut().addSampleToBuffer();
                            }
                        }

                        self.bus.borrow_mut().setControllerEvents(input.clone());
                        self.bus.borrow_mut().getControllerInput();

                        if canPressEscape && input.key_pressed(VirtualKeyCode::Escape) {
                            canPressEscape = false;
                            pixelBuffer = imgBytes.clone();
                            self.copyBufferToPixels(&pixelBuffer);
                            self.returnToSplashScreen();
                        }

                        self.window.request_redraw();
                    }
                }

                if input.key_released(VirtualKeyCode::Escape) {
                    canPressEscape = true;
                }
            }

            match event {
                Event::WindowEvent { event, .. } => {
                    self.gui.handleEvent(&event);
                    match *self.guiCommands.borrow() {
                        GuiCommands::Default => {}
                        GuiCommands::LoadGame => {

                            let rom = FileDialog::new()
                                .add_filter("rom", &["nes"])
                                .set_directory(home::home_dir().unwrap())
                                .pick_file();

                            let cartridge = Rc::new(RefCell::new(Cartridge::new(rom.unwrap().as_path())));
                            self.bus.borrow_mut().attachCartridge(cartridge.clone());
                            self.ppu.borrow_mut().attachCartridge(cartridge.clone());
                            self.cpu.borrow_mut().init();
                            self.gameState = GameState::Loaded;
                        }
                        GuiCommands::SaveState => {

                            let path = FileDialog::new()
                                .add_filter("json", &["json"])
                                .set_directory(home::home_dir().unwrap())
                                .save_file().unwrap();

                            SaveState::save(
                                path,
                                self.cpu.clone(),
                                self.ppu.clone(),
                                self.apu.clone(),
                                self.cartridge.as_ref().unwrap().clone()
                            );
                        }
                        GuiCommands::LoadState => {

                            let path = FileDialog::new()
                                .add_filter("json", &["json"])
                                .set_directory(home::home_dir().unwrap())
                                .pick_file().unwrap();

                            SaveState::load(
                                path,
                                self.cpu.clone(),
                                self.ppu.clone(),
                                self.apu.clone(),
                                self.cartridge.as_ref().unwrap().clone()
                            )
                        }
                    }
                    *self.guiCommands.borrow_mut() = GuiCommands::Default;

                    match event {
                        CloseRequested => {
                            *controlFlow = ControlFlow::Exit;
                        }
                        _ => {}
                    }
                }
                Event::RedrawRequested(_) => {

                    self.copyBufferToPixels(&pixelBuffer);
                    self.gui.prepareGui(&self.window);

                    self.pixels.render_with(|encoder, target, context| {

                        context.scaling_renderer.render(encoder, target);
                        self.gui.render(encoder, target, context).unwrap();

                        return Ok(());
                    });
                }
                _ => {}
            }


        });
    }
}