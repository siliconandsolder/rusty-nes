#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

extern crate sdl2;

use crate::cpu::Cpu;
use crate::ppu::Ppu;
use crate::cartridge::Cartridge;
use crate::data_bus::DataBus;
use crate::clock::Clocked;
use std::time::{SystemTime, Duration};
use std::thread::sleep;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use sdl2::EventPump;
use self::sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::gfx::framerate::FPSManager;
use crate::ppu_bus::PpuBus;
use crate::controller::Controller;
use crate::apu::Apu;
use crate::save_load::SaveState;
use self::sdl2::audio::AudioSpecDesired;
use sdl2::image::{InitFlag, LoadTexture};
use self::sdl2::mouse::SystemCursor::No;
use std::process::exit;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::WindowContext;
use self::sdl2::event::Event;
use self::sdl2::keyboard::Keycode;
use self::sdl2::render::{Canvas, WindowCanvas};
use self::sdl2::{Sdl, AudioSubsystem};
use self::sdl2::messagebox::{show_simple_message_box, MessageBoxFlag};

const MASTER_CLOCK_NANO: u8 = 47;
// should be about 46.56, but the std::thread functions don't allow decimals
const CPU_HERTZ_PER_CYCLE: f64 = 1.0 / 1789773.0;
const CPU_HERTZ: f64 = 1789773.0;
const AUDIO_HERTZ: u16 = 44100;
const AUDIO_HERTZ_PER_SAMPLE: f64 = 1.0 / 44100.0;

const NANO_PER_FRAME: u128 = ((1.0 / 60.0) * 1000.0 * 1000000.0) as u128;

struct MsgFont<'a, 'b> {
    ttfContext: Sdl2TtfContext,
    font: Option<Font<'a, 'b>>,
    msgBox: Rect,
    textureCreator: TextureCreator<WindowContext>,
    texture: Option<Texture<'a>>
}

impl<'a, 'b> MsgFont<'a, 'b> {
    fn new(creator: TextureCreator<WindowContext>) -> Self {
        MsgFont {
            ttfContext: sdl2::ttf::init().unwrap(),
            font: None,
            msgBox: Rect::new(50, 600, 100, 35),
            textureCreator: creator,
            texture: None
        }
    }

    fn generateTexture(&mut self) -> () {
        self.font = Some(self.ttfContext.load_font("./nes.ttf", 128).unwrap());
        let surface = self.font.as_ref().unwrap().render("HELP").solid(Color::RGB(255, 255, 255)).unwrap();
        self.texture = Some(self.textureCreator.create_texture_from_surface(surface).unwrap());
    }

    fn updateMsgTexture(&mut self, msg: &str) -> () {
        if self.texture.is_none() {
            self.generateTexture();
        }
        let surface = self.font.as_ref().unwrap().render(msg).solid(Color::RGB(255, 255, 255)).unwrap();
        self.texture.as_mut().unwrap().update(self.msgBox, &surface.without_lock().unwrap(), (surface.width() * 3) as usize);
    }

    fn getTexture(&self) -> Option<&Texture<'a>> {
        return self.texture.as_ref();
    }

    fn getMsgBox(&self) -> &Rect {
        return &self.msgBox;
    }
}

pub struct Console<'a, 'b> {
    canvas: Rc<RefCell<WindowCanvas>>,
    audioSystem: Rc<RefCell<AudioSubsystem>>,
    eventPump: Rc<RefCell<EventPump>>,
    msgBox: MsgFont<'a, 'b>,
    cpu: Rc<RefCell<Cpu<'a>>>,
    ppu: Rc<RefCell<Ppu<'a>>>,
    apu: Rc<RefCell<Apu<'a>>>,
    bus: Rc<RefCell<DataBus<'a>>>,
    cartridge: Rc<RefCell<Cartridge>>,
}

impl<'a, 'b> Console<'a, 'b> {
    pub fn new(game: Option<&str>) -> Self {

        // sdl setup
        let sdl = sdl2::init().unwrap();

        let vid = sdl.video().unwrap();
        let audioSystem = Rc::new(RefCell::new(sdl.audio().unwrap()));

        let window = vid
            .window("RustyNES", 768, 720)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        // will eventually pass this to the controller
        let eventPump = Rc::new(RefCell::new(sdl.event_pump().unwrap()));
        let canvas = Rc::new(RefCell::new(window.into_canvas().software().build().unwrap()));

        return Console::assembleConsole(canvas, audioSystem, eventPump, game);
    }

