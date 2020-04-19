#![allow(non_snake_case)]

pub trait Clocked {
    fn cycle(&mut self);
}