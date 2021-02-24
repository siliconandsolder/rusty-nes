#![allow(non_snake_case)]
#![allow(warnings)]

use std::rc::Rc;
use std::cell::RefCell;
use sdl2::EventPump;
use crate::clock::Clocked;
use std::process::exit;
use sdl2::keyboard::Keycode;
use sdl2::event::Event;

pub struct Controller {
    eventPump: Rc<RefCell<EventPump>>,
    controllerState: u8,
    controllerIdx: u8,
    strobe: bool,
}

impl Controller {
    pub fn new(eventPump: Rc<RefCell<EventPump>>) -> Self {
        Controller {
            eventPump,
            controllerState: 0,
            controllerIdx: 0,
            strobe: false,
        }
    }

    pub fn getState(&mut self) -> u8 {
        if self.strobe {
            self.controllerIdx = 0;
        }

        let state = (self.controllerState & (1 << self.controllerIdx)) >> self.controllerIdx;
        self.controllerIdx = (self.controllerIdx + 1) & 7;

        return state;
    }

    pub fn writeState(&mut self, val: u8) -> () {
        if (val & 1) == 1 {
            self.strobe = true;
            self.controllerIdx = 0;
        } else {
            self.strobe = false;
        }
    }
}

const A_POS: u8 = 0;
const B_POS: u8 = 1;
const SEL_POS: u8 = 2;
const STR_POS: u8 = 3;
const UP_POS: u8 = 4;
const DWN_POS: u8 = 5;
const LFT_POS: u8 = 6;
const RGT_POS: u8 = 7;

impl Clocked for Controller {
    fn cycle(&mut self) {
        for event in self.eventPump.borrow_mut().poll_iter() {
            match event {
                Event::Quit { .. } => { exit(0); }
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => { exit(0) }

                Event::KeyDown { keycode: Some(Keycode::Z), .. } => { self.controllerState |= (1 << A_POS); }
                Event::KeyDown { keycode: Some(Keycode::X), .. } => { self.controllerState |= (1 << B_POS); }
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => { self.controllerState |= (1 << LFT_POS); }
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => { self.controllerState |= (1 << RGT_POS); }
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => { self.controllerState |= (1 << UP_POS); }
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => { self.controllerState |= (1 << DWN_POS); }
                Event::KeyDown { keycode: Some(Keycode::Return), .. } => { self.controllerState |= (1 << STR_POS); }
                Event::KeyDown { keycode: Some(Keycode::RShift), .. } => { self.controllerState |= (1 << SEL_POS); }

                Event::KeyUp { keycode: Some(Keycode::Z), .. } => { self.controllerState &= !(1 << A_POS); }
                Event::KeyUp { keycode: Some(Keycode::X), .. } => { self.controllerState &= !(1 << B_POS); }
                Event::KeyUp { keycode: Some(Keycode::Left), .. } => { self.controllerState &= !(1 << LFT_POS); }
                Event::KeyUp { keycode: Some(Keycode::Right), .. } => { self.controllerState &= !(1 << RGT_POS); }
                Event::KeyUp { keycode: Some(Keycode::Up), .. } => { self.controllerState &= !(1 << UP_POS); }
                Event::KeyUp { keycode: Some(Keycode::Down), .. } => { self.controllerState &= !(1 << DWN_POS); }
                Event::KeyUp { keycode: Some(Keycode::Return), .. } => { self.controllerState &= !(1 << STR_POS); }
                Event::KeyUp { keycode: Some(Keycode::RShift), .. } => { self.controllerState &= !(1 << SEL_POS); }

                _ => {}
            }
        }
    }
}