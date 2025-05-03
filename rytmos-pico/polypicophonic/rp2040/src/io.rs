use embedded_hal::digital::v2::InputPin;
use rp_pico::hal::gpio::{bank0::*, FunctionSioInput, Pin, PullUp};

use polypicophonic::{
    clavier::KeyId,
    io::{ClavierPins, Fifo},
};

pub struct SioFifo(pub rp_pico::hal::sio::SioFifo);

impl Fifo for SioFifo {
    fn write(&mut self, value: u32) {
        self.0.write(value);
    }
}

pub struct Rp2040Clavier {
    c_pin: Pin<Gpio4, FunctionSioInput, PullUp>,
    cis_pin: Pin<Gpio5, FunctionSioInput, PullUp>,
    d_pin: Pin<Gpio6, FunctionSioInput, PullUp>,
    dis_pin: Pin<Gpio7, FunctionSioInput, PullUp>,
    e_pin: Pin<Gpio8, FunctionSioInput, PullUp>,
    f_pin: Pin<Gpio9, FunctionSioInput, PullUp>,
    fis_pin: Pin<Gpio10, FunctionSioInput, PullUp>,
    g_pin: Pin<Gpio11, FunctionSioInput, PullUp>,
    gis_pin: Pin<Gpio19, FunctionSioInput, PullUp>,
    a_pin: Pin<Gpio18, FunctionSioInput, PullUp>,
    ais_pin: Pin<Gpio17, FunctionSioInput, PullUp>,
    b_pin: Pin<Gpio16, FunctionSioInput, PullUp>,
    fn0_pin: Pin<Gpio3, FunctionSioInput, PullUp>,
    fn1_pin: Pin<Gpio2, FunctionSioInput, PullUp>,
    fn2_pin: Pin<Gpio1, FunctionSioInput, PullUp>,
    fn3_pin: Pin<Gpio0, FunctionSioInput, PullUp>,
}

impl Rp2040Clavier {
    pub fn new(pins: rp_pico::Pins) -> Self {
        Self {
            c_pin: pins.gpio4.into_pull_up_input(),
            cis_pin: pins.gpio5.into_pull_up_input(),
            d_pin: pins.gpio6.into_pull_up_input(),
            dis_pin: pins.gpio7.into_pull_up_input(),
            e_pin: pins.gpio8.into_pull_up_input(),
            f_pin: pins.gpio9.into_pull_up_input(),
            fis_pin: pins.gpio10.into_pull_up_input(),
            g_pin: pins.gpio11.into_pull_up_input(),
            gis_pin: pins.gpio19.into_pull_up_input(),
            a_pin: pins.gpio18.into_pull_up_input(),
            ais_pin: pins.gpio17.into_pull_up_input(),
            b_pin: pins.gpio16.into_pull_up_input(),
            fn0_pin: pins.gpio3.into_pull_up_input(),
            fn1_pin: pins.gpio2.into_pull_up_input(),
            fn2_pin: pins.gpio1.into_pull_up_input(),
            fn3_pin: pins.gpio0.into_pull_up_input(),
        }
    }
}

impl ClavierPins for Rp2040Clavier {
    fn read(&self, id: KeyId) -> bool {
        match id {
            KeyId::NoteC => self.c_pin.is_low().unwrap(),
            KeyId::NoteCis => self.cis_pin.is_low().unwrap(),
            KeyId::NoteD => self.d_pin.is_low().unwrap(),
            KeyId::NoteDis => self.dis_pin.is_low().unwrap(),
            KeyId::NoteE => self.e_pin.is_low().unwrap(),
            KeyId::NoteF => self.f_pin.is_low().unwrap(),
            KeyId::NoteFis => self.fis_pin.is_low().unwrap(),
            KeyId::NoteG => self.g_pin.is_low().unwrap(),
            KeyId::NoteGis => self.gis_pin.is_low().unwrap(),
            KeyId::NoteA => self.a_pin.is_low().unwrap(),
            KeyId::NoteAis => self.ais_pin.is_low().unwrap(),
            KeyId::NoteB => self.b_pin.is_low().unwrap(),
            KeyId::Fn0 => self.fn0_pin.is_low().unwrap(),
            KeyId::Fn1 => self.fn1_pin.is_low().unwrap(),
            KeyId::Fn2 => self.fn2_pin.is_low().unwrap(),
            KeyId::Fn3 => self.fn3_pin.is_low().unwrap(),
        }
    }
}
