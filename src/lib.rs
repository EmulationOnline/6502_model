// 6502 Model
// Cycle accurate model based on observing behavior of a real chip.
//
// The goal of this project is to perfectly recreate all the behavior of the chip,
// not necessarily implement it in the same way.
//
// Representing Signals in Rust:
// - Buses can be represented by the unsigned int of appropriate size.
// - Tri state pins are represented by Option<bool>, and None indicates floating / HighZ.
use std::collections::VecDeque;

mod trace_tests;

// Small internal instructions that perform the work for each
// cycle of a user-facing instruction.
#[derive(Clone, Copy, Debug)]
enum UOp {
    Nop,
    Fetch,
    ResetRegs,
    ReadPC{first: bool, addr: u16},
}

struct W6502 {
    outputs: Outputs,
    prev_clk: bool,

    //
    // Internal Execution State
    // Analogs of these may or may not not exist in the real chip, but are important for managing
    // execution.
    //

    // Most instructions take several cycles. The queue
    // holds remaining steps for the last fetched
    // instruction.
    queue: VecDeque<UOp>,
    active_uop: UOp,

    //
    // Registers
    // These are real internal state documented in the chip.
    //
    pc: u16,
    acc: u8,
    x: u8,
    y: u8,
    sp: u8,       // The top of stack is 0x0100 + sp
    flags: u8,    // NZCIDV
}

// Pins read by the 6502
#[derive(Clone, Copy)]
struct Inputs {
    clk: bool,
    n_reset: bool,    // active low reset
    data: u8,
}

// Pins set by the 6502.
struct Outputs {
    address: u16,
    data: Option<u8>,   // None if reading, Some if writing.
    rwb: bool,          // true for read, false for write
}

impl Outputs {
    fn new() -> Outputs {
        Outputs {
            address: 0xFFFF,
            data: None,
            rwb: true,
        }
    }
}

impl W6502 {
    pub fn new() -> W6502 {
        W6502 {
            outputs: Outputs::new(),
            prev_clk: false,
            queue: VecDeque::new(),
            active_uop: UOp::Nop,

            // "random" nonzero values before reset
            pc: 0xcafe,
            acc: 0xAA,
            flags: 0xFF,
            sp: 0xfc,
            x: 0xbc,
            y: 0xca,
        }
    }

    // Utility, lower and raise the clock for a given
    // input.
    pub fn cycle(&mut self, inputs: &Inputs) {
        let mut inputs = inputs.clone();
        inputs.clk = false;
        self.tick(&inputs);
        inputs.clk = true;
        self.tick(&inputs);
    }

    pub fn tick(&mut self, inputs: &Inputs) {
        let posedge =!self.prev_clk && inputs.clk; 
        let op = if posedge {
            if self.queue.len() > 0 {
                self.queue.pop_front().unwrap()
            } else {
                UOp::Fetch
            }
        } else {
            self.active_uop
        };
        self.active_uop = op;

        if !inputs.n_reset {
            // unspecified behavior for 6 cycles, then
            // read the reset vector, then set pc
            self.queue.clear();
            for i in 0 .. 6 {
                self.queue.push_back(UOp::Nop);
            }
            self.queue.push_back(UOp::ReadPC{first: true, addr: 0xFFFC});
            self.queue.push_back(UOp::ReadPC{first: false, addr: 0xFFFD});
            return;
        }

        // Execute uops.
        match op {
            UOp::Nop => {
                // nop reads past the opcode while stalling.
                self.set_addr(self.pc);
            },
            UOp::Fetch => {
                if posedge {
                    self.set_addr(self.pc);
                } else {
                    self.decode_op(inputs.data);
                }
            },
            UOp::ResetRegs => {
                // TODO: initialize registers for reset
            },
            UOp::ReadPC{first, addr} => {
                if posedge {
                    self.set_addr(addr);
                } else {
                    if first {
                        self.pc = (self.pc & 0xFF00) | (inputs.data as u16);
                    } else {
                        self.pc = (self.pc & 0x00FF) | ((inputs.data as u16) << 8);
                    }
                }
            },
        }

        self.prev_clk = inputs.clk;
    }
    pub fn outputs(&self) -> &Outputs {
        &self.outputs
    }

    // decode_op is called at the end of a fetch, when the
    // cpu has just read the opcode for the next byte.
    //
    // This function is responsible for decoding the opcode byte,
    // and setting up the queue to execute the rest of the instruction.
    // After decoding, PC should point to the next instruction.
    fn decode_op(&mut self, opcode: u8) {
        assert_eq!(0, self.queue.len());
        match opcode {
            0x4C => {
                // jmp abs
                self.queue.push_back(UOp::ReadPC{first: true, addr: self.pc+1});
                self.queue.push_back(UOp::ReadPC{first: false, addr: self.pc+2});
                self.pc += 3;
            },
            0xEA => {
                self.queue.push_back(UOp::Nop);
                // nop
                self.pc += 1;
            },
            _ => {
                assert!(false, "Unsupported opcode: {opcode:2X}");
            },
        }
    }

    fn set_addr(&mut self, value: u16) {
        self.outputs.address = value;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn clock(cpu: &mut W6502, inputs: &Inputs) {
        let mut inputs = *inputs;
        // toggle the clock low->high while asserting inputs.
        inputs.clk = false;
        cpu.tick(&inputs);
        inputs.clk = true;
        cpu.tick(&inputs);
    }

    #[test]
    fn test_reset() {
        // After clocking the chip with reset low, the chip will run for 6 cycles
        // before reading from the reset vector. The chip will then begin executing
        // from the address found.
        //
        // The standard trace tests ignore the trace before the reset vector read, since it is
        // varies based on the previous state of the chip. This is why reset needs a non-trace
        // test.
        // 
        // Reset involves clocking the chip with n_reset held low for two cycles. After 6 cycles,
        // the reset vector will be read from 0xFFFC and 0xFFFD, then the chip will execute
        // from that address.
        let mut cpu = W6502::new();
        const RESET_CYCLES : usize = 2;
        const PRE_VECTOR_CYCLES : usize = 6;

        let mut inputs = Inputs {
            data: 0xFF,
            n_reset: false,
            clk: false,
        };

        for i in 0 .. RESET_CYCLES {
            cpu.cycle(&inputs);
        }

        inputs.n_reset = true;
        // for the next 6 cycles, the cpu should be reading only.
        for i in 0 .. PRE_VECTOR_CYCLES {
            cpu.cycle(&inputs);
            assert_eq!(true, cpu.outputs().rwb);
        }

        // Then it should read the reset vector
        // Vector read 1
        clock(&mut cpu, &inputs);
        assert_eq!(0xFFFC, cpu.outputs().address);
        inputs.data = 0xAD;

        // Vector read 2
        clock(&mut cpu, &inputs);
        assert_eq!(0xFFFD, cpu.outputs().address);
        inputs.data = 0xDE;

        // start reading from target address
        clock(&mut cpu, &inputs);
        assert_eq!(0xDEAD, cpu.outputs().address);
    }
}
