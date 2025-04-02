use drum_machine_bsp::hal::gpio::{DynPinId, FunctionSioOutput, Pin, PullNone};
use embedded_hal::digital::v2::OutputPin;

pub struct Cd4051Addressor {
    pub addr0: Pin<DynPinId, FunctionSioOutput, PullNone>,
    pub addr1: Pin<DynPinId, FunctionSioOutput, PullNone>,
    pub addr2: Pin<DynPinId, FunctionSioOutput, PullNone>,
}

impl Cd4051Addressor {
    const BUSY_WAIT_CYCLES: usize = 10;

    pub fn set(&mut self, address: u8) {
        let addr0_bit = (address & 0b1) == 1;
        let addr1_bit = ((address & 0b10) >> 1) == 1;
        let addr2_bit = ((address & 0b100) >> 2) == 1;

        self.addr0.set_state(addr0_bit.into()).unwrap();
        self.addr1.set_state(addr1_bit.into()).unwrap();
        self.addr2.set_state(addr2_bit.into()).unwrap();

        for _ in 0..Self::BUSY_WAIT_CYCLES {
            cortex_m::asm::nop();
        }
    }
}