    fn assembleConsole(canvas: Rc<RefCell<WindowCanvas>>, audioSystem: Rc<RefCell<AudioSubsystem>>, eventPump: Rc<RefCell<EventPump>>, game: Option<&str>) -> Self {
        let fileName = match game {
            None => {
                Console::loadSplashScreen(canvas.clone(), eventPump.clone())
            }
            Some(fileName) => {
                String::from(fileName)
            }
        };
        let filePath = Path::new(fileName.as_str());



        let controller1 = Rc::new(RefCell::new(Controller::new(eventPump.clone())));
        let cartridge = Rc::new(RefCell::new(Cartridge::new(filePath)));
        let bus = Rc::new(RefCell::new(DataBus::new()));
        bus.borrow_mut().attachController1(controller1);
        bus.borrow_mut().attachCartridge(cartridge.clone());
        let cpu = Rc::new(RefCell::new(Cpu::new(bus.clone())));
        bus.borrow_mut().attachCpu(cpu.clone());
        let apu = Rc::new(RefCell::new(Apu::new(bus.clone(), audioSystem.clone())));
        bus.borrow_mut().attachApu(apu.clone());
        let ppuBus = PpuBus::new(cartridge.clone());
        let ppu = Rc::new(RefCell::new(Ppu::new(bus.clone(), canvas.clone(), ppuBus)));
        bus.borrow_mut().attachPpu(ppu.clone());

        let mut msgBox = MsgFont::new(canvas.borrow().texture_creator());

        Console {
            canvas,
            audioSystem,
            eventPump,
            msgBox,
            cpu,
            ppu,
            apu,
            bus,
            cartridge,
        }
    }

    fn loadSplashScreen(canvas: Rc<RefCell<WindowCanvas>>, eventPump: Rc<RefCell<EventPump>>) -> String {
        let textureCreator = canvas.borrow_mut().texture_creator();
        let imgBytes = include_bytes!("./resources/rustynes_splash_screen.png");
        let imgTexture = textureCreator.load_texture_bytes(imgBytes).unwrap();
        canvas.borrow_mut().copy(&imgTexture, None, None).unwrap();
        canvas.borrow_mut().present();

        loop {
            for event in eventPump.borrow_mut().poll_event() {
                match event {
                    Event::DropFile {filename, .. } => { return filename; }
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => { exit(0) }
                    Event::Quit { .. } => { exit(0); }
                    _ => {}
                }
            }
        }
    }

    fn checkSDLEvents(&mut self, events: Vec<Event>) -> () {
        for event in events {
            match event {
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => { self.returnToSplashScreen(); }
                Event::KeyDown { keycode: Some(Keycode::I), .. } => { self.showControls(); }
                Event::KeyDown { keycode: Some(Keycode::S), .. } => { SaveState::save(
                    self.cpu.clone(),
                    self.ppu.clone(),
                    self.apu.clone(),
                    self.cartridge.clone())
                }
                Event::KeyDown { keycode: Some(Keycode::D), .. } => { SaveState::load(
                    self.cpu.clone(),
                    self.ppu.clone(),
                    self.apu.clone(),
                    self.cartridge.clone())
                }
                _ => {}
            }
        }
    }

    fn returnToSplashScreen(&mut self) -> () {
        *self = Console::assembleConsole(self.canvas.clone(), self.audioSystem.clone(), self.eventPump.clone(), None);
    }

    fn showControls(&self) -> () {
        show_simple_message_box(
            MessageBoxFlag::INFORMATION,
            "Controls",
            "Arrow Keys - Move\nZ - A button\nX - B button\nEnter/Return - Start button\nRight Shift - Select button",
            self.canvas.borrow_mut().window()
        ).unwrap();
    }

    fn renderFrame(&self, texture: &Texture<'a>) -> () {
        self.canvas.borrow_mut().clear();
        self.canvas.borrow_mut().copy(texture, None, None).unwrap();

        if let Some(fontTexture) = self.msgBox.getTexture() {
            self.canvas.borrow_mut().copy(fontTexture, None, Some(Rect::new(50, 600, 100, 35))).unwrap();
        }

        self.canvas.borrow_mut().present();
    }

    fn updateMsgBox(&mut self, msg: &str) -> () {
        self.msgBox.updateMsgTexture(msg);
    }
}

impl<'a, 'b> Clocked for Console<'a, 'b> {
    #[inline]
    fn cycle(&mut self) {

        let mut fps = FPSManager::new();
        fps.set_framerate(60);
        let mut audioTime: f64 = 0.0;
        self.updateMsgBox("HELP");

        'game: loop {

            // one frame (approximately)
            for _ in 0..=29781 {

                for _ in 0..3 {
                    if let Some(texture) = self.ppu.borrow_mut().cycleAndPrepareTexture() {
                        self.renderFrame(texture)
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

            let events = self.eventPump.borrow_mut().poll_event().into_iter().collect::<Vec<Event>>();
            self.bus.borrow_mut().setControllerEvents(events.clone());
            self.bus.borrow_mut().getControllerInput();
            self.checkSDLEvents(events);
            fps.delay();
        }
    }
}