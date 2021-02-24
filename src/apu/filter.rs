#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(exceeding_bitshifts)]

use std::f32::consts::PI;

pub struct Filter {
    B0: f32,
    B1: f32,
    A1: f32,
    prevX: f32,
    prevY: f32,
}

impl Filter {
    pub fn LowPassFilter(sampleHertz: f32, cutOffFreq: f32) -> Self {
        let c = sampleHertz / PI / cutOffFreq;
        let ai = 1.0 / (1.0 + c);

        return Filter {
            B0: ai,
            B1: ai,
            A1: (1.0 - c) * ai,
            prevX: 0.0,
            prevY: 0.0,
        };
    }

    pub fn HighPassFilter(sampleHertz: f32, cutOffFreq: f32) -> Self {
        let c = sampleHertz / PI / cutOffFreq;
        let ai = 1.0 / (1.0 + c);

        return Filter {
            B0: c * ai,
            B1: -c * ai,
            A1: (1.0 - c) * ai,
            prevX: 0.0,
            prevY: 0.0,
        };
    }

    pub fn Step(&mut self, x: f32) -> f32 {
        let y = self.B0 * x + self.B1 * self.prevX - self.A1 * self.prevY;
        self.prevY = y;
        self.prevX = x;
        return y;
    }
}

