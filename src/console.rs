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
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::{Window, WindowBuilder};
use winit_input_helper::WinitInputHelper;
use crate::gui::Gui;
use crate::gui_commands::GuiCommands;

const CPU_HERTZ_PER_CYCLE: f64 = 1.0 / 1789773.0;
const AUDIO_HERTZ_PER_SAMPLE: f64 = 1.0 / 44100.0;

const SCREEN_WIDTH: u32 = SCREEN_WIDTH;
const SCREEN_HEIGHT: u32 = 720;

const PIXEL_WIDTH: u32 = PIXEL_WIDTH;
const PIXEL_HEIGHT: u32 = PIXEL_HEIGHT;


enum GameState {
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

        return Console::assembleConsole(audioSystem, game);
    }

    fn assembleConsole(audioSystem: Rc<RefCell<AudioSubsystem>>, game: Option<&str>) -> Self {

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


        let controller1 = Rc::new(RefCell::new(Controller::new()));
        let bus = Rc::new(RefCell::new(DataBus::new()));
        bus.borrow_mut().attachController1(controller1);
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


    fn checkInputEvents(&mut self, input: WinitInputHelper, flow: &mut ControlFlow) -> () {

        if input.key_pressed(VirtualKeyCode::Escape) {
            self.returnToSplashScreen();
        }

        if input.quit() {
            *flow = ControlFlow::Exit;
        }
    }

    fn returnToSplashScreen(&mut self) -> () {
        *self = Console::assembleConsole(self.audioSystem.clone(), None);
    }

    pub fn run(mut self) {

        //let mut fps = FPSManager::new();
        //fps.set_framerate(60);
        let mut audioTime: f64 = 0.0;
        let mut input = WinitInputHelper::new();
        let imgBytes = include_bytes!("./resources/rustynes_splash_screen.png");
        let bytes = image::load_from_memory(imgBytes).unwrap().to_rgb8().into_raw();
        //self.updateMsgBox("HELP");

        self.eventLoop.take().unwrap().run(move |event, _, controlFlow | {

            let state = &self.gameState;
            match *state {
                GameState::NotLoaded => {
                    let frame = self.pixels.get_frame();
                    for (idx, pixel) in frame.chunks_exact_mut(4).into_iter().enumerate() {
                        let realIdx = idx * 3;
                        pixel[0] = bytes[realIdx];
                        pixel[1] = bytes[realIdx + 1];
                        pixel[2] = bytes[realIdx + 2];
                        pixel[3] = 0xFF;
                    }

                    if input.update(&event) && (input.key_pressed(VirtualKeyCode::Escape) || input.quit()) {
                        *controlFlow = ControlFlow::Exit;
                    }

                    self.window.request_redraw();
                }
                GameState::Loaded => {
                    for _ in 0..3 {
                        if let Some(buffer) = self.ppu.borrow_mut().cycleAndPrepareTexture().cloned() {
                            let frame = self.pixels.get_frame();
                            for (idx, pixel) in frame.chunks_exact_mut(4).into_iter().enumerate() {
                                let realIdx = idx * 3;
                                pixel[0] = buffer[realIdx];
                                pixel[1] = buffer[realIdx + 1];
                                pixel[2] = buffer[realIdx + 2];
                                pixel[3] = 0xFF;
                            }

                            self.window.request_redraw();
                        }
                    }

                    self.cpu.borrow_mut().cycle();
                    self.apu.borrow_mut().cycle();

                    audioTime += CPU_HERTZ_PER_CYCLE;
                    if audioTime >= AUDIO_HERTZ_PER_SAMPLE {
                        audioTime -= AUDIO_HERTZ_PER_SAMPLE;
                        self.apu.borrow_mut().addSampleToBuffer();
                    }

                    if input.update(&event) {
                        self.bus.borrow_mut().setControllerEvents(input.clone());
                        self.bus.borrow_mut().getControllerInput();
                        self.checkInputEvents(input.clone(), controlFlow);
                    }
                }
            }

            match event {
                Event::Suspended => {
                    *controlFlow = ControlFlow::Wait;
                }
                Event::Resumed => {
                    *controlFlow = ControlFlow::Poll;
                }
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
                }
                Event::RedrawRequested(_) => {

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