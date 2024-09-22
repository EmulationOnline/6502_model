// 6502 Model
// Cycle accurate model based on observing behavior of a real chip.
//
// The goal of this project is to perfectly recreate all the behavior of the chip,
// not necessarily implement it in the same way.
//
// Representing Signals in Rust:
// - Buses can be represented by the unsigned int of appropriate size.
// - Tri state pins are represented by Option<bool>, and None indicates floating / HighZ.

struct W6502 {
}

// Pins read by the 6502
struct Inputs {
}

// Pins set by the 6502.
struct Outputs {
}

impl W6502 {
    pub fn new() -> W6502 {
        W6502 {
        }
    }

    pub fn tick(inputs: &Inputs) {
    }

    pub fn outputs() -> &Outputs {
    }
}
